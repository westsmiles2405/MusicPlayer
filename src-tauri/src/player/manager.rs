use std::sync::Arc;

use crossbeam_channel::{Receiver, Sender};
use parking_lot::Mutex;
use tauri::{AppHandle, Emitter};

use crate::db::{self, Database};
use crate::error::{AppError, AppResult};
use crate::player::engine::AudioEngine;
use crate::player::state::{
    now_ms, EngineCommand, EngineTrack, PlaybackErrorCode, PlaybackErrorEvent, PlaybackSession,
    PlayerCommand, PlayerEvent, PlayerSnapshot, SessionEndReason,
};
use crate::system::now_playing::NowPlaying;

pub struct PlayerManager {
    player_cmd_tx: Sender<PlayerCommand>,
    engine_cmd_tx: Sender<EngineCommand>,
    snapshot: Arc<Mutex<PlayerSnapshot>>,
    worker: Mutex<Option<std::thread::JoinHandle<()>>>,
    db: Database,
}

impl PlayerManager {
    pub fn new(app: AppHandle, db: Database) -> Self {
        let (player_cmd_tx, player_cmd_rx) = crossbeam_channel::unbounded();
        let (engine_cmd_tx, engine_cmd_rx) = crossbeam_channel::unbounded();
        let (event_tx, event_rx) = crossbeam_channel::unbounded();

        let worker = AudioEngine::spawn(engine_cmd_rx, event_tx.clone());
        let snapshot = Arc::new(Mutex::new(PlayerSnapshot::default()));

        let mut now_playing = NowPlaying::new();
        now_playing.set_command_sender(engine_cmd_tx.clone());

        spawn_event_bridge(app, db.clone(), snapshot.clone(), now_playing, event_rx);
        spawn_player_command_handler(db.clone(), player_cmd_rx, engine_cmd_tx.clone(), event_tx);

        Self {
            player_cmd_tx,
            engine_cmd_tx,
            snapshot,
            worker: Mutex::new(Some(worker)),
            db,
        }
    }

    pub fn snapshot(&self) -> PlayerSnapshot {
        self.snapshot.lock().clone()
    }

    /// Send a PlayerCommand (with raw IDs). The command handler thread resolves
    /// track data from DB and forwards EngineCommands to the engine.
    pub fn send(&self, command: PlayerCommand) -> AppResult<()> {
        self.player_cmd_tx
            .send(command)
            .map_err(|_| AppError::Playback("player engine is not running".into()))
    }

    pub fn play_queue(
        &self,
        track_id: i64,
        queue_track_ids: Option<Vec<i64>>,
        queue_index: Option<usize>,
    ) -> AppResult<()> {
        let (queue, index) = normalize_queue_args(track_id, queue_track_ids, queue_index)?;
        // Resolve tracks here on the calling thread (typically command handler)
        let tracks = resolve_tracks(&self.db, &queue)?;
        // Verify the selected track exists
        if tracks.get(index).is_none_or(|t| t.missing_at.is_some()) {
            return Err(AppError::FileNotFound(format!(
                "selected track {track_id} is missing or not found"
            )));
        }
        self.engine_cmd_tx
            .send(EngineCommand::LoadQueueAndPlay { tracks, index })
            .map_err(|_| AppError::Playback("player engine is not running".into()))
    }

    pub fn shutdown(&self) {
        let _ = self.engine_cmd_tx.send(EngineCommand::Shutdown);
        if let Some(worker) = self.worker.lock().take() {
            let _ = worker.join();
        }
    }
}

// ── player command handler thread ───────────────────────────────
//
// Receives PlayerCommands (raw IDs), resolves tracks via DB, and
// forwards EngineCommands to the engine. Runs in a dedicated thread
// so DB access never blocks the audio thread.

