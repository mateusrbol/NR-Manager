use std::path::PathBuf;

use winreg::enums::HKEY_CURRENT_USER;
use winreg::enums::HKEY_LOCAL_MACHINE;
use winreg::RegKey;

use crate::keyvalues;
use crate::logging::Logger;
use crate::models::Game;
use crate::util;

/// Detecta jogos instalados via Steam (multiplas bibliotecas/drives).
pub fn detect(logger: &Logger) -> Vec<Game> {
    let mut games = Vec::new();
    let Some(steam) = install_path() else {
        logger.info("Steam nao encontrado.");
        return games;
    };
    logger.info(format!("Steam em {}", steam.display()));

    for library in library_folders(&steam, logger) {
        let steamapps = library.join("steamapps");
        let Ok(entries) = std::fs::read_dir(&steamapps) else {
            continue;
        };
        for entry in entries.flatten() {
            let fname = entry.file_name().to_string_lossy().to_string();
            let lower = fname.to_lowercase();
            if !lower.starts_with("appmanifest_") || !lower.ends_with(".acf") {
                continue;
            }
            let Ok(kv) = keyvalues::parse_file(&entry.path()) else {
                continue;
            };
            let Some(app) = kv.get("AppState") else {
                continue;
            };
            let app_id = app.get_str("appid").unwrap_or_default().to_string();
            let name = app.get_str("name").unwrap_or_default().to_string();
            let installdir = app.get_str("installdir").unwrap_or_default().to_string();
            if app_id.is_empty() || installdir.is_empty() || is_tool(&name) {
                continue;
            }
            let install_dir = steamapps.join("common").join(&installdir);
            if !install_dir.exists() {
                continue;
            }
            games.push(Game {
                id: format!("steam:{app_id}"),
                name,
                source: "steam".into(),
                app_id: Some(app_id.clone()),
                install_dir: util::to_win_string(&install_dir),
                cover_url: Some(format!(
                    "https://cdn.cloudflare.steamstatic.com/steam/apps/{app_id}/library_600x900.jpg"
                )),
                ..Default::default()
            });
        }
    }
    games
}

fn is_tool(name: &str) -> bool {
    let l = name.to_lowercase();
    l.starts_with("steamworks")
        || l.starts_with("steam linux runtime")
        || l.starts_with("proton")
        || l.contains("redistributables")
}

fn install_path() -> Option<PathBuf> {
    if let Ok(key) = RegKey::predef(HKEY_CURRENT_USER).open_subkey("Software\\Valve\\Steam") {
        if let Ok(v) = key.get_value::<String, _>("SteamPath") {
            let p = PathBuf::from(v.replace('/', "\\"));
            if p.exists() {
                return Some(p);
            }
        }
    }
    if let Ok(key) =
        RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey("SOFTWARE\\WOW6432Node\\Valve\\Steam")
    {
        if let Ok(v) = key.get_value::<String, _>("InstallPath") {
            let p = PathBuf::from(v.replace('/', "\\"));
            if p.exists() {
                return Some(p);
            }
        }
    }
    let fallback = PathBuf::from("C:\\Program Files (x86)\\Steam");
    fallback.exists().then_some(fallback)
}

fn library_folders(steam: &std::path::Path, logger: &Logger) -> Vec<PathBuf> {
    let mut libs: Vec<PathBuf> = vec![steam.to_path_buf()];
    let vdf = steam.join("steamapps").join("libraryfolders.vdf");
    let Ok(kv) = keyvalues::parse_file(&vdf) else {
        logger.warn(format!("Nao foi possivel ler {}", vdf.display()));
        return libs;
    };
    let Some(root) = kv.get("libraryfolders") else {
        return libs;
    };
    for (_, value) in root.iter() {
        if let Some(path) = value.get_str("path") {
            let p = PathBuf::from(path.replace("\\\\", "\\"));
            if p.exists() && !libs.iter().any(|l| util::path_eq(l, &p)) {
                libs.push(p);
            }
        }
    }
    libs
}
