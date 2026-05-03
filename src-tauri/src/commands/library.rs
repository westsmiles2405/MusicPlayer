//! Library IPC commands. Each command takes State<Database> and delegates to db::*.

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager, State};

use crate::db::{
    albums, artists, play_history, scan_folders as folders_repo, search, tracks, Database,
};
use crate::error::{AppError, AppResult};
use crate::library::scanner::{self, AbortFlag, ScanProgress, ScanReport};

fn resolve_cover_path(covers_dir: &std::path::Path, cover_path: &mut Option<String>) {
    if let Some(ref p) = cover_path {
        let abs = covers_dir.join(p);
        if abs.exists() {
            *cover_path = Some(abs.to_string_lossy().into_owned());
        }
    }
}

// ---- query params ----

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListParams {
    #[serde(default = "default_sort")]
    pub sort: tracks::TrackSort,
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_sort() -> tracks::TrackSort {
    tracks::TrackSort::Title
}
fn default_limit() -> i64 {
    500
}

fn now_ms() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

// ---- scan state managed by Tauri ----

pub struct ScanManager {
    pub running: Arc<AtomicBool>,
    pub abort: AbortFlag,
}

impl Default for ScanManager {
    fn default() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            abort: AbortFlag::new(),
        }
    }
}

// ---- existing library query commands ----

#[tauri::command]
pub async fn get_tracks(
    db: State<'_, Database>,
    params: Option<ListParams>,
) -> AppResult<Vec<tracks::TrackView>> {
    let p = params.unwrap_or(ListParams {
        sort: default_sort(),
        limit: default_limit(),
        offset: 0,
    });
    let result = db.with_conn(|c| tracks::list(c, p.sort, p.limit, p.offset))?;
    Ok(result)
}

#[tauri::command]
pub async fn get_albums(app: AppHandle, db: State<'_, Database>) -> AppResult<Vec<albums::AlbumView>> {
    let covers_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Scan(format!("app_data_dir: {e}")))?;
    let mut result = db.with_conn(albums::get_all)?;
    for a in &mut result {
        resolve_cover_path(&covers_dir, &mut a.album.cover_path);
    }
    Ok(result)
}

#[tauri::command]
pub async fn get_artists(db: State<'_, Database>) -> AppResult<Vec<artists::Artist>> {
    db.with_conn(artists::get_all)
}

#[tauri::command]
pub async fn get_album(
    app: AppHandle,
    db: State<'_, Database>,
    album_id: i64,
) -> AppResult<Option<albums::AlbumView>> {
    let covers_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Scan(format!("app_data_dir: {e}")))?;
    let mut result = db.with_conn(|c| albums::get_by_id(c, album_id))?;
    if let Some(ref mut a) = result {
        resolve_cover_path(&covers_dir, &mut a.album.cover_path);
    }
    Ok(result)
}

#[tauri::command]
pub async fn get_artist(
    db: State<'_, Database>,
    artist_id: i64,
) -> AppResult<Option<artists::Artist>> {
    db.with_conn(|c| artists::get_by_id(c, artist_id))
}

#[tauri::command]
pub async fn get_album_tracks(
    db: State<'_, Database>,
    album_id: i64,
) -> AppResult<Vec<tracks::TrackView>> {
    db.with_conn(|c| tracks::list_by_album(c, album_id))
}

#[tauri::command]
pub async fn get_artist_tracks(
    db: State<'_, Database>,
    artist_id: i64,
) -> AppResult<Vec<tracks::TrackView>> {
    db.with_conn(|c| tracks::list_by_artist(c, artist_id))
}

#[tauri::command]
pub async fn get_recently_added(
    db: State<'_, Database>,
    limit: Option<i64>,
) -> AppResult<Vec<tracks::TrackView>> {
    db.with_conn(|c| tracks::recently_added(c, limit.unwrap_or(50)))
}

#[tauri::command]
pub async fn get_recent_plays(
    db: State<'_, Database>,
    limit: Option<i64>,
) -> AppResult<Vec<play_history::PlayHistoryEntry>> {
    db.with_conn(|c| play_history::get_recent(c, limit.unwrap_or(50)))
}

#[tauri::command]
pub async fn set_favorite(db: State<'_, Database>, track_id: i64, favorite: bool) -> AppResult<()> {
    db.with_conn(|c| tracks::set_favorite(c, track_id, favorite, now_ms()))
}

#[tauri::command]
pub async fn record_play(
    db: State<'_, Database>,
    track_id: i64,
    duration_played_ms: i64,
    completed: bool,
) -> AppResult<i64> {
    db.with_conn(|c| play_history::record(c, track_id, now_ms(), duration_played_ms, completed))
}

#[tauri::command]
pub async fn search(
    db: State<'_, Database>,
    query: String,
    limit: Option<i64>,
) -> AppResult<Vec<tracks::TrackView>> {
    db.with_conn(|c| search::search_tracks(c, &query, limit.unwrap_or(50)))
}

