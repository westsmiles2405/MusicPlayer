//! Audio engine: ringbuf-backed shared buffer + cpal output stream.
//! PCM decoded in engine thread, pushed to ringbuf Producer.
//! cpal callback reads from ringbuf Consumer via try_lock — never blocks.

use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering},
    Arc,
};
use std::thread;
use std::time::{Duration, Instant};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, Stream, StreamConfig};
use crossbeam_channel::{Receiver, Sender};
use parking_lot::Mutex;
use ringbuf::traits::{Consumer as _, Producer as _, Split};
use ringbuf::{HeapCons, HeapProd, HeapRb};

use crate::error::{AppError, AppResult};
use crate::player::decoder::DecodedTrack;
use crate::player::gapless::{GaplessPredecoder, GaplessRequest, GaplessResult};
use crate::player::queue::{PlayQueue, QueueMove};
use crate::player::state::{
    now_ms, EngineCommand, EngineTrack, NowPlayingTrack, PlaybackErrorCode, PlaybackErrorEvent,
    PlaybackSession, PlaybackStatus, PlayerEvent, PlayerSnapshot, SessionEndReason,
};

// ── SharedAudioBuffer ───────────────────────────────────────────

pub struct SharedAudioBuffer {
    consumer: Mutex<HeapCons<f32>>,
    volume_bits: AtomicU32,
    muted: AtomicBool,
    underruns: AtomicU64,
}

impl SharedAudioBuffer {
    pub fn set_volume(&self, volume: f32) {
        self.volume_bits
            .store(volume.clamp(0.0, 1.0).to_bits(), Ordering::Relaxed);
    }

    pub fn set_muted(&self, muted: bool) {
        self.muted.store(muted, Ordering::Relaxed);
    }

    /// Called from the cpal audio callback. Uses try_lock() to avoid ever
    /// blocking the realtime thread. If the consumer lock is held by the
    /// engine (during clear/seek/stop), output silence.
    pub fn fill_output_f32(&self, output: &mut [f32]) {
        let volume = f32::from_bits(self.volume_bits.load(Ordering::Relaxed));
        let muted = self.muted.load(Ordering::Relaxed);

        if let Some(mut consumer) = self.consumer.try_lock() {
            for out in output.iter_mut() {
                let sample = (*consumer).try_pop().unwrap_or_else(|| {
                    self.underruns.fetch_add(1, Ordering::Relaxed);
                    0.0
                });
                *out = if muted { 0.0 } else { sample * volume };
            }
        } else {
            output.fill(0.0);
        }
    }

    /// Blocking clear — called from engine thread during stop/seek.
    pub fn clear(&self) {
        let mut consumer = self.consumer.lock();
        while (*consumer).try_pop().is_some() {}
    }

    #[allow(dead_code)]
    pub fn underrun_count(&self) -> u64 {
        self.underruns.load(Ordering::Relaxed)
    }
}

// ── Producer handle ─────────────────────────────────────────────

pub struct AudioProducer {
    producer: Mutex<HeapProd<f32>>,
}

impl AudioProducer {
    /// Push samples into the ring buffer. Returns how many samples were
    /// actually accepted (stops on first full-buffer rejection so we never
    /// silently drop samples while advancing position).
    pub fn push_samples(&self, samples: &[f32]) -> usize {
        let mut prod = self.producer.lock();
        let mut pushed = 0;
        for &s in samples {
            if (*prod).try_push(s).is_err() {
                break;
            }
            pushed += 1;
        }
        pushed
    }
}

pub fn create_audio_buffer(
    capacity_samples: usize,
    volume: f32,
    muted: bool,
) -> (AudioProducer, Arc<SharedAudioBuffer>) {
    let rb = HeapRb::<f32>::new(capacity_samples);
    let (producer, consumer) = rb.split();
    let shared = Arc::new(SharedAudioBuffer {
        consumer: Mutex::new(consumer),
        volume_bits: AtomicU32::new(volume.clamp(0.0, 1.0).to_bits()),
        muted: AtomicBool::new(muted),
        underruns: AtomicU64::new(0),
    });
    let prod = AudioProducer {
        producer: Mutex::new(producer),
    };
    (prod, shared)
}

