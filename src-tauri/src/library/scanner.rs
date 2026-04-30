#![allow(dead_code)]
//! Library scanner public types.
//! 实际扫描编排在 Task 3.3 实现，本任务只确立 IPC 边界用的 DTO。

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

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
