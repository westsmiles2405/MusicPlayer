#![allow(dead_code, unused_imports)]
use crate::error::AppResult;
use tauri::State;

#[tauri::command]
pub async fn play(track_id: i64) -> AppResult<()> {
    let _ = track_id;
    Ok(())
}

#[tauri::command]
pub async fn pause() -> AppResult<()> {
    Ok(())
}

#[tauri::command]
pub async fn resume() -> AppResult<()> {
    Ok(())
}

#[tauri::command]
pub async fn seek(position_ms: i64) -> AppResult<()> {
    let _ = position_ms;
    Ok(())
}

#[tauri::command]
pub async fn next() -> AppResult<()> {
    Ok(())
}

#[tauri::command]
pub async fn prev() -> AppResult<()> {
    Ok(())
}
