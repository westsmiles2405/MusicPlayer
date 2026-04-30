//! Album queries.

use rusqlite::{params, Connection, OptionalExtension, Row};
use serde::{Deserialize, Serialize};

use crate::error::AppResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Album {
    pub id: i64,
    pub name: String,
    pub album_artist_id: i64,
    pub year: Option<i32>,
    pub cover_path: Option<String>,
    pub added_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlbumView {
    #[serde(flatten)]
    pub album: Album,
    pub album_artist_name: String,
    pub track_count: i64,
}

impl Album {
    fn from_row(row: &Row<'_>) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            name: row.get("name")?,
            album_artist_id: row.get("album_artist_id")?,
            year: row.get("year")?,
            cover_path: row.get("cover_path")?,
            added_at: row.get("added_at")?,
            updated_at: row.get("updated_at")?,
        })
    }
}

#[allow(dead_code)]
fn _placeholder(_: &Connection) -> AppResult<()> {
    let _ = (Album::from_row, params!(), |_: &mut Option<Album>| {});
    Ok(())
}
