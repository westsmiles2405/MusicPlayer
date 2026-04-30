//! Track queries (read + write).

use rusqlite::{params, Connection, OptionalExtension, Row};
use serde::{Deserialize, Serialize};

use crate::error::AppResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Track {
    pub id: i64,
    pub file_path: String,
    pub file_size: i64,
    pub file_modified_at: i64,
    pub hash: Option<String>,
    pub title: String,
    pub album_id: Option<i64>,
    pub primary_artist_id: Option<i64>,
    pub album_artist_id: Option<i64>,
    pub track_no: Option<i32>,
    pub disc_no: Option<i32>,
    pub year: Option<i32>,
    pub genre: Option<String>,
    pub duration_ms: i64,
    pub bitrate: Option<i32>,
    pub sample_rate: Option<i32>,
    pub channels: Option<i32>,
    pub codec: Option<String>,
    pub is_favorite: bool,
    pub play_count: i64,
    pub last_played_at: Option<i64>,
    pub last_seen_at: i64,
    pub missing_at: Option<i64>,
    pub added_at: i64,
    pub updated_at: i64,
}

/// 列表/搜索用的反规范化视图：附带专辑名 / 主艺人名，省一次 round-trip。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackView {
    #[serde(flatten)]
    pub track: Track,
    pub album_name: Option<String>,
    pub primary_artist_name: Option<String>,
}

/// 写入用：尚未拿到 id 的新 track。`scanner` 在 Task 2.5 用到。
#[derive(Debug, Clone)]
pub struct NewTrack {
    pub file_path: String,
    pub file_size: i64,
    pub file_modified_at: i64,
    pub hash: Option<String>,
    pub title: String,
    pub album_id: Option<i64>,
    pub primary_artist_id: Option<i64>,
    pub album_artist_id: Option<i64>,
    pub track_no: Option<i32>,
    pub disc_no: Option<i32>,
    pub year: Option<i32>,
    pub genre: Option<String>,
    pub duration_ms: i64,
    pub bitrate: Option<i32>,
    pub sample_rate: Option<i32>,
    pub channels: Option<i32>,
    pub codec: Option<String>,
}

impl Track {
    fn from_row(row: &Row<'_>) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            file_path: row.get("file_path")?,
            file_size: row.get("file_size")?,
            file_modified_at: row.get("file_modified_at")?,
            hash: row.get("hash")?,
            title: row.get("title")?,
            album_id: row.get("album_id")?,
            primary_artist_id: row.get("primary_artist_id")?,
            album_artist_id: row.get("album_artist_id")?,
            track_no: row.get("track_no")?,
            disc_no: row.get("disc_no")?,
            year: row.get("year")?,
            genre: row.get("genre")?,
            duration_ms: row.get("duration_ms")?,
            bitrate: row.get("bitrate")?,
            sample_rate: row.get("sample_rate")?,
            channels: row.get("channels")?,
            codec: row.get("codec")?,
            is_favorite: row.get::<_, i64>("is_favorite")? != 0,
            play_count: row.get("play_count")?,
            last_played_at: row.get("last_played_at")?,
            last_seen_at: row.get("last_seen_at")?,
            missing_at: row.get("missing_at")?,
            added_at: row.get("added_at")?,
            updated_at: row.get("updated_at")?,
        })
    }
}

/// 插入单首歌；同时把 (track_id, primary_artist_id) 写到 track_artists（role='main', position=0）。
/// 返回新 track id。
pub fn insert(conn: &Connection, t: &NewTrack, now_ms: i64) -> AppResult<i64> {
    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "INSERT INTO tracks (
            file_path, file_size, file_modified_at, hash, title,
            album_id, primary_artist_id, album_artist_id,
            track_no, disc_no, year, genre,
            duration_ms, bitrate, sample_rate, channels, codec,
            is_favorite, play_count, last_played_at,
            last_seen_at, missing_at, added_at, updated_at
         ) VALUES (
            ?1, ?2, ?3, ?4, ?5,
            ?6, ?7, ?8,
            ?9, ?10, ?11, ?12,
            ?13, ?14, ?15, ?16, ?17,
            0, 0, NULL,
            ?18, NULL, ?18, ?18
         )",
        params![
            t.file_path, t.file_size, t.file_modified_at, t.hash, t.title,
            t.album_id, t.primary_artist_id, t.album_artist_id,
            t.track_no, t.disc_no, t.year, t.genre,
            t.duration_ms, t.bitrate, t.sample_rate, t.channels, t.codec,
            now_ms,
        ],
    )?;
    let id = tx.last_insert_rowid();
    if let Some(aid) = t.primary_artist_id {
        tx.execute(
            "INSERT OR IGNORE INTO track_artists (track_id, artist_id, role, position) VALUES (?1, ?2, 'main', 0)",
            params![id, aid],
        )?;
    }
    tx.commit()?;
    Ok(id)
}

