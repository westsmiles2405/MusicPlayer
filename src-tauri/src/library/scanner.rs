#![allow(dead_code)]
//! Library scanner public types.
//! 实际扫描编排在 Task 3.3 实现，本任务只确立 IPC 边界用的 DTO。

use serde::{Deserialize, Serialize};
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
}
