mod error;
mod db;
mod commands;
mod player;
mod library;
mod metadata;
mod system;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            log::info!("MusicPlayer v{} 启动", app.package_info().version);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::player::play,
            commands::player::pause,
            commands::player::resume,
            commands::player::seek,
            commands::player::next,
            commands::player::prev,
            commands::library::get_tracks,
            commands::library::get_albums,
            commands::library::get_artists,
            commands::library::search,
            commands::library::add_folder,
            commands::playlist::get_playlists,
            commands::playlist::create_playlist,
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
