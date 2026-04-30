-- V3: scanner support — root scoping, hash index, row-level FTS sync.
-- v0.2.1 V2 created tracks_fts as a standalone (non-contentless) FTS5 table,
-- but kept 'rebuild' fallback for UPDATE/DELETE which is O(N). v0.3.0 will
-- routinely UPDATE on rescan / moved-path, so we replace with row-level sync.

ALTER TABLE tracks ADD COLUMN root_folder_id INTEGER REFERENCES scan_folders(id);
CREATE INDEX idx_tracks_root_folder ON tracks(root_folder_id);
CREATE INDEX idx_tracks_hash ON tracks(hash);

DROP TRIGGER IF EXISTS trg_tracks_fts_insert;
DROP TRIGGER IF EXISTS trg_tracks_fts_update;
DROP TRIGGER IF EXISTS trg_tracks_fts_delete;
DROP TRIGGER IF EXISTS trg_albums_fts_update;
DROP TRIGGER IF EXISTS trg_artists_fts_update;

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

CREATE TRIGGER trg_tracks_fts_update AFTER UPDATE ON tracks
WHEN OLD.title IS NOT NEW.title
  OR OLD.album_id IS NOT NEW.album_id
  OR OLD.primary_artist_id IS NOT NEW.primary_artist_id
  OR OLD.missing_at IS NOT NEW.missing_at
BEGIN
  DELETE FROM tracks_fts WHERE rowid = OLD.id;
  INSERT INTO tracks_fts(rowid, title, album_name, artist_name)
  SELECT
    NEW.id,
    NEW.title,
    (SELECT name FROM albums WHERE id = NEW.album_id),
    (SELECT name FROM artists WHERE id = NEW.primary_artist_id)
  WHERE NEW.missing_at IS NULL;
END;

CREATE TRIGGER trg_tracks_fts_delete AFTER DELETE ON tracks
BEGIN
  DELETE FROM tracks_fts WHERE rowid = OLD.id;
END;

CREATE TRIGGER trg_albums_fts_update AFTER UPDATE OF name ON albums
WHEN NEW.name IS NOT OLD.name
BEGIN
  DELETE FROM tracks_fts
   WHERE rowid IN (SELECT id FROM tracks WHERE album_id = NEW.id);
  INSERT INTO tracks_fts(rowid, title, album_name, artist_name)
  SELECT t.id, t.title, NEW.name, ar.name
    FROM tracks t
    LEFT JOIN artists ar ON ar.id = t.primary_artist_id
   WHERE t.album_id = NEW.id AND t.missing_at IS NULL;
END;

CREATE TRIGGER trg_artists_fts_update AFTER UPDATE OF name ON artists
WHEN NEW.name IS NOT OLD.name
BEGIN
  DELETE FROM tracks_fts
   WHERE rowid IN (SELECT id FROM tracks WHERE primary_artist_id = NEW.id);
  INSERT INTO tracks_fts(rowid, title, album_name, artist_name)
  SELECT t.id, t.title, al.name, NEW.name
    FROM tracks t
    LEFT JOIN albums al ON al.id = t.album_id
   WHERE t.primary_artist_id = NEW.id AND t.missing_at IS NULL;
END;

-- Rebuild FTS content once, cleaning dirty data from v0.2.1 old 'rebuild' triggers.
DELETE FROM tracks_fts;
INSERT INTO tracks_fts(rowid, title, album_name, artist_name)
SELECT t.id, t.title, al.name, ar.name
  FROM tracks t
  LEFT JOIN albums al ON al.id = t.album_id
  LEFT JOIN artists ar ON ar.id = t.primary_artist_id
 WHERE t.missing_at IS NULL;
