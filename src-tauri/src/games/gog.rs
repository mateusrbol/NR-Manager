use std::path::{Path, PathBuf};

use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};
use winreg::RegKey;

use crate::logging::Logger;
use crate::models::Game;
use crate::util;

/// Detecta jogos do GOG Galaxy/offline via registro.
pub fn detect(logger: &Logger) -> Vec<Game> {
    let mut games = Vec::new();
    let roots = [
        (HKEY_LOCAL_MACHINE, "SOFTWARE\\WOW6432Node\\GOG.com\\Games"),
        (HKEY_LOCAL_MACHINE, "SOFTWARE\\GOG.com\\Games"),
        (HKEY_CURRENT_USER, "Software\\GOG.com\\Games"),
    ];

    for (hive, sub) in roots {
        let Ok(key) = RegKey::predef(hive).open_subkey(sub) else {
            continue;
        };
        for id in key.enum_keys().flatten() {
            let Ok(game_key) = key.open_subkey(&id) else {
                continue;
            };
            let name = game_key.get_value::<String, _>("gameName").unwrap_or_default();
            let path = game_key.get_value::<String, _>("path").unwrap_or_default();
            if name.is_empty() || path.is_empty() {
                continue;
            }
            let install = PathBuf::from(&path);
            if !install.exists() {
                continue;
            }
            let game_id = game_key
                .get_value::<String, _>("gameID")
                .unwrap_or_else(|_| id.clone());

            let exe_val = game_key.get_value::<String, _>("exe").unwrap_or_default();
            let exe = if exe_val.is_empty() {
                None
            } else {
                let p = Path::new(&exe_val);
                let full = if p.is_absolute() { p.to_path_buf() } else { install.join(p) };
                full.exists().then_some(full)
            };

            games.push(Game {
                id: format!("gog:{game_id}"),
                name,
                source: "gog".into(),
                app_id: Some(game_id),
                install_dir: util::to_win_string(&install),
                exe_path: exe.map(|p| util::to_win_string(&p)),
                ..Default::default()
            });
        }
    }
    if games.is_empty() {
        logger.info("GOG nao encontrado.");
    }
    games
}
