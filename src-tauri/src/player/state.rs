#![allow(dead_code)]
//! Player state machine: Idle → Loading → Playing → Paused → Stopped

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum PlayerState {
    Idle,
    Loading,
    Playing,
    Paused,
    Stopped,
}

pub struct StateManager {
    state: PlayerState,
}

impl StateManager {
    pub fn new() -> Self {
        Self {
            state: PlayerState::Idle,
        }
    }

    pub fn state(&self) -> PlayerState {
        self.state
    }
}
