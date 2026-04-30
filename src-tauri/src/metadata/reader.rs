#![allow(dead_code)]
//! Read tag/duration/codec metadata + sample-hash + cover bytes from one audio file.

use crate::error::{AppError, AppResult};
use lofty::file::{AudioFile, TaggedFileExt};
use lofty::tag::{ItemKey, Tag};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

const HASH_BYTES: usize = 64 * 1024;
pub const UNKNOWN_ARTIST: &str = "Unknown Artist";
pub const UNKNOWN_ALBUM: &str = "Unknown Album";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RawTrack {
    pub path: PathBuf,
    pub hash: String,
    pub mtime_ms: i64,
    pub size_bytes: i64,
    pub title: String,
    pub artists: Vec<String>,
    pub album: String,
    pub album_artist: Option<String>,
    pub track_no: Option<i32>,
    pub disc_no: Option<i32>,
    pub year: Option<i32>,
    pub genre: Option<String>,
    pub duration_ms: i64,
    pub bitrate: Option<i32>,
    pub sample_rate: Option<i32>,
    pub channels: Option<i32>,
    pub codec: Option<String>,
    pub cover: Option<Vec<u8>>,
}

pub fn read_track(path: &Path) -> AppResult<RawTrack> {
    let meta = fs::metadata(path)
        .map_err(|e| AppError::Metadata(format!("stat {}: {}", path.display(), e)))?;
    let size_bytes = meta.len() as i64;
    let mtime_ms = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);

    let hash = sample_hash(path).map_err(|e| AppError::Metadata(format!("hash: {e}")))?;

    let tagged = lofty::read_from_path(path)
        .map_err(|e| AppError::Metadata(format!("lofty {}: {}", path.display(), e)))?;
    let props = tagged.properties();
    let duration_ms = props.duration().as_millis() as i64;
    let bitrate = props.audio_bitrate().map(|b| b as i32);
    let sample_rate = props.sample_rate().map(|s| s as i32);
    let channels = props.channels().map(|c| c as i32);
    let codec = Some(format!("{:?}", tagged.file_type()));

    let tag = tagged.primary_tag();
    let stem_title = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown")
        .to_string();
    let (title, artists, album, album_artist, track_no, disc_no, year, genre, cover) =
        extract_tag_fields(tag, stem_title);

    Ok(RawTrack {
        path: path.to_path_buf(),
        hash,
        mtime_ms,
        size_bytes,
        title,
        artists,
        album,
        album_artist,
        track_no,
        disc_no,
        year,
        genre,
        duration_ms,
        bitrate,
        sample_rate,
        channels,
        codec,
        cover,
    })
}

#[allow(clippy::type_complexity)]
fn extract_tag_fields(
    tag: Option<&Tag>,
    stem_title: String,
) -> (
    String,
    Vec<String>,
    String,
    Option<String>,
    Option<i32>,
    Option<i32>,
    Option<i32>,
    Option<String>,
    Option<Vec<u8>>,
) {
    let title = tag
        .and_then(|t| t.get_string(ItemKey::TrackTitle))
        .map(str::to_string)
        .unwrap_or(stem_title);
    let primary_artist = tag
        .and_then(|t| t.get_string(ItemKey::TrackArtist))
        .map(str::to_string)
        .unwrap_or_else(|| UNKNOWN_ARTIST.to_string());
    let artists = vec![primary_artist];
    let album = tag
        .and_then(|t| t.get_string(ItemKey::AlbumTitle))
        .map(str::to_string)
        .unwrap_or_else(|| UNKNOWN_ALBUM.to_string());
    let album_artist = tag
        .and_then(|t| t.get_string(ItemKey::AlbumArtist))
        .map(str::to_string);
    let track_no = tag
        .and_then(|t| t.get_string(ItemKey::TrackNumber))
        .and_then(|s| s.split('/').next().and_then(|n| n.parse().ok()));
    let disc_no = tag
        .and_then(|t| t.get_string(ItemKey::DiscNumber))
        .and_then(|s| s.split('/').next().and_then(|n| n.parse().ok()));
    let year = tag
        .and_then(|t| t.get_string(ItemKey::Year))
        .and_then(|s| s.parse().ok());
    let genre = tag
        .and_then(|t| t.get_string(ItemKey::Genre))
        .map(str::to_string);
    let cover = tag.and_then(|t| t.pictures().first().map(|p| p.data().to_vec()));
    (
        title,
        artists,
        album,
        album_artist,
        track_no,
        disc_no,
        year,
        genre,
        cover,
    )
}

fn sample_hash(path: &Path) -> std::io::Result<String> {
    use xxhash_rust::xxh3::Xxh3;
    let mut f = fs::File::open(path)?;
    let mut buf = vec![0u8; HASH_BYTES];
    let n = f.read(&mut buf)?;
    let mut h = Xxh3::new();
    h.update(&buf[..n]);
    Ok(format!("{:016x}", h.digest()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/audio")
            .join(name)
    }

    #[test]
    fn reads_mp3_with_full_tag() {
        let r = read_track(&fixture("a.mp3")).unwrap();
        assert_eq!(r.title, "Track A");
        assert_eq!(r.artists, vec!["Test Artist 1"]);
        assert_eq!(r.album, "Test Album 1");
        assert_eq!(r.track_no, Some(1));
        assert!(r.duration_ms >= 900 && r.duration_ms <= 1100);
        assert!(r.size_bytes > 0);
        assert_eq!(r.hash.len(), 16);
    }

    #[test]
    fn reads_flac_with_full_tag() {
        let r = read_track(&fixture("b.flac")).unwrap();
        assert_eq!(r.title, "Track B");
        assert_eq!(r.album, "Test Album 1");
        assert_eq!(r.track_no, Some(2));
    }

    #[test]
    fn reads_m4a() {
        let r = read_track(&fixture("c.m4a")).unwrap();
        assert_eq!(r.title, "Track C");
        assert_eq!(r.artists, vec!["Test Artist 2"]);
    }

    #[test]
    fn reads_wav() {
        let r = read_track(&fixture("d.wav")).unwrap();
        assert!(r.duration_ms >= 900);
        assert!(!r.title.is_empty());
    }

    #[test]
    fn reads_no_tag_falls_back_to_filename() {
        let r = read_track(&fixture("e_no_tag.mp3")).unwrap();
        assert_eq!(r.title, "e_no_tag");
        assert_eq!(r.artists, vec![UNKNOWN_ARTIST]);
        assert_eq!(r.album, UNKNOWN_ALBUM);
    }

    #[test]
    fn hash_is_stable_across_calls() {
        let h1 = read_track(&fixture("a.mp3")).unwrap().hash;
        let h2 = read_track(&fixture("a.mp3")).unwrap().hash;
        assert_eq!(h1, h2);
    }

    #[test]
    fn different_files_have_different_hash() {
        let a = read_track(&fixture("a.mp3")).unwrap().hash;
        let b = read_track(&fixture("b.flac")).unwrap().hash;
        assert_ne!(a, b);
    }
}
