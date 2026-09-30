use parking_lot::Mutex;

use crate::error::Result;
use crate::github::FeedCache;
use crate::logging::Logger;
use crate::models::Game;
use crate::paths::Paths;
use crate::settings::Settings;

/// Compatibilidade embutida, copiada para o AppData no primeiro uso.
const BUNDLED_COMPAT: &str = include_str!("../../compat.json");

/// Estado global compartilhado entre os comandos.
pub struct AppState {
    pub paths: Paths,
    pub logger: Logger,
    pub settings: Mutex<Settings>,
    pub library: Mutex<Vec<Game>>,
    pub client: reqwest::Client,
    pub feed: Mutex<Option<FeedCache>>,
}

impl AppState {
    pub fn init(paths: Paths) -> Result<Self> {
        paths.ensure()?;
        let logger = Logger::new(&paths);

        // Configuracoes.
        let settings = load_settings(&paths).unwrap_or_default();
        let _ = save_json(&paths.config_file(), &settings);

        // compat.json: semeia a partir do arquivo embutido na primeira execucao.
        if !paths.compat_file().exists() {
            let _ = std::fs::write(paths.compat_file(), BUNDLED_COMPAT);
            logger.info("compat.json inicial criado no AppData.");
        }

        // Biblioteca.
        let library = load_library(&paths);

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .connect_timeout(std::time::Duration::from_secs(15))
            .build()
            .unwrap_or_default();

        Ok(Self {
            paths,
            logger,
            settings: Mutex::new(settings),
            library: Mutex::new(library),
            client,
            feed: Mutex::new(None),
        })
    }

    pub fn settings(&self) -> Settings {
        self.settings.lock().clone()
    }

    pub fn save_settings(&self) -> Result<()> {
        let s = self.settings.lock().clone();
        save_json(&self.paths.config_file(), &s)
    }

    pub fn save_library(&self) -> Result<()> {
        let lib = self.library.lock().clone();
        save_json(&self.paths.library_file(), &lib)
    }

    pub fn compat_text(&self) -> String {
        std::fs::read_to_string(self.paths.compat_file()).unwrap_or_else(|_| BUNDLED_COMPAT.to_string())
    }
}

pub fn load_settings(paths: &Paths) -> Option<Settings> {
    let text = std::fs::read_to_string(paths.config_file()).ok()?;
    serde_json::from_str(&text).ok()
}

pub fn load_library(paths: &Paths) -> Vec<Game> {
    std::fs::read_to_string(paths.library_file())
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_default()
}

pub fn save_json<T: serde::Serialize>(path: &std::path::Path, value: &T) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let text = serde_json::to_string_pretty(value)?;
    std::fs::write(path, text)?;
    Ok(())
}
