#![allow(dead_code)]
//! Library scanner public types.
//! 实际扫描编排在 Task 3.3 实现，本任务只确立 IPC 边界用的 DTO。

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::db::scan_folders as folders_repo;
use crate::db::Database;
use crate::error::AppResult;
use crate::library::indexer;
use crate::metadata::art;
use crate::metadata::reader::{self, RawTrack};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ScanProgress {
    pub done: usize,
    pub total: usize,
    pub current_file: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ScanReport {
    pub added: usize,
    pub updated: usize,
    pub moved: usize,
    pub unchanged: usize,
    pub missing: usize,
    pub errors: Vec<ScanError>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ScanError {
    pub path: String,
    pub message: String,
}

#[derive(Debug, Clone, Default)]
pub struct AbortFlag(Arc<AtomicBool>);

impl AbortFlag {
    pub fn new() -> Self {
        Self(Arc::new(AtomicBool::new(false)))
    }
    pub fn signal(&self) {
        self.0.store(true, Ordering::Relaxed);
    }
    pub fn is_aborted(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }
    pub fn reset(&self) {
        self.0.store(false, Ordering::Relaxed);
    }
}

const AUDIO_EXTS: &[&str] = &["mp3", "m4a", "flac", "wav", "aac"];

pub fn canonicalize_roots(roots: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut canon: Vec<_> = roots
        .into_iter()
        .filter_map(|p| p.canonicalize().ok())
        .collect();
    canon.sort();
    let mut out: Vec<PathBuf> = Vec::new();
    for p in canon {
        if !out.iter().any(|r| p.starts_with(r)) {
            out.push(p);
        }
    }
    out
}

pub fn walk_audio_files(roots: &[PathBuf]) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for root in roots {
        for entry in walkdir::WalkDir::new(root)
            .follow_links(false)
            .into_iter()
            .filter_entry(|e| e.depth() == 0 || !is_hidden(e.path()))
            .filter_map(Result::ok)
        {
            if !entry.file_type().is_file() {
                continue;
            }
            if has_audio_ext(entry.path()) {
                out.push(entry.into_path());
            }
        }
    }
    out
}

fn is_hidden(p: &Path) -> bool {
    p.file_name()
        .and_then(|n| n.to_str())
        .map(|n| n.starts_with('.'))
        .unwrap_or(false)
}

fn has_audio_ext(p: &Path) -> bool {
    p.extension()
        .and_then(|e| e.to_str())
        .map(|e| {
            let lower = e.to_ascii_lowercase();
            AUDIO_EXTS.contains(&lower.as_str())
        })
        .unwrap_or(false)
}

// ---------------------------------------------------------------------------
// Scanner orchestrator (Task 3.3)
// ---------------------------------------------------------------------------

pub struct ScanContext<'a> {
    pub db: &'a Database,
    pub cache_dir: PathBuf,
}

const TXN_BATCH: usize = 50;

