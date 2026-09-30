use std::path::{Path, PathBuf};

use serde::Serialize;
use tauri::{AppHandle, Emitter, State};

use crate::error::{AppError, Result};
use crate::games;
use crate::github;
use crate::gpu;
use crate::install;
use crate::models::{
    CachedRelease, CompatEntry, Game, GameStatus, GpuInfo, LogEntry, ReleaseFeed, RepairReport,
};
use crate::settings::Settings;
use crate::state::AppState;
use crate::util;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    pub app_data: String,
    pub cache_dir: String,
    pub log_file: String,
    pub elevated: bool,
    pub version: String,
}

#[tauri::command]
pub fn get_app_info(state: State<'_, AppState>) -> AppInfo {
    let settings = state.settings();
    AppInfo {
        app_data: util::to_win_string(&state.paths.app_data),
        cache_dir: util::to_win_string(&state.paths.cache_dir(&settings)),
        log_file: util::to_win_string(&state.paths.log_file()),
        elevated: install::is_elevated(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    }
}

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Settings {
    state.settings()
}

#[tauri::command]
pub fn save_settings(state: State<'_, AppState>, mut settings: Settings) -> Result<Settings> {
    if settings.repo_parts().is_none() {
        return Err(AppError::msg(
            "URL do repositorio invalida. Use o formato https://github.com/usuario/repo",
        ));
    }
    // Normaliza e persiste.
    settings.repo_url = settings.repo_url.trim().to_string();
    *state.settings.lock() = settings.clone();
    state.save_settings()?;
    // Invalida cache em memoria do feed.
    *state.feed.lock() = None;
    state.logger.info("Configuracoes salvas.");
    Ok(settings)
}

#[tauri::command]
pub fn detect_gpu() -> GpuInfo {
    gpu::detect()
}

#[tauri::command]
pub fn is_elevated() -> bool {
    install::is_elevated()
}

#[tauri::command]
pub fn restart_elevated() -> Result<()> {
    install::restart_elevated()
}

#[tauri::command]
pub fn list_games(state: State<'_, AppState>) -> Vec<Game> {
    state.library.lock().clone()
}

/// Redetecta a biblioteca e reavalia compatibilidade/estado.
#[tauri::command]
pub async fn scan_library(app: AppHandle, state: State<'_, AppState>) -> Result<Vec<Game>> {
    let logger = state.logger.clone();
    let detected = tauri::async_runtime::spawn_blocking(move || games::detect_all(&logger))
        .await
        .map_err(|e| AppError::msg(format!("Falha na deteccao: {e}")))?;

    let compat = games::load_compat(&state.paths);
    let existing = state.library.lock().clone();

    let mut merged: Vec<Game> = Vec::new();
    for mut g in detected {
        if let Some(old) = existing.iter().find(|o| o.id == g.id) {
            // Preserva correcoes manuais do usuario.
            if old.exe_path.is_some() {
                g.exe_path = old.exe_path.clone();
            }
            if old.mod_dir.is_some() {
                g.mod_dir = old.mod_dir.clone();
            }
            g.added_manually = old.added_manually;
        }
        merged.push(g);
    }
    for old in existing {
        if !merged.iter().any(|m| m.id == old.id) {
            merged.push(old);
        }
    }

    for g in merged.iter_mut() {
        games::enrich(g, &compat, &state.logger);
        if let Err(e) = games::refresh_install_state(g, &state.paths) {
            state.logger.warn(format!("Estado de '{}': {e}", g.name));
        }
    }

    *state.library.lock() = merged.clone();
    state.save_library()?;
    let _ = app.emit("nr-library-updated", ());
    Ok(merged)
}

/// Reavalia a compatibilidade de todos os jogos sem varrer os lancadores.
#[tauri::command]
pub fn refresh_compat(state: State<'_, AppState>) -> Vec<Game> {
    let compat = games::load_compat(&state.paths);
    let mut lib = state.library.lock();
    for g in lib.iter_mut() {
        games::enrich(g, &compat, &state.logger);
        let _ = games::refresh_install_state(g, &state.paths);
    }
    let out = lib.clone();
    drop(lib);
    let _ = state.save_library();
    out
}

#[tauri::command]
pub fn add_game_manual(
    state: State<'_, AppState>,
    path: String,
    name: Option<String>,
) -> Result<Game> {
    let p = PathBuf::from(path.trim());
    if !p.exists() {
        return Err(AppError::NotFound(path));
    }

    let (install_dir, exe_path, default_name) = if p.is_file() {
        let parent = p
            .parent()
            .ok_or_else(|| AppError::msg("Caminho de arquivo invalido."))?
            .to_path_buf();
        let stem = p
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "Jogo".into());
        (parent, Some(p.clone()), stem)
    } else {
        let exe = games::find_executable(&p, &p.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default());
        let name = p
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "Jogo".into());
        (p, exe, name)
    };

    let game_name = name.filter(|n| !n.trim().is_empty()).unwrap_or(default_name);
    let id = format!(
        "manual:{}",
        chrono::Local::now().format("%Y%m%d%H%M%S%3f")
    );

    let mut game = Game {
        id,
        name: game_name,
        source: "manual".into(),
        app_id: None,
        install_dir: util::to_win_string(&install_dir),
        exe_path: exe_path.map(|e| util::to_win_string(&e)),
        mod_dir: None,
        compat: "untested".into(),
        graphics_api: "unknown".into(),
        added_manually: true,
        ..Default::default()
    };
    if game.exe_path.is_some() {
        game.mod_dir = game
            .exe_path
            .as_ref()
            .and_then(|e| Path::new(e).parent())
            .map(util::to_win_string);
    }

    let compat = games::load_compat(&state.paths);
    games::enrich(&mut game, &compat, &state.logger);
    let _ = games::refresh_install_state(&mut game, &state.paths);

    install::upsert(&state, game.clone());
    state.save_library()?;
    state.logger.info(format!("Jogo adicionado manualmente: {}", game.name));
    Ok(game)
}

