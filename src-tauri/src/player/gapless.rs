//! Gapless predecode: opens and reads the first chunk of the next track
//! while the current track is still playing, so transition has zero gap.

use std::path::PathBuf;
use std::thread::JoinHandle;

use crossbeam_channel::{Receiver, Sender};

use crate::player::decoder::{DecodedTrack, PcmChunk};

#[derive(Debug, Clone)]
pub struct GaplessRequest {
    pub track_id: i64,
    pub file_path: PathBuf,
    pub output_sample_rate: u32,
    pub output_channels: u16,
}

pub enum GaplessResult {
    Ready {
        track_id: i64,
        decoder: DecodedTrack,
        first_chunk: PcmChunk,
    },
    Failed {
        track_id: i64,
        message: String,
    },
}

pub struct GaplessPredecoder {
    current_track_id: Option<i64>,
    worker: Option<JoinHandle<()>>,
    result_rx: Receiver<GaplessResult>,
    result_tx: Sender<GaplessResult>,
}

impl GaplessPredecoder {
    pub fn new() -> Self {
        let (result_tx, result_rx) = crossbeam_channel::bounded(1);
        Self {
            current_track_id: None,
            worker: None,
            result_rx,
            result_tx,
        }
    }

    pub fn start(&mut self, request: GaplessRequest) {
        if self.current_track_id == Some(request.track_id) {
            return;
        }
        self.cancel();
        self.current_track_id = Some(request.track_id);
        let tx = self.result_tx.clone();
        self.worker = Some(std::thread::spawn(move || {
            let result = decode_first_chunk(request);
            let _ = tx.send(result);
        }));
    }

    pub fn poll(&mut self) -> Option<GaplessResult> {
        self.result_rx.try_recv().ok()
    }

    pub fn cancel(&mut self) {
        self.current_track_id = None;
        self.worker.take();
        while self.result_rx.try_recv().is_ok() {}
    }
}

fn decode_first_chunk(request: GaplessRequest) -> GaplessResult {
    match DecodedTrack::open(
        &request.file_path,
        request.output_sample_rate,
        request.output_channels,
    )
    .and_then(|mut decoder| {
        let chunk = decoder
            .read_chunk(4096)?
            .ok_or_else(|| crate::error::AppError::Playback("empty gapless track".into()))?;
        Ok((decoder, chunk))
    }) {
        Ok((decoder, first_chunk)) => GaplessResult::Ready {
            track_id: request.track_id,
            decoder,
            first_chunk,
        },
        Err(err) => GaplessResult::Failed {
            track_id: request.track_id,
            message: err.to_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gapless_predecode_reports_failure_for_missing_file() {
        let req = GaplessRequest {
            track_id: 1,
            file_path: PathBuf::from("/tmp/musicplayer-missing-file.mp3"),
            output_sample_rate: 48_000,
            output_channels: 2,
        };
        let result = decode_first_chunk(req);
        assert!(matches!(result, GaplessResult::Failed { track_id: 1, .. }));
    }
}
