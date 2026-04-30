//! Track queries (read + write).

use rusqlite::{params, Connection, OptionalExtension, Row};
use serde::{Deserialize, Serialize};

use crate::error::AppResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Track {
    pub id: i64,
    pub file_path: String,
    pub file_size: i64,
    pub file_modified_at: i64,
    pub hash: Option<String>,
    pub title: String,
    pub album_id: Option<i64>,
    pub primary_artist_id: Option<i64>,
    pub album_artist_id: Option<i64>,
    pub track_no: Option<i32>,
    pub disc_no: Option<i32>,
    pub year: Option<i32>,
    pub genre: Option<String>,
    pub duration_ms: i64,
    pub bitrate: Option<i32>,
    pub sample_rate: Option<i32>,
    pub channels: Option<i32>,
    pub codec: Option<String>,
    pub is_favorite: bool,
    pub play_count: i64,
    pub last_played_at: Option<i64>,
    pub last_seen_at: i64,
    pub missing_at: Option<i64>,
    pub added_at: i64,
    pub updated_at: i64,
}

/// 列表/搜索用的反规范化视图：附带专辑名 / 主艺人名，省一次 round-trip。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackView {
    #[serde(flatten)]
    pub track: Track,
    pub album_name: Option<String>,
    pub primary_artist_name: Option<String>,
}

/// 写入用：尚未拿到 id 的新 track。`scanner` 在 Task 2.5 用到。
#[derive(Debug, Clone)]
pub struct NewTrack {
    pub file_path: String,
    pub file_size: i64,
    pub file_modified_at: i64,
    pub hash: Option<String>,
    pub title: String,
    pub album_id: Option<i64>,
    pub primary_artist_id: Option<i64>,
    pub album_artist_id: Option<i64>,
    pub track_no: Option<i32>,
    pub disc_no: Option<i32>,
    pub year: Option<i32>,
    pub genre: Option<String>,
    pub duration_ms: i64,
    pub bitrate: Option<i32>,
    pub sample_rate: Option<i32>,
    pub channels: Option<i32>,
    pub codec: Option<String>,
}

impl Track {
    fn from_row(row: &Row<'_>) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            file_path: row.get("file_path")?,
            file_size: row.get("file_size")?,
            file_modified_at: row.get("file_modified_at")?,
            hash: row.get("hash")?,
            title: row.get("title")?,
            album_id: row.get("album_id")?,
            primary_artist_id: row.get("primary_artist_id")?,
            album_artist_id: row.get("album_artist_id")?,
            track_no: row.get("track_no")?,
            disc_no: row.get("disc_no")?,
            year: row.get("year")?,
            genre: row.get("genre")?,
            duration_ms: row.get("duration_ms")?,
            bitrate: row.get("bitrate")?,
            sample_rate: row.get("sample_rate")?,
            channels: row.get("channels")?,
            codec: row.get("codec")?,
            is_favorite: row.get::<_, i64>("is_favorite")? != 0,
            play_count: row.get("play_count")?,
            last_played_at: row.get("last_played_at")?,
            last_seen_at: row.get("last_seen_at")?,
            missing_at: row.get("missing_at")?,
            added_at: row.get("added_at")?,
            updated_at: row.get("updated_at")?,
        })
    }
}

#[allow(dead_code)]
fn _placeholder(_: &Connection) -> AppResult<()> {
    let _ = (Track::from_row, params!(), NewTrack { file_path: String::new(), file_size: 0, file_modified_at: 0, hash: None, title: String::new(), album_id: None, primary_artist_id: None, album_artist_id: None, track_no: None, disc_no: None, year: None, genre: None, duration_ms: 0, bitrate: None, sample_rate: None, channels: None, codec: None });
    Ok(())
}
