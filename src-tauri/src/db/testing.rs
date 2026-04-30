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

/// 插入一条完整 track，返回 id。用于其他 repo 测试的 fixture。
pub(crate) fn make_basic_track(conn: &Connection, title: &str) -> i64 {
    let artist = crate::db::artists::upsert_by_name(conn, "TestArtist", 100).unwrap();
    let album = crate::db::albums::upsert(conn, "TestAlbum", artist, Some(2024), 100).unwrap();
    let nt = crate::db::tracks::NewTrack {
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
        root_folder_id: None,
    };
    crate::db::tracks::insert(conn, &nt, 100).unwrap()
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
