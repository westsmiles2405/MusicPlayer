use crate::error::AppResult;
use crate::player::manager::PlayerManager;
use crate::player::state::{PlayerCommand, PlayerSnapshot};
use tauri::State;

// ── new primary commands ───────────────────────────────────────

#[tauri::command]
pub async fn player_play(
    manager: State<'_, PlayerManager>,
    track_id: i64,
    queue_track_ids: Option<Vec<i64>>,
    queue_index: Option<usize>,
) -> AppResult<()> {
    manager.play_queue(track_id, queue_track_ids, queue_index)
}

#[tauri::command]
pub async fn player_pause(manager: State<'_, PlayerManager>) -> AppResult<()> {
    manager.send(PlayerCommand::Pause)
}

#[tauri::command]
pub async fn player_resume(manager: State<'_, PlayerManager>) -> AppResult<()> {
    manager.send(PlayerCommand::Resume)
}

#[tauri::command]
pub async fn player_toggle(manager: State<'_, PlayerManager>) -> AppResult<()> {
    manager.send(PlayerCommand::Toggle)
}

#[tauri::command]
pub async fn player_stop(manager: State<'_, PlayerManager>) -> AppResult<()> {
    manager.send(PlayerCommand::Stop)
}

#[tauri::command]
pub async fn player_seek(manager: State<'_, PlayerManager>, position_ms: i64) -> AppResult<()> {
    manager.send(PlayerCommand::Seek { position_ms })
}

#[tauri::command]
pub async fn player_next(manager: State<'_, PlayerManager>) -> AppResult<()> {
    manager.send(PlayerCommand::Next)
}

#[tauri::command]
pub async fn player_previous(manager: State<'_, PlayerManager>) -> AppResult<()> {
    manager.send(PlayerCommand::Previous)
}

#[tauri::command]
pub async fn player_set_volume(manager: State<'_, PlayerManager>, volume: f32) -> AppResult<()> {
    manager.send(PlayerCommand::SetVolume { volume })
}

#[tauri::command]
pub async fn player_set_muted(manager: State<'_, PlayerManager>, muted: bool) -> AppResult<()> {
    manager.send(PlayerCommand::SetMuted { muted })
}

#[tauri::command]
pub async fn player_toggle_mute(manager: State<'_, PlayerManager>) -> AppResult<()> {
    manager.send(PlayerCommand::ToggleMute)
}

#[tauri::command]
pub async fn player_get_state(manager: State<'_, PlayerManager>) -> AppResult<PlayerSnapshot> {
    Ok(manager.snapshot())
}

// ── compatibility wrappers ─────────────────────────────────────

#[tauri::command]
pub async fn play(manager: State<'_, PlayerManager>, track_id: i64) -> AppResult<()> {
    player_play(manager, track_id, None, None).await
}

#[tauri::command]
pub async fn pause(manager: State<'_, PlayerManager>) -> AppResult<()> {
    player_pause(manager).await
}

#[tauri::command]
pub async fn resume(manager: State<'_, PlayerManager>) -> AppResult<()> {
    player_resume(manager).await
}

#[tauri::command]
pub async fn seek(manager: State<'_, PlayerManager>, position_ms: i64) -> AppResult<()> {
    player_seek(manager, position_ms).await
}

#[tauri::command]
pub async fn next(manager: State<'_, PlayerManager>) -> AppResult<()> {
    player_next(manager).await
}

#[tauri::command]
pub async fn prev(manager: State<'_, PlayerManager>) -> AppResult<()> {
    player_previous(manager).await
}
