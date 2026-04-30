//! Artist queries.

use rusqlite::{params, Connection, OptionalExtension, Row};
use serde::{Deserialize, Serialize};

use crate::error::AppResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Artist {
    pub id: i64,
    pub name: String,
    pub added_at: i64,
    pub updated_at: i64,
}

impl Artist {
    fn from_row(row: &Row<'_>) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            name: row.get("name")?,
            added_at: row.get("added_at")?,
            updated_at: row.get("updated_at")?,
        })
    }
}

// Implementations follow in Task 2.2.
#[allow(dead_code)]
fn _placeholder(conn: &Connection) -> AppResult<Option<Artist>> {
    let _ = conn;
    let _ = Artist::from_row;
    Ok(None::<Artist>)
        .or_else(|_: AppResult<Option<Artist>>| Ok(None))
}