fn spawn_player_command_handler(
    db: Database,
    rx: Receiver<PlayerCommand>,
    engine_tx: Sender<EngineCommand>,
    _event_tx: Sender<PlayerEvent>,
) {
    std::thread::spawn(move || {
        while let Ok(cmd) = rx.recv() {
            match cmd {
                PlayerCommand::LoadQueueAndPlayIds { queue, index } => {
                    let tracks = match resolve_tracks(&db, &queue) {
                        Ok(t) => t,
                        Err(e) => {
                            let _ = _event_tx.send(PlayerEvent::Error(PlaybackErrorEvent {
                                track_id: queue.get(index).copied(),
                                code: PlaybackErrorCode::FileNotFound,
                                message: e.to_string(),
                                recoverable: false,
                            }));
                            continue;
                        }
                    };
                    let _ = engine_tx.send(EngineCommand::LoadQueueAndPlay { tracks, index });
                }
                PlayerCommand::Pause => {
                    let _ = engine_tx.send(EngineCommand::Pause);
                }
                PlayerCommand::Resume => {
                    let _ = engine_tx.send(EngineCommand::Resume);
                }
                PlayerCommand::Toggle => {
                    let _ = engine_tx.send(EngineCommand::Toggle);
                }
                PlayerCommand::Stop => {
                    let _ = engine_tx.send(EngineCommand::Stop);
                }
                PlayerCommand::Seek { position_ms } => {
                    let _ = engine_tx.send(EngineCommand::Seek { position_ms });
                }
                PlayerCommand::Next => {
                    let _ = engine_tx.send(EngineCommand::Next);
                }
                PlayerCommand::Previous => {
                    let _ = engine_tx.send(EngineCommand::Previous);
                }
                PlayerCommand::SetVolume { volume } => {
                    let _ = engine_tx.send(EngineCommand::SetVolume { volume });
                }
                PlayerCommand::SetMuted { muted } => {
                    let _ = engine_tx.send(EngineCommand::SetMuted { muted });
                }
                PlayerCommand::ToggleMute => {
                    let _ = engine_tx.send(EngineCommand::ToggleMute);
                }
                PlayerCommand::SetRepeatMode { mode } => {
                    let _ = engine_tx.send(EngineCommand::SetRepeatMode { mode });
                }
                PlayerCommand::SetShuffle { enabled } => {
                    let _ = engine_tx.send(EngineCommand::SetShuffle { enabled });
                }
                PlayerCommand::Shutdown => {
                    let _ = engine_tx.send(EngineCommand::Shutdown);
                    break;
                }
            }
        }
    });
}

fn resolve_tracks(db: &Database, ids: &[i64]) -> AppResult<Vec<EngineTrack>> {
    db.with_conn(|conn| {
        ids.iter()
            .map(|&id| {
                let view = db::tracks::get_view_by_id(conn, id)?
                    .ok_or_else(|| AppError::NotFound(format!("track {id}")))?;
                Ok(EngineTrack {
                    id: view.track.id,
                    file_path: view.track.file_path.clone(),
                    title: view.track.title.clone(),
                    album_name: view.album_name.clone(),
                    artist_name: view.primary_artist_name.clone(),
                    duration_ms: view.track.duration_ms,
                    missing_at: view.track.missing_at,
                })
            })
            .collect()
    })
}

// ── event bridge ────────────────────────────────────────────────

fn spawn_event_bridge(
    app: AppHandle,
    db: Database,
    snapshot: Arc<Mutex<PlayerSnapshot>>,
    mut now_playing: NowPlaying,
    event_rx: Receiver<PlayerEvent>,
) {
    std::thread::spawn(move || {
        while let Ok(event) = event_rx.recv() {
            match event {
                PlayerEvent::Snapshot(next) => {
                    *snapshot.lock() = next.clone();
                    now_playing.update(&next);
                    let _ = app.emit("playback_state", next);
                }
                PlayerEvent::Progress {
                    position_ms,
                    duration_ms,
                } => {
                    {
                        let mut guard = snapshot.lock();
                        guard.position_ms = position_ms;
                        guard.duration_ms = duration_ms;
                    }
                    now_playing.update_progress(position_ms, duration_ms);
                    let _ = app.emit(
                        "playback_progress",
                        serde_json::json!({
                            "positionMs": position_ms,
                            "durationMs": duration_ms
                        }),
                    );
                }
                PlayerEvent::TrackChanged(track) => {
                    let _ = app.emit("track_changed", track);
                }
                PlayerEvent::Error(error) => {
                    let _ = app.emit("playback_error", error);
                }
                PlayerEvent::SessionEnded {
                    mut session,
                    duration_ms,
                    reason,
                } => {
                    flush_session(&db, &mut session, duration_ms, reason);
                }
                PlayerEvent::ShutdownComplete => break,
            }
        }
    });
}

