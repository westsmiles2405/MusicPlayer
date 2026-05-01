use std::{fs::File, path::Path};

use symphonia::core::{
    audio::{AudioBufferRef, Signal},
    codecs::{Decoder, DecoderOptions},
    errors::Error as SymphoniaError,
    formats::{FormatOptions, FormatReader, SeekMode, SeekTo},
    io::MediaSourceStream,
    meta::MetadataOptions,
    probe::Hint,
    units::Time,
};

use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, PartialEq)]
pub struct PcmChunk {
    pub samples: Vec<f32>,
    pub frames: usize,
    pub channels: u16,
    pub sample_rate: u32,
    pub start_ms: i64,
    pub duration_ms: i64,
}

pub struct DecodedTrack {
    format: Box<dyn FormatReader>,
    decoder: Box<dyn Decoder>,
    track_id: u32,
    source_sample_rate: u32,
    source_channels: u16,
    output_sample_rate: u32,
    output_channels: u16,
    next_start_ms: i64,
}

impl DecodedTrack {
    pub fn open(path: &Path, output_sample_rate: u32, output_channels: u16) -> AppResult<Self> {
        let file = File::open(path).map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => AppError::FileNotFound(path.display().to_string()),
            std::io::ErrorKind::PermissionDenied => {
                AppError::PermissionDenied(path.display().to_string())
            }
            _ => AppError::Playback(e.to_string()),
        })?;
        let mss = MediaSourceStream::new(Box::new(file), Default::default());
        let mut hint = Hint::new();
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            hint.with_extension(ext);
        }
        let probed = symphonia::default::get_probe()
            .format(
                &hint,
                mss,
                &FormatOptions::default(),
                &MetadataOptions::default(),
            )
            .map_err(|e| AppError::Playback(format!("decode probe failed: {e}")))?;
        let format = probed.format;
        let track = format
            .tracks()
            .iter()
            .find(|t| t.codec_params.codec != symphonia::core::codecs::CODEC_TYPE_NULL)
            .ok_or_else(|| AppError::Playback("no supported audio track".into()))?;
        let track_id = track.id;
        let codec_params = &track.codec_params;
        let source_sample_rate = codec_params.sample_rate.unwrap_or(output_sample_rate);
        let source_channels = codec_params
            .channels
            .map(|c| c.count() as u16)
            .unwrap_or(output_channels);
        if source_channels == 0 || source_channels > 2 {
            return Err(AppError::Playback(format!(
                "unsupported channel count {source_channels}"
            )));
        }
        let decoder = symphonia::default::get_codecs()
            .make(codec_params, &DecoderOptions::default())
            .map_err(|e| AppError::Playback(format!("decoder init failed: {e}")))?;
        Ok(Self {
            format,
            decoder,
            track_id,
            source_sample_rate,
            source_channels,
            output_sample_rate,
            output_channels,
            next_start_ms: 0,
        })
    }

    pub fn read_chunk(&mut self, max_frames: usize) -> AppResult<Option<PcmChunk>> {
        loop {
            let packet = match self.format.next_packet() {
                Ok(packet) => packet,
                Err(SymphoniaError::IoError(_)) => return Ok(None),
                Err(e) => {
                    return Err(AppError::Playback(format!("packet read failed: {e}")))
                }
            };
            if packet.track_id() != self.track_id {
                continue;
            }
            let decoded = match self.decoder.decode(&packet) {
                Ok(decoded) => decoded,
                Err(SymphoniaError::DecodeError(_)) => continue,
                Err(e) => return Err(AppError::Playback(format!("decode failed: {e}"))),
            };
            let source = audio_ref_to_interleaved_f32(decoded)?;
            let converted = convert_channels(&source, self.source_channels, self.output_channels)?;
            let resampled = linear_resample(
                &converted,
                self.source_sample_rate,
                self.output_sample_rate,
                self.output_channels,
            );
            let frames = (resampled.len() / self.output_channels as usize).min(max_frames);
            let sample_len = frames * self.output_channels as usize;
            let samples = resampled[..sample_len].to_vec();
            let duration_ms = frames_to_ms(frames, self.output_sample_rate);
            let chunk = PcmChunk {
                samples,
                frames,
                channels: self.output_channels,
                sample_rate: self.output_sample_rate,
                start_ms: self.next_start_ms,
                duration_ms,
            };
            self.next_start_ms += duration_ms;
            return Ok(Some(chunk));
        }
    }

    pub fn seek_ms(&mut self, position_ms: i64) -> AppResult<i64> {
        let seconds = position_ms.max(0) as f64 / 1000.0;
        self.format
            .seek(
                SeekMode::Accurate,
                SeekTo::Time {
                    time: Time::from(seconds),
                    track_id: Some(self.track_id),
                },
            )
            .map_err(|e| AppError::Playback(format!("seek failed: {e}")))?;
        self.decoder.reset();
        self.next_start_ms = position_ms.max(0);
        Ok(self.next_start_ms)
    }
}

