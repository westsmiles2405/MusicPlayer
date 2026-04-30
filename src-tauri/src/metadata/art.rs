#![allow(dead_code)]
//! Cache album cover images on disk by content hash, deduplicating across tracks.

use crate::error::{AppError, AppResult};
use std::fs;
use std::path::Path;

/// Write image bytes to `<cache_dir>/<hash16>.<ext>` and return the relative path
/// `covers/<hash16>.<ext>` suitable for `albums.cover_path`. Idempotent: same bytes
/// hashing to existing file is a no-op.
pub fn cache_cover_bytes(bytes: &[u8], cache_dir: &Path) -> AppResult<String> {
    if bytes.is_empty() {
        return Err(AppError::Metadata("empty cover bytes".into()));
    }
    let hash16 = content_hash16(bytes);
    let ext = detect_image_ext(bytes);
    let filename = format!("{hash16}.{ext}");
    let abs = cache_dir.join(&filename);
    if !abs.exists() {
        fs::create_dir_all(cache_dir)
            .map_err(|e| AppError::Metadata(format!("mkdir {}: {}", cache_dir.display(), e)))?;
        fs::write(&abs, bytes)
            .map_err(|e| AppError::Metadata(format!("write {}: {}", abs.display(), e)))?;
    }
    Ok(format!("covers/{filename}"))
}

fn content_hash16(bytes: &[u8]) -> String {
    use xxhash_rust::xxh3::Xxh3;
    let mut h = Xxh3::new();
    h.update(bytes);
    format!("{:016x}", h.digest())
}

fn detect_image_ext(bytes: &[u8]) -> &'static str {
    if bytes.len() >= 3 && &bytes[..3] == b"\xFF\xD8\xFF" {
        "jpg"
    } else if bytes.len() >= 8 && &bytes[..8] == b"\x89PNG\r\n\x1a\n" {
        "png"
    } else if bytes.len() >= 12 && &bytes[..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        "webp"
    } else if bytes.len() >= 6 && (&bytes[..6] == b"GIF89a" || &bytes[..6] == b"GIF87a") {
        "gif"
    } else {
        "bin"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    const JPEG_HEAD: &[u8] = b"\xFF\xD8\xFF\xE0PADPAD";
    const PNG_HEAD: &[u8] = b"\x89PNG\r\n\x1a\nMOREDATA";

    #[test]
    fn detects_jpeg_extension() {
        assert_eq!(detect_image_ext(JPEG_HEAD), "jpg");
    }

    #[test]
    fn detects_png_extension() {
        assert_eq!(detect_image_ext(PNG_HEAD), "png");
    }

    #[test]
    fn detects_unknown_as_bin() {
        assert_eq!(detect_image_ext(b"random garbage"), "bin");
    }

    #[test]
    fn cache_writes_file_first_time() {
        let dir = TempDir::new().unwrap();
        let rel = cache_cover_bytes(JPEG_HEAD, dir.path()).unwrap();
        assert!(rel.starts_with("covers/"));
        assert!(rel.ends_with(".jpg"));
        let abs = dir.path().join(rel.strip_prefix("covers/").unwrap());
        assert!(abs.exists());
    }

    #[test]
    fn cache_is_idempotent_for_same_bytes() {
        let dir = TempDir::new().unwrap();
        let r1 = cache_cover_bytes(JPEG_HEAD, dir.path()).unwrap();
        let r2 = cache_cover_bytes(JPEG_HEAD, dir.path()).unwrap();
        assert_eq!(r1, r2);
        let count = std::fs::read_dir(dir.path()).unwrap().count();
        assert_eq!(count, 1);
    }

    #[test]
    fn different_bytes_produce_different_files() {
        let dir = TempDir::new().unwrap();
        let r1 = cache_cover_bytes(JPEG_HEAD, dir.path()).unwrap();
        let r2 = cache_cover_bytes(PNG_HEAD, dir.path()).unwrap();
        assert_ne!(r1, r2);
        let count = std::fs::read_dir(dir.path()).unwrap().count();
        assert_eq!(count, 2);
    }

    #[test]
    fn empty_bytes_returns_metadata_error() {
        let dir = TempDir::new().unwrap();
        let r = cache_cover_bytes(&[], dir.path());
        assert!(matches!(r, Err(AppError::Metadata(_))));
    }
}