fn flush_session(
    db: &Database,
    session: &mut PlaybackSession,
    duration_ms: i64,
    reason: SessionEndReason,
) {
    if matches!(
        reason,
        SessionEndReason::DecodeError
            | SessionEndReason::FileMissing
            | SessionEndReason::PermissionDenied
            | SessionEndReason::OutputError
    ) {
        return;
    }
    if session.completed_written {
        return;
    }
    let completed = session.qualifies_completed(duration_ms);
    let played_at = now_ms();
    let duration_played = session.real_played_ms.min(duration_ms.max(0));
    let result = db.with_conn(|conn| {
        db::play_history::record(
            conn,
            session.track_id,
            played_at,
            duration_played,
            completed,
        )
    });
    if result.is_ok() && completed {
        session.completed_written = true;
    }
}

// ── queue arg normalization ─────────────────────────────────────

pub fn normalize_queue_args(
    track_id: i64,
    queue_track_ids: Option<Vec<i64>>,
    queue_index: Option<usize>,
) -> AppResult<(Vec<i64>, usize)> {
    let queue = match queue_track_ids {
        Some(ids) if !ids.is_empty() => ids,
        _ => vec![track_id],
    };
    let index =
        queue_index.unwrap_or_else(|| queue.iter().position(|id| *id == track_id).unwrap_or(0));
    if index >= queue.len() {
        return Err(AppError::InvalidInput(format!(
            "queueIndex {index} out of range for queue length {}",
            queue.len()
        )));
    }
    if queue[index] != track_id {
        return Err(AppError::InvalidInput(format!(
            "trackId {track_id} does not match queueTrackIds[{index}] {}",
            queue[index]
        )));
    }
    Ok((queue, index))
}

// ── tests ───────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::testing::{make_basic_track, test_db};

    #[test]
    fn queue_args_default_to_single_track() {
        let (queue, index) = normalize_queue_args(5, None, None).unwrap();
        assert_eq!(queue, vec![5]);
        assert_eq!(index, 0);
    }

    #[test]
    fn queue_args_reject_index_mismatch() {
        let err = normalize_queue_args(5, Some(vec![4, 5]), Some(0)).unwrap_err();
        assert!(matches!(err, AppError::InvalidInput(_)));
    }

    #[test]
    fn queue_args_reject_out_of_range() {
        let err = normalize_queue_args(5, Some(vec![5]), Some(1)).unwrap_err();
        assert!(matches!(err, AppError::InvalidInput(_)));
    }

    #[test]
    fn flush_session_writes_completed_once() {
        let conn = test_db();
        let track_id = make_basic_track(&conn, "Song");
        let db = Database::from_conn(conn);
        let mut session = PlaybackSession::new(track_id, 100);
        session.mark_position(95_000);
        session.add_real_played_ms(30_000);
        flush_session(&db, &mut session, 100_000, SessionEndReason::Next);
        assert!(session.completed_written);
        flush_session(&db, &mut session, 100_000, SessionEndReason::Next);
        let count: i64 = db
            .with_conn(|conn| {
                conn.query_row(
                    "SELECT play_count FROM tracks WHERE id=?1",
                    rusqlite::params![track_id],
                    |r| r.get(0),
                )
            })
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn decode_error_does_not_write_history() {
        let conn = test_db();
        let track_id = make_basic_track(&conn, "Song");
        let db = Database::from_conn(conn);
        let mut session = PlaybackSession::new(track_id, 100);
        session.mark_position(95_000);
        session.add_real_played_ms(30_000);
        flush_session(&db, &mut session, 100_000, SessionEndReason::DecodeError);
        let rows: i64 = db
            .with_conn(|conn| {
                conn.query_row(
                    "SELECT COUNT(*) FROM play_history WHERE track_id=?1",
                    rusqlite::params![track_id],
                    |r| r.get(0),
                )
            })
            .unwrap();
        assert_eq!(rows, 0);
    }
}
