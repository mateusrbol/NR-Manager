use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::logging::Logger;
use crate::models::Game;
use crate::util;

#[derive(Deserialize)]
struct EpicManifest {
    #[serde(rename = "DisplayName")]
    display_name: Option<String>,
    #[serde(rename = "InstallLocation")]
    install_location: Option<String>,
    #[serde(rename = "AppName")]
    app_name: Option<String>,
    #[serde(rename = "LaunchExecutable")]
    launch_executable: Option<String>,
    #[serde(rename = "CatalogNamespace")]
    catalog_namespace: Option<String>,
}

/// Detecta jogos da Epic Games lendo os manifestos em ProgramData.
pub fn detect(logger: &Logger) -> Vec<Game> {
    let mut games = Vec::new();
    let manifest_dir = PathBuf::from("C:\\ProgramData\\Epic\\EpicGamesLauncher\\Data\\Manifests");
    let Ok(entries) = std::fs::read_dir(&manifest_dir) else {
        logger.info("Epic Games nao encontrado.");
        return games;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().map(|e| e != "item").unwrap_or(true) {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        let Ok(m) = serde_json::from_str::<EpicManifest>(&text) else {
            continue;
        };
        let (Some(name), Some(location), Some(app_name)) =
            (m.display_name, m.install_location, m.app_name)
        else {
            continue;
        };
        let install = Path::new(&location);
        if !install.exists() {
            continue;
        }
        // Ignora engines/plugins sem executavel de jogo.
        if m.launch_executable.as_deref().unwrap_or("").is_empty()
            && m.catalog_namespace.as_deref().unwrap_or("").is_empty()
        {
            continue;
        }
        let exe = m
            .launch_executable
            .as_deref()
            .map(|e| install.join(e.replace('/', "\\")));
        games.push(Game {
            id: format!("epic:{app_name}"),
            name,
            source: "epic".into(),
            app_id: Some(app_name),
            install_dir: util::to_win_string(install),
            exe_path: exe.filter(|p| p.exists()).map(|p| util::to_win_string(&p)),
            ..Default::default()
        });
    }
    games
}
