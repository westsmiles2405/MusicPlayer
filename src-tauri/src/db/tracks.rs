#![allow(dead_code)]
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
    pub root_folder_id: Option<i64>,
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
    pub root_folder_id: Option<i64>,
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
            root_folder_id: row.get("root_folder_id")?,
        })
    }

    #[doc(hidden)]
    pub(crate) fn from_row_via_helper(row: &Row<'_>) -> rusqlite::Result<Self> {
        Self::from_row(row)
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
            last_seen_at, missing_at, added_at, updated_at,
            root_folder_id
         ) VALUES (
            ?1, ?2, ?3, ?4, ?5,
            ?6, ?7, ?8,
            ?9, ?10, ?11, ?12,
            ?13, ?14, ?15, ?16, ?17,
            0, 0, NULL,
            ?18, NULL, ?18, ?18,
            ?19
         )",
        params![
            t.file_path,
            t.file_size,
            t.file_modified_at,
            t.hash,
            t.title,
            t.album_id,
            t.primary_artist_id,
            t.album_artist_id,
            t.track_no,
            t.disc_no,
            t.year,
            t.genre,
            t.duration_ms,
            t.bitrate,
            t.sample_rate,
            t.channels,
            t.codec,
            now_ms,
            t.root_folder_id,
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

/// 在已有事务中插入 track（不打开新事务）。
pub fn insert_in_tx(conn: &Connection, t: &NewTrack, now_ms: i64) -> AppResult<i64> {
    conn.execute(
        "INSERT INTO tracks (
            file_path, file_size, file_modified_at, hash, title,
            album_id, primary_artist_id, album_artist_id,
            track_no, disc_no, year, genre,
            duration_ms, bitrate, sample_rate, channels, codec,
            is_favorite, play_count, last_played_at,
            last_seen_at, missing_at, added_at, updated_at,
            root_folder_id
         ) VALUES (
            ?1, ?2, ?3, ?4, ?5,
            ?6, ?7, ?8,
            ?9, ?10, ?11, ?12,
            ?13, ?14, ?15, ?16, ?17,
            0, 0, NULL,
            ?18, NULL, ?18, ?18,
            ?19
         )",
        params![
            t.file_path,
            t.file_size,
            t.file_modified_at,
            t.hash,
            t.title,
            t.album_id,
            t.primary_artist_id,
            t.album_artist_id,
            t.track_no,
            t.disc_no,
            t.year,
            t.genre,
            t.duration_ms,
            t.bitrate,
            t.sample_rate,
            t.channels,
            t.codec,
            now_ms,
            t.root_folder_id,
        ],
    )?;
    let id = conn.last_insert_rowid();
    if let Some(aid) = t.primary_artist_id {
        conn.execute(
            "INSERT OR IGNORE INTO track_artists (track_id, artist_id, role, position) VALUES (?1, ?2, 'main', 0)",
            params![id, aid],
        )?;
    }
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
            rusqlite::Error::QueryReturnedNoRows => {
                crate::error::AppError::NotFound(format!("track at {}", t.file_path))
            }
            other => other.into(),
        })?;
    conn.execute(
        "UPDATE tracks SET
            file_size = ?2, file_modified_at = ?3, hash = ?4, title = ?5,
            album_id = ?6, primary_artist_id = ?7, album_artist_id = ?8,
            track_no = ?9, disc_no = ?10, year = ?11, genre = ?12,
            duration_ms = ?13, bitrate = ?14, sample_rate = ?15, channels = ?16, codec = ?17,
            last_seen_at = ?18, missing_at = NULL, updated_at = ?18,
            root_folder_id = ?19
         WHERE id = ?1",
        params![
            id,
            t.file_size,
            t.file_modified_at,
            t.hash,
            t.title,
            t.album_id,
            t.primary_artist_id,
            t.album_artist_id,
            t.track_no,
            t.disc_no,
            t.year,
            t.genre,
            t.duration_ms,
            t.bitrate,
            t.sample_rate,
            t.channels,
            t.codec,
            now_ms,
            t.root_folder_id,
        ],
    )?;
    Ok(id)
}

