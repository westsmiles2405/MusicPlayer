#![allow(dead_code, unused_imports)]
//! Album queries.

use rusqlite::{params, Connection, OptionalExtension, Row};
use serde::{Deserialize, Serialize};

use crate::error::AppResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Album {
    pub id: i64,
    pub name: String,
    pub album_artist_id: i64,
    pub year: Option<i32>,
    pub cover_path: Option<String>,
    pub added_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlbumView {
    #[serde(flatten)]
    pub album: Album,
    pub album_artist_name: String,
    pub track_count: i64,
}

impl Album {
    fn from_row(row: &Row<'_>) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            name: row.get("name")?,
            album_artist_id: row.get("album_artist_id")?,
            year: row.get("year")?,
            cover_path: row.get("cover_path")?,
            added_at: row.get("added_at")?,
            updated_at: row.get("updated_at")?,
        })
    }
}

pub fn upsert(
    conn: &Connection,
    name: &str,
    album_artist_id: i64,
    year: Option<i32>,
    now_ms: i64,
) -> AppResult<i64> {
    conn.execute(
        "INSERT OR IGNORE INTO albums (name, album_artist_id, year, added_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?4)",
        params![name, album_artist_id, year, now_ms],
    )?;
    let id: i64 = conn.query_row(
        "SELECT id FROM albums WHERE name = ?1 AND album_artist_id = ?2",
        params![name, album_artist_id],
        |r| r.get(0),
    )?;
    Ok(id)
}

/// 在已有事务中 upsert album（不打开新事务）。
pub fn upsert_in_tx(
    conn: &Connection,
    name: &str,
    album_artist_id: i64,
    year: Option<i32>,
    now_ms: i64,
) -> AppResult<i64> {
    upsert(conn, name, album_artist_id, year, now_ms)
}

pub fn get_all(conn: &Connection) -> AppResult<Vec<AlbumView>> {
    let mut stmt = conn.prepare(
        "SELECT a.id, a.name, a.album_artist_id, a.year, a.cover_path, a.added_at, a.updated_at,
                ar.name AS album_artist_name,
                (SELECT COUNT(*) FROM tracks t WHERE t.album_id = a.id AND t.missing_at IS NULL) AS track_count
         FROM albums a
         JOIN artists ar ON ar.id = a.album_artist_id
         ORDER BY a.name COLLATE NOCASE",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(AlbumView {
            album: Album::from_row(row)?,
            album_artist_name: row.get("album_artist_name")?,
            track_count: row.get("track_count")?,
        })
    })?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

pub fn get_by_id(conn: &Connection, id: i64) -> AppResult<Option<AlbumView>> {
    let opt = conn
        .query_row(
            "SELECT a.id, a.name, a.album_artist_id, a.year, a.cover_path, a.added_at, a.updated_at,
                    ar.name AS album_artist_name,
                    (SELECT COUNT(*) FROM tracks t WHERE t.album_id = a.id AND t.missing_at IS NULL) AS track_count
             FROM albums a
             JOIN artists ar ON ar.id = a.album_artist_id
             WHERE a.id = ?1",
            params![id],
            |row| {
                Ok(AlbumView {
                    album: Album::from_row(row)?,
                    album_artist_name: row.get("album_artist_name")?,
                    track_count: row.get("track_count")?,
                })
            },
        )
        .optional()?;
    Ok(opt)
}

pub fn search_by_name(conn: &Connection, query: &str, limit: i64) -> AppResult<Vec<AlbumView>> {
    let pattern = format!("%{query}%");
    let mut stmt = conn.prepare(
        "SELECT a.id, a.name, a.album_artist_id, a.year, a.cover_path, a.added_at, a.updated_at,
                ar.name AS album_artist_name,
                (SELECT COUNT(*) FROM tracks t WHERE t.album_id = a.id AND t.missing_at IS NULL) AS track_count
         FROM albums a
         JOIN artists ar ON ar.id = a.album_artist_id
         WHERE a.name LIKE ?1
         ORDER BY a.name COLLATE NOCASE
         LIMIT ?2",
    )?;
    let rows = stmt.query_map(rusqlite::params![pattern, limit.max(1)], |row| {
        Ok(AlbumView {
            album: Album::from_row(row)?,
            album_artist_name: row.get("album_artist_name")?,
            track_count: row.get("track_count")?,
        })
    })?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

pub fn set_cover_path(conn: &Connection, id: i64, cover_path: &str, now_ms: i64) -> AppResult<()> {
    let n = conn.execute(
        "UPDATE albums SET cover_path = ?1, updated_at = ?2 WHERE id = ?3",
        params![cover_path, now_ms, id],
    )?;
    if n == 0 {
        return Err(crate::error::AppError::NotFound(format!("album {id}")));
    }
    Ok(())
}

/// 在已有事务中设置封面路径。
pub fn set_cover_path_in_tx(
    conn: &Connection,
    id: i64,
    cover_path: &str,
    now_ms: i64,
) -> AppResult<()> {
    set_cover_path(conn, id, cover_path, now_ms)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{artists, testing::test_db};

    fn make_artist(conn: &Connection, name: &str) -> i64 {
        artists::upsert_by_name(conn, name, 100).unwrap()
    }

    #[test]
    fn upsert_unique_on_name_plus_album_artist() {
        let conn = test_db();
        let a1 = make_artist(&conn, "Beatles");
        let a2 = make_artist(&conn, "Stones");
        let alb1 = upsert(&conn, "Greatest Hits", a1, Some(1970), 100).unwrap();
        let alb2 = upsert(&conn, "Greatest Hits", a2, Some(1971), 100).unwrap();
        let alb3 = upsert(&conn, "Greatest Hits", a1, Some(1970), 200).unwrap();
        assert_ne!(
            alb1, alb2,
            "same name different artist must produce different albums"
        );
        assert_eq!(
            alb1, alb3,
            "second call with same (name, artist) must return existing id"
        );
    }

    #[test]
    fn get_all_includes_artist_name_and_track_count() {
        let conn = test_db();
        let a = make_artist(&conn, "Beatles");
        upsert(&conn, "Abbey Road", a, Some(1969), 100).unwrap();
        let all = get_all(&conn).unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].album_artist_name, "Beatles");
        assert_eq!(all[0].track_count, 0);
    }

    #[test]
    fn get_by_id_missing_returns_none() {
        let conn = test_db();
        assert!(get_by_id(&conn, 999).unwrap().is_none());
    }

    #[test]
    fn set_cover_path_persists() {
        let conn = test_db();
        let a = make_artist(&conn, "Beatles");
        let alb = upsert(&conn, "Abbey Road", a, Some(1969), 100).unwrap();
        set_cover_path(&conn, alb, "/cache/covers/abc.jpg", 200).unwrap();
        let fetched = get_by_id(&conn, alb).unwrap().unwrap();
        assert_eq!(
            fetched.album.cover_path.as_deref(),
            Some("/cache/covers/abc.jpg")
        );
    }
}
