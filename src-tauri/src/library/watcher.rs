#![allow(dead_code)]
//! Filesystem watcher using notify crate
//! Detects new/modified/deleted files with 2s debounce

#[derive(Default)]
pub struct Watcher {}

impl Watcher {
    pub fn new() -> Self {
        Self {}
    }
}