/// 在已有事务中按 file_path 整行覆盖。
pub fn update_by_path_in_tx(conn: &Connection, t: &NewTrack, now_ms: i64) -> AppResult<i64> {
    let n = conn.execute(
        "UPDATE tracks
            SET file_size = ?2, file_modified_at = ?3, hash = ?4, title = ?5,
                album_id = ?6, primary_artist_id = ?7, album_artist_id = ?8,
                track_no = ?9, disc_no = ?10, year = ?11, genre = ?12,
                duration_ms = ?13, bitrate = ?14, sample_rate = ?15,
                channels = ?16, codec = ?17,
                last_seen_at = ?18, missing_at = NULL, updated_at = ?18,
                root_folder_id = ?19
          WHERE file_path = ?1",
        params![
            t.file_path,
            t.file_size,
            t.file_modified_at,
            t.hash,
            t.title,
            t.album_id,
            t.primary_artist_id,
            t.album_artist_id,
            t.track_no,
            t.disc_no,
            t.year,
            t.genre,
            t.duration_ms,
            t.bitrate,
            t.sample_rate,
            t.channels,
            t.codec,
            now_ms,
            t.root_folder_id,
        ],
    )?;
    if n == 0 {
        return Err(crate::error::AppError::NotFound(format!(
            "track path {}",
            t.file_path
        )));
    }
    let id: i64 = conn.query_row(
        "SELECT id FROM tracks WHERE file_path = ?1",
        [&t.file_path],
        |r| r.get(0),
    )?;
    Ok(id)
}

/// 在已有事务中标记单首 track 重新出现（清除 missing_at，更新 last_seen_at）。
pub fn mark_present_in_tx(conn: &Connection, track_id: i64, now_ms: i64) -> AppResult<()> {
    conn.execute(
        "UPDATE tracks SET missing_at = NULL, last_seen_at = ?1, updated_at = ?1 WHERE id = ?2",
        params![now_ms, track_id],
    )?;
    Ok(())
}

/// 文件移动检测：对 hash 匹配的 track 更新路径/mtime/size/root。
pub fn update_path_for_move_in_tx(
    conn: &Connection,
    track_id: i64,
    new_path: &str,
    file_size: i64,
    file_modified_at: i64,
    now_ms: i64,
    root_folder_id: Option<i64>,
) -> AppResult<()> {
    conn.execute(
        "UPDATE tracks
            SET file_path = ?2, file_size = ?3, file_modified_at = ?4,
                last_seen_at = ?5, missing_at = NULL, updated_at = ?5,
                root_folder_id = ?6
          WHERE id = ?1",
        params![
            track_id,
            new_path,
            file_size,
            file_modified_at,
            now_ms,
            root_folder_id,
        ],
    )?;
    Ok(())
}

/// 软删除：设置 missing_at = now，保留 row 不破坏 playlist/play_history 引用。
pub fn mark_missing(conn: &Connection, ids: &[i64], now_ms: i64) -> AppResult<usize> {
    if ids.is_empty() {
        return Ok(0);
    }
    let mut stmt =
        conn.prepare("UPDATE tracks SET missing_at = ?1, updated_at = ?1 WHERE id = ?2")?;
    let mut updated = 0;
    for &id in ids {
        updated += stmt.execute(params![now_ms, id])?;
    }
    Ok(updated)
}

/// 标记为重新发现：清除 missing_at，更新 last_seen_at。
pub fn mark_present(conn: &Connection, ids: &[i64], now_ms: i64) -> AppResult<usize> {
    if ids.is_empty() {
        return Ok(0);
    }
    let mut stmt = conn.prepare(
        "UPDATE tracks SET missing_at = NULL, last_seen_at = ?1, updated_at = ?1 WHERE id = ?2",
    )?;
    let mut updated = 0;
    for &id in ids {
        updated += stmt.execute(params![now_ms, id])?;
    }
    Ok(updated)
}

/// 将某个 root 下所有非 missing track 标记为 missing（移除文件夹时用）。
pub fn mark_missing_by_root(
    conn: &Connection,
    root_folder_id: i64,
    now_ms: i64,
) -> AppResult<usize> {
    let n = conn.execute(
        "UPDATE tracks SET missing_at = ?1, updated_at = ?1
          WHERE root_folder_id = ?2 AND missing_at IS NULL",
        params![now_ms, root_folder_id],
    )?;
    Ok(n)
}

