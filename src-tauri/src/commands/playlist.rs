use crate::error::AppResult;

#[tauri::command]
pub async fn get_playlists() -> AppResult<Vec<()>> {
    Ok(vec![])
}

#[tauri::command]
pub async fn create_playlist(name: String) -> AppResult<()> {
    let _ = name;
    Ok(())
}

#[tauri::command]
pub async fn delete_playlist(id: i64) -> AppResult<()> {
    let _ = id;
    Ok(())
}

#[tauri::command]
pub async fn add_to_playlist(playlist_id: i64, track_id: i64) -> AppResult<()> {
    let _ = (playlist_id, track_id);
    Ok(())
}

#[tauri::command]
pub async fn remove_from_playlist(playlist_id: i64, track_id: i64) -> AppResult<()> {
    let _ = (playlist_id, track_id);
    Ok(())
}

#[tauri::command]
pub async fn reorder_playlist(playlist_id: i64, from: i32, to: i32) -> AppResult<()> {
    let _ = (playlist_id, from, to);
    Ok(())
}
