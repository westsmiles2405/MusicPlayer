#![allow(dead_code)]
//! Audio engine: symphonia decode + cpal output
//! PCM decoded in Rust, output directly to system audio device
//! (does NOT go through IPC)

pub struct AudioEngine {}

impl AudioEngine {
    pub fn new() -> Self {
        Self {}
    }
}