/// 解除 track 与 root_folder 的关联（移除文件夹清理时用）。
pub fn unlink_root(conn: &Connection, root_folder_id: i64) -> AppResult<usize> {
    let n = conn.execute(
        "UPDATE tracks SET root_folder_id = NULL WHERE root_folder_id = ?1",
        [root_folder_id],
    )?;
    Ok(n)
}

/// 用 hash 找已知曲目（用于"文件被移动后保留收藏/播放次数"的身份匹配）。
pub fn find_by_hash(conn: &Connection, hash: &str) -> AppResult<Vec<Track>> {
    let mut stmt =
        conn.prepare("SELECT * FROM tracks WHERE hash = ?1 AND missing_at IS NULL ORDER BY id")?;
    let rows = stmt.query_map(params![hash], Track::from_row)?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

pub fn find_by_path(conn: &Connection, path: &str) -> AppResult<Option<Track>> {
    let opt = conn
        .query_row(
            "SELECT * FROM tracks WHERE file_path = ?1",
            params![path],
            Track::from_row,
        )
        .optional()?;
    Ok(opt)
}

/// 用 hash + file_size 精确匹配 track id（用于文件移动检测）。
pub fn find_ids_by_hash_size(conn: &Connection, hash: &str, size: i64) -> AppResult<Vec<i64>> {
    let mut stmt = conn.prepare("SELECT id FROM tracks WHERE hash = ?1 AND file_size = ?2")?;
    let ids: Vec<i64> = stmt
        .query_map(params![hash, size], |r| r.get(0))?
        .filter_map(Result::ok)
        .collect();
    Ok(ids)
}

/// 关联 (track_id, artist_id, role)。同 (track_id, role) 已存在时无视。
pub fn link_artist(
    conn: &Connection,
    track_id: i64,
    artist_id: i64,
    role: &str,
    position: i32,
) -> AppResult<()> {
    conn.execute(
        "INSERT OR IGNORE INTO track_artists (track_id, artist_id, role, position) VALUES (?1, ?2, ?3, ?4)",
        params![track_id, artist_id, role, position],
    )?;
    Ok(())
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TrackSort {
    Title,
    Artist,
    Album,
    AddedAt,
    LastPlayed,
}

/// 列表查询。`missing_at IS NULL` 自动过滤已软删除的歌曲。
pub fn list(
    conn: &Connection,
    sort: TrackSort,
    limit: i64,
    offset: i64,
) -> AppResult<Vec<TrackView>> {
    let order_by = match sort {
        TrackSort::Title => "t.title COLLATE NOCASE ASC",
        TrackSort::Artist => "ar.name COLLATE NOCASE ASC, t.title COLLATE NOCASE ASC",
        TrackSort::Album => "al.name COLLATE NOCASE ASC, t.disc_no, t.track_no",
        TrackSort::AddedAt => "t.added_at DESC",
        TrackSort::LastPlayed => "t.last_played_at DESC NULLS LAST",
    };
    let sql = format!(
        "{} ORDER BY {order_by} LIMIT ?1 OFFSET ?2",
        base_track_view_select(" WHERE t.missing_at IS NULL")
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params![limit, offset], track_view_from_row)?;
    collect(rows)
}

pub fn get_view_by_id(conn: &Connection, id: i64) -> AppResult<Option<TrackView>> {
    let sql = format!("{} WHERE t.id = ?1", base_track_view_select(""));
    let opt = conn
        .query_row(&sql, params![id], track_view_from_row)
        .optional()?;
    Ok(opt)
}

/// 按 id 获取单首 track（不含 view 联表）。
pub fn get_by_id(conn: &Connection, id: i64) -> AppResult<Track> {
    conn.query_row(
        "SELECT * FROM tracks WHERE id = ?1",
        params![id],
        Track::from_row,
    )
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => {
            crate::error::AppError::NotFound(format!("track {id}"))
        }
        other => other.into(),
    })
}

pub fn list_by_album(conn: &Connection, album_id: i64) -> AppResult<Vec<TrackView>> {
    let sql = format!(
        "{} ORDER BY t.disc_no NULLS LAST, t.track_no NULLS LAST, t.title COLLATE NOCASE",
        base_track_view_select(" WHERE t.album_id = ?1 AND t.missing_at IS NULL"),
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params![album_id], track_view_from_row)?;
    collect(rows)
}

