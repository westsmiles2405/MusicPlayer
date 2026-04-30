//! Test-only helpers. Compiled out of release builds.

#![cfg(test)]

use rusqlite::Connection;

/// 打开内存 SQLite，应用所有迁移，返回连接。
/// 每个测试都用独立连接，避免共享状态。
pub(crate) fn test_db() -> Connection {
    let conn = Connection::open_in_memory().expect("open in-memory DB");
    conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
    crate::db::schema::apply_pending(&conn).expect("apply migrations");
    conn
}

#[test]
fn test_db_applies_v1_schema() {
    let conn = test_db();
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='tracks'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(count, 1, "tracks table should exist after migrations");
}
