//! Database migration runner. Applies SQL migrations in order.
//! Tracks applied versions in the schema_migrations table.

use crate::error::AppResult;
use rusqlite::Connection;

/// 已编译进二进制的迁移列表，按版本号升序。新增 V_N 时在此追加一项。
const MIGRATIONS: &[(i64, &str)] = &[
    (1, include_str!("../../migrations/V1__init.sql")),
    (
        2,
        include_str!("../../migrations/V2__fix_fts5_triggers.sql"),
    ),
    (
        3,
        include_str!("../../migrations/V3__scanner_support.sql"),
    ),
];

/// 按 schema_migrations 表中的最新版本号增量执行所有未应用迁移。
/// 调用幂等：再调一次什么都不会发生。
pub fn apply_pending(conn: &Connection) -> AppResult<()> {
    ensure_migrations_table(conn)?;
    let applied = max_applied_version(conn)?;
    for &(version, sql) in MIGRATIONS {
        if version <= applied {
            continue;
        }
        log::info!("applying migration V{version}");
        let tx = conn.unchecked_transaction()?;
        tx.execute_batch(sql)?;
        tx.execute(
            "INSERT INTO schema_migrations (version, applied_at) VALUES (?1, ?2)",
            rusqlite::params![version, now_unix_ms()],
        )?;
        tx.commit()?;
    }
    Ok(())
}

fn ensure_migrations_table(conn: &Connection) -> AppResult<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            applied_at INTEGER NOT NULL
        );",
    )?;
    Ok(())
}

fn max_applied_version(conn: &Connection) -> AppResult<i64> {
    let v: Option<i64> = conn
        .query_row("SELECT MAX(version) FROM schema_migrations", [], |r| {
            r.get(0)
        })
        .ok()
        .flatten();
    Ok(v.unwrap_or(0))
}