/// 按 file_path 整行覆盖（扫描器二次扫描时用，元数据可能变了）。
pub fn update_by_path(conn: &Connection, t: &NewTrack, now_ms: i64) -> AppResult<i64> {
    let id: i64 = conn
        .query_row(
            "SELECT id FROM tracks WHERE file_path = ?1",
            params![t.file_path],
            |r| r.get(0),
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => crate::error::AppError::NotFound(format!("track at {}", t.file_path)),
            other => other.into(),
        })?;
    conn.execute(
        "UPDATE tracks SET
            file_size = ?2, file_modified_at = ?3, hash = ?4, title = ?5,
            album_id = ?6, primary_artist_id = ?7, album_artist_id = ?8,
            track_no = ?9, disc_no = ?10, year = ?11, genre = ?12,
            duration_ms = ?13, bitrate = ?14, sample_rate = ?15, channels = ?16, codec = ?17,
            last_seen_at = ?18, missing_at = NULL, updated_at = ?18
         WHERE id = ?1",
        params![
            id, t.file_size, t.file_modified_at, t.hash, t.title,
            t.album_id, t.primary_artist_id, t.album_artist_id,
            t.track_no, t.disc_no, t.year, t.genre,
            t.duration_ms, t.bitrate, t.sample_rate, t.channels, t.codec,
            now_ms,
        ],
    )?;
    Ok(id)
}

/// 软删除：设置 missing_at = now，保留 row 不破坏 playlist/play_history 引用。
pub fn mark_missing(conn: &Connection, ids: &[i64], now_ms: i64) -> AppResult<usize> {
    if ids.is_empty() { return Ok(0); }
    let mut stmt = conn.prepare("UPDATE tracks SET missing_at = ?1, updated_at = ?1 WHERE id = ?2")?;
    let mut updated = 0;
    for &id in ids {
        updated += stmt.execute(params![now_ms, id])?;
    }
    Ok(updated)
}

/// 标记为重新发现：清除 missing_at，更新 last_seen_at。
pub fn mark_present(conn: &Connection, ids: &[i64], now_ms: i64) -> AppResult<usize> {
    if ids.is_empty() { return Ok(0); }
    let mut stmt = conn.prepare("UPDATE tracks SET missing_at = NULL, last_seen_at = ?1, updated_at = ?1 WHERE id = ?2")?;
    let mut updated = 0;
    for &id in ids {
        updated += stmt.execute(params![now_ms, id])?;
    }
    Ok(updated)
}

/// 用 hash 找已知曲目（用于"文件被移动后保留收藏/播放次数"的身份匹配）。
pub fn find_by_hash(conn: &Connection, hash: &str) -> AppResult<Vec<Track>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM tracks WHERE hash = ?1 AND missing_at IS NULL ORDER BY id",
    )?;
    let rows = stmt.query_map(params![hash], Track::from_row)?;
    let mut out = Vec::new();
    for r in rows { out.push(r?); }
    Ok(out)
}

pub fn find_by_path(conn: &Connection, path: &str) -> AppResult<Option<Track>> {
    let opt = conn
        .query_row("SELECT * FROM tracks WHERE file_path = ?1", params![path], Track::from_row)
        .optional()?;
    Ok(opt)
}

