//! Playlists table queries
use rusqlite::Connection;
use crate::error::AppResult;

pub struct PlaylistRepo {}

impl PlaylistRepo {
    pub fn new() -> Self { Self {} }

    pub fn get_all(conn: &Connection) -> AppResult<Vec<()>> {
        let _ = conn;
        Ok(vec![])
    }
}
