mod commands;
pub mod db;
pub mod error;
pub mod library;
mod metadata;
mod player;
mod system;

use commands::library::ScanManager;
use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(ScanManager::default())
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir().expect("no app_data_dir");
            std::fs::create_dir_all(&app_data_dir).ok();
            let db_path = app_data_dir.join("musicplayer.db");
            log::info!("opening DB at {}", db_path.display());
            let db = db::Database::open(&db_path).expect("failed to open database");

            let player = player::manager::PlayerManager::new(app.handle().clone(), db.clone());
            app.manage(db);
            app.manage(player);

            log::info!("MusicPlayer v{} 启动", app.package_info().version);
            Ok(())
        })
        .on_window_event(|window, event| {
            if matches!(event, tauri::WindowEvent::CloseRequested { .. }) {
                if let Some(manager) = window
                    .app_handle()
                    .try_state::<player::manager::PlayerManager>()
                {
                    manager.shutdown();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::player::player_play,
            commands::player::player_pause,
            commands::player::player_resume,
            commands::player::player_toggle,
            commands::player::player_stop,
            commands::player::player_seek,
            commands::player::player_next,
            commands::player::player_previous,
            commands::player::player_set_volume,
            commands::player::player_set_muted,
            commands::player::player_toggle_mute,
            commands::player::player_get_state,
            commands::player::play,
            commands::player::pause,
            commands::player::resume,
            commands::player::seek,
            commands::player::next,
            commands::player::prev,
            commands::library::get_tracks,
            commands::library::get_albums,
            commands::library::get_artists,
            commands::library::get_album,
            commands::library::get_artist,
            commands::library::get_album_tracks,
            commands::library::get_artist_tracks,
            commands::library::get_recently_added,
            commands::library::get_recent_plays,
            commands::library::set_favorite,
            commands::library::record_play,
            commands::library::search,
            commands::library::add_music_folder,
            commands::library::list_music_folders,
            commands::library::remove_music_folder,
            commands::library::start_scan,
            commands::library::cancel_scan,
            commands::library::add_folder,
            commands::playlist::get_playlists,
            commands::playlist::get_playlist_tracks,
            commands::playlist::create_playlist,
            commands::playlist::rename_playlist,
            commands::playlist::delete_playlist,
            commands::playlist::add_to_playlist,
            commands::playlist::remove_from_playlist,
            commands::playlist::reorder_playlist,
            commands::prefs::get_pref,
            commands::prefs::set_pref,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
