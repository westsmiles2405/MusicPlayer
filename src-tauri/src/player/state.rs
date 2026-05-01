//! Player state types: status, commands (dual-layer), events, snapshot, session.
//!
//! Commands split into two layers:
//!   PlayerCommand — manager layer, carries raw track IDs
//!   EngineCommand — engine layer, carries resolved EngineTrack payloads
//! The manager resolves IDs via DB and translates between the two.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── status & modes ──────────────────────────────────────────────

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum PlaybackStatus {
    Idle,
    Loading,
    Buffering,
    Playing,
    Paused,
    Stopped,
    Ended,
    Error,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum RepeatMode {
    Off,
    One,
    All,
}

// ── track payloads ──────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NowPlayingTrack {
    pub id: i64,
    pub title: String,
    pub album_name: Option<String>,
    pub artist_name: Option<String>,
    pub duration_ms: i64,
    pub cover_path: Option<String>,
}

/// Resolved track used inside the engine. Never carries IDs alone.
#[derive(Debug, Clone, PartialEq)]
pub struct EngineTrack {
    pub id: i64,
    pub file_path: String,
    pub title: String,
    pub album_name: Option<String>,
    pub artist_name: Option<String>,
    pub duration_ms: i64,
    pub missing_at: Option<i64>,
}

impl From<EngineTrack> for NowPlayingTrack {
    fn from(t: EngineTrack) -> Self {
        Self {
            id: t.id,
            title: t.title,
            album_name: t.album_name,
            artist_name: t.artist_name,
            duration_ms: t.duration_ms,
            cover_path: None,
        }
    }
}

// ── snapshot ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PlayerSnapshot {
    pub status: PlaybackStatus,
    pub current: Option<NowPlayingTrack>,
    pub position_ms: i64,
    pub duration_ms: i64,
    pub volume: f32,
    pub muted: bool,
    pub queue_index: Option<usize>,
    pub queue_len: usize,
    pub repeat_mode: RepeatMode,
    pub shuffle: bool,
}

impl Default for PlayerSnapshot {
    fn default() -> Self {
        Self {
            status: PlaybackStatus::Idle,
            current: None,
            position_ms: 0,
            duration_ms: 0,
            volume: 0.8,
            muted: false,
            queue_index: None,
            queue_len: 0,
            repeat_mode: RepeatMode::Off,
            shuffle: false,
        }
    }
}

// ── error types ─────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum PlaybackErrorCode {
    FileNotFound,
    PermissionDenied,
    DecodeFailed,
    OutputUnavailable,
    StreamError,
    InvalidInput,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PlaybackErrorEvent {
    pub track_id: Option<i64>,
    pub code: PlaybackErrorCode,
    pub message: String,
    pub recoverable: bool,
}

// ── session ─────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlaybackSession {
    pub id: String,
    pub track_id: i64,
    pub started_at_ms: i64,
    pub max_position_ms: i64,
    pub real_played_ms: i64,
    pub completed_written: bool,
}

