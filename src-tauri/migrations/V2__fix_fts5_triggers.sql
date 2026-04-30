-- V2: replace V1's full-rebuild FTS5 triggers with row-level external-content sync.
-- V1 triggers were correct but pathological: each track insert triggered a full FTS rebuild.

DROP TRIGGER IF EXISTS trg_tracks_fts_insert;
DROP TRIGGER IF EXISTS trg_tracks_fts_update;
DROP TRIGGER IF EXISTS trg_tracks_fts_delete;
DROP TRIGGER IF EXISTS trg_albums_fts_update;
DROP TRIGGER IF EXISTS trg_artists_fts_update;

-- Row-level sync. tracks_fts is contentless (content='tracks_search_view'),
-- so we feed it title/album_name/artist_name explicitly via the rowid=tracks.id link.

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

CREATE TRIGGER trg_tracks_fts_delete AFTER DELETE ON tracks
BEGIN
  INSERT INTO tracks_fts(tracks_fts, rowid, title, album_name, artist_name)
  VALUES (
    'delete',
    OLD.id,
    OLD.title,
    (SELECT name FROM albums WHERE id = OLD.album_id),
    (SELECT name FROM artists WHERE id = OLD.primary_artist_id)
  );
END;

CREATE TRIGGER trg_tracks_fts_update AFTER UPDATE ON tracks
BEGIN
  INSERT INTO tracks_fts(tracks_fts, rowid, title, album_name, artist_name)
  VALUES (
    'delete',
    OLD.id,
    OLD.title,
    (SELECT name FROM albums WHERE id = OLD.album_id),
    (SELECT name FROM artists WHERE id = OLD.primary_artist_id)
  );
  INSERT INTO tracks_fts(rowid, title, album_name, artist_name)
  SELECT
    NEW.id,
    NEW.title,
    (SELECT name FROM albums WHERE id = NEW.album_id),
    (SELECT name FROM artists WHERE id = NEW.primary_artist_id)
  WHERE NEW.missing_at IS NULL;
END;

-- Albums/Artists 表的 name 改名是罕见操作（用户编辑标签或合并艺人），
-- 用 rebuild 兜底，避免在触发器里写复杂的多行更新。
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