#[tauri::command]
pub fn remove_game(state: State<'_, AppState>, id: String) -> Result<()> {
    {
        let mut lib = state.library.lock();
        let before = lib.len();
        lib.retain(|g| g.id != id);
        if lib.len() == before {
            return Err(AppError::msg("Jogo nao encontrado."));
        }
    }
    state.save_library()?;
    Ok(())
}

/// Atualiza campos editaveis de um jogo e reavalia heuristica/estado.
#[tauri::command]
pub fn update_game(state: State<'_, AppState>, mut game: Game) -> Result<Game> {
    if !Path::new(&game.install_dir).exists() {
        return Err(AppError::NotFound(game.install_dir.clone()));
    }
    // So deriva a pasta do mod do executavel quando o usuario nao definiu uma.
    let mod_dir_empty = game
        .mod_dir
        .as_deref()
        .map(|s| s.trim().is_empty())
        .unwrap_or(true);
    if mod_dir_empty {
        if let Some(exe) = &game.exe_path {
            if let Some(parent) = Path::new(exe).parent() {
                game.mod_dir = Some(util::to_win_string(parent));
            }
        }
    }
    let compat = games::load_compat(&state.paths);
    games::enrich(&mut game, &compat, &state.logger);
    let _ = games::refresh_install_state(&mut game, &state.paths);
    install::upsert(&state, game.clone());
    state.save_library()?;
    Ok(game)
}

#[tauri::command]
pub fn find_executable(install_dir: String, name: String) -> Option<String> {
    games::find_executable(Path::new(&install_dir), &name).map(|p| util::to_win_string(&p))
}

#[tauri::command]
pub async fn apply_mod(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
    tag: Option<String>,
) -> Result<Game> {
    install::apply_mod(&app, &state, &id, tag).await
}

#[tauri::command]
pub async fn remove_mod(app: AppHandle, state: State<'_, AppState>, id: String) -> Result<Game> {
    install::uninstall(&app, &state, &id).await
}

#[tauri::command]
pub fn repair_mod(state: State<'_, AppState>, id: String) -> Result<RepairReport> {
    install::repair(&state, &id)
}

#[tauri::command]
pub fn get_game_status(state: State<'_, AppState>, id: String) -> Result<GameStatus> {
    install::game_status(&state, &id)
}

#[tauri::command]
pub async fn list_releases(
    state: State<'_, AppState>,
    force: bool,
) -> Result<ReleaseFeed> {
    let settings = state.settings();
    github::get_feed(
        &state.client,
        &state.paths,
        &settings,
        &state.logger,
        &state.feed,
        force,
    )
    .await
}

#[tauri::command]
pub fn list_cached_releases(state: State<'_, AppState>) -> Vec<CachedRelease> {
    let settings = state.settings();
    github::list_cached(&state.paths, &settings)
}

#[tauri::command]
pub async fn download_release(
    app: AppHandle,
    state: State<'_, AppState>,
    tag: String,
) -> Result<CachedRelease> {
    let settings = state.settings();
    let feed = github::get_feed(
        &state.client,
        &state.paths,
        &settings,
        &state.logger,
        &state.feed,
        false,
    )
    .await?;
    let release = feed
        .releases
        .iter()
        .find(|r| r.tag == tag)
        .cloned()
        .ok_or_else(|| AppError::msg("Versao nao encontrada nas releases."))?;
    github::download_setup(
        &app,
        &state.client,
        &state.paths,
        &settings,
        &state.logger,
        &release,
    )
    .await
}

#[tauri::command]
pub async fn import_custom_setup(
    state: State<'_, AppState>,
    path: String,
) -> Result<CachedRelease> {
    let settings = state.settings();
    github::import_custom_setup(&state.paths, &settings, &state.logger, Path::new(&path)).await
}

