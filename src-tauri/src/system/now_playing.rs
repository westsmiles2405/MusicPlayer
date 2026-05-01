//! macOS Now Playing integration (P2 — full impl in Task 9).
//! Non-macOS builds use a no-op bridge so the manager always compiles.

use crate::player::state::{EngineCommand, PlayerSnapshot};

pub struct NowPlaying {
    #[cfg(target_os = "macos")]
    inner: platform::NowPlayingPlatform,
}

impl NowPlaying {
    pub fn new() -> Self {
        #[cfg(target_os = "macos")]
        {
            Self {
                inner: platform::NowPlayingPlatform::new(),
            }
        }
        #[cfg(not(target_os = "macos"))]
        {
            Self {}
        }
    }

    pub fn set_command_sender(&mut self, tx: crossbeam_channel::Sender<EngineCommand>) {
        #[cfg(target_os = "macos")]
        self.inner.set_command_sender(tx);
        #[cfg(not(target_os = "macos"))]
        let _ = tx;
    }

    pub fn update(&mut self, snapshot: &PlayerSnapshot) {
        #[cfg(target_os = "macos")]
        self.inner.update(snapshot);
        #[cfg(not(target_os = "macos"))]
        let _ = snapshot;
    }

    pub fn update_progress(&mut self, position_ms: i64, duration_ms: i64) {
        #[cfg(target_os = "macos")]
        self.inner.update_progress(position_ms, duration_ms);
        #[cfg(not(target_os = "macos"))]
        {
            let _ = position_ms;
            let _ = duration_ms;
        }
    }

    pub fn clear(&mut self) {
        #[cfg(target_os = "macos")]
        self.inner.clear();
    }
}

#[cfg(target_os = "macos")]
mod platform {
    use crate::player::state::{EngineCommand, PlayerSnapshot};

    pub struct NowPlayingPlatform;

    impl NowPlayingPlatform {
        pub fn new() -> Self {
            Self
        }

        pub fn set_command_sender(&mut self, _tx: crossbeam_channel::Sender<EngineCommand>) {}

        pub fn update(&mut self, _snapshot: &PlayerSnapshot) {}

        pub fn update_progress(&mut self, _position_ms: i64, _duration_ms: i64) {}

        pub fn clear(&mut self) {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn now_playing_bridge_accepts_snapshot_update() {
        let mut bridge = NowPlaying::new();
        bridge.update(&PlayerSnapshot::default());
        bridge.update_progress(0, 0);
        bridge.clear();
    }
}