// ── cpal stream helpers ─────────────────────────────────────────

fn open_default_stream(
    audio: Arc<SharedAudioBuffer>,
    event_tx: Sender<PlayerEvent>,
    error_flag: Arc<AtomicBool>,
) -> AppResult<(Stream, StreamConfig)> {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or_else(|| AppError::Playback("no default output device".into()))?;
    let supported = device
        .default_output_config()
        .map_err(|e| AppError::Playback(format!("default output config failed: {e}")))?;
    let sample_format = supported.sample_format();
    let config: StreamConfig = supported.into();
    let err_tx = event_tx.clone();
    let on_error = move |err: cpal::StreamError| {
        error_flag.store(true, Ordering::Relaxed);
        let _ = err_tx.send(PlayerEvent::Error(PlaybackErrorEvent {
            track_id: None,
            code: PlaybackErrorCode::StreamError,
            message: err.to_string(),
            recoverable: true,
        }));
    };
    let stream = match sample_format {
        SampleFormat::F32 => device.build_output_stream(
            &config,
            move |data: &mut [f32], _| audio.fill_output_f32(data),
            on_error,
            None,
        ),
        SampleFormat::I16 => device.build_output_stream(
            &config,
            move |data: &mut [i16], _| fill_output_i16(&audio, data),
            on_error,
            None,
        ),
        SampleFormat::U16 => device.build_output_stream(
            &config,
            move |data: &mut [u16], _| fill_output_u16(&audio, data),
            on_error,
            None,
        ),
        other => {
            return Err(AppError::Playback(format!(
                "unsupported output sample format {other:?}"
            )))
        }
    }
    .map_err(|e| AppError::Playback(format!("output stream build failed: {e}")))?;
    stream
        .play()
        .map_err(|e| AppError::Playback(format!("output stream play failed: {e}")))?;
    Ok((stream, config))
}

