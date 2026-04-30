//! scan_folders CRUD.
use crate::error::AppResult;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ScanFolder {
    pub id: i64,
    pub path: String,
    pub added_at: i64,
    pub last_scanned_at: Option<i64>,
}

pub fn add(conn: &Connection, path: &str, now_ms: i64) -> AppResult<ScanFolder> {
    conn.execute(
        "INSERT INTO scan_folders (path, added_at) VALUES (?1, ?2)",
        params![path, now_ms],
    )?;
    let id = conn.last_insert_rowid();
    Ok(ScanFolder {
        id,
        path: path.into(),
        added_at: now_ms,
        last_scanned_at: None,
    })
}

pub fn list(conn: &Connection) -> AppResult<Vec<ScanFolder>> {
    let mut stmt =
        conn.prepare("SELECT id, path, added_at, last_scanned_at FROM scan_folders ORDER BY id")?;
    let rows: Vec<ScanFolder> = stmt
        .query_map([], |r| {
            Ok(ScanFolder {
                id: r.get(0)?,
                path: r.get(1)?,
                added_at: r.get(2)?,
                last_scanned_at: r.get(3)?,
            })
        })?
        .filter_map(Result::ok)
        .collect();
    Ok(rows)
}

pub fn delete(conn: &Connection, id: i64) -> AppResult<()> {
    conn.execute("DELETE FROM scan_folders WHERE id = ?1", [id])?;
    Ok(())
}

pub fn update_last_scanned(conn: &Connection, id: i64, now_ms: i64) -> AppResult<()> {
    conn.execute(
        "UPDATE scan_folders SET last_scanned_at = ?1 WHERE id = ?2",
        params![now_ms, id],
    )?;
    Ok(())
}
