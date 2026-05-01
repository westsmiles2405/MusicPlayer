//! Playlist queries.

use rusqlite::{params, Connection, Row};
use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Playlist {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub cover_path: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistSummary {
    #[serde(flatten)]
    pub playlist: Playlist,
    pub track_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistTrackView {
    #[serde(flatten)]
    pub track: crate::db::tracks::TrackView,
    pub playlist_position: i64,
}

fn validate_name(name: &str) -> AppResult<&str> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(AppError::InvalidInput(
            "playlist name cannot be empty".into(),
        ));
    }
    Ok(trimmed)
}

fn compact_positions(conn: &Connection, playlist_id: i64) -> AppResult<()> {
    let mut stmt = conn.prepare(
        "SELECT rowid, position FROM playlist_tracks
         WHERE playlist_id = ?1
         ORDER BY position, added_at, rowid",
    )?;
    let rows: Vec<(i64, i64)> = stmt
        .query_map(params![playlist_id], |row| Ok((row.get(0)?, row.get(1)?)))?
        .collect::<Result<_, _>>()?;
    drop(stmt);

    for (new_pos, (rowid, _old_pos)) in rows.into_iter().enumerate() {
        conn.execute(
            "UPDATE playlist_tracks
             SET position = ?2
             WHERE rowid = ?1",
            params![rowid, new_pos as i64],
        )?;
    }
    Ok(())
}

impl Playlist {
    fn from_row(row: &Row<'_>) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            name: row.get("name")?,
            description: row.get("description")?,
            cover_path: row.get("cover_path")?,
            created_at: row.get("created_at")?,
            updated_at: row.get("updated_at")?,
        })
    }
}

