-- V2: replace V1's full-rebuild FTS5 triggers with row-level sync for INSERT
-- (the most common path during scanning) and keep rebuild for UPDATE/DELETE.
-- V1 tracks_fts used content='tracks_search_view' (content-sync), which conflicts with
-- manual row-level INSERT/DELETE. V2 recreates tracks_fts as contentless.

DROP TRIGGER IF EXISTS trg_tracks_fts_insert;
DROP TRIGGER IF EXISTS trg_tracks_fts_update;
DROP TRIGGER IF EXISTS trg_tracks_fts_delete;
DROP TRIGGER IF EXISTS trg_albums_fts_update;
DROP TRIGGER IF EXISTS trg_artists_fts_update;

DROP TABLE IF EXISTS tracks_fts;
CREATE VIRTUAL TABLE tracks_fts USING fts5(
  title, album_name, artist_name,
  tokenize='unicode61 remove_diacritics 2'
);

-- INSERT: row-level sync. The most common operation during scanning.
CREATE TRIGGER trg_tracks_fts_insert AFTER INSERT ON tracks
WHEN NEW.missing_at IS NULL
BEGIN
  INSERT INTO tracks_fts(rowid, title, album_name, artist_name)
  VALUES (
    NEW.id,
    NEW.title,
    (SELECT name FROM albums WHERE id = NEW.album_id),
    (SELECT name FROM artists WHERE id = NEW.primary_artist_id)
  );
END;

-- UPDATE / DELETE: use rebuild. These are less frequent and rebuild avoids
-- the complexity of contentless delete verification.
CREATE TRIGGER trg_tracks_fts_delete AFTER DELETE ON tracks
BEGIN
  INSERT INTO tracks_fts(tracks_fts) VALUES('rebuild');
END;

CREATE TRIGGER trg_tracks_fts_update AFTER UPDATE ON tracks
BEGIN
  INSERT INTO tracks_fts(tracks_fts) VALUES('rebuild');
END;

-- Albums/Artists rename (rare) still uses rebuild.
CREATE TRIGGER trg_albums_fts_update AFTER UPDATE OF name ON albums
WHEN NEW.name IS NOT OLD.name
BEGIN
  INSERT INTO tracks_fts(tracks_fts) VALUES('rebuild');
END;

CREATE TRIGGER trg_artists_fts_update AFTER UPDATE OF name ON artists
WHEN NEW.name IS NOT OLD.name
BEGIN
  INSERT INTO tracks_fts(tracks_fts) VALUES('rebuild');
END;
