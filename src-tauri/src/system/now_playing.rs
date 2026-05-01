//! macOS Now Playing integration.
//! Bridges MPNowPlayingInfoCenter + MPRemoteCommandCenter.
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

    #[allow(dead_code)]
    pub fn clear(&mut self) {
        #[cfg(target_os = "macos")]
        self.inner.clear();
    }
}

#[cfg(target_os = "macos")]
mod platform {
    use core::ptr::NonNull;

    use block2::RcBlock;
    use objc2::msg_send;
    use objc2_foundation::{NSDictionary, NSMutableDictionary, NSNumber, NSString};
    use objc2_media_player::{
        MPChangePlaybackPositionCommandEvent, MPMediaItemPropertyAlbumTitle,
        MPMediaItemPropertyArtist, MPMediaItemPropertyPlaybackDuration, MPMediaItemPropertyTitle,
        MPNowPlayingInfoCenter, MPNowPlayingInfoPropertyElapsedPlaybackTime,
        MPNowPlayingInfoPropertyPlaybackRate, MPRemoteCommand, MPRemoteCommandCenter,
        MPRemoteCommandEvent, MPRemoteCommandHandlerStatus,
    };

    use crate::player::state::{EngineCommand, PlaybackStatus, PlayerSnapshot};

    pub struct NowPlayingPlatform {
        command_tx: Option<crossbeam_channel::Sender<EngineCommand>>,
    }

    impl NowPlayingPlatform {
        pub fn new() -> Self {
            Self { command_tx: None }
        }

        pub fn set_command_sender(&mut self, tx: crossbeam_channel::Sender<EngineCommand>) {
            self.command_tx = Some(tx);
            self.register_remote_commands();
        }

        pub fn update(&mut self, snapshot: &PlayerSnapshot) {
            let rate = if snapshot.status == PlaybackStatus::Playing {
                1.0_f64
            } else {
                0.0_f64
            };
            let title = snapshot
                .current
                .as_ref()
                .map(|t| t.title.clone())
                .unwrap_or_default();
            let artist = snapshot
                .current
                .as_ref()
                .and_then(|t| t.artist_name.clone())
                .unwrap_or_default();
            let album = snapshot
                .current
                .as_ref()
                .and_then(|t| t.album_name.clone())
                .unwrap_or_default();
            let duration = f64::max(snapshot.duration_ms as f64 / 1000.0, 0.0);
            let elapsed = f64::max(snapshot.position_ms as f64 / 1000.0, 0.0);
            self.write_now_playing_info(&title, &artist, &album, duration, elapsed, rate);
        }

        pub fn update_progress(&mut self, position_ms: i64, duration_ms: i64) {
            let elapsed = f64::max(position_ms as f64 / 1000.0, 0.0);
            let duration = f64::max(duration_ms as f64 / 1000.0, 0.0);
            self.write_progress(elapsed, duration);
        }

        pub fn clear(&mut self) {
            unsafe {
                MPNowPlayingInfoCenter::defaultCenter().setNowPlayingInfo(None);
            }
        }

        fn register_remote_commands(&mut self) {
            let Some(ref tx) = self.command_tx else {
                return;
            };

            unsafe {
                let center = MPRemoteCommandCenter::sharedCommandCenter();
                register_command(&center.playCommand(), tx.clone(), EngineCommand::Resume);
                register_command(&center.pauseCommand(), tx.clone(), EngineCommand::Pause);
                register_command(
                    &center.togglePlayPauseCommand(),
                    tx.clone(),
                    EngineCommand::Toggle,
                );
                register_command(&center.nextTrackCommand(), tx.clone(), EngineCommand::Next);
                register_command(
                    &center.previousTrackCommand(),
                    tx.clone(),
                    EngineCommand::Previous,
                );

                let seek_tx = tx.clone();
                let block = RcBlock::new(move |event: NonNull<MPRemoteCommandEvent>| {
                    let pos_event: NonNull<MPChangePlaybackPositionCommandEvent> = event.cast();
                    let seconds = pos_event.as_ref().positionTime();
                    let _ = seek_tx.send(EngineCommand::Seek {
                        position_ms: (seconds * 1000.0) as i64,
                    });
                    MPRemoteCommandHandlerStatus::Success
                });
                center
                    .changePlaybackPositionCommand()
                    .addTargetWithHandler(&block);
            }
        }