pub fn list_by_artist(conn: &Connection, artist_id: i64) -> AppResult<Vec<TrackView>> {
    let sql = format!(
        "{} JOIN track_artists ta ON ta.track_id = t.id WHERE ta.artist_id = ?1 AND t.missing_at IS NULL ORDER BY t.title COLLATE NOCASE",
        base_track_view_select(""),
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params![artist_id], track_view_from_row)?;
    collect(rows)
}

pub fn recently_added(conn: &Connection, limit: i64) -> AppResult<Vec<TrackView>> {
    let sql = format!(
        "{} ORDER BY t.added_at DESC LIMIT ?1",
        base_track_view_select(" WHERE t.missing_at IS NULL"),
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params![limit], track_view_from_row)?;
    collect(rows)
}

pub fn set_favorite(conn: &Connection, id: i64, favorite: bool, now_ms: i64) -> AppResult<()> {
    let n = conn.execute(
        "UPDATE tracks SET is_favorite = ?1, updated_at = ?2 WHERE id = ?3",
        params![favorite as i64, now_ms, id],
    )?;
    if n == 0 {
        return Err(crate::error::AppError::NotFound(format!("track {id}")));
    }
    Ok(())
}

pub fn list_favorite_tracks(conn: &Connection) -> AppResult<Vec<TrackView>> {
    let sql = format!(
        "{} AND t.is_favorite = 1 ORDER BY t.title COLLATE NOCASE ASC",
        base_track_view_select(" WHERE t.missing_at IS NULL"),
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], track_view_from_row)?;
    collect(rows)
}

// ---- internal helpers ----

pub(crate) fn base_track_view_select(extra_where: &str) -> String {
    format!(
        "SELECT t.id, t.file_path, t.file_size, t.file_modified_at, t.hash, t.title,
                t.album_id, t.primary_artist_id, t.album_artist_id,
                t.track_no, t.disc_no, t.year, t.genre,
                t.duration_ms, t.bitrate, t.sample_rate, t.channels, t.codec,
                t.is_favorite, t.play_count, t.last_played_at,
                t.last_seen_at, t.missing_at, t.added_at, t.updated_at,
                t.root_folder_id,
                al.name AS album_name,
                ar.name AS primary_artist_name
         FROM tracks t
         LEFT JOIN albums al ON al.id = t.album_id
         LEFT JOIN artists ar ON ar.id = t.primary_artist_id{extra_where}",
    )
}

pub(crate) fn track_view_from_row(row: &Row<'_>) -> rusqlite::Result<TrackView> {
    Ok(TrackView {
        track: Track::from_row(row)?,
        album_name: row.get("album_name")?,
        primary_artist_name: row.get("primary_artist_name")?,
    })
}

