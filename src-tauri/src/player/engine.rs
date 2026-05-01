//! Audio engine: ringbuf-backed shared buffer + cpal output stream.
//! PCM decoded in engine thread, pushed to ringbuf Producer.
//! cpal callback reads from ringbuf Consumer via try_lock — never blocks.

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
use crate::player::state::{
    EngineCommand, PlaybackErrorCode, PlaybackErrorEvent, PlaybackStatus, PlayerEvent,
    PlayerSnapshot,
};

// ── SharedAudioBuffer ───────────────────────────────────────────
//
// Producer side: engine thread pushes decoded PCM.
// Consumer side: cpal callback pops via try_lock() — if lock is
// contended (buffer being cleared) the callback writes silence.

pub struct SharedAudioBuffer {
    consumer: Mutex<HeapCons<f32>>,
    volume_bits: AtomicU32,
    muted: AtomicBool,
    underruns: AtomicU64,
}

impl SharedAudioBuffer {
    pub fn new(capacity_samples: usize, volume: f32, muted: bool) -> Self {
        let rb = HeapRb::<f32>::new(capacity_samples);
        let (_, consumer) = rb.split();
        Self {
            consumer: Mutex::new(consumer),
            volume_bits: AtomicU32::new(volume.clamp(0.0, 1.0).to_bits()),
            muted: AtomicBool::new(muted),
            underruns: AtomicU64::new(0),
        }
    }

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
    pub fn push_samples(&self, samples: &[f32]) {
        let mut prod = self.producer.lock();
        for &s in samples {
            let _ = (*prod).try_push(s);
        }
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

// ── AudioEngine skeleton ────────────────────────────────────────

pub struct AudioEngine {
    command_rx: Receiver<EngineCommand>,
    event_tx: Sender<PlayerEvent>,
    producer: AudioProducer,
    audio: Arc<SharedAudioBuffer>,
    stream: Option<Stream>,
    snapshot: PlayerSnapshot,
    shutdown: bool,
    // playback fields added in Task 7
    queue: Option<crate::player::queue::PlayQueue>,
    tracks: Vec<crate::player::state::EngineTrack>,
    decoder: Option<crate::player::decoder::DecodedTrack>,
    session: Option<crate::player::state::PlaybackSession>,
    last_tick: Instant,
}

impl AudioEngine {
    pub fn spawn(
        command_rx: Receiver<EngineCommand>,
        event_tx: Sender<PlayerEvent>,
    ) -> thread::JoinHandle<()> {
        thread::spawn(move || {
            let (producer, audio) = create_audio_buffer(48_000 * 2 * 5, 0.8, false);
            let mut engine = Self {
                command_rx,
                event_tx,
                producer,
                audio,
                stream: None,
                snapshot: PlayerSnapshot::default(),
                shutdown: false,
                queue: None,
                tracks: Vec::new(),
                decoder: None,
                session: None,
                last_tick: Instant::now(),
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
        }
        let _ = self.event_tx.send(PlayerEvent::ShutdownComplete);
    }

    fn handle_command(&mut self, command: EngineCommand) {
        match command {
            EngineCommand::SetVolume { volume } => {
                let volume = volume.clamp(0.0, 1.0);
                self.audio.set_volume(volume);
                self.snapshot.volume = volume;
                let _ = self.event_tx.send(PlayerEvent::Snapshot(self.snapshot.clone()));
            }
            EngineCommand::SetMuted { muted } => {
                self.audio.set_muted(muted);
                self.snapshot.muted = muted;
                let _ = self.event_tx.send(PlayerEvent::Snapshot(self.snapshot.clone()));
            }
            EngineCommand::ToggleMute => {
                let muted = !self.snapshot.muted;
                self.audio.set_muted(muted);
                self.snapshot.muted = muted;
                let _ = self.event_tx.send(PlayerEvent::Snapshot(self.snapshot.clone()));
            }
            EngineCommand::Stop => self.stop(),
            EngineCommand::Shutdown => {
                self.stop();
                self.shutdown = true;
            }
            _ => {} // LoadQueueAndPlay, Pause, Resume, Toggle, Seek, Next, Previous handled in Task 7
        }
    }

    fn stop(&mut self) {
        self.audio.clear();
        self.stream = None;
        self.decoder = None;
        self.snapshot.status = PlaybackStatus::Stopped;
        self.snapshot.position_ms = 0;
        let _ = self.event_tx.send(PlayerEvent::Snapshot(self.snapshot.clone()));
    }

    /// Stub — full implementation in Task 7.
    fn tick_playback(&mut self) {
        // Placeholder: push silence/progress when active
    }

    fn emit_error(&self, track_id: Option<i64>, code: PlaybackErrorCode, message: &str, recoverable: bool) {
        let _ = self.event_tx.send(PlayerEvent::Error(PlaybackErrorEvent {
            track_id,
            code,
            message: message.into(),
            recoverable,
        }));
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
}