// ---- scan-folder CRUD commands ----

#[tauri::command]
pub fn add_music_folder(
    db: State<'_, Database>,
    path: String,
) -> AppResult<folders_repo::ScanFolder> {
    let now = now_ms();
    folders_repo::add(&db.lock_conn(), &path, now)
}

#[tauri::command]
pub fn list_music_folders(db: State<'_, Database>) -> AppResult<Vec<folders_repo::ScanFolder>> {
    folders_repo::list(&db.lock_conn())
}

#[tauri::command]
pub fn remove_music_folder(db: State<'_, Database>, id: i64) -> AppResult<()> {
    let now = now_ms();
    let conn = db.lock_conn();
    let _ = tracks::mark_missing_by_root(&conn, id, now)?;
    let _ = tracks::unlink_root(&conn, id)?;
    folders_repo::delete(&conn, id)
}

// ---- scan control commands ----

#[tauri::command]
pub async fn start_scan(
    app: AppHandle,
    db: State<'_, Database>,
    manager: State<'_, ScanManager>,
) -> AppResult<()> {
    if manager
        .running
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return Err(AppError::Busy);
    }

    let abort = manager.abort.clone();
    abort.reset();
    let db_for_scan: Database = (*db).clone();
    let running = Arc::clone(&manager.running);

    let cache_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Scan(format!("app_data_dir: {e}")))?
        .join("covers");
    std::fs::create_dir_all(&cache_dir).ok();

    tauri::async_runtime::spawn_blocking(move || {
        let ctx = scanner::ScanContext {
            db: &db_for_scan,
            cache_dir,
        };
        let last_emit = std::sync::Mutex::new(None::<Instant>);
        let on_progress = |p: ScanProgress| {
            let mut last = last_emit.lock().unwrap();
            let now = Instant::now();
            let should = last
                .map(|t| now.duration_since(t) >= Duration::from_millis(100))
                .unwrap_or(true);
            if should {
                *last = Some(now);
                let _ = app.emit("scan_progress", &p);
            }
        };
        let result = scanner::scan_folders(&ctx, &abort, &on_progress);
        match result {
            Ok(report) => {
                let _ = app.emit("scan_done", &report);
            }
            Err(e) => {
                let err = ScanReport {
                    errors: vec![scanner::ScanError {
                        path: String::new(),
                        message: format!("{e}"),
                    }],
                    ..Default::default()
                };
                let _ = app.emit("scan_done", &err);
            }
        }
        running.store(false, Ordering::SeqCst);
    });

    Ok(())
}

#[tauri::command]
pub fn cancel_scan(manager: State<'_, ScanManager>) -> AppResult<()> {
    manager.abort.signal();
    Ok(())
}

// ---- v0.6.0: grouped search, favorites, recent plays ----

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResultDto {
    pub tracks: Vec<tracks::TrackView>,
    pub albums: Vec<albums::AlbumView>,
    pub artists: Vec<artists::Artist>,
    pub playlists: Vec<crate::db::playlists::PlaylistSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecentPlayedTrackDto {
    #[serde(flatten)]
    pub track: tracks::TrackView,
    pub last_played_at: i64,
}

#[tauri::command]
pub async fn library_search_all(
    db: State<'_, Database>,
    query: String,
    limit_per_group: Option<i64>,
) -> AppResult<SearchResultDto> {
    let trimmed = query.trim().to_string();
    if trimmed.is_empty() {
        return Ok(SearchResultDto {
            tracks: vec![],
            albums: vec![],
            artists: vec![],
            playlists: vec![],
        });
    }
    let result =
        db.with_conn(|c| search::search_all(c, &trimmed, limit_per_group.unwrap_or(10)))?;
    Ok(SearchResultDto {
        tracks: result.tracks,
        albums: result.albums,
        artists: result.artists,
        playlists: result.playlists,
    })
}

#[tauri::command]
pub async fn library_get_favorite_tracks(
    db: State<'_, Database>,
) -> AppResult<Vec<tracks::TrackView>> {
    db.with_conn(tracks::list_favorite_tracks)
}

#[tauri::command]
pub async fn library_get_recent_played_tracks(
    db: State<'_, Database>,
    limit: Option<i64>,
) -> AppResult<Vec<RecentPlayedTrackDto>> {
    let rows = db.with_conn(|c| play_history::list_recent_played_tracks(c, limit.unwrap_or(50)))?;
    Ok(rows
        .into_iter()
        .map(|row| RecentPlayedTrackDto {
            track: row.track,
            last_played_at: row.last_played_at,
        })
        .collect())
}

// ---- legacy alias ----

/// 向后兼容旧 API：v0.3.0 之前 add_folder 只写 scan_folders 表不做扫描。
/// 现在等同于 add_music_folder。
#[tauri::command]
pub fn add_folder(db: State<'_, Database>, path: String) -> AppResult<()> {
    add_music_folder(db, path).map(|_| ())
}
