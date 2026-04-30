//! Play history queries. Records every play attempt; bumps tracks.play_count when completed.

use rusqlite::{params, Connection, Row};
use serde::{Deserialize, Serialize};

use crate::error::AppResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayHistoryEntry {
    pub id: i64,
    pub track_id: i64,
    pub played_at: i64,
    pub duration_played_ms: i64,
    pub completed: bool,
    pub track_title: String,
    pub album_name: Option<String>,
    pub primary_artist_name: Option<String>,
}

/// 写一次播放事件。当 completed=true 时同时累加 tracks.play_count 并更新 last_played_at。
pub fn record(conn: &Connection, track_id: i64, played_at_ms: i64, duration_played_ms: i64, completed: bool) -> AppResult<i64> {
    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "INSERT INTO play_history (track_id, played_at, duration_played_ms, completed) VALUES (?1, ?2, ?3, ?4)",
        params![track_id, played_at_ms, duration_played_ms, completed as i64],
    )?;
    let id = tx.last_insert_rowid();
    if completed {
        tx.execute(
            "UPDATE tracks SET play_count = play_count + 1, last_played_at = ?1, updated_at = ?1 WHERE id = ?2",
            params![played_at_ms, track_id],
        )?;
    } else {
        tx.execute(
            "UPDATE tracks SET last_played_at = ?1, updated_at = ?1 WHERE id = ?2",
            params![played_at_ms, track_id],
        )?;
    }
    tx.commit()?;
    Ok(id)
}

pub fn get_recent(conn: &Connection, limit: i64) -> AppResult<Vec<PlayHistoryEntry>> {
    let mut stmt = conn.prepare(
        "SELECT h.id, h.track_id, h.played_at, h.duration_played_ms, h.completed,
                t.title AS track_title,
                al.name AS album_name,
                ar.name AS primary_artist_name
         FROM play_history h
         JOIN tracks t ON t.id = h.track_id
         LEFT JOIN albums al ON al.id = t.album_id
         LEFT JOIN artists ar ON ar.id = t.primary_artist_id
         ORDER BY h.played_at DESC
         LIMIT ?1",
    )?;
    let rows = stmt.query_map(params![limit], from_row)?;
    let mut out = Vec::new();
    for r in rows { out.push(r?); }
    Ok(out)
}

fn from_row(row: &Row<'_>) -> rusqlite::Result<PlayHistoryEntry> {
    Ok(PlayHistoryEntry {
        id: row.get("id")?,
        track_id: row.get("track_id")?,
        played_at: row.get("played_at")?,
        duration_played_ms: row.get("duration_played_ms")?,
        completed: row.get::<_, i64>("completed")? != 0,
        track_title: row.get("track_title")?,
        album_name: row.get("album_name")?,
        primary_artist_name: row.get("primary_artist_name")?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::testing::{self, test_db};

    #[test]
    fn completed_play_increments_play_count() {
        let conn = test_db();
        let id = testing::make_basic_track(&conn, "Song");
        record(&conn, id, 1000, 240_000, true).unwrap();
        let count: i64 = conn.query_row("SELECT play_count FROM tracks WHERE id=?1", params![id], |r| r.get(0)).unwrap();
        assert_eq!(count, 1);
        record(&conn, id, 2000, 240_000, true).unwrap();
        let count: i64 = conn.query_row("SELECT play_count FROM tracks WHERE id=?1", params![id], |r| r.get(0)).unwrap();
        assert_eq!(count, 2);
    }

    #[test]
    fn skipped_play_only_updates_last_played_at() {
        let conn = test_db();
        let id = testing::make_basic_track(&conn, "Song");
        record(&conn, id, 1500, 30_000, false).unwrap();
        let count: i64 = conn.query_row("SELECT play_count FROM tracks WHERE id=?1", params![id], |r| r.get(0)).unwrap();
        let last: i64 = conn.query_row("SELECT last_played_at FROM tracks WHERE id=?1", params![id], |r| r.get(0)).unwrap();
        assert_eq!(count, 0);
        assert_eq!(last, 1500);
    }

    #[test]
    fn get_recent_orders_desc_with_track_metadata() {
        let conn = test_db();
        let id = testing::make_basic_track(&conn, "Song");
        record(&conn, id, 100, 100, true).unwrap();
        record(&conn, id, 999, 100, true).unwrap();
        let recent = get_recent(&conn, 10).unwrap();
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0].played_at, 999);
        assert_eq!(recent[0].track_title, "Song");
        assert_eq!(recent[0].album_name.as_deref(), Some("TestAlbum"));
    }
}
