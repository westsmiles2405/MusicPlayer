#![allow(dead_code)]
//! Gapless playback: double-buffered decoding
//! Pre-decodes next track while current is still playing

pub struct GaplessBuffer {}

impl GaplessBuffer {
    pub fn new() -> Self {
        Self {}
    }
}
