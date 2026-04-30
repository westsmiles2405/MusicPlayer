use serde::Serialize;

/// 统一错误类型，经 Tauri IPC 传到前端
#[derive(Debug, thiserror::Error, Serialize)]
#[serde(tag = "kind", content = "message")]
pub enum AppError {
    #[error("未找到: {0}")]
    NotFound(String),

    #[error("数据库错误: {0}")]
    Database(String),

    #[error("文件未找到: {0}")]
    FileNotFound(String),

    #[error("无权限访问: {0}")]
    PermissionDenied(String),

    #[error("播放失败: {0}")]
    Playback(String),

    #[error("元数据解析失败: {0}")]
    Metadata(String),

    #[error("扫描错误: {0}")]
    Scan(String),

    #[error("无效输入: {0}")]
    InvalidInput(String),

    #[error("内部错误: {0}")]
    Internal(String),
}

impl From<rusqlite::Error> for AppError {
    fn from(e: rusqlite::Error) -> Self {
        AppError::Database(e.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_found_serializes_with_kind_tag() {
        let err = AppError::NotFound("playlist 5".into());
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains(r#""kind":"NotFound""#));
        assert!(json.contains(r#""message":"playlist 5""#));
    }

    #[test]
    fn database_error_from_rusqlite_preserves_message() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        let r = conn.execute("SELECT no_such_column FROM no_such_table", []);
        let err: AppError = r.unwrap_err().into();
        match err {
            AppError::Database(msg) => assert!(msg.contains("no_such_table") || msg.contains("no such")),
            other => panic!("expected Database, got {other:?}"),
        }
    }
}
