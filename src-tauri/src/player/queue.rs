//! Deterministic play queue. Owns track-id ordering, shuffle, and repeat.
//! Does NOT know playback position — that belongs to the engine or manager.

use crate::player::state::RepeatMode;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayQueue {
    original: Vec<i64>,
    order: Vec<usize>,
    index: Option<usize>,
    repeat_mode: RepeatMode,
    shuffle: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueueMove {
    Track(i64),
    RestartCurrent(i64),
    Ended,
}

impl PlayQueue {
    pub fn from_tracks(track_ids: Vec<i64>, index: usize) -> Result<Self, String> {
        if track_ids.is_empty() {
            return Err("queue cannot be empty".into());
        }
        if index >= track_ids.len() {
            return Err(format!(
                "queueIndex {index} out of range for queue length {}",
                track_ids.len()
            ));
        }
        let order = (0..track_ids.len()).collect();
        Ok(Self {
            original: track_ids,
            order,
            index: Some(index),
            repeat_mode: RepeatMode::Off,
            shuffle: false,
        })
    }

    pub fn current(&self) -> Option<i64> {
        self.index.map(|i| self.original[self.order[i]])
    }

    pub fn current_index(&self) -> Option<usize> {
        self.index
    }

    pub fn len(&self) -> usize {
        self.original.len()
    }

    pub fn repeat_mode(&self) -> RepeatMode {
        self.repeat_mode
    }

    pub fn shuffle(&self) -> bool {
        self.shuffle
    }

    pub fn set_repeat_mode(&mut self, mode: RepeatMode) {
        self.repeat_mode = mode;
    }

    pub fn set_shuffle(&mut self, enabled: bool) {
        if self.shuffle == enabled {
            return;
        }
        let current_track = self.current();
        self.shuffle = enabled;
        self.order = if enabled {
            deterministic_shuffle(self.original.len())
        } else {
            (0..self.original.len()).collect()
        };
        self.index = current_track
            .and_then(|id| self.order.iter().position(|&oi| self.original[oi] == id));
    }

    pub fn next(&mut self) -> QueueMove {
        let Some(index) = self.index else {
            return QueueMove::Ended;
        };
        if self.repeat_mode == RepeatMode::One {
            return QueueMove::Track(self.original[self.order[index]]);
        }
        let next = index + 1;
        if next < self.order.len() {
            self.index = Some(next);
            return QueueMove::Track(self.original[self.order[next]]);
        }
        if self.repeat_mode == RepeatMode::All {
            self.index = Some(0);
            return QueueMove::Track(self.original[self.order[0]]);
        }
        self.index = None;
        QueueMove::Ended
    }

    pub fn previous(&mut self) -> QueueMove {
        let Some(index) = self.index else {
            return QueueMove::Ended;
        };
        if index > 0 {
            let prev = index - 1;
            self.index = Some(prev);
            return QueueMove::Track(self.original[self.order[prev]]);
        }
        if self.repeat_mode == RepeatMode::All {
            let last = self.order.len() - 1;
            self.index = Some(last);
            return QueueMove::Track(self.original[self.order[last]]);
        }
        QueueMove::RestartCurrent(self.original[self.order[index]])
    }

    pub fn remove_current_unplayable(&mut self) -> QueueMove {
        let Some(index) = self.index else {
            return QueueMove::Ended;
        };
        let original_idx = self.order[index];
        self.original.remove(original_idx);
        if self.original.is_empty() {
            self.order.clear();
            self.index = None;
            return QueueMove::Ended;
        }
        self.order = (0..self.original.len()).collect();
        if self.shuffle {
            self.order = deterministic_shuffle(self.original.len());
        }
        self.index = Some(index.min(self.order.len() - 1));
        QueueMove::Track(self.original[self.order[self.index.unwrap()]])
    }
}

fn deterministic_shuffle(len: usize) -> Vec<usize> {
    let mut order: Vec<usize> = (0..len).collect();
    if len > 2 {
        order[1..].reverse();
    }
    order
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_empty_queue_and_bad_index() {
        assert!(PlayQueue::from_tracks(vec![], 0).is_err());
        assert!(PlayQueue::from_tracks(vec![1, 2], 2).is_err());
    }

    #[test]
    fn next_ends_without_repeat() {
        let mut q = PlayQueue::from_tracks(vec![1, 2], 0).unwrap();
        assert_eq!(q.next(), QueueMove::Track(2));
        assert_eq!(q.next(), QueueMove::Ended);
        assert_eq!(q.current(), None);
    }

    #[test]
    fn repeat_all_wraps_to_start() {
        let mut q = PlayQueue::from_tracks(vec![1, 2], 1).unwrap();
        q.set_repeat_mode(RepeatMode::All);
        assert_eq!(q.next(), QueueMove::Track(1));
        assert_eq!(q.current_index(), Some(0));
    }

    #[test]
    fn shuffle_keeps_current_track() {
        let mut q = PlayQueue::from_tracks(vec![10, 20, 30, 40], 2).unwrap();
        q.set_shuffle(true);
        assert_eq!(q.current(), Some(30));
        assert!(q.shuffle());
    }

    #[test]
    fn shuffle_off_restores_original_index_for_current() {
        let mut q = PlayQueue::from_tracks(vec![10, 20, 30, 40], 2).unwrap();
        q.set_shuffle(true);
        q.set_shuffle(false);
        assert_eq!(q.current(), Some(30));
        assert_eq!(q.current_index(), Some(2));
    }

    #[test]
    fn previous_at_start_restarts_current_when_repeat_off() {
        let mut q = PlayQueue::from_tracks(vec![1, 2], 0).unwrap();
        assert_eq!(q.previous(), QueueMove::RestartCurrent(1));
    }

    #[test]
    fn remove_current_skips_to_next_available() {
        let mut q = PlayQueue::from_tracks(vec![1, 2, 3], 1).unwrap();
        assert_eq!(q.remove_current_unplayable(), QueueMove::Track(3));
        assert_eq!(q.len(), 2);
    }
}
