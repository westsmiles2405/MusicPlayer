use crate::error::AppResult;

#[tauri::command]
pub async fn get_pref(key: String) -> AppResult<Option<String>> {
    let _ = key;
    Ok(None)
}

#[tauri::command]
pub async fn set_pref(key: String, value: String) -> AppResult<()> {
    let _ = (key, value);
    Ok(())
}
