//! FTS5 search across track titles, album names, artist names.

use rusqlite::{params, Connection};

use crate::db::tracks::{Track, TrackView};
use crate::error::AppResult;

/// 搜索曲目。空 query 返回空结果（不要把"无"翻译成 MATCH '*' 触发 FTS5 错误）。
/// 用前缀匹配让用户输入"hel"也能命中"Hello World"。
pub fn search_tracks(conn: &Connection, query: &str, limit: i64) -> AppResult<Vec<TrackView>> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }
    let escaped = build_fts_query(trimmed);

    let mut stmt = conn.prepare(
        "SELECT t.id, t.file_path, t.file_size, t.file_modified_at, t.hash, t.title,
                t.album_id, t.primary_artist_id, t.album_artist_id,
                t.track_no, t.disc_no, t.year, t.genre,
                t.duration_ms, t.bitrate, t.sample_rate, t.channels, t.codec,
                t.is_favorite, t.play_count, t.last_played_at,
                t.last_seen_at, t.missing_at, t.added_at, t.updated_at,
                t.root_folder_id,
                al.name AS album_name,
                ar.name AS primary_artist_name
         FROM tracks_fts f
         JOIN tracks t ON t.id = f.rowid
         LEFT JOIN albums al ON al.id = t.album_id
         LEFT JOIN artists ar ON ar.id = t.primary_artist_id
         WHERE tracks_fts MATCH ?1 AND t.missing_at IS NULL
         ORDER BY rank
         LIMIT ?2",
    )?;
    let rows = stmt.query_map(params![escaped, limit], |row| {
        Ok(TrackView {
            track: Track::from_row_via_helper(row)?,
            album_name: row.get("album_name")?,
            primary_artist_name: row.get("primary_artist_name")?,
        })
    })?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

/// 把用户输入转成 FTS5 安全的查询字符串。
/// 策略：拆词 → 每个 token 用双引号包成 phrase + 末尾加 * 做前缀；token 之间用 AND（默认）。
/// 例: "hello world" → `"hello"* "world"*`
fn build_fts_query(input: &str) -> String {
    input
        .split_whitespace()
        .filter(|t| !t.is_empty())
        .map(|tok| {
            // 转义内嵌引号
            let safe = tok.replace('"', r#""""#);
            format!(r#""{safe}"*"#)
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{testing, testing::test_db, tracks};

    #[test]
    fn empty_query_returns_empty() {
        let conn = test_db();
        let _ = testing::make_basic_track(&conn, "Hello World");
        assert!(search_tracks(&conn, "", 10).unwrap().is_empty());
        assert!(search_tracks(&conn, "   ", 10).unwrap().is_empty());
    }

    #[test]
    fn finds_by_title_prefix() {
        let conn = test_db();
        let _ = testing::make_basic_track(&conn, "Hello World");
        let r = search_tracks(&conn, "hel", 10).unwrap();
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].track.title, "Hello World");
    }

    #[test]
    fn finds_by_artist_name() {
        let conn = test_db();
        let _ = testing::make_basic_track(&conn, "Generic");
        let r = search_tracks(&conn, "TestArtist", 10).unwrap();
        assert_eq!(r.len(), 1);
    }

    #[test]
    fn finds_by_album_name() {
        let conn = test_db();
        let _ = testing::make_basic_track(&conn, "Generic");
        let r = search_tracks(&conn, "TestAlbum", 10).unwrap();
        assert_eq!(r.len(), 1);
    }

    #[test]
    fn excludes_missing_tracks_from_results() {
        let conn = test_db();
        let id = testing::make_basic_track(&conn, "Hidden");
        tracks::mark_missing(&conn, &[id], 1000).unwrap();
        let r = search_tracks(&conn, "Hidden", 10).unwrap();
        assert!(r.is_empty(), "missing tracks must not appear in search");
    }

    #[test]
    fn quote_in_query_does_not_crash() {
        let conn = test_db();
        let _ = testing::make_basic_track(&conn, "Innocent");
        let r = search_tracks(&conn, r#"that "tricky" input"#, 10).unwrap();
        // 不强制结果数量，只确认不 panic 也不返回 Err
        let _ = r;
    }

    #[test]
    fn build_fts_query_escapes_internal_quotes() {
        let q = build_fts_query(r#"hello "world""#);
        assert!(q.contains(r#""hello"*"#));
        assert!(q.contains(r#""""world"""*"#)); // 双引号被转义
    }
}
