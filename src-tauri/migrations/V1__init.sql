-- V1: Initial schema for MusicPlayer
-- Applies: artists, albums, tracks, track_artists, playlists, playlist_tracks,
--          play_history, scan_folders, app_state, FTS5 search, schema_migrations

-- ===========================
-- Artists
-- ===========================
CREATE TABLE artists (
  id INTEGER PRIMARY KEY,
  name TEXT NOT NULL UNIQUE,
  added_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL
);

-- ===========================
-- Albums
-- ===========================
CREATE TABLE albums (
  id INTEGER PRIMARY KEY,
  name TEXT NOT NULL,
  album_artist_id INTEGER NOT NULL REFERENCES artists(id),
  year INTEGER,
  cover_path TEXT,
  added_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  UNIQUE(name, album_artist_id)
);

-- ===========================
-- Tracks (core table)
-- ===========================
CREATE TABLE tracks (
  id INTEGER PRIMARY KEY,
  file_path TEXT NOT NULL UNIQUE,
  file_size INTEGER NOT NULL,
  file_modified_at INTEGER NOT NULL,
  hash TEXT,
  title TEXT NOT NULL,
  album_id INTEGER REFERENCES albums(id),
  primary_artist_id INTEGER REFERENCES artists(id),
  album_artist_id INTEGER REFERENCES artists(id),
  track_no INTEGER,
  disc_no INTEGER,
  year INTEGER,
  genre TEXT,
  duration_ms INTEGER NOT NULL,
  bitrate INTEGER,
  sample_rate INTEGER,
  channels INTEGER,
  codec TEXT,
  is_favorite INTEGER NOT NULL DEFAULT 0,
  play_count INTEGER NOT NULL DEFAULT 0,
  last_played_at INTEGER,
  last_seen_at INTEGER NOT NULL,
  missing_at INTEGER,
  added_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL
);

CREATE INDEX idx_tracks_album ON tracks(album_id);
CREATE INDEX idx_tracks_primary_artist ON tracks(primary_artist_id);
CREATE INDEX idx_tracks_title ON tracks(title);
CREATE INDEX idx_tracks_missing ON tracks(missing_at);

-- ===========================
-- Track-Artist many-to-many (with role)
-- ===========================
CREATE TABLE track_artists (
  track_id INTEGER NOT NULL REFERENCES tracks(id) ON DELETE CASCADE,
  artist_id INTEGER NOT NULL REFERENCES artists(id) ON DELETE CASCADE,
  role TEXT NOT NULL DEFAULT 'main',
  position INTEGER NOT NULL DEFAULT 0,
  PRIMARY KEY (track_id, artist_id, role)
);

-- ===========================
-- User Playlists
-- ===========================
CREATE TABLE playlists (
  id INTEGER PRIMARY KEY,
  name TEXT NOT NULL,
  description TEXT,
  cover_path TEXT,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL
);

CREATE TABLE playlist_tracks (
  playlist_id INTEGER NOT NULL REFERENCES playlists(id) ON DELETE CASCADE,
  track_id INTEGER NOT NULL REFERENCES tracks(id) ON DELETE CASCADE,
  position INTEGER NOT NULL,
  added_at INTEGER NOT NULL,
  PRIMARY KEY (playlist_id, track_id, position)
);

CREATE INDEX idx_playlist_tracks ON playlist_tracks(playlist_id, position);

-- ===========================
-- Play History
-- ===========================
CREATE TABLE play_history (
  id INTEGER PRIMARY KEY,
  track_id INTEGER NOT NULL REFERENCES tracks(id) ON DELETE CASCADE,
  played_at INTEGER NOT NULL,
  duration_played_ms INTEGER NOT NULL,
  completed INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX idx_play_history_played_at ON play_history(played_at DESC);

-- ===========================
-- Scan Folders
-- ===========================
CREATE TABLE scan_folders (
  id INTEGER PRIMARY KEY,
  path TEXT NOT NULL UNIQUE,
  bookmark_data BLOB,
  added_at INTEGER NOT NULL,
  last_scanned_at INTEGER
);

-- ===========================
-- App State (KV store)
-- ===========================
CREATE TABLE app_state (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL,
  updated_at INTEGER NOT NULL
);

-- ===========================
-- Full-Text Search (FTS5)
-- ===========================
CREATE VIEW tracks_search_view AS
SELECT
  tracks.id AS id,
  tracks.title AS title,
  albums.name AS album_name,
  artists.name AS artist_name
FROM tracks
LEFT JOIN albums ON albums.id = tracks.album_id
LEFT JOIN artists ON artists.id = tracks.primary_artist_id
WHERE tracks.missing_at IS NULL;

CREATE VIRTUAL TABLE tracks_fts USING fts5(
  title, album_name, artist_name,
  content='tracks_search_view',
  content_rowid='id',
  tokenize='unicode61 remove_diacritics 2'
);

-- Triggers to keep FTS5 in sync with the content view's underlying tables

CREATE TRIGGER trg_tracks_fts_insert AFTER INSERT ON tracks
BEGIN
  INSERT INTO tracks_fts(tracks_fts) VALUES('rebuild');
END;

CREATE TRIGGER trg_tracks_fts_update AFTER UPDATE ON tracks
BEGIN
  INSERT INTO tracks_fts(tracks_fts) VALUES('rebuild');
END;

CREATE TRIGGER trg_tracks_fts_delete AFTER DELETE ON tracks
BEGIN
  INSERT INTO tracks_fts(tracks_fts) VALUES('rebuild');
END;

CREATE TRIGGER trg_albums_fts_update AFTER UPDATE ON albums
BEGIN
  INSERT INTO tracks_fts(tracks_fts) VALUES('rebuild');
END;

CREATE TRIGGER trg_artists_fts_update AFTER UPDATE ON artists
BEGIN
  INSERT INTO tracks_fts(tracks_fts) VALUES('rebuild');
END;

-- ===========================
-- Schema Migrations Tracker
-- ===========================
CREATE TABLE IF NOT EXISTS schema_migrations (
  version INTEGER PRIMARY KEY,
  applied_at INTEGER NOT NULL
);
