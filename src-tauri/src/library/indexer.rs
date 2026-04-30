#![allow(dead_code)]
//! Single-thread indexer: takes RawTrack -> upserts artist/album/track in an
//! externally-managed transaction.

use rusqlite::Connection;

use crate::db::{albums, artists, tracks};
use crate::error::AppResult;
use crate::metadata::reader::RawTrack;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpsertKind {
    Added,
    Updated,
    Moved,
    Unchanged,
}

#[derive(Debug, Clone)]
pub struct UpsertOutcome {
    pub kind: UpsertKind,
    pub track_id: i64,
}

/// Idempotent upsert. Caller wraps in transaction.
///
/// Resolution order (B1):
/// 1. file_path hit?
///    a. (size, mtime) match → Unchanged
///    b. (size, mtime) differ → Updated
/// 2. file_path miss; hash exists in db?
///    a. unique hit → Moved
///    b. multi-hit → fall through to Added
/// 3. otherwise → Added
pub fn upsert_track(
    conn: &Connection,
    raw: &RawTrack,
    cover_relpath: Option<&str>,
    scan_id: i64,
    root_folder_id: i64,
) -> AppResult<UpsertOutcome> {
    let path_str = raw.path.to_string_lossy().to_string();
    let primary_artist_id = artists::upsert_by_name_in_tx(conn, &raw.artists[0], scan_id)?;
    let album_artist_id = match &raw.album_artist {
        Some(name) => Some(artists::upsert_by_name_in_tx(conn, name, scan_id)?),
        None => None,
    };
    // 专辑艺人默认 fallback 到主艺人
    let album_artist_for_album = album_artist_id.unwrap_or(primary_artist_id);
    let album_id =
        albums::upsert_in_tx(conn, &raw.album, album_artist_for_album, raw.year, scan_id)?;
    if let Some(rel) = cover_relpath {
        albums::set_cover_path_in_tx(conn, album_id, rel, scan_id)?;
    }

    // Step 1: path hit?
    if let Some(existing) = tracks::find_by_path(conn, &path_str)? {
        if existing.file_size == raw.size_bytes && existing.file_modified_at == raw.mtime_ms {
            tracks::mark_present_in_tx(conn, existing.id, scan_id)?;
            return Ok(UpsertOutcome {
                kind: UpsertKind::Unchanged,
                track_id: existing.id,
            });
        }
        let nt = build_new_track(
            raw,
            album_id,
            primary_artist_id,
            album_artist_id,
            root_folder_id,
        );
        tracks::update_by_path_in_tx(conn, &nt, scan_id)?;
        return Ok(UpsertOutcome {
            kind: UpsertKind::Updated,
            track_id: existing.id,
        });
    }

    // Step 2: hash hit?
    let by_hash = tracks::find_ids_by_hash_size(conn, &raw.hash, raw.size_bytes)?;
    if by_hash.len() == 1 {
        let id = by_hash[0];
        tracks::update_path_for_move_in_tx(
            conn,
            id,
            &path_str,
            raw.size_bytes,
            raw.mtime_ms,
            scan_id,
            Some(root_folder_id),
        )?;
        return Ok(UpsertOutcome {
            kind: UpsertKind::Moved,
            track_id: id,
        });
    }

    // Step 3: Added.
    let nt = build_new_track(
        raw,
        album_id,
        primary_artist_id,
        album_artist_id,
        root_folder_id,
    );
    let id = tracks::insert_in_tx(conn, &nt, scan_id)?;
    Ok(UpsertOutcome {
        kind: UpsertKind::Added,
        track_id: id,
    })
}