pub fn create(
    conn: &Connection,
    name: &str,
    description: Option<&str>,
    now_ms: i64,
) -> AppResult<i64> {
    let name = validate_name(name)?;
    conn.execute(
        "INSERT INTO playlists (name, description, created_at, updated_at) VALUES (?1, ?2, ?3, ?3)",
        params![name, description, now_ms],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn delete(conn: &Connection, id: i64) -> AppResult<()> {
    let n = conn.execute("DELETE FROM playlists WHERE id = ?1", params![id])?;
    if n == 0 {
        return Err(AppError::NotFound(format!("playlist {id}")));
    }
    Ok(())
}

pub fn rename(conn: &Connection, id: i64, name: &str, now_ms: i64) -> AppResult<()> {
    let name = validate_name(name)?;
    let n = conn.execute(
        "UPDATE playlists SET name = ?1, updated_at = ?2 WHERE id = ?3",
        params![name, now_ms, id],
    )?;
    if n == 0 {
        return Err(AppError::NotFound(format!("playlist {id}")));
    }
    Ok(())
}

pub fn list(conn: &Connection) -> AppResult<Vec<PlaylistSummary>> {
    let mut stmt = conn.prepare(
        "SELECT p.id, p.name, p.description, p.cover_path, p.created_at, p.updated_at,
                (SELECT COUNT(*) FROM playlist_tracks pt WHERE pt.playlist_id = p.id) AS track_count
         FROM playlists p
         ORDER BY p.updated_at DESC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(PlaylistSummary {
            playlist: Playlist::from_row(row)?,
            track_count: row.get("track_count")?,
        })
    })?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

pub fn search_by_name(
    conn: &Connection,
    query: &str,
    limit: i64,
) -> AppResult<Vec<PlaylistSummary>> {
    let pattern = format!("%{query}%");
    let mut stmt = conn.prepare(
        "SELECT p.id, p.name, p.description, p.cover_path, p.created_at, p.updated_at,
                (SELECT COUNT(*) FROM playlist_tracks pt WHERE pt.playlist_id = p.id) AS track_count
         FROM playlists p
         WHERE p.name LIKE ?1
         ORDER BY p.name COLLATE NOCASE
         LIMIT ?2",
    )?;
    let rows = stmt.query_map(params![pattern, limit.max(1)], |row| {
        Ok(PlaylistSummary {
            playlist: Playlist::from_row(row)?,
            track_count: row.get("track_count")?,
        })
    })?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

pub fn get_tracks(conn: &Connection, playlist_id: i64) -> AppResult<Vec<PlaylistTrackView>> {
    use crate::db::tracks::{Track, TrackView};
    let mut stmt = conn.prepare(
        "SELECT t.id, t.file_path, t.file_size, t.file_modified_at, t.hash, t.title,
                t.album_id, t.primary_artist_id, t.album_artist_id,
                t.track_no, t.disc_no, t.year, t.genre,
                t.duration_ms, t.bitrate, t.sample_rate, t.channels, t.codec,
                t.is_favorite, t.play_count, t.last_played_at,
                t.last_seen_at, t.missing_at, t.added_at, t.updated_at,
                t.root_folder_id,
                al.name AS album_name,
                ar.name AS primary_artist_name,
                pt.position AS playlist_position
         FROM playlist_tracks pt
         JOIN tracks t ON t.id = pt.track_id
         LEFT JOIN albums al ON al.id = t.album_id
         LEFT JOIN artists ar ON ar.id = t.primary_artist_id
         WHERE pt.playlist_id = ?1
         ORDER BY pt.position",
    )?;
    let rows = stmt.query_map(params![playlist_id], |row| {
        Ok(PlaylistTrackView {
            track: TrackView {
                track: Track::from_row_via_helper(row)?,
                album_name: row.get("album_name")?,
                primary_artist_name: row.get("primary_artist_name")?,
            },
            playlist_position: row.get("playlist_position")?,
        })
    })?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

pub fn append_track(
    conn: &Connection,
    playlist_id: i64,
    track_id: i64,
    now_ms: i64,
) -> AppResult<i64> {
    let next_pos: i64 = conn.query_row(
        "SELECT COALESCE(MAX(position), -1) + 1 FROM playlist_tracks WHERE playlist_id = ?1",
        params![playlist_id],
        |r| r.get(0),
    )?;
    conn.execute(
        "INSERT INTO playlist_tracks (playlist_id, track_id, position, added_at) VALUES (?1, ?2, ?3, ?4)",
        params![playlist_id, track_id, next_pos, now_ms],
    )?;
    Ok(next_pos)
}

pub fn remove_track(
    conn: &Connection,
    playlist_id: i64,
    track_id: i64,
    position: i64,
) -> AppResult<()> {
    let n = conn.execute(
        "DELETE FROM playlist_tracks WHERE playlist_id = ?1 AND track_id = ?2 AND position = ?3",
        params![playlist_id, track_id, position],
    )?;
    if n == 0 {
        return Err(AppError::NotFound(format!(
            "playlist_track ({playlist_id},{track_id},pos={position})"
        )));
    }
    compact_positions(conn, playlist_id)?;
    Ok(())
}

pub fn reorder(
    conn: &Connection,
    playlist_id: i64,
    source_position: i64,
    destination_position: i64,
) -> AppResult<()> {
    compact_positions(conn, playlist_id)?;

    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM playlist_tracks WHERE playlist_id = ?1",
        params![playlist_id],
        |r| r.get(0),
    )?;

    if source_position < 0
        || destination_position < 0
        || source_position >= count
        || destination_position >= count
    {
        return Err(AppError::InvalidInput(format!(
            "reorder out of bounds: {source_position} -> {destination_position} (count={count})"
        )));
    }

    if source_position == destination_position {
        return Ok(());
    }

    // Select rowid of the moved row
    let rowid: i64 = conn.query_row(
        "SELECT rowid FROM playlist_tracks
         WHERE playlist_id = ?1 AND position = ?2",
        params![playlist_id, source_position],
        |r| r.get(0),
    )?;

    let tx = conn.unchecked_transaction()?;

    // Move to temporary position
    tx.execute(
        "UPDATE playlist_tracks SET position = -1 WHERE rowid = ?1",
        params![rowid],
    )?;

    // Shift intervening rows
    if source_position < destination_position {
        tx.execute(
            "UPDATE playlist_tracks SET position = position - 1
             WHERE playlist_id = ?1 AND position > ?2 AND position <= ?3",
            params![playlist_id, source_position, destination_position],
        )?;
    } else {
        tx.execute(
            "UPDATE playlist_tracks SET position = position + 1
             WHERE playlist_id = ?1 AND position >= ?2 AND position < ?3",
            params![playlist_id, destination_position, source_position],
        )?;
    }

    // Move to destination
    tx.execute(
        "UPDATE playlist_tracks SET position = ?2 WHERE rowid = ?1",
        params![rowid, destination_position],
    )?;

    tx.commit()?;
    compact_positions(conn, playlist_id)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{testing, testing::test_db, tracks};

    fn setup_with_three_tracks(conn: &Connection) -> (i64, [i64; 3]) {
        let pid = create(conn, "Mix", None, 100).unwrap();
        let t1 = testing::make_basic_track(conn, "T1");
        let artist = crate::db::artists::find_by_name(conn, "TestArtist")
            .unwrap()
            .unwrap()
            .id;
        let album = crate::db::albums::upsert(conn, "TestAlbum", artist, Some(2024), 100).unwrap();
        let mk = |path: &str, title: &str| -> i64 {
            let nt = crate::db::tracks::NewTrack {
                file_path: path.into(),
                file_size: 1,
                file_modified_at: 0,
                hash: None,
                title: title.into(),
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
            crate::db::tracks::insert(conn, &nt, 100).unwrap()
        };
        let t2 = mk("/m/T2.mp3", "T2");
        let t3 = mk("/m/T3.mp3", "T3");
        append_track(conn, pid, t1, 100).unwrap();
        append_track(conn, pid, t2, 100).unwrap();
        append_track(conn, pid, t3, 100).unwrap();
        (pid, [t1, t2, t3])
    }

    // ── original tests preserved ────────────────────────────────

    #[test]
    fn create_and_list() {
        let conn = test_db();
        let pid = create(&conn, "Favs", Some("My favorites"), 100).unwrap();
        let lst = list(&conn).unwrap();
        assert_eq!(lst.len(), 1);
        assert_eq!(lst[0].playlist.id, pid);
        assert_eq!(lst[0].playlist.name, "Favs");
        assert_eq!(lst[0].track_count, 0);
    }

    #[test]
    fn append_track_increments_position() {
        let conn = test_db();
        let (pid, ids) = setup_with_three_tracks(&conn);
        let positions: Vec<i64> = conn
            .prepare("SELECT position FROM playlist_tracks WHERE playlist_id=?1 ORDER BY position")
            .unwrap()
            .query_map(params![pid], |r| r.get(0))
            .unwrap()
            .map(|r| r.unwrap())
            .collect();
        assert_eq!(positions, vec![0, 1, 2]);
        let views = get_tracks(&conn, pid).unwrap();
        assert_eq!(views[0].track.track.id, ids[0]);
        assert_eq!(views[2].track.track.id, ids[2]);
    }

    #[test]
    fn reorder_moves_row_and_shifts_others() {
        let conn = test_db();
        let (pid, ids) = setup_with_three_tracks(&conn);
        reorder(&conn, pid, 0, 2).unwrap();
        let views = get_tracks(&conn, pid).unwrap();
        let titles: Vec<&str> = views.iter().map(|v| v.track.track.title.as_str()).collect();
        assert_eq!(titles, vec!["T2", "T3", "T1"]);
        let new_t1_pos = conn
            .query_row(
                "SELECT position FROM playlist_tracks WHERE playlist_id=?1 AND track_id=?2",
                params![pid, ids[0]],
                |r| r.get::<_, i64>(0),
            )
            .unwrap();
        reorder(&conn, pid, new_t1_pos, 0).unwrap();
        let views = get_tracks(&conn, pid).unwrap();
        let titles: Vec<&str> = views.iter().map(|v| v.track.track.title.as_str()).collect();
        assert_eq!(titles, vec!["T1", "T2", "T3"]);
    }

    #[test]
    fn delete_cascades_playlist_tracks() {
        let conn = test_db();
        let (pid, _) = setup_with_three_tracks(&conn);
        delete(&conn, pid).unwrap();
        let n: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM playlist_tracks WHERE playlist_id=?1",
                params![pid],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(n, 0);
    }

    #[test]
    fn rename_updates_row() {
        let conn = test_db();
        let pid = create(&conn, "Old", None, 100).unwrap();
        rename(&conn, pid, "New", 200).unwrap();
        let n: String = conn
            .query_row(
                "SELECT name FROM playlists WHERE id=?1",
                params![pid],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(n, "New");
    }

    // ── new v0.5.0 tests ────────────────────────────────────────

    #[test]
    fn create_rejects_empty_name_after_trim() {
        let conn = test_db();
        let err = create(&conn, "   ", None, 100).unwrap_err();
        assert!(matches!(err, AppError::InvalidInput(_)));
    }

    #[test]
    fn rename_rejects_empty_name_after_trim() {
        let conn = test_db();
        let pid = create(&conn, "Mix", None, 100).unwrap();
        let err = rename(&conn, pid, "   ", 200).unwrap_err();
        assert!(matches!(err, AppError::InvalidInput(_)));
    }

    #[test]
    fn get_tracks_includes_playlist_position() {
        let conn = test_db();
        let (pid, ids) = setup_with_three_tracks(&conn);
        let views = get_tracks(&conn, pid).unwrap();
        assert_eq!(views.len(), 3);
        assert_eq!(views[0].track.track.id, ids[0]);
        assert_eq!(views[0].playlist_position, 0);
        assert_eq!(views[1].playlist_position, 1);
        assert_eq!(views[2].playlist_position, 2);
    }

    #[test]
    fn get_tracks_keeps_missing_tracks_for_playlists() {
        let conn = test_db();
        let (pid, ids) = setup_with_three_tracks(&conn);
        tracks::mark_missing(&conn, &[ids[1]], 300).unwrap();
        let views = get_tracks(&conn, pid).unwrap();
        assert_eq!(views.len(), 3);
        assert!(views[1].track.track.missing_at.is_some());
        assert_eq!(views[1].playlist_position, 1);
    }

    #[test]
    fn remove_track_compacts_remaining_positions() {
        let conn = test_db();
        let (pid, ids) = setup_with_three_tracks(&conn);
        remove_track(&conn, pid, ids[1], 1).unwrap();
        let views = get_tracks(&conn, pid).unwrap();
        assert_eq!(views.len(), 2);
        assert_eq!(views[0].track.track.id, ids[0]);
        assert_eq!(views[0].playlist_position, 0);
        assert_eq!(views[1].track.track.id, ids[2]);
        assert_eq!(views[1].playlist_position, 1);
    }

    #[test]
    fn reorder_rejects_out_of_bounds_without_changing_order() {
        let conn = test_db();
        let (pid, _) = setup_with_three_tracks(&conn);
        let err = reorder(&conn, pid, 0, 99).unwrap_err();
        assert!(matches!(err, AppError::InvalidInput(_)));
        let titles: Vec<String> = get_tracks(&conn, pid)
            .unwrap()
            .into_iter()
            .map(|v| v.track.track.title)
            .collect();
        assert_eq!(titles, vec!["T1", "T2", "T3"]);
    }

    #[test]
    fn compact_positions_handles_duplicate_track_rows() {
        let conn = test_db();
        let (pid, ids) = setup_with_three_tracks(&conn);
        append_track(&conn, pid, ids[0], 400).unwrap();
        remove_track(&conn, pid, ids[1], 1).unwrap();
        let views = get_tracks(&conn, pid).unwrap();
        let positions: Vec<i64> = views.iter().map(|v| v.playlist_position).collect();
        let ids_after: Vec<i64> = views.iter().map(|v| v.track.track.id).collect();
        assert_eq!(positions, vec![0, 1, 2]);
        assert_eq!(ids_after, vec![ids[0], ids[2], ids[0]]);
    }
}
