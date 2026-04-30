//! Database migration runner. Applies SQL migrations in order.
//! Tracks applied versions in the schema_migrations table.

use rusqlite::Connection;
use crate::error::AppResult;

pub fn run_migrations(conn: &Connection) -> AppResult<()> {
    // TODO: Task 0.6 — apply V1__init.sql
    let _ = conn;
    Ok(())
}