fn build_new_track(
    raw: &RawTrack,
    album_id: i64,
    primary_artist_id: i64,
    album_artist_id: Option<i64>,
    root_folder_id: i64,
) -> tracks::NewTrack {
    tracks::NewTrack {
        file_path: raw.path.to_string_lossy().to_string(),
        file_size: raw.size_bytes,
        file_modified_at: raw.mtime_ms,
        hash: Some(raw.hash.clone()),
        title: raw.title.clone(),
        album_id: Some(album_id),
        primary_artist_id: Some(primary_artist_id),
        album_artist_id,
        track_no: raw.track_no,
        disc_no: raw.disc_no,
        year: raw.year,
        genre: raw.genre.clone(),
        duration_ms: raw.duration_ms,
        bitrate: raw.bitrate,
        sample_rate: raw.sample_rate,
        channels: raw.channels,
        codec: raw.codec.clone(),
        root_folder_id: Some(root_folder_id),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::testing::test_db;
    use std::path::PathBuf;

    fn raw(name: &str, hash: &str, size: i64, mtime: i64) -> RawTrack {
        RawTrack {
            path: PathBuf::from(name),
            hash: hash.into(),
            mtime_ms: mtime,
            size_bytes: size,
            title: name.trim_end_matches(".mp3").to_string(),
            artists: vec!["A1".into()],
            album: "Album1".into(),
            album_artist: Some("A1".into()),
            track_no: Some(1),
            disc_no: None,
            year: Some(2020),
            genre: None,
            duration_ms: 1000,
            bitrate: Some(128),
            sample_rate: Some(44100),
            channels: Some(2),
            codec: Some("MP3".into()),
            cover: None,
        }
    }

    fn fresh_root(conn: &Connection) -> i64 {
        conn.execute(
            "INSERT INTO scan_folders (path, added_at) VALUES ('/m', 0)",
            [],
        )
        .unwrap();
        conn.last_insert_rowid()
    }

    #[test]
    fn first_insert_is_added() {
        let mut conn = test_db();
        let root = fresh_root(&conn);
        let tx = conn.transaction().unwrap();
        let r = raw("/m/a.mp3", "h1", 100, 0);
        let out = upsert_track(&tx, &r, None, 1000, root).unwrap();
        assert_eq!(out.kind, UpsertKind::Added);
        tx.commit().unwrap();
    }

    #[test]
    fn second_insert_same_metadata_is_unchanged() {
        let mut conn = test_db();
        let root = fresh_root(&conn);
        let r = raw("/m/a.mp3", "h1", 100, 0);
        {
            let tx = conn.transaction().unwrap();
            upsert_track(&tx, &r, None, 1000, root).unwrap();
            tx.commit().unwrap();
        }
        {
            let tx = conn.transaction().unwrap();
            let out = upsert_track(&tx, &r, None, 2000, root).unwrap();
            assert_eq!(out.kind, UpsertKind::Unchanged);
            tx.commit().unwrap();
        }
    }

    #[test]
    fn changed_size_triggers_updated() {
        let mut conn = test_db();
        let root = fresh_root(&conn);
        let mut r = raw("/m/a.mp3", "h1", 100, 0);
        {
            let tx = conn.transaction().unwrap();
            upsert_track(&tx, &r, None, 1000, root).unwrap();
            tx.commit().unwrap();
        }
        r.size_bytes = 200;
        r.hash = "h2".into();
        let tx = conn.transaction().unwrap();
        let out = upsert_track(&tx, &r, None, 2000, root).unwrap();
        assert_eq!(out.kind, UpsertKind::Updated);
        tx.commit().unwrap();
    }

    #[test]
    fn moved_path_with_same_hash_is_moved() {
        let mut conn = test_db();
        let root = fresh_root(&conn);
        let r1 = raw("/m/a.mp3", "h1", 100, 0);
        {
            let tx = conn.transaction().unwrap();
            upsert_track(&tx, &r1, None, 1000, root).unwrap();
            tx.commit().unwrap();
        }
        let r2 = raw("/m/sub/a.mp3", "h1", 100, 0);
        let tx = conn.transaction().unwrap();
        let out = upsert_track(&tx, &r2, None, 2000, root).unwrap();
        assert_eq!(out.kind, UpsertKind::Moved);
        tx.commit().unwrap();
    }
}