impl PlaybackSession {
    pub fn new(track_id: i64, started_at_ms: i64) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            track_id,
            started_at_ms,
            max_position_ms: 0,
            real_played_ms: 0,
            completed_written: false,
        }
    }

    pub fn mark_position(&mut self, position_ms: i64) {
        self.max_position_ms = self.max_position_ms.max(position_ms.max(0));
    }

    pub fn add_real_played_ms(&mut self, delta_ms: i64) {
        self.real_played_ms += delta_ms.max(0);
    }

    /// A session "qualifies" as completed when:
    /// - not already written
    /// - position reached >= 95% of duration
    /// - at least 30 s of real-time playback (or half the track for short tracks)
    pub fn qualifies_completed(&self, duration_ms: i64) -> bool {
        if self.completed_written || duration_ms <= 0 {
            return false;
        }
        let reached_end =
            self.max_position_ms.saturating_mul(100) >= duration_ms.saturating_mul(95);
        let required_real = 30_000_i64.min(duration_ms / 2);
        reached_end && self.real_played_ms >= required_real
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum SessionEndReason {
    Completed,
    Stop,
    Next,
    Previous,
    Replaced,
    Shutdown,
    DecodeError,
    FileMissing,
    PermissionDenied,
    OutputError,
}

// ── dual-layer commands ─────────────────────────────────────────
// Manager receives PlayerCommand (raw IDs), resolves tracks,
// translates to EngineCommand for the engine.

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum PlayerCommand {
    LoadQueueAndPlayIds { queue: Vec<i64>, index: usize },
    Pause,
    Resume,
    Toggle,
    Stop,
    Seek { position_ms: i64 },
    Next,
    Previous,
    SetVolume { volume: f32 },
    SetMuted { muted: bool },
    ToggleMute,
    SetRepeatMode { mode: RepeatMode },
    SetShuffle { enabled: bool },
    Shutdown,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EngineCommand {
    LoadQueueAndPlay {
        tracks: Vec<EngineTrack>,
        index: usize,
    },
    Pause,
    Resume,
    Toggle,
    Stop,
    Seek {
        position_ms: i64,
    },
    Next,
    Previous,
    SetVolume {
        volume: f32,
    },
    SetMuted {
        muted: bool,
    },
    ToggleMute,
    Shutdown,
}

// ── events ──────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum PlayerEvent {
    Snapshot(PlayerSnapshot),
    Progress {
        position_ms: i64,
        duration_ms: i64,
    },
    TrackChanged(Option<NowPlayingTrack>),
    Error(PlaybackErrorEvent),
    SessionEnded {
        session: PlaybackSession,
        duration_ms: i64,
        reason: SessionEndReason,
    },
    ShutdownComplete,
}

// ── helpers ─────────────────────────────────────────────────────

pub fn now_ms() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

// ── tests ───────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_serializes_camel_case() {
        let snapshot = PlayerSnapshot {
            status: PlaybackStatus::Playing,
            current: Some(NowPlayingTrack {
                id: 7,
                title: "Song".into(),
                album_name: Some("Album".into()),
                artist_name: Some("Artist".into()),
                duration_ms: 120_000,
                cover_path: None,
            }),
            position_ms: 1_000,
            duration_ms: 120_000,
            volume: 0.8,
            muted: false,
            queue_index: Some(0),
            queue_len: 1,
            repeat_mode: RepeatMode::Off,
            shuffle: false,
        };
        let json = serde_json::to_string(&snapshot).unwrap();
        assert!(json.contains(r#""positionMs":1000"#));
        assert!(json.contains(r#""artistName":"Artist""#));
        assert!(json.contains(r#""repeatMode":"off""#));
    }

    #[test]
    fn seek_to_end_without_real_play_time_does_not_complete() {
        let mut session = PlaybackSession::new(1, 100);
        session.mark_position(95_000);
        assert!(!session.qualifies_completed(100_000));
    }

    #[test]
    fn completed_requires_position_and_real_play_time() {
        let mut session = PlaybackSession::new(1, 100);
        session.mark_position(95_000);
        session.add_real_played_ms(30_000);
        assert!(session.qualifies_completed(100_000));
        session.completed_written = true;
        assert!(!session.qualifies_completed(100_000));
    }

    #[test]
    fn short_track_completion_uses_half_duration() {
        let mut session = PlaybackSession::new(1, 100);
        session.mark_position(9_500);
        session.add_real_played_ms(4_999);
        assert!(!session.qualifies_completed(10_000));
        session.add_real_played_ms(1);
        assert!(session.qualifies_completed(10_000));
    }

    #[test]
    fn engine_track_converts_to_now_playing() {
        let et = EngineTrack {
            id: 3,
            file_path: "/music/x.mp3".into(),
            title: "X".into(),
            album_name: Some("Album".into()),
            artist_name: Some("Artist".into()),
            duration_ms: 50_000,
            missing_at: None,
        };
        let np: NowPlayingTrack = et.into();
        assert_eq!(np.id, 3);
        assert_eq!(np.title, "X");
        assert_eq!(np.duration_ms, 50_000);
    }
}
