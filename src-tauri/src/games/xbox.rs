use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use crate::logging::Logger;
use crate::models::Game;
use crate::util;

/// Detecta jogos Xbox/Game Pass escaneando pastas XboxGames em cada drive.
pub fn detect(logger: &Logger) -> Vec<Game> {
    let mut games = Vec::new();
    for drive in b'A'..=b'Z' {
        let root = PathBuf::from(format!("{}:\\XboxGames", drive as char));
        if !root.exists() {
            continue;
        }
        let Ok(entries) = std::fs::read_dir(&root) else {
            continue;
        };
        for entry in entries.flatten() {
            if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                continue;
            }
            let name = entry.file_name().to_string_lossy().to_string();
            // Alguns diretorios internos nao sao jogos.
            if name.starts_with('.') || name.eq_ignore_ascii_case("Microsoft") {
                continue;
            }
            let dir = entry.path();
            if let Some(exe) = largest_exe(&dir) {
                games.push(Game {
                    id: format!("xbox:{name}"),
                    name,
                    source: "xbox".into(),
                    app_id: None,
                    install_dir: util::to_win_string(&dir),
                    exe_path: Some(util::to_win_string(&exe)),
                    ..Default::default()
                });
            }
        }
    }
    if games.is_empty() {
        logger.info("Xbox/Game Pass nao encontrado (ou sem jogos em XboxGames).");
    }
    games
}

fn largest_exe(dir: &Path) -> Option<PathBuf> {
    let mut best: Option<(u64, PathBuf)> = None;
    for entry in WalkDir::new(dir).max_depth(3).into_iter().flatten() {
        if !entry.file_type().is_file() {
            continue;
        }
        let is_exe = entry
            .path()
            .extension()
            .map(|e| e.eq_ignore_ascii_case("exe"))
            .unwrap_or(false);
        if !is_exe {
            continue;
        }
        let len = entry.metadata().map(|m| m.len()).unwrap_or(0);
        if best.as_ref().map(|(b, _)| len > *b).unwrap_or(true) {
            best = Some((len, entry.path().to_path_buf()));
        }
    }
    best.map(|(_, p)| p)
}
