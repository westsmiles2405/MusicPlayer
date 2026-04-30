//! End-to-end scanner tests against real fixture audio files.

use musicplayer_lib::db::scan_folders as folders_repo;
use musicplayer_lib::db::Database;
use musicplayer_lib::library::scanner::{scan_folders, AbortFlag, ScanContext};
use rusqlite::Connection;
use std::path::PathBuf;
use tempfile::TempDir;

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/audio")
}

fn make_test_db() -> Database {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
    musicplayer_lib::db::schema::apply_pending(&conn).unwrap();
    Database::from_conn(conn)
}

fn setup() -> (TempDir, Database, PathBuf) {
    let db = make_test_db();
    let cover_dir = TempDir::new().unwrap();
    let fixtures = fixture_dir();
    folders_repo::add(&db.lock_conn(), fixtures.to_str().unwrap(), 1000).unwrap();
    (cover_dir, db, fixtures)
}

#[test]
fn fresh_scan_indexes_all_fixtures() {
    let (cover_dir, db, _fix) = setup();
    let ctx = ScanContext {
        db: &db,
        cache_dir: cover_dir.path().to_path_buf(),
    };
    let abort = AbortFlag::new();
    let report = scan_folders(&ctx, &abort, &|_| {}).unwrap();
    assert_eq!(report.added, 5, "expected 5 added, got {report:?}");
    assert!(
        report.errors.is_empty(),
        "unexpected errors: {:?}",
        report.errors
    );

    let count: i64 = db
        .lock_conn()
        .query_row("SELECT COUNT(*) FROM tracks", [], |r| r.get(0))
        .unwrap();
    assert_eq!(count, 5);
    let albums: i64 = db
        .lock_conn()
        .query_row("SELECT COUNT(*) FROM albums", [], |r| r.get(0))
        .unwrap();
    assert!(albums >= 2, "expected at least 2 albums, got {albums}");
}

#[test]
fn second_scan_is_all_unchanged() {
    let (cover_dir, db, _fix) = setup();
    let ctx = ScanContext {
        db: &db,
        cache_dir: cover_dir.path().to_path_buf(),
    };
    let abort = AbortFlag::new();
    let _ = scan_folders(&ctx, &abort, &|_| {}).unwrap();
    let report = scan_folders(&ctx, &abort, &|_| {}).unwrap();
    assert!(report.errors.is_empty(), "errors: {:?}", report.errors);
    assert_eq!(report.added, 0);
    assert_eq!(report.updated, 0);
    assert_eq!(report.unchanged, 5);
    assert_eq!(report.missing, 0);
}

#[test]
fn deleted_file_marked_missing_on_rescan() {
    // Use a temporary directory with a copy of a fixture file so we never
    // touch the shared fixture directory (avoids cross-test interference).
    let db = make_test_db();
    let cover_dir = TempDir::new().unwrap();
    let work_dir = TempDir::new().unwrap();
    let src = fixture_dir().join("a.mp3");
    let copy = work_dir.path().join("a.mp3");
    std::fs::copy(&src, &copy).unwrap();

    folders_repo::add(&db.lock_conn(), work_dir.path().to_str().unwrap(), 1000).unwrap();

    let ctx = ScanContext {
        db: &db,
        cache_dir: cover_dir.path().to_path_buf(),
    };
    let abort = AbortFlag::new();

    // First scan: find the copied file.
    let _ = scan_folders(&ctx, &abort, &|_| {}).unwrap();

    // Delete the file and scan again — it should be marked missing.
    std::fs::remove_file(&copy).unwrap();
    let res = scan_folders(&ctx, &abort, &|_| {}).unwrap();
    assert_eq!(res.missing, 1);
}

#[test]
fn progress_callback_fires_at_least_twice() {
    let (cover_dir, db, _fix) = setup();
    let ctx = ScanContext {
        db: &db,
        cache_dir: cover_dir.path().to_path_buf(),
    };
    let abort = AbortFlag::new();
    let count = std::sync::atomic::AtomicUsize::new(0);
    let _ = scan_folders(&ctx, &abort, &|_p| {
        count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    })
    .unwrap();
    assert!(
        count.load(std::sync::atomic::Ordering::Relaxed) >= 2,
        "progress callback fired {} times, expected >= 2",
        count.load(std::sync::atomic::Ordering::Relaxed)
    );
}
