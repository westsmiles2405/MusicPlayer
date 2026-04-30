//! Database connection wrapper. Registered as Tauri State so commands can `state.with_conn(...)`.

pub mod schema;
pub mod tracks;
pub mod albums;
pub mod artists;
pub mod playlists;
pub mod play_history;
pub mod search;

#[cfg(test)]
pub(crate) mod testing;

use std::path::Path;
use parking_lot::Mutex;
use rusqlite::Connection;

use crate::error::AppResult;

/// 持有 SQLite 连接 + WAL 模式 + 已应用迁移。
/// 注册为 `tauri::State<Database>` 后被所有 IPC 命令共享。
pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    /// 打开（或创建）数据库文件，开启 WAL，应用所有未应用的迁移。
    pub fn open(path: &Path) -> AppResult<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA foreign_keys = ON;
             PRAGMA synchronous = NORMAL;
             PRAGMA journal_size_limit = 67108864;",
        )?;
        schema::apply_pending(&conn)?;
        Ok(Self { conn: Mutex::new(conn) })
    }

    /// 临时拿到连接执行查询；在锁的作用域内运行 closure。
    /// rusqlite 不是 Send-safe across awaits，所以保持同步即可（命令体不要 await DB）。
    pub fn with_conn<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&Connection) -> T,
    {
        let guard = self.conn.lock();
        f(&guard)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_in_temp_file_applies_migrations() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.db");
        let db = Database::open(&path).unwrap();
        let v: i64 = db.with_conn(|c| {
            c.query_row("SELECT MAX(version) FROM schema_migrations", [], |r| r.get(0)).unwrap()
        });
        assert_eq!(v, 2);
    }
}
