//! Playlist IPC commands.

use tauri::State;

use crate::db::{playlists, tracks, Database};
use crate::error::AppResult;

fn now_ms() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

#[tauri::command]
pub async fn get_playlists(db: State<'_, Database>) -> AppResult<Vec<playlists::PlaylistSummary>> {
    db.with_conn(playlists::list)
}

#[tauri::command]
pub async fn get_playlist_tracks(
    db: State<'_, Database>,
    playlist_id: i64,
) -> AppResult<Vec<tracks::TrackView>> {
    db.with_conn(|c| playlists::get_tracks(c, playlist_id))
}

#[tauri::command]
pub async fn create_playlist(
    db: State<'_, Database>,
    name: String,
    description: Option<String>,
) -> AppResult<i64> {
    db.with_conn(|c| playlists::create(c, &name, description.as_deref(), now_ms()))
}

#[tauri::command]
pub async fn rename_playlist(
    db: State<'_, Database>,
    playlist_id: i64,
    name: String,
) -> AppResult<()> {
    db.with_conn(|c| playlists::rename(c, playlist_id, &name, now_ms()))
}

#[tauri::command]
pub async fn delete_playlist(db: State<'_, Database>, playlist_id: i64) -> AppResult<()> {
    db.with_conn(|c| playlists::delete(c, playlist_id))
}

#[tauri::command]
pub async fn add_to_playlist(
    db: State<'_, Database>,
    playlist_id: i64,
    track_id: i64,
) -> AppResult<i64> {
    db.with_conn(|c| playlists::append_track(c, playlist_id, track_id, now_ms()))
}

#[tauri::command]
pub async fn remove_from_playlist(
    db: State<'_, Database>,
    playlist_id: i64,
    track_id: i64,
    position: i64,
) -> AppResult<()> {
    db.with_conn(|c| playlists::remove_track(c, playlist_id, track_id, position))
}

#[tauri::command]
pub async fn reorder_playlist(
    db: State<'_, Database>,
    playlist_id: i64,
    from_position: i64,
    to_position: i64,
) -> AppResult<()> {
    db.with_conn(|c| playlists::reorder(c, playlist_id, from_position, to_position))
}