        fn write_now_playing_info(
            &self,
            title: &str,
            artist: &str,
            album: &str,
            duration: f64,
            elapsed: f64,
            rate: f64,
        ) {
            unsafe {
                let info = NSMutableDictionary::<NSString, objc2::runtime::AnyObject>::new();

                let title_ns = NSString::from_str(title);
                let artist_ns = NSString::from_str(artist);
                let album_ns = NSString::from_str(album);
                let dur_ns = NSNumber::new_f64(duration);
                let elapsed_ns = NSNumber::new_f64(elapsed);
                let rate_ns = NSNumber::new_f64(rate);

                let _: () =
                    msg_send![&info, setObject: &*title_ns, forKey: MPMediaItemPropertyTitle];
                let _: () =
                    msg_send![&info, setObject: &*artist_ns, forKey: MPMediaItemPropertyArtist];
                let _: () =
                    msg_send![&info, setObject: &*album_ns, forKey: MPMediaItemPropertyAlbumTitle];
                let _: () = msg_send![&info, setObject: &*dur_ns, forKey: MPMediaItemPropertyPlaybackDuration];
                let _: () = msg_send![&info, setObject: &*elapsed_ns, forKey: MPNowPlayingInfoPropertyElapsedPlaybackTime];
                let _: () = msg_send![&info, setObject: &*rate_ns, forKey: MPNowPlayingInfoPropertyPlaybackRate];

                let dict: &NSDictionary<NSString, objc2::runtime::AnyObject> = &info;
                MPNowPlayingInfoCenter::defaultCenter().setNowPlayingInfo(Some(dict));
            }
        }

        fn write_progress(&self, elapsed: f64, duration: f64) {
            unsafe {
                let existing = MPNowPlayingInfoCenter::defaultCenter().nowPlayingInfo();
                let info = NSMutableDictionary::<NSString, objc2::runtime::AnyObject>::new();
                if let Some(ref existing) = existing {
                    copy_existing_now_playing_info(existing, &info);
                }

                let dur_ns = NSNumber::new_f64(duration);
                let elapsed_ns = NSNumber::new_f64(elapsed);
                let _: () = msg_send![&info, setObject: &*dur_ns, forKey: MPMediaItemPropertyPlaybackDuration];
                let _: () = msg_send![&info, setObject: &*elapsed_ns, forKey: MPNowPlayingInfoPropertyElapsedPlaybackTime];

                let dict: &NSDictionary<NSString, objc2::runtime::AnyObject> = &info;
                MPNowPlayingInfoCenter::defaultCenter().setNowPlayingInfo(Some(dict));
            }
        }
    }

    unsafe fn register_command(
        command: &MPRemoteCommand,
        tx: crossbeam_channel::Sender<EngineCommand>,
        player_command: EngineCommand,
    ) {
        command.setEnabled(true);
        let block = RcBlock::new(move |_event: NonNull<MPRemoteCommandEvent>| {
            let _ = tx.send(player_command.clone());
            MPRemoteCommandHandlerStatus::Success
        });
        command.addTargetWithHandler(&block);
    }

    unsafe fn copy_existing_now_playing_info(
        existing: &objc2::rc::Retained<NSDictionary<NSString, objc2::runtime::AnyObject>>,
        target: &NSMutableDictionary<NSString, objc2::runtime::AnyObject>,
    ) {
        for key in [
            MPMediaItemPropertyTitle,
            MPMediaItemPropertyArtist,
            MPMediaItemPropertyAlbumTitle,
            MPNowPlayingInfoPropertyPlaybackRate,
        ] {
            if let Some(value) = existing.objectForKey(key) {
                let _: () = msg_send![target, setObject: &*value, forKey: key];
            }
        }
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