pub fn scan_folders(
    ctx: &ScanContext<'_>,
    abort: &AbortFlag,
    on_progress: &(dyn Fn(ScanProgress) + Sync),
) -> AppResult<ScanReport> {
    abort.reset();
    let conn_guard = ctx.db.lock_conn();
    let conn: &rusqlite::Connection = &conn_guard;

    let folders = folders_repo::list(conn)?;
    if folders.is_empty() {
        return Ok(ScanReport::default());
    }

    let canon = canonicalize_roots(folders.iter().map(|f| PathBuf::from(&f.path)).collect());
    let canon_to_id: Vec<(PathBuf, i64)> = folders
        .iter()
        .filter_map(|f| {
            PathBuf::from(&f.path)
                .canonicalize()
                .ok()
                .map(|c| (c, f.id))
        })
        .collect();

    let files = walk_audio_files(&canon);
    let total = files.len();
    on_progress(ScanProgress {
        done: 0,
        total,
        current_file: None,
    });

    let mut report = ScanReport::default();
    let mut buffer: Vec<Result<RawTrack, ScanError>> = Vec::with_capacity(TXN_BATCH);
    let mut done = 0usize;
    let scan_id = now_ms();

    for path in &files {
        if abort.is_aborted() {
            break;
        }
        let display_path = path.to_string_lossy().to_string();
        match reader::read_track(path) {
            Ok(raw) => buffer.push(Ok(raw)),
            Err(e) => buffer.push(Err(ScanError {
                path: display_path.clone(),
                message: format!("{e}"),
            })),
        }
        done += 1;

        if buffer.len() >= TXN_BATCH || done == total {
            commit_batch(
                conn,
                &ctx.cache_dir,
                &canon_to_id,
                &mut buffer,
                scan_id,
                &mut report,
            )?;
            on_progress(ScanProgress {
                done,
                total,
                current_file: Some(display_path),
            });
        }
    }

    // Flush remaining if abort interrupted mid-batch
    if !buffer.is_empty() {
        commit_batch(
            conn,
            &ctx.cache_dir,
            &canon_to_id,
            &mut buffer,
            scan_id,
            &mut report,
        )?;
    }

    // Mark missing within scanned roots.
    // Use scan_id (the stamp on last_seen_at) as the threshold so tracks just
    // scanned are NOT marked missing; only tracks from prior scans are.
    let scanned_ids: Vec<i64> = canon_to_id.iter().map(|(_, id)| *id).collect();
    if !scanned_ids.is_empty() && !abort.is_aborted() {
        let missing = mark_missing_scanned_roots(conn, &scanned_ids, scan_id)?;
        report.missing = missing;
        for fid in &scanned_ids {
            folders_repo::update_last_scanned(conn, *fid, scan_id).ok();
        }
    }

    on_progress(ScanProgress {
        done,
        total,
        current_file: None,
    });
    Ok(report)
}

fn commit_batch(
    conn: &rusqlite::Connection,
    cache_dir: &Path,
    canon_to_id: &[(PathBuf, i64)],
    buffer: &mut Vec<Result<RawTrack, ScanError>>,
    scan_id: i64,
    report: &mut ScanReport,
) -> AppResult<()> {
    let tx = conn.unchecked_transaction()?;
    for item in buffer.drain(..) {
        match item {
            Ok(raw) => {
                let root_id = match find_root_for_path(&raw.path, canon_to_id) {
                    Some(id) => id,
                    None => {
                        report.errors.push(ScanError {
                            path: raw.path.to_string_lossy().to_string(),
                            message: "no matching scan_folder root".into(),
                        });
                        continue;
                    }
                };
                let cover_rel = if let Some(bytes) = &raw.cover {
                    art::cache_cover_bytes(bytes, cache_dir)
                        .map_err(|e| {
                            report.errors.push(ScanError {
                                path: raw.path.to_string_lossy().to_string(),
                                message: format!("cover: {e}"),
                            });
                        })
                        .ok()
                } else {
                    None
                };
                match indexer::upsert_track(&tx, &raw, cover_rel.as_deref(), scan_id, root_id) {
                    Ok(out) => match out.kind {
                        indexer::UpsertKind::Added => report.added += 1,
                        indexer::UpsertKind::Updated => report.updated += 1,
                        indexer::UpsertKind::Moved => report.moved += 1,
                        indexer::UpsertKind::Unchanged => report.unchanged += 1,
                    },
                    Err(e) => report.errors.push(ScanError {
                        path: raw.path.to_string_lossy().to_string(),
                        message: format!("upsert: {e}"),
                    }),
                }
            }
            Err(scan_err) => report.errors.push(scan_err),
        }
    }
    tx.commit()?;
    Ok(())
}

fn find_root_for_path(p: &Path, canon_to_id: &[(PathBuf, i64)]) -> Option<i64> {
    canon_to_id
        .iter()
        .filter(|(root, _)| p.starts_with(root))
        .max_by_key(|(root, _)| root.as_os_str().len())
        .map(|(_, id)| *id)
}

fn mark_missing_scanned_roots(
    conn: &rusqlite::Connection,
    root_ids: &[i64],
    now_ms: i64,
) -> AppResult<usize> {
    let placeholders = std::iter::repeat_n("?", root_ids.len())
        .collect::<Vec<_>>()
        .join(",");
    let sql = format!(
        "UPDATE tracks SET missing_at = ?1, updated_at = ?1
          WHERE missing_at IS NULL
            AND last_seen_at < ?1
            AND root_folder_id IN ({placeholders})"
    );
    let mut stmt = conn.prepare(&sql)?;
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = vec![Box::new(now_ms)];
    for id in root_ids {
        params.push(Box::new(*id));
    }
    Ok(stmt.execute(rusqlite::params_from_iter(
        params.iter().map(|p| p.as_ref()),
    ))?)
}

