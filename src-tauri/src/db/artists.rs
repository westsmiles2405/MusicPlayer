//! Artist queries.

use rusqlite::{params, Connection, OptionalExtension, Row};
use serde::{Deserialize, Serialize};

use crate::error::AppResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Artist {
    pub id: i64,
    pub name: String,
    pub added_at: i64,
    pub updated_at: i64,
}

impl Artist {
    fn from_row(row: &Row<'_>) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            name: row.get("name")?,
            added_at: row.get("added_at")?,
            updated_at: row.get("updated_at")?,
        })
    }
}

/// 按名查找；不存在则插入。返回 artist id。
/// 用于扫描器：每首歌的艺人名 → 唯一 artist_id。
pub fn upsert_by_name(conn: &Connection, name: &str, now_ms: i64) -> AppResult<i64> {
    // SQLite UNIQUE 在 name 上：先尝试 INSERT OR IGNORE，再 SELECT id。
    conn.execute(
        "INSERT OR IGNORE INTO artists (name, added_at, updated_at) VALUES (?1, ?2, ?2)",
        params![name, now_ms],
    )?;
    let id: i64 = conn.query_row(
        "SELECT id FROM artists WHERE name = ?1",
        params![name],
        |r| r.get(0),
    )?;
    Ok(id)
}

pub fn get_all(conn: &Connection) -> AppResult<Vec<Artist>> {
    let mut stmt = conn.prepare("SELECT id, name, added_at, updated_at FROM artists ORDER BY name COLLATE NOCASE")?;
    let rows = stmt.query_map([], Artist::from_row)?;
    let mut out = Vec::new();
    for r in rows { out.push(r?); }
    Ok(out)
}

pub fn get_by_id(conn: &Connection, id: i64) -> AppResult<Option<Artist>> {
    let opt = conn
        .query_row(
            "SELECT id, name, added_at, updated_at FROM artists WHERE id = ?1",
            params![id],
            Artist::from_row,
        )
        .optional()?;
    Ok(opt)
}

pub fn find_by_name(conn: &Connection, name: &str) -> AppResult<Option<Artist>> {
    let opt = conn
        .query_row(
            "SELECT id, name, added_at, updated_at FROM artists WHERE name = ?1",
            params![name],
            Artist::from_row,
        )
        .optional()?;
    Ok(opt)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::testing::test_db;

    #[test]
    fn upsert_creates_then_returns_existing_id() {
        let conn = test_db();
        let id1 = upsert_by_name(&conn, "Radiohead", 100).unwrap();
        let id2 = upsert_by_name(&conn, "Radiohead", 200).unwrap();
        assert_eq!(id1, id2, "second call must return same id");

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM artists", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn get_all_returns_alphabetical() {
        let conn = test_db();
        upsert_by_name(&conn, "Zoo", 100).unwrap();
        upsert_by_name(&conn, "Alpha", 100).unwrap();
        upsert_by_name(&conn, "Mu", 100).unwrap();
        let all = get_all(&conn).unwrap();
        let names: Vec<_> = all.iter().map(|a| a.name.as_str()).collect();
        assert_eq!(names, vec!["Alpha", "Mu", "Zoo"]);
    }

    #[test]
    fn get_by_id_returns_none_for_missing() {
        let conn = test_db();
        assert!(get_by_id(&conn, 999).unwrap().is_none());
    }

    #[test]
    fn find_by_name_is_case_sensitive() {
        let conn = test_db();
        upsert_by_name(&conn, "Björk", 100).unwrap();
        assert!(find_by_name(&conn, "Björk").unwrap().is_some());
        assert!(find_by_name(&conn, "björk").unwrap().is_none());
    }
}
