use crate::error::AppResult;

#[tauri::command]
pub async fn get_tracks() -> AppResult<Vec<()>> {
    Ok(vec![])
}

#[tauri::command]
pub async fn get_albums() -> AppResult<Vec<()>> {
    Ok(vec![])
}

#[tauri::command]
pub async fn get_artists() -> AppResult<Vec<()>> {
    Ok(vec![])
}

#[tauri::command]
pub async fn search(query: String) -> AppResult<Vec<()>> {
    let _ = query;
    Ok(vec![])
}

#[tauri::command]
pub async fn add_folder(path: String) -> AppResult<()> {
    let _ = path;
    Ok(())
}
