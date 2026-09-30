mod commands;
mod error;
mod games;
mod github;
mod gpu;
mod install;
mod keyvalues;
mod logging;
mod manifest;
mod models;
mod paths;
mod settings;
mod state;
mod util;

use tauri::Manager;

use crate::paths::Paths;
use crate::state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_data = app.path().app_data_dir()?;
            let cache = app.path().app_cache_dir()?;
            let paths = Paths::new(app_data, cache);
            let state = AppState::init(paths)?;
            state.logger.attach(app.handle().clone());
            state.logger.info("NR Manager iniciado.");
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_app_info,
            commands::get_settings,
            commands::save_settings,
            commands::detect_gpu,
            commands::is_elevated,
            commands::restart_elevated,
            commands::list_games,
            commands::scan_library,
            commands::refresh_compat,
            commands::add_game_manual,
            commands::remove_game,
            commands::update_game,
            commands::find_executable,
            commands::apply_mod,
            commands::remove_mod,
            commands::repair_mod,
            commands::get_game_status,
            commands::list_releases,
            commands::list_cached_releases,
            commands::download_release,
            commands::import_custom_setup,
            commands::remove_cached_release,
            commands::update_all,
            commands::get_compat_raw,
            commands::get_compat_entries,
            commands::save_compat,
            commands::get_logs,
            commands::clear_logs,
            commands::set_start_with_windows,
            commands::find_mod_config,
            commands::reveal_in_explorer,
        ])
        .run(tauri::generate_context!())
        .expect("erro ao executar o NR Manager");
}