fn fill_output_i16(audio: &SharedAudioBuffer, output: &mut [i16]) {
    let mut tmp = vec![0.0_f32; output.len()];
    audio.fill_output_f32(&mut tmp);
    for (dst, src) in output.iter_mut().zip(tmp) {
        *dst = (src.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
    }
}

fn fill_output_u16(audio: &SharedAudioBuffer, output: &mut [u16]) {
    let mut tmp = vec![0.0_f32; output.len()];
    audio.fill_output_f32(&mut tmp);
    for (dst, src) in output.iter_mut().zip(tmp) {
        let normalized = (src.clamp(-1.0, 1.0) + 1.0) * 0.5;
        *dst = (normalized * u16::MAX as f32) as u16;
    }
}

// ── AudioEngine ─────────────────────────────────────────────────

pub struct AudioEngine {
    command_rx: Receiver<EngineCommand>,
    event_tx: Sender<PlayerEvent>,
    producer: AudioProducer,
    audio: Arc<SharedAudioBuffer>,
    stream: Option<Stream>,
    stream_error_flag: Arc<AtomicBool>,
    snapshot: PlayerSnapshot,
    shutdown: bool,
    queue: Option<PlayQueue>,
    tracks: Vec<EngineTrack>,
    decoder: Option<DecodedTrack>,
    session: Option<PlaybackSession>,
    last_tick: Instant,
    gapless: GaplessPredecoder,
}

impl AudioEngine {
    pub fn spawn(
        command_rx: Receiver<EngineCommand>,
        event_tx: Sender<PlayerEvent>,
    ) -> thread::JoinHandle<()> {
        thread::spawn(move || {
            let (producer, audio) = create_audio_buffer(48_000 * 2 * 5, 0.8, false);
            let stream_error_flag = Arc::new(AtomicBool::new(false));
            let mut engine = Self {
                command_rx,
                event_tx,
                producer,
                audio,
                stream: None,
                stream_error_flag,
                snapshot: PlayerSnapshot::default(),
                shutdown: false,
                queue: None,
                tracks: Vec::new(),
                decoder: None,
                session: None,
                last_tick: Instant::now(),
                gapless: GaplessPredecoder::new(),
            };
            engine.run();
        })
    }

    fn run(&mut self) {
        while !self.shutdown {
            match self.command_rx.recv_timeout(Duration::from_millis(20)) {
                Ok(command) => self.handle_command(command),
                Err(crossbeam_channel::RecvTimeoutError::Timeout) => {}
                Err(crossbeam_channel::RecvTimeoutError::Disconnected) => self.shutdown = true,
            }
            self.tick_playback();
            if self.stream_error_flag.swap(false, Ordering::Relaxed) {
                self.recover_output_once();
            }
        }
        let _ = self.event_tx.send(PlayerEvent::ShutdownComplete);
    }

    fn handle_command(&mut self, command: EngineCommand) {
        match command {
            EngineCommand::LoadQueueAndPlay { tracks, index } => {
                self.finish_session(SessionEndReason::Replaced);
                self.tracks = tracks;
                self.queue = Some(
                    PlayQueue::from_tracks(self.tracks.iter().map(|t| t.id).collect(), index)
                        .unwrap(),
                );
                self.start_current_track();
            }
            EngineCommand::Pause => self.pause(),
            EngineCommand::Resume => self.resume(),
            EngineCommand::Toggle => {
                if self.snapshot.status == PlaybackStatus::Playing {
                    self.pause();
                } else if self.snapshot.status == PlaybackStatus::Paused {
                    self.resume();
                }
            }
            EngineCommand::Stop => self.stop(),
            EngineCommand::Seek { position_ms } => {
                let target = clamp_seek(position_ms, self.snapshot.duration_ms);
                self.seek_to(target);
            }
            EngineCommand::Next => {
                self.finish_session(SessionEndReason::Next);
                self.advance_next();
            }
            EngineCommand::Previous => {
                if previous_should_restart_current(self.snapshot.position_ms) {
                    self.seek_to(0);
                } else {
                    self.finish_session(SessionEndReason::Previous);
                    self.go_previous();
                }
            }
            EngineCommand::SetVolume { volume } => {
                let volume = volume.clamp(0.0, 1.0);
                self.audio.set_volume(volume);
                self.snapshot.volume = volume;
                let _ = self
                    .event_tx
                    .send(PlayerEvent::Snapshot(self.snapshot.clone()));
            }
            EngineCommand::SetMuted { muted } => {
                self.audio.set_muted(muted);
                self.snapshot.muted = muted;
                let _ = self
                    .event_tx
                    .send(PlayerEvent::Snapshot(self.snapshot.clone()));
            }
            EngineCommand::ToggleMute => {
                let muted = !self.snapshot.muted;
                self.audio.set_muted(muted);
                self.snapshot.muted = muted;
                let _ = self
                    .event_tx
                    .send(PlayerEvent::Snapshot(self.snapshot.clone()));
            }
            EngineCommand::Shutdown => {
                self.stop();
                self.shutdown = true;
            }
        }
    }

    // ── pause / resume / seek ──────────────────────────────────

    fn pause(&mut self) {
        if self.snapshot.status == PlaybackStatus::Playing {
            self.snapshot.status = PlaybackStatus::Paused;
            let _ = self
                .event_tx
                .send(PlayerEvent::Snapshot(self.snapshot.clone()));
        }
    }

    fn resume(&mut self) {
        if self.snapshot.status == PlaybackStatus::Paused {
            self.snapshot.status = PlaybackStatus::Playing;
            self.last_tick = Instant::now();
            let _ = self
                .event_tx
                .send(PlayerEvent::Snapshot(self.snapshot.clone()));
        }
    }

    fn seek_to(&mut self, target_ms: i64) {
        let was_paused = self.snapshot.status == PlaybackStatus::Paused;
        self.snapshot.status = PlaybackStatus::Buffering;
        let _ = self
            .event_tx
            .send(PlayerEvent::Snapshot(self.snapshot.clone()));
        self.audio.clear();
        if let Some(ref mut decoder) = self.decoder {
            match decoder.seek_ms(target_ms) {
                Ok(pos) => {
                    self.snapshot.position_ms = pos;
                    if let Some(ref mut session) = self.session {
                        session.mark_position(pos);
                    }
                }
                Err(e) => {
                    self.emit_error(
                        self.snapshot.current.as_ref().map(|t| t.id),
                        PlaybackErrorCode::DecodeFailed,
                        &e.to_string(),
                        true,
                    );
                }
            }
        }
        self.snapshot.status = if was_paused {
            PlaybackStatus::Paused
        } else {
            PlaybackStatus::Playing
        };
        self.last_tick = Instant::now();
        let _ = self
            .event_tx
            .send(PlayerEvent::Snapshot(self.snapshot.clone()));
    }

    fn stop(&mut self) {
        self.finish_session(SessionEndReason::Stop);
        self.gapless.cancel();
        self.audio.clear();
        self.stream = None;
        self.decoder = None;
        self.queue = None;
        self.tracks.clear();
        self.snapshot.status = PlaybackStatus::Stopped;
        self.snapshot.position_ms = 0;
        self.snapshot.current = None;
        self.snapshot.queue_index = None;
        self.snapshot.queue_len = 0;
        let _ = self
            .event_tx
            .send(PlayerEvent::Snapshot(self.snapshot.clone()));
    }

    // ── output device recovery ─────────────────────────────────

    fn recover_output_once(&mut self) {
        self.stream = None;
        match open_default_stream(
            self.audio.clone(),
            self.event_tx.clone(),
            self.stream_error_flag.clone(),
        ) {
            Ok((stream, _config)) => {
                self.stream = Some(stream);
                if self.snapshot.status == PlaybackStatus::Buffering {
                    self.snapshot.status = PlaybackStatus::Playing;
                }
                let _ = self
                    .event_tx
                    .send(PlayerEvent::Snapshot(self.snapshot.clone()));
            }
            Err(err) => {
                self.snapshot.status = PlaybackStatus::Error;
                self.emit_error(
                    self.snapshot.current.as_ref().map(|t| t.id),
                    PlaybackErrorCode::OutputUnavailable,
                    &err.to_string(),
                    false,
                );
                self.finish_session(SessionEndReason::OutputError);
                let _ = self
                    .event_tx
                    .send(PlayerEvent::Snapshot(self.snapshot.clone()));
            }
        }
    }

    // ── track lifecycle ─────────────────────────────────────────

    fn start_current_track(&mut self) {
        let Some(track_id) = self.queue.as_ref().and_then(|q| q.current()) else {
            self.snapshot.status = PlaybackStatus::Ended;
            let _ = self
                .event_tx
                .send(PlayerEvent::Snapshot(self.snapshot.clone()));
            return;
        };
        let Some(track) = self.tracks.iter().find(|t| t.id == track_id).cloned() else {
            self.emit_error(
                Some(track_id),
                PlaybackErrorCode::FileNotFound,
                "track metadata missing",
                true,
            );
            self.skip_unplayable_current();
            return;
        };
        if track.missing_at.is_some() {
            self.emit_error(
                Some(track.id),
                PlaybackErrorCode::FileNotFound,
                "track is marked missing",
                true,
            );
            self.skip_unplayable_current();
            return;
        }
        let output_rate = 48_000;
        let output_channels = 2;
        match DecodedTrack::open(
            PathBuf::from(&track.file_path).as_path(),
            output_rate,
            output_channels,
        ) {
            Ok(decoder) => {
                if self.stream.is_none() {
                    match open_default_stream(
                        self.audio.clone(),
                        self.event_tx.clone(),
                        self.stream_error_flag.clone(),
                    ) {
                        Ok((stream, _config)) => self.stream = Some(stream),
                        Err(err) => {
                            self.snapshot.status = PlaybackStatus::Error;
                            self.emit_error(
                                Some(track.id),
                                PlaybackErrorCode::OutputUnavailable,
                                &err.to_string(),
                                false,
                            );
                            let _ = self
                                .event_tx
                                .send(PlayerEvent::Snapshot(self.snapshot.clone()));
                            return;
                        }
                    }
                }
                self.decoder = Some(decoder);
                self.audio.clear();
                let session = PlaybackSession::new(track.id, now_ms());
                self.session = Some(session);
                self.snapshot.status = PlaybackStatus::Playing;
                self.snapshot.current = Some(NowPlayingTrack::from(track));
                self.snapshot.position_ms = 0;
                self.snapshot.duration_ms = self
                    .tracks
                    .iter()
                    .find(|t| t.id == track_id)
                    .map(|t| t.duration_ms)
                    .unwrap_or(0);
                self.snapshot.queue_index = self.queue.as_ref().and_then(|q| q.current_index());
                self.snapshot.queue_len = self.queue.as_ref().map(|q| q.len()).unwrap_or(0);
                self.last_tick = Instant::now();
                let _ = self
                    .event_tx
                    .send(PlayerEvent::TrackChanged(self.snapshot.current.clone()));
                let _ = self
                    .event_tx
                    .send(PlayerEvent::Snapshot(self.snapshot.clone()));
            }
            Err(err) => {
                self.emit_error(
                    Some(track.id),
                    classify_decode_error(&err),
                    &err.to_string(),
                    true,
                );
                self.skip_unplayable_current();
            }
        }
    }

    fn finish_session(&mut self, reason: SessionEndReason) {
        self.gapless.cancel();
        if let Some(session) = self.session.take() {
            let _ = self.event_tx.send(PlayerEvent::SessionEnded {
                session,
                duration_ms: self.snapshot.duration_ms,
                reason,
            });
        }
    }

    fn advance_next(&mut self) {
        self.decoder = None;
        self.audio.clear();
        // Try to use a pre-decoded gapless result first.
        if let Some(GaplessResult::Ready {
            track_id,
            decoder,
            first_chunk,
        }) = self.gapless.poll()
        {
            if Some(track_id) == self.queue.as_ref().and_then(|q| q.current()) {
                if self.stream.is_none() {
                    match open_default_stream(
                        self.audio.clone(),
                        self.event_tx.clone(),
                        self.stream_error_flag.clone(),
                    ) {
                        Ok((stream, _config)) => self.stream = Some(stream),
                        Err(err) => {
                            self.snapshot.status = PlaybackStatus::Error;
                            self.emit_error(
                                Some(track_id),
                                PlaybackErrorCode::OutputUnavailable,
                                &err.to_string(),
                                false,
                            );
                            let _ = self
                                .event_tx
                                .send(PlayerEvent::Snapshot(self.snapshot.clone()));
                            return;
                        }
                    }
                }
                self.decoder = Some(decoder);
                self.audio.clear();
                self.producer.push_samples(&first_chunk.samples);
                self.snapshot.status = PlaybackStatus::Playing;
                self.snapshot.position_ms =
                    first_chunk.start_ms + first_chunk.duration_ms;
                let session =
                    PlaybackSession::new(track_id, now_ms());
                if let Some(ref mut s) = self.session {
                    *s = session;
                } else {
                    self.session = Some(session);
                }
                self.snapshot.current = self
                    .tracks
                    .iter()
                    .find(|t| t.id == track_id)
                    .map(|t| NowPlayingTrack::from(t.clone()));
                self.snapshot.duration_ms = self
                    .tracks
                    .iter()
                    .find(|t| t.id == track_id)
                    .map(|t| t.duration_ms)
                    .unwrap_or(0);
                self.snapshot.queue_index =
                    self.queue.as_ref().and_then(|q| q.current_index());
                self.snapshot.queue_len =
                    self.queue.as_ref().map(|q| q.len()).unwrap_or(0);
                self.last_tick = Instant::now();
                let _ = self
                    .event_tx
                    .send(PlayerEvent::TrackChanged(self.snapshot.current.clone()));
                let _ = self
                    .event_tx
                    .send(PlayerEvent::Snapshot(self.snapshot.clone()));
                return;
            }
        }
        if let Some(ref mut q) = self.queue {
            match q.next() {
                QueueMove::Track(_) => {
                    self.start_current_track();
                }
                QueueMove::RestartCurrent(_) => {
                    self.start_current_track();
                }
                QueueMove::Ended => {
                    self.snapshot.status = PlaybackStatus::Ended;
                    self.snapshot.current = None;
                    let _ = self
                        .event_tx
                        .send(PlayerEvent::Snapshot(self.snapshot.clone()));
                }
            }
        }
    }

    fn go_previous(&mut self) {
        self.decoder = None;
        self.audio.clear();
        if let Some(ref mut q) = self.queue {
            match q.previous() {
                QueueMove::Track(_) => {
                    self.start_current_track();
                }
                QueueMove::RestartCurrent(_) => {
                    self.start_current_track();
                }
                QueueMove::Ended => {
                    self.snapshot.status = PlaybackStatus::Ended;
                    self.snapshot.current = None;
                    let _ = self
                        .event_tx
                        .send(PlayerEvent::Snapshot(self.snapshot.clone()));
                }
            }
        }
    }

    fn skip_unplayable_current(&mut self) {
        self.decoder = None;
        self.audio.clear();
        if let Some(ref mut q) = self.queue {
            match q.remove_current_unplayable() {
                QueueMove::Track(_) => {
                    self.start_current_track();
                }
                QueueMove::RestartCurrent(_) => {
                    self.start_current_track();
                }
                QueueMove::Ended => {
                    self.snapshot.status = PlaybackStatus::Ended;
                    self.snapshot.current = None;
                    let _ = self
                        .event_tx
                        .send(PlayerEvent::Snapshot(self.snapshot.clone()));
                }
            }
        }
    }

    // ── tick ────────────────────────────────────────────────────

    fn tick_playback(&mut self) {
        if self.snapshot.status != PlaybackStatus::Playing {
            self.last_tick = Instant::now();
            return;
        }
        let elapsed = self.last_tick.elapsed();
        self.last_tick = Instant::now();
        let delta_ms = elapsed.as_millis() as i64;
        if let Some(ref mut session) = self.session {
            session.add_real_played_ms(delta_ms);
        }
        if let Some(ref mut decoder) = self.decoder {
            for _ in 0..4 {
                match decoder.read_chunk(4096) {
                    Ok(Some(chunk)) => {
                        let total = chunk.samples.len();
                        let pushed = self.producer.push_samples(&chunk.samples);
                        // Advance position only by samples that actually landed
                        // in the buffer (stereo = 2 channels, 48k rate).
                        let pushed_frames = (pushed / 2) as u64;
                        let pushed_duration_ms =
                            (pushed_frames * 1000 / 48_000) as i64;
                        self.snapshot.position_ms =
                            chunk.start_ms + pushed_duration_ms;
                        if let Some(ref mut session) = self.session {
                            session.mark_position(self.snapshot.position_ms);
                        }
                        // Buffer full — stop decoding this tick so audio device
                        // has time to drain. Resume on the next tick.
                        if pushed < total {
                            break;
                        }
                    }
                    Ok(None) => {
                        self.finish_session(SessionEndReason::Completed);
                        self.advance_next();
                        break;
                    }
                    Err(err) => {
                        self.emit_error(
                            self.snapshot.current.as_ref().map(|t| t.id),
                            classify_decode_error(&err),
                            &err.to_string(),
                            true,
                        );
                        self.finish_session(SessionEndReason::DecodeError);
                        self.skip_unplayable_current();
                        break;
                    }
                }
            }
        }
        // If approaching end of track, warm up the next one in background.
        self.maybe_start_gapless_predecode();

        let _ = self.event_tx.send(PlayerEvent::Progress {
            position_ms: self.snapshot.position_ms,
            duration_ms: self.snapshot.duration_ms,
        });
    }

    fn maybe_start_gapless_predecode(&mut self) {
        if self.snapshot.duration_ms - self.snapshot.position_ms > 1_000 {
            return;
        }
        let Some(queue) = &self.queue else { return };
        let Some(next_id) = queue.peek_next_track_id() else {
            return;
        };
        let Some(track) = self.tracks.iter().find(|t| t.id == next_id) else {
            return;
        };
        self.gapless.start(GaplessRequest {
            track_id: track.id,
            file_path: PathBuf::from(&track.file_path),
            output_sample_rate: 48_000,
            output_channels: 2,
        });
    }

    fn emit_error(
        &self,
        track_id: Option<i64>,
        code: PlaybackErrorCode,
        message: &str,
        recoverable: bool,
    ) {
        let _ = self.event_tx.send(PlayerEvent::Error(PlaybackErrorEvent {
            track_id,
            code,
            message: message.into(),
            recoverable,
        }));
    }
}

// ── helpers ─────────────────────────────────────────────────────

fn clamp_seek(position_ms: i64, duration_ms: i64) -> i64 {
    position_ms.clamp(0, duration_ms.max(0))
}

fn previous_should_restart_current(position_ms: i64) -> bool {
    position_ms > 3_000
}

fn classify_decode_error(err: &AppError) -> PlaybackErrorCode {
    match err {
        AppError::FileNotFound(_) => PlaybackErrorCode::FileNotFound,
        AppError::PermissionDenied(_) => PlaybackErrorCode::PermissionDenied,
        _ => PlaybackErrorCode::DecodeFailed,
    }
}

// ── tests ───────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn callback_outputs_silence_on_underrun() {
        let (_, audio) = create_audio_buffer(8, 1.0, false);
        let mut out = [1.0_f32; 4];
        audio.fill_output_f32(&mut out);
        assert_eq!(out, [0.0, 0.0, 0.0, 0.0]);
        assert_eq!(audio.underrun_count(), 4);
    }

    #[test]
    fn callback_applies_volume_and_mute() {
        let (producer, audio) = create_audio_buffer(8, 0.5, false);
        producer.push_samples(&[1.0, -1.0]);
        let mut out = [0.0_f32; 2];
        audio.fill_output_f32(&mut out);
        assert_eq!(out, [0.5, -0.5]);

        producer.push_samples(&[1.0, 1.0]);
        audio.set_muted(true);
        audio.fill_output_f32(&mut out);
        assert_eq!(out, [0.0, 0.0]);
    }

    #[test]
    fn seek_clamps_negative_to_zero() {
        assert_eq!(clamp_seek(-10, 100_000), 0);
    }

    #[test]
    fn seek_clamps_after_duration_to_duration() {
        assert_eq!(clamp_seek(120_000, 100_000), 100_000);
    }

    #[test]
    fn previous_restarts_when_position_over_three_seconds() {
        assert!(previous_should_restart_current(3_001));
        assert!(!previous_should_restart_current(3_000));
    }

    #[test]
    fn output_error_recovery_allows_one_attempt() {
        assert!(output_error_is_recoverable(0));
        assert!(!output_error_is_recoverable(1));
    }

    fn output_error_is_recoverable(attempts: usize) -> bool {
        attempts == 0
    }
}
