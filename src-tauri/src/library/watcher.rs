//! Filesystem watcher using notify crate
//! Detects new/modified/deleted files with 2s debounce

pub struct Watcher {}

impl Watcher {
    pub fn new() -> Self {
        Self {}
    }
}
