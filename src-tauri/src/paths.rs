use std::path::{Path, PathBuf};

use crate::error::Result;
use crate::settings::{Settings, MANIFEST_FILE_NAME};

/// Diretorios de dados do aplicativo.
#[derive(Clone, Debug)]
pub struct Paths {
    /// %APPDATA%\com.nrmanager.app  (config, library, compat, logs)
    pub app_data: PathBuf,
    /// %LOCALAPPDATA%\com.nrmanager.app  (cache padrao)
    pub default_cache: PathBuf,
}

impl Paths {
    pub fn new(app_data: PathBuf, default_cache: PathBuf) -> Self {
        Self {
            app_data,
            default_cache,
        }
    }

    pub fn ensure(&self) -> Result<()> {
        let dirs = [
            self.app_data.clone(),
            self.backups_dir(),
            self.manifests_dir(),
            self.logs_dir(),
        ];
        for p in dirs {
            std::fs::create_dir_all(p)?;
        }
        Ok(())
    }

    pub fn config_file(&self) -> PathBuf {
        self.app_data.join("config.json")
    }
    pub fn library_file(&self) -> PathBuf {
        self.app_data.join("library.json")
    }
    pub fn compat_file(&self) -> PathBuf {
        self.app_data.join("compat.json")
    }
    pub fn backups_dir(&self) -> PathBuf {
        self.app_data.join("backups")
    }
    pub fn manifests_dir(&self) -> PathBuf {
        self.app_data.join("manifests")
    }
    pub fn logs_dir(&self) -> PathBuf {
        self.app_data.join("logs")
    }
    pub fn log_file(&self) -> PathBuf {
        self.logs_dir().join("nrmanager.log")
    }
    pub fn manifest_file(&self, game_id: &str) -> PathBuf {
        let safe = sanitize(game_id);
        self.manifests_dir().join(format!("{safe}.json"))
    }
    pub fn backup_dir_for(&self, game_id: &str) -> PathBuf {
        let safe = sanitize(game_id);
        self.backups_dir().join(safe)
    }

    /// Resolve a pasta de cache conforme as configuracoes.
    pub fn cache_dir(&self, settings: &Settings) -> PathBuf {
        if settings.cache_dir.trim().is_empty() {
            self.default_cache.join("cache")
        } else {
            PathBuf::from(settings.cache_dir.trim())
        }
    }

    pub fn releases_dir(&self, settings: &Settings) -> PathBuf {
        self.cache_dir(settings).join("releases")
    }
}

/// Caminho do manifesto dentro da pasta do jogo.
pub fn manifest_in_game_dir(mod_dir: &Path) -> PathBuf {
    mod_dir.join(MANIFEST_FILE_NAME)
}

fn sanitize(id: &str) -> String {
    id.chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect()
}