/// 关联 (track_id, artist_id, role)。同 (track_id, role) 已存在时无视。
pub fn link_artist(conn: &Connection, track_id: i64, artist_id: i64, role: &str, position: i32) -> AppResult<()> {
    conn.execute(
        "INSERT OR IGNORE INTO track_artists (track_id, artist_id, role, position) VALUES (?1, ?2, ?3, ?4)",
        params![track_id, artist_id, role, position],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{albums, artists, testing::test_db};

    pub(crate) fn make_basic_track(conn: &Connection, title: &str) -> i64 {
        let artist = artists::upsert_by_name(conn, "TestArtist", 100).unwrap();
        let album = albums::upsert(conn, "TestAlbum", artist, Some(2024), 100).unwrap();
        let nt = NewTrack {
            file_path: format!("/music/{title}.mp3"),
            file_size: 4_000_000,
            file_modified_at: 1000,
            hash: Some(format!("hash-{title}")),
            title: title.into(),
            album_id: Some(album),
            primary_artist_id: Some(artist),
            album_artist_id: Some(artist),
            track_no: Some(1),
            disc_no: Some(1),
            year: Some(2024),
            genre: Some("Indie".into()),
            duration_ms: 240_000,
            bitrate: Some(320),
            sample_rate: Some(44_100),
            channels: Some(2),
            codec: Some("mp3".into()),
        };
        insert(conn, &nt, 100).unwrap()
    }

    #[test]
    fn insert_creates_track_and_main_artist_link() {
        let conn = test_db();
        let id = make_basic_track(&conn, "Hello");
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM tracks WHERE id = ?1", params![id], |r| r.get(0)).unwrap();
        assert_eq!(count, 1);
        let role: String = conn.query_row("SELECT role FROM track_artists WHERE track_id = ?1", params![id], |r| r.get(0)).unwrap();
        assert_eq!(role, "main");
    }

    #[test]
    fn insert_duplicate_path_fails() {
        let conn = test_db();
        let _ = make_basic_track(&conn, "Hello");
        let artist = artists::find_by_name(&conn, "TestArtist").unwrap().unwrap().id;
        let album = albums::upsert(&conn, "TestAlbum", artist, Some(2024), 100).unwrap();
        let dup = NewTrack {
            file_path: "/music/Hello.mp3".into(),
            file_size: 1, file_modified_at: 0, hash: None, title: "X".into(),
            album_id: Some(album), primary_artist_id: Some(artist), album_artist_id: Some(artist),
            track_no: None, disc_no: None, year: None, genre: None,
            duration_ms: 0, bitrate: None, sample_rate: None, channels: None, codec: None,
        };
        assert!(insert(&conn, &dup, 100).is_err());
    }

    #[test]
    fn update_by_path_overwrites_metadata() {
        let conn = test_db();
        let id = make_basic_track(&conn, "Hello");
        let artist = artists::find_by_name(&conn, "TestArtist").unwrap().unwrap().id;
        let album = albums::upsert(&conn, "TestAlbum", artist, Some(2024), 100).unwrap();
        let edited = NewTrack {
            file_path: "/music/Hello.mp3".into(),
            file_size: 5_000_000, file_modified_at: 9000,
            hash: Some("new-hash".into()), title: "Hello (Remastered)".into(),
            album_id: Some(album), primary_artist_id: Some(artist), album_artist_id: Some(artist),
            track_no: Some(2), disc_no: Some(1), year: Some(2025), genre: None,
            duration_ms: 250_000, bitrate: Some(320), sample_rate: Some(44_100), channels: Some(2), codec: Some("mp3".into()),
        };
        let same_id = update_by_path(&conn, &edited, 9999).unwrap();
        assert_eq!(same_id, id);
        let title: String = conn.query_row("SELECT title FROM tracks WHERE id=?1", params![id], |r| r.get(0)).unwrap();
        assert_eq!(title, "Hello (Remastered)");
    }

    #[test]
    fn mark_missing_then_present_roundtrip() {
        let conn = test_db();
        let id = make_basic_track(&conn, "X");
        assert_eq!(mark_missing(&conn, &[id], 1000).unwrap(), 1);
        let m: Option<i64> = conn.query_row("SELECT missing_at FROM tracks WHERE id=?1", params![id], |r| r.get(0)).unwrap();
        assert_eq!(m, Some(1000));
        assert_eq!(mark_present(&conn, &[id], 2000).unwrap(), 1);
        let m: Option<i64> = conn.query_row("SELECT missing_at FROM tracks WHERE id=?1", params![id], |r| r.get(0)).unwrap();
        assert_eq!(m, None);
    }

    #[test]
    fn find_by_hash_returns_match() {
        let conn = test_db();
        let id = make_basic_track(&conn, "Hello");
        let found = find_by_hash(&conn, "hash-Hello").unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].id, id);
    }

    #[test]
    fn link_artist_idempotent_on_same_role() {
        let conn = test_db();
        let id = make_basic_track(&conn, "Hello");
        let other = artists::upsert_by_name(&conn, "Featured", 100).unwrap();
        link_artist(&conn, id, other, "featured", 0).unwrap();
        link_artist(&conn, id, other, "featured", 0).unwrap();  // 第二次必须不报错
        let n: i64 = conn.query_row("SELECT COUNT(*) FROM track_artists WHERE track_id=?1", params![id], |r| r.get(0)).unwrap();
        assert_eq!(n, 2, "main + featured");
    }
}