pub(crate) fn collect<T, I>(iter: I) -> AppResult<Vec<T>>
where
    I: IntoIterator<Item = rusqlite::Result<T>>,
{
    let mut out = Vec::new();
    for r in iter {
        out.push(r?);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{albums, artists, testing::test_db};

    pub(crate) fn make_basic_track(conn: &Connection, title: &str) -> i64 {
        crate::db::testing::make_basic_track(conn, title)
    }

    #[test]
    fn insert_creates_track_and_main_artist_link() {
        let conn = test_db();
        let id = make_basic_track(&conn, "Hello");
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM tracks WHERE id = ?1",
                params![id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
        let role: String = conn
            .query_row(
                "SELECT role FROM track_artists WHERE track_id = ?1",
                params![id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(role, "main");
    }

    #[test]
    fn insert_duplicate_path_fails() {
        let conn = test_db();
        let _ = make_basic_track(&conn, "Hello");
        let artist = artists::find_by_name(&conn, "TestArtist")
            .unwrap()
            .unwrap()
            .id;
        let album = albums::upsert(&conn, "TestAlbum", artist, Some(2024), 100).unwrap();
        let dup = NewTrack {
            file_path: "/music/Hello.mp3".into(),
            file_size: 1,
            file_modified_at: 0,
            hash: None,
            title: "X".into(),
            album_id: Some(album),
            primary_artist_id: Some(artist),
            album_artist_id: Some(artist),
            track_no: None,
            disc_no: None,
            year: None,
            genre: None,
            duration_ms: 0,
            bitrate: None,
            sample_rate: None,
            channels: None,
            codec: None,
            root_folder_id: None,
        };
        assert!(insert(&conn, &dup, 100).is_err());
    }

    #[test]
    fn update_by_path_overwrites_metadata() {
        let conn = test_db();
        let id = make_basic_track(&conn, "Hello");
        let artist = artists::find_by_name(&conn, "TestArtist")
            .unwrap()
            .unwrap()
            .id;
        let album = albums::upsert(&conn, "TestAlbum", artist, Some(2024), 100).unwrap();
        let edited = NewTrack {
            file_path: "/music/Hello.mp3".into(),
            file_size: 5_000_000,
            file_modified_at: 9000,
            hash: Some("new-hash".into()),
            title: "Hello (Remastered)".into(),
            album_id: Some(album),
            primary_artist_id: Some(artist),
            album_artist_id: Some(artist),
            track_no: Some(2),
            disc_no: Some(1),
            year: Some(2025),
            genre: None,
            duration_ms: 250_000,
            bitrate: Some(320),
            sample_rate: Some(44_100),
            channels: Some(2),
            codec: Some("mp3".into()),
            root_folder_id: None,
        };
        let same_id = update_by_path(&conn, &edited, 9999).unwrap();
        assert_eq!(same_id, id);
        let title: String = conn
            .query_row("SELECT title FROM tracks WHERE id=?1", params![id], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(title, "Hello (Remastered)");
    }

    #[test]
    fn mark_missing_then_present_roundtrip() {
        let conn = test_db();
        let id = make_basic_track(&conn, "X");
        assert_eq!(mark_missing(&conn, &[id], 1000).unwrap(), 1);
        let m: Option<i64> = conn
            .query_row(
                "SELECT missing_at FROM tracks WHERE id=?1",
                params![id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(m, Some(1000));
        assert_eq!(mark_present(&conn, &[id], 2000).unwrap(), 1);
        let m: Option<i64> = conn
            .query_row(
                "SELECT missing_at FROM tracks WHERE id=?1",
                params![id],
                |r| r.get(0),
            )
            .unwrap();
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
        link_artist(&conn, id, other, "featured", 0).unwrap(); // 第二次必须不报错
        let n: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM track_artists WHERE track_id=?1",
                params![id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(n, 2, "main + featured");
    }

    #[test]
    fn list_returns_track_view_with_album_and_artist_names() {
        let conn = test_db();
        let _ = make_basic_track(&conn, "Song1");
        let views = list(&conn, TrackSort::Title, 100, 0).unwrap();
        assert_eq!(views.len(), 1);
        assert_eq!(views[0].album_name.as_deref(), Some("TestAlbum"));
        assert_eq!(views[0].primary_artist_name.as_deref(), Some("TestArtist"));
    }

    #[test]
    fn list_excludes_missing_tracks() {
        let conn = test_db();
        let id = make_basic_track(&conn, "Hidden");
        mark_missing(&conn, &[id], 1000).unwrap();
        let views = list(&conn, TrackSort::Title, 100, 0).unwrap();
        assert!(views.is_empty());
    }

    #[test]
    fn list_by_album_orders_by_disc_and_track_no() {
        let conn = test_db();
        let _ = make_basic_track(&conn, "Track1"); // track_no=1
        let artist = artists::find_by_name(&conn, "TestArtist")
            .unwrap()
            .unwrap()
            .id;
        let album = albums::upsert(&conn, "TestAlbum", artist, Some(2024), 100).unwrap();
        let nt2 = NewTrack {
            file_path: "/music/Track0.mp3".into(),
            file_size: 1,
            file_modified_at: 0,
            hash: None,
            title: "Track0".into(),
            album_id: Some(album),
            primary_artist_id: Some(artist),
            album_artist_id: Some(artist),
            track_no: Some(0),
            disc_no: Some(1),
            year: None,
            genre: None,
            duration_ms: 0,
            bitrate: None,
            sample_rate: None,
            channels: None,
            codec: None,
            root_folder_id: None,
        };
        insert(&conn, &nt2, 100).unwrap();
        let views = list_by_album(&conn, album).unwrap();
        assert_eq!(views.len(), 2);
        assert_eq!(views[0].track.track_no, Some(0));
        assert_eq!(views[1].track.track_no, Some(1));
    }

    #[test]
    fn recently_added_orders_desc_by_added_at() {
        let conn = test_db();
        let id_old = make_basic_track(&conn, "Old");
        // 手动改 added_at 让"Old"早于另一首
        conn.execute(
            "UPDATE tracks SET added_at = 100 WHERE id = ?1",
            params![id_old],
        )
        .unwrap();
        let _id_new = make_basic_track(&conn, "New");
        conn.execute(
            "UPDATE tracks SET added_at = 999 WHERE file_path = '/music/New.mp3'",
            [],
        )
        .unwrap();
        let views = recently_added(&conn, 10).unwrap();
        assert_eq!(views[0].track.title, "New");
        assert_eq!(views[1].track.title, "Old");
    }

    #[test]
    fn set_favorite_toggles() {
        let conn = test_db();
        let id = make_basic_track(&conn, "Star");
        set_favorite(&conn, id, true, 1000).unwrap();
        let v = get_view_by_id(&conn, id).unwrap().unwrap();
        assert!(v.track.is_favorite);
        set_favorite(&conn, id, false, 2000).unwrap();
        let v = get_view_by_id(&conn, id).unwrap().unwrap();
        assert!(!v.track.is_favorite);
    }

    fn make_basic_artist_album(conn: &rusqlite::Connection) -> (i64, i64) {
        let now = 1000i64;
        conn.execute(
            "INSERT INTO artists (name, added_at, updated_at) VALUES ('A', ?1, ?1)",
            [now],
        )
        .unwrap();
        let aid = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO albums (name, album_artist_id, added_at, updated_at) VALUES ('Al', ?1, ?2, ?2)",
            rusqlite::params![aid, now],
        )
        .unwrap();
        let albid = conn.last_insert_rowid();
        (aid, albid)
    }

    #[test]
    fn insert_with_root_folder_round_trips() {
        let conn = test_db();
        // First create a scan_folder
        conn.execute(
            "INSERT INTO scan_folders (id, path, added_at) VALUES (10, '/m', 0)",
            [],
        )
        .unwrap();
        let (artist_id, album_id) = make_basic_artist_album(&conn);
        let nt = NewTrack {
            file_path: "/m/x.mp3".into(),
            file_size: 1,
            file_modified_at: 0,
            hash: Some("abcdef0123456789".into()),
            title: "X".into(),
            album_id: Some(album_id),
            primary_artist_id: Some(artist_id),
            album_artist_id: None,
            track_no: None,
            disc_no: None,
            year: None,
            genre: None,
            duration_ms: 1000,
            bitrate: None,
            sample_rate: None,
            channels: None,
            codec: None,
            root_folder_id: Some(10),
        };
        let id = insert(&conn, &nt, 1000).unwrap();
        let t: Track = conn
            .query_row(
                "SELECT t.id, t.file_path, t.file_size, t.file_modified_at, t.hash, t.title,
                    t.album_id, t.primary_artist_id, t.album_artist_id,
                    t.track_no, t.disc_no, t.year, t.genre,
                    t.duration_ms, t.bitrate, t.sample_rate, t.channels, t.codec,
                    t.is_favorite, t.play_count, t.last_played_at,
                    t.last_seen_at, t.missing_at, t.added_at, t.updated_at,
                    t.root_folder_id
               FROM tracks t WHERE t.id = ?1",
                [id],
                Track::from_row,
            )
            .unwrap();
        assert_eq!(t.root_folder_id, Some(10));
    }

    #[test]
    fn list_favorite_tracks_returns_only_favorites() {
        let conn = test_db();
        let t1 = make_basic_track(&conn, "Fav");
        let t2 = make_basic_track(&conn, "Other");
        set_favorite(&conn, t1, true, 100).unwrap();
        let favorites = list_favorite_tracks(&conn).unwrap();
        assert_eq!(favorites.len(), 1);
        assert_eq!(favorites[0].track.id, t1);
        assert_ne!(favorites[0].track.id, t2);
    }
}