#[tauri::command]
pub fn remove_cached_release(state: State<'_, AppState>, tag: String) -> Result<()> {
    let settings = state.settings();
    github::remove_cached(&state.paths, &settings, &tag)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateFailure {
    pub game_id: String,
    pub name: String,
    pub message: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAllResult {
    pub updated: Vec<Game>,
    pub failed: Vec<UpdateFailure>,
}

#[tauri::command]
pub async fn update_all(
    app: AppHandle,
    state: State<'_, AppState>,
    ids: Vec<String>,
    tag: Option<String>,
) -> Result<UpdateAllResult> {
    let mut updated = Vec::new();
    let mut failed = Vec::new();
    for id in ids {
        let name = install::find_game(&state, &id)
            .map(|g| g.name)
            .unwrap_or_else(|_| id.clone());
        match install::apply_mod(&app, &state, &id, tag.clone()).await {
            Ok(g) => updated.push(g),
            Err(e) => failed.push(UpdateFailure {
                game_id: id,
                name,
                message: e.to_string(),
            }),
        }
    }
    Ok(UpdateAllResult { updated, failed })
}

#[tauri::command]
pub fn get_compat_raw(state: State<'_, AppState>) -> String {
    state.compat_text()
}

#[tauri::command]
pub fn get_compat_entries(state: State<'_, AppState>) -> Vec<CompatEntry> {
    games::load_compat(&state.paths)
}

#[tauri::command]
pub fn save_compat(state: State<'_, AppState>, content: String) -> Result<()> {
    // Valida o JSON antes de gravar.
    #[derive(serde::Deserialize)]
    struct Wrapper {
        #[allow(dead_code)]
        #[serde(default)]
        entries: Vec<CompatEntry>,
    }
    let parsed: Wrapper = serde_json::from_str(&content)
        .map_err(|e| AppError::msg(format!("compat.json invalido: {e}")))?;
    let _ = parsed;
    std::fs::write(state.paths.compat_file(), &content)?;
    // Reavalia heuristica.
    let entries = games::load_compat(&state.paths);
    {
        let mut lib = state.library.lock();
        for g in lib.iter_mut() {
            games::enrich(g, &entries, &state.logger);
        }
    }
    state.save_library()?;
    state.logger.info("compat.json atualizado.");
    Ok(())
}

#[tauri::command]
pub fn get_logs(state: State<'_, AppState>) -> Vec<LogEntry> {
    state.logger.snapshot()
}

#[tauri::command]
pub fn clear_logs(state: State<'_, AppState>) {
    state.logger.clear();
}

#[tauri::command]
pub fn set_start_with_windows(state: State<'_, AppState>, enabled: bool) -> Result<()> {
    let exe = std::env::current_exe()?;
    let key_path = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
    let key = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER)
        .open_subkey_with_flags(key_path, winreg::enums::KEY_SET_VALUE)
        .map_err(|e| AppError::Permission(format!("Nao foi possivel abrir o registro: {e}")))?;
    if enabled {
        key.set_value("NRManager", &exe.to_string_lossy().to_string())
            .map_err(|e| AppError::Permission(format!("Falha ao gravar no registro: {e}")))?;
    } else {
        let _ = key.delete_value("NRManager");
    }
    {
        let mut s = state.settings.lock();
        s.start_with_windows = enabled;
    }
    state.save_settings()?;
    Ok(())
}

/// Localiza o arquivo de configuracao do mod na pasta do jogo.
#[tauri::command]
pub fn find_mod_config(state: State<'_, AppState>, id: String) -> Result<Option<String>> {
    let game = install::find_game(&state, &id)?;
    let Some(mod_dir) = game.mod_dir else {
        return Ok(None);
    };
    let dir = PathBuf::from(&mod_dir);
    if !dir.exists() {
        return Ok(None);
    }
    let candidates = [
        "dlssnr_on_amd.ini",
        "dlssnr_on_amd.cfg",
        "dlssnr.ini",
        "nvngx.ini",
        "dlssnr_on_amd.json",
    ];
    for c in candidates {
        let p = dir.join(c);
        if p.exists() {
            return Ok(Some(util::to_win_string(&p)));
        }
    }
    // Procura por qualquer arquivo relacionado.
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for e in entries.flatten() {
            let n = e.file_name().to_string_lossy().to_lowercase();
            if (n.contains("dlssnr") || n.contains("nvngx"))
                && (n.ends_with(".ini") || n.ends_with(".cfg") || n.ends_with(".json"))
            {
                return Ok(Some(util::to_win_string(&e.path())));
            }
        }
    }
    Ok(None)
}

/// Abre a pasta/arquivo no Explorer (usado como fallback).
#[tauri::command]
pub fn reveal_in_explorer(path: String) -> Result<()> {
    let p = PathBuf::from(&path);
    if !p.exists() {
        return Err(AppError::NotFound(path));
    }
    if p.is_dir() {
        std::process::Command::new("explorer").arg(&p).spawn()?;
    } else {
        std::process::Command::new("explorer")
            .arg(format!("/select,{}", p.display()))
            .spawn()?;
    }
    Ok(())
}