fn now_unix_ms() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_run_applies_v1() {
        let conn = Connection::open_in_memory().unwrap();
        apply_pending(&conn).unwrap();
        let v: i64 = conn
            .query_row("SELECT MAX(version) FROM schema_migrations", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert!(v >= 1);
    }

    #[test]
    fn second_run_is_noop() {
        let conn = Connection::open_in_memory().unwrap();
        apply_pending(&conn).unwrap();
        let count_before: i64 = conn
            .query_row("SELECT COUNT(*) FROM schema_migrations", [], |r| r.get(0))
            .unwrap();
        apply_pending(&conn).unwrap();
        let count_after: i64 = conn
            .query_row("SELECT COUNT(*) FROM schema_migrations", [], |r| r.get(0))
            .unwrap();
        assert_eq!(
            count_before, count_after,
            "second run must not insert duplicate row"
        );
    }

    #[test]
    fn all_v1_tables_created() {
        let conn = Connection::open_in_memory().unwrap();
        apply_pending(&conn).unwrap();
        for table in [
            "artists",
            "albums",
            "tracks",
            "track_artists",
            "playlists",
            "playlist_tracks",
            "play_history",
            "scan_folders",
            "app_state",
            "tracks_fts",
        ] {
            let n: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM sqlite_master WHERE name = ?1",
                    rusqlite::params![table],
                    |r| r.get(0),
                )
                .unwrap();
            assert!(n >= 1, "expected '{table}' to exist");
        }
    }

    #[test]
    fn applies_through_v3() {
        let conn = Connection::open_in_memory().unwrap();
        apply_pending(&conn).unwrap();
        let v: i64 = conn
            .query_row("SELECT MAX(version) FROM schema_migrations", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(v, 3);
    }

    #[test]
    fn track_insert_does_not_rebuild_full_fts() {
        // V2 行级触发器：插入一行只该新增一条 FTS 记录，不该重建整个表。
        let conn = Connection::open_in_memory().unwrap();
        apply_pending(&conn).unwrap();
        let now = 0_i64;
        conn.execute(
            "INSERT INTO artists (id, name, added_at, updated_at) VALUES (1, 'A', ?1, ?1)",
            rusqlite::params![now],
        )
        .unwrap();
        conn.execute("INSERT INTO albums (id, name, album_artist_id, added_at, updated_at) VALUES (1, 'Alb', 1, ?1, ?1)", rusqlite::params![now]).unwrap();
        conn.execute(
            "INSERT INTO tracks (id, file_path, file_size, file_modified_at, title, album_id, primary_artist_id, duration_ms, last_seen_at, added_at, updated_at)
             VALUES (1, '/x.mp3', 0, 0, 'Hello World', 1, 1, 1000, ?1, ?1, ?1)",
            rusqlite::params![now]
        ).unwrap();
        let n: i64 = conn
            .query_row("SELECT COUNT(*) FROM tracks_fts", [], |r| r.get(0))
            .unwrap();
        assert_eq!(n, 1);
        let title: String = conn
            .query_row("SELECT title FROM tracks_fts WHERE rowid=1", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(title, "Hello World");
    }

    #[test]
    fn applies_v3_and_creates_root_folder_column() {
        let conn = Connection::open_in_memory().unwrap();
        apply_pending(&conn).unwrap();
        let v: i64 = conn
            .query_row("SELECT MAX(version) FROM schema_migrations", [], |r| r.get(0))
            .unwrap();
        assert_eq!(v, 3, "expected V3 applied, got version={v}");

        let cols: Vec<String> = conn
            .prepare("PRAGMA table_info(tracks)")
            .unwrap()
            .query_map([], |r| r.get::<_, String>(1))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();
        assert!(cols.contains(&"root_folder_id".to_string()));

        let idx: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='index' AND tbl_name='tracks'")
            .unwrap()
            .query_map([], |r| r.get::<_, String>(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();
        assert!(idx.iter().any(|n| n == "idx_tracks_hash"));
        assert!(idx.iter().any(|n| n == "idx_tracks_root_folder"));
    }

    #[test]
    fn fts_update_replaces_old_term_after_v3() {
        let conn = Connection::open_in_memory().unwrap();
        apply_pending(&conn).unwrap();
        let now = 1000i64;
        conn.execute(
            "INSERT INTO artists (id, name, added_at, updated_at) VALUES (1, 'Alpha', ?1, ?1)",
            [now],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO albums (id, name, album_artist_id, added_at, updated_at)
             VALUES (1, 'Album1', 1, ?1, ?1)",
            [now],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO tracks (
                id, file_path, file_size, file_modified_at, title,
                album_id, primary_artist_id, duration_ms,
                last_seen_at, added_at, updated_at
             ) VALUES (1, '/x/a.mp3', 1, 0, 'Song', 1, 1, 1000, ?1, ?1, ?1)",
            [now],
        )
        .unwrap();

        let hits: Vec<i64> = conn
            .prepare("SELECT rowid FROM tracks_fts WHERE tracks_fts MATCH 'Song'")
            .unwrap()
            .query_map([], |r| r.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();
        assert_eq!(hits, vec![1]);

        conn.execute(
            "UPDATE tracks SET title='Renamed', updated_at=?1 WHERE id=1",
            [now + 1],
        )
        .unwrap();

        let old: Vec<i64> = conn
            .prepare("SELECT rowid FROM tracks_fts WHERE tracks_fts MATCH 'Song'")
            .unwrap()
            .query_map([], |r| r.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();
        assert!(old.is_empty(), "old term still searchable after UPDATE");

        let new_hits: Vec<i64> = conn
            .prepare("SELECT rowid FROM tracks_fts WHERE tracks_fts MATCH 'Renamed'")
            .unwrap()
            .query_map([], |r| r.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();
        assert_eq!(new_hits, vec![1]);
    }

    #[test]
    fn fts_missing_track_excluded_from_search() {
        let conn = Connection::open_in_memory().unwrap();
        apply_pending(&conn).unwrap();
        let now = 1000i64;
        conn.execute(
            "INSERT INTO artists (id, name, added_at, updated_at) VALUES (1, 'A', ?1, ?1)",
            [now],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO albums (id, name, album_artist_id, added_at, updated_at)
             VALUES (1, 'Al', 1, ?1, ?1)",
            [now],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO tracks (
                id, file_path, file_size, file_modified_at, title,
                album_id, primary_artist_id, duration_ms,
                last_seen_at, added_at, updated_at
             ) VALUES (1, '/x/y.mp3', 1, 0, 'Hidden', 1, 1, 1000, ?1, ?1, ?1)",
            [now],
        )
        .unwrap();
        conn.execute(
            "UPDATE tracks SET missing_at=?1, updated_at=?1 WHERE id=1",
            [now + 1],
        )
        .unwrap();
        let hits: Vec<i64> = conn
            .prepare("SELECT rowid FROM tracks_fts WHERE tracks_fts MATCH 'Hidden'")
            .unwrap()
            .query_map([], |r| r.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();
        assert!(hits.is_empty(), "missing track still searchable");
    }
}