fn now_ms() -> i64 {
    use std::sync::atomic::AtomicI64;
    static LAST: AtomicI64 = AtomicI64::new(0);
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);
    // Ensure strictly-increasing values; rapid successive calls within the
    // same millisecond must not return the same stamp.
    loop {
        let prev = LAST.load(std::sync::atomic::Ordering::Relaxed);
        let next = std::cmp::max(now, prev + 1);
        if LAST
            .compare_exchange(
                prev,
                next,
                std::sync::atomic::Ordering::Relaxed,
                std::sync::atomic::Ordering::Relaxed,
            )
            .is_ok()
        {
            return next;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scan_progress_serializes_camel_case() {
        let p = ScanProgress {
            done: 1,
            total: 5,
            current_file: Some("a.mp3".into()),
        };
        let json = serde_json::to_string(&p).unwrap();
        assert!(json.contains(r#""currentFile":"a.mp3""#));
    }

    #[test]
    fn scan_report_default_is_zero() {
        let r = ScanReport::default();
        assert_eq!(r.added, 0);
        assert!(r.errors.is_empty());
    }

    #[test]
    fn abort_flag_round_trip() {
        let f = AbortFlag::new();
        assert!(!f.is_aborted());
        f.signal();
        assert!(f.is_aborted());
        f.reset();
        assert!(!f.is_aborted());
    }

    #[test]
    fn abort_flag_clones_share_state() {
        let a = AbortFlag::new();
        let b = a.clone();
        a.signal();
        assert!(b.is_aborted());
    }

    use std::path::PathBuf;
    use tempfile::TempDir;

    #[test]
    fn canonicalize_dedups_nested_roots() {
        let dir = TempDir::new().unwrap();
        let parent = dir.path().to_path_buf();
        let child = parent.join("sub");
        std::fs::create_dir_all(&child).unwrap();
        let out = canonicalize_roots(vec![child.clone(), parent.clone()]);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0], parent.canonicalize().unwrap());
    }

    #[test]
    fn canonicalize_keeps_disjoint_roots() {
        let a = TempDir::new().unwrap();
        let b = TempDir::new().unwrap();
        let out = canonicalize_roots(vec![a.path().to_path_buf(), b.path().to_path_buf()]);
        assert_eq!(out.len(), 2);
    }

    #[test]
    fn canonicalize_drops_nonexistent() {
        let out = canonicalize_roots(vec![PathBuf::from("/no/such/path/v0_3_0_test")]);
        assert!(out.is_empty());
    }

    #[test]
    fn walk_finds_audio_files_recursively() {
        let dir = TempDir::new().unwrap();
        let nested = dir.path().join("sub/deep");
        std::fs::create_dir_all(&nested).unwrap();
        std::fs::write(dir.path().join("a.mp3"), b"x").unwrap();
        std::fs::write(nested.join("b.flac"), b"y").unwrap();
        std::fs::write(dir.path().join("readme.txt"), b"z").unwrap();
        std::fs::write(dir.path().join("noext"), b"q").unwrap();

        let files = walk_audio_files(&[dir.path().to_path_buf()]);
        let names: Vec<String> = files
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
            .collect();
        assert_eq!(files.len(), 2);
        assert!(names.contains(&"a.mp3".to_string()));
        assert!(names.contains(&"b.flac".to_string()));
    }

    #[test]
    fn walk_skips_hidden_directories() {
        let dir = TempDir::new().unwrap();
        let hidden = dir.path().join(".Trash");
        std::fs::create_dir_all(&hidden).unwrap();
        std::fs::write(hidden.join("x.mp3"), b"x").unwrap();
        std::fs::write(dir.path().join("y.mp3"), b"y").unwrap();

        let files = walk_audio_files(&[dir.path().to_path_buf()]);
        assert_eq!(files.len(), 1);
        assert!(files[0].file_name().unwrap() == "y.mp3");
    }
}
