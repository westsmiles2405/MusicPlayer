//! Library IPC commands. Each command takes State<Database> and delegates to db::*.

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::db::{albums, artists, play_history, playlists, search, tracks, Database};
use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListParams {
    #[serde(default = "default_sort")]
    pub sort: tracks::TrackSort,
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_sort() -> tracks::TrackSort { tracks::TrackSort::Title }
fn default_limit() -> i64 { 500 }

fn now_ms() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as i64).unwrap_or(0)
}

#[tauri::command]
pub async fn get_tracks(db: State<'_, Database>, params: Option<ListParams>) -> AppResult<Vec<tracks::TrackView>> {
    let p = params.unwrap_or(ListParams { sort: default_sort(), limit: default_limit(), offset: 0 });
    db.with_conn(|c| tracks::list(c, p.sort, p.limit, p.offset))
}

#[tauri::command]
pub async fn get_albums(db: State<'_, Database>) -> AppResult<Vec<albums::AlbumView>> {
    db.with_conn(albums::get_all)
}

#[tauri::command]
pub async fn get_artists(db: State<'_, Database>) -> AppResult<Vec<artists::Artist>> {
    db.with_conn(artists::get_all)
}

#[tauri::command]
pub async fn get_album_tracks(db: State<'_, Database>, album_id: i64) -> AppResult<Vec<tracks::TrackView>> {
    db.with_conn(|c| tracks::list_by_album(c, album_id))
}

#[tauri::command]
pub async fn get_artist_tracks(db: State<'_, Database>, artist_id: i64) -> AppResult<Vec<tracks::TrackView>> {
    db.with_conn(|c| tracks::list_by_artist(c, artist_id))
}

#[tauri::command]
pub async fn get_recently_added(db: State<'_, Database>, limit: Option<i64>) -> AppResult<Vec<tracks::TrackView>> {
    db.with_conn(|c| tracks::recently_added(c, limit.unwrap_or(50)))
}

#[tauri::command]
pub async fn get_recent_plays(db: State<'_, Database>, limit: Option<i64>) -> AppResult<Vec<play_history::PlayHistoryEntry>> {
    db.with_conn(|c| play_history::get_recent(c, limit.unwrap_or(50)))
}

#[tauri::command]
pub async fn set_favorite(db: State<'_, Database>, track_id: i64, favorite: bool) -> AppResult<()> {
    db.with_conn(|c| tracks::set_favorite(c, track_id, favorite, now_ms()))
}

#[tauri::command]
pub async fn record_play(db: State<'_, Database>, track_id: i64, duration_played_ms: i64, completed: bool) -> AppResult<i64> {
    db.with_conn(|c| play_history::record(c, track_id, now_ms(), duration_played_ms, completed))
}

#[tauri::command]
pub async fn search(db: State<'_, Database>, query: String, limit: Option<i64>) -> AppResult<Vec<tracks::TrackView>> {
    db.with_conn(|c| search::search_tracks(c, &query, limit.unwrap_or(50)))
}

/// 占位：v0.3.0 的 scanner 任务里再实现真正逻辑。
/// 当前只把路径写进 scan_folders 表；扫描动作不做。
#[tauri::command]
pub async fn add_folder(db: State<'_, Database>, path: String) -> AppResult<()> {
    db.with_conn(|c| -> AppResult<()> {
        c.execute(
            "INSERT OR IGNORE INTO scan_folders (path, added_at) VALUES (?1, ?2)",
            rusqlite::params![path, now_ms()],
        )?;
        Ok(())
    })?;
    let _ = playlists::list;  // 抑制未使用警告
    let _ = AppError::NotFound("".into());
    Ok(())
}