fn audio_ref_to_interleaved_f32(buffer: AudioBufferRef<'_>) -> AppResult<Vec<f32>> {
    match &buffer {
        AudioBufferRef::F32(buf) => {
            let spec = *buf.spec();
            let channels = spec.channels.count();
            let frames = buf.frames();
            let mut out = Vec::with_capacity(frames * channels);
            for f in 0..frames {
                for ch in 0..channels {
                    out.push(buf.chan(ch)[f]);
                }
            }
            Ok(out)
        }
        AudioBufferRef::U8(buf) => {
            let spec = *buf.spec();
            let channels = spec.channels.count();
            let frames = buf.frames();
            let mut out = Vec::with_capacity(frames * channels);
            for f in 0..frames {
                for ch in 0..channels {
                    out.push(u8_to_f32(buf.chan(ch)[f]));
                }
            }
            Ok(out)
        }
        AudioBufferRef::U16(buf) => {
            let spec = *buf.spec();
            let channels = spec.channels.count();
            let frames = buf.frames();
            let mut out = Vec::with_capacity(frames * channels);
            for f in 0..frames {
                for ch in 0..channels {
                    out.push(u16_to_f32(buf.chan(ch)[f]));
                }
            }
            Ok(out)
        }
        AudioBufferRef::U24(buf) => {
            let spec = *buf.spec();
            let channels = spec.channels.count();
            let frames = buf.frames();
            let mut out = Vec::with_capacity(frames * channels);
            for f in 0..frames {
                for ch in 0..channels {
                    out.push(u24_to_f32(buf.chan(ch)[f].inner()));
                }
            }
            Ok(out)
        }
        AudioBufferRef::U32(buf) => {
            let spec = *buf.spec();
            let channels = spec.channels.count();
            let frames = buf.frames();
            let mut out = Vec::with_capacity(frames * channels);
            for f in 0..frames {
                for ch in 0..channels {
                    out.push(u32_to_f32(buf.chan(ch)[f]));
                }
            }
            Ok(out)
        }
        AudioBufferRef::S8(buf) => {
            let spec = *buf.spec();
            let channels = spec.channels.count();
            let frames = buf.frames();
            let mut out = Vec::with_capacity(frames * channels);
            for f in 0..frames {
                for ch in 0..channels {
                    out.push(s8_to_f32(buf.chan(ch)[f]));
                }
            }
            Ok(out)
        }
        AudioBufferRef::S16(buf) => {
            let spec = *buf.spec();
            let channels = spec.channels.count();
            let frames = buf.frames();
            let mut out = Vec::with_capacity(frames * channels);
            for f in 0..frames {
                for ch in 0..channels {
                    out.push(s16_to_f32(buf.chan(ch)[f]));
                }
            }
            Ok(out)
        }
        AudioBufferRef::S24(buf) => {
            let spec = *buf.spec();
            let channels = spec.channels.count();
            let frames = buf.frames();
            let mut out = Vec::with_capacity(frames * channels);
            for f in 0..frames {
                for ch in 0..channels {
                    out.push(s24_to_f32(buf.chan(ch)[f].inner()));
                }
            }
            Ok(out)
        }
        AudioBufferRef::S32(buf) => {
            let spec = *buf.spec();
            let channels = spec.channels.count();
            let frames = buf.frames();
            let mut out = Vec::with_capacity(frames * channels);
            for f in 0..frames {
                for ch in 0..channels {
                    out.push(s32_to_f32(buf.chan(ch)[f]));
                }
            }
            Ok(out)
        }
        AudioBufferRef::F64(buf) => {
            let spec = *buf.spec();
            let channels = spec.channels.count();
            let frames = buf.frames();
            let mut out = Vec::with_capacity(frames * channels);
            for f in 0..frames {
                for ch in 0..channels {
                    out.push(buf.chan(ch)[f] as f32);
                }
            }
            Ok(out)
        }
    }
}

fn u8_to_f32(s: u8) -> f32 {
    (s as f32 / u8::MAX as f32) * 2.0 - 1.0
}
fn u16_to_f32(s: u16) -> f32 {
    (s as f32 / u16::MAX as f32) * 2.0 - 1.0
}
fn u24_to_f32(s: u32) -> f32 {
    s as f32 / 8_388_607.0 * 2.0 - 1.0
}
fn u32_to_f32(s: u32) -> f32 {
    (s as f64 / u32::MAX as f64) as f32 * 2.0 - 1.0
}
fn s8_to_f32(s: i8) -> f32 {
    s as f32 / i8::MAX as f32
}
fn s16_to_f32(s: i16) -> f32 {
    s as f32 / i16::MAX as f32
}
fn s24_to_f32(s: i32) -> f32 {
    (s as f64 / 8_388_607.0_f64) as f32
}
fn s32_to_f32(s: i32) -> f32 {
    (s as f64 / i32::MAX as f64) as f32
}

pub fn convert_channels(
    samples: &[f32],
    source_channels: u16,
    output_channels: u16,
) -> AppResult<Vec<f32>> {
    match (source_channels, output_channels) {
        (1, 1) | (2, 2) => Ok(samples.to_vec()),
        (1, 2) => Ok(samples.iter().flat_map(|s| [*s, *s]).collect()),
        (2, 1) => Ok(samples
            .chunks_exact(2)
            .map(|f| (f[0] + f[1]) * 0.5)
            .collect()),
        _ => Err(AppError::Playback(format!(
            "unsupported channel conversion {source_channels} to {output_channels}"
        ))),
    }
}

pub fn linear_resample(
    samples: &[f32],
    source_rate: u32,
    output_rate: u32,
    channels: u16,
) -> Vec<f32> {
    if source_rate == output_rate || samples.is_empty() {
        return samples.to_vec();
    }
    let channels = channels as usize;
    let in_frames = samples.len() / channels;
    let out_frames =
        ((in_frames as u64 * output_rate as u64) / source_rate as u64).max(1) as usize;
    let mut out = vec![0.0; out_frames * channels];
    for out_frame in 0..out_frames {
        let src_pos = out_frame as f64 * source_rate as f64 / output_rate as f64;
        let left = src_pos.floor() as usize;
        let right = (left + 1).min(in_frames - 1);
        let frac = (src_pos - left as f64) as f32;
        for ch in 0..channels {
            let a = samples[left * channels + ch];
            let b = samples[right * channels + ch];
            out[out_frame * channels + ch] = a + (b - a) * frac;
        }
    }
    out
}

fn frames_to_ms(frames: usize, sample_rate: u32) -> i64 {
    ((frames as u64 * 1000) / sample_rate as u64) as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn fixture(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/audio")
            .join(name)
    }

    #[test]
    fn mono_to_stereo_duplicates_samples() {
        let out = convert_channels(&[0.25, -0.5], 1, 2).unwrap();
        assert_eq!(out, vec![0.25, 0.25, -0.5, -0.5]);
    }

    #[test]
    fn stereo_to_stereo_preserves_samples() {
        let out = convert_channels(&[0.1, 0.2, 0.3, 0.4], 2, 2).unwrap();
        assert_eq!(out, vec![0.1, 0.2, 0.3, 0.4]);
    }

    #[test]
    fn resamples_44100_to_48000() {
        let input = vec![0.0; 44_100 * 2];
        let output = linear_resample(&input, 44_100, 48_000, 2);
        assert_eq!(output.len(), 48_000 * 2);
    }

    #[test]
    fn decodes_mp3_fixture_to_f32_pcm() {
        let mut decoded = DecodedTrack::open(&fixture("a.mp3"), 48_000, 2).unwrap();
        let chunk = decoded.read_chunk(4096).unwrap().unwrap();
        assert_eq!(chunk.channels, 2);
        assert_eq!(chunk.sample_rate, 48_000);
        assert!(chunk.frames > 0);
        assert_eq!(chunk.samples.len(), chunk.frames * 2);
    }

    #[test]
    fn decodes_wav_fixture_to_f32_pcm() {
        let mut decoded = DecodedTrack::open(&fixture("d.wav"), 48_000, 2).unwrap();
        let chunk = decoded.read_chunk(4096).unwrap().unwrap();
        assert!(chunk.frames > 0);
    }
}
