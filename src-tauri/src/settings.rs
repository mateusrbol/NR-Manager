use serde::{Deserialize, Serialize};

pub const DEFAULT_REPO: &str = "https://github.com/danielblnc/DLSS-NR-on-AMD";
pub const SETUP_ASSET_NAME: &str = "dlssnr_on_amd_setup.exe";
pub const DLSSNR_DLL_NAME: &str = "nvngx_dlssnr.dll";
pub const MANIFEST_FILE_NAME: &str = "nrmanager.manifest.json";

fn default_repo() -> String {
    DEFAULT_REPO.to_string()
}
fn default_true() -> bool {
    true
}
fn default_theme() -> String {
    "dark".into()
}
fn default_lang() -> String {
    "pt-BR".into()
}
fn default_cap() -> u64 {
    512
}

/// Configuracoes persistidas em config.json (AppData).
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    #[serde(default = "default_repo")]
    pub repo_url: String,
    /// Pasta de cache dos downloads. Vazio = <AppData>\cache.
    #[serde(default)]
    pub cache_dir: String,
    /// Caminho do nvngx_dlssnr.dll fornecido pelo usuario.
    #[serde(default)]
    pub dlssnr_dll_path: String,
    #[serde(default = "default_true")]
    pub check_updates_on_start: bool,
    #[serde(default)]
    pub start_with_windows: bool,
    #[serde(default = "default_theme")]
    pub theme: String,
    /// Versao fixada (rollback). Quando definida, ignora automatico.
    #[serde(default)]
    pub pinned_version: Option<String>,
    /// Setup local importado manualmente (build do Discord etc.).
    #[serde(default)]
    pub custom_setup_path: String,
    /// Argumentos silenciosos opcionais para o setup (vazio = interativo).
    #[serde(default)]
    pub setup_silent_args: String,
    /// Arquivos acima deste tamanho (MB) nao recebem hash nem backup.
    #[serde(default = "default_cap")]
    pub large_file_cap_mb: u64,
    #[serde(default = "default_lang")]
    pub language: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            repo_url: default_repo(),
            cache_dir: String::new(),
            dlssnr_dll_path: String::new(),
            check_updates_on_start: true,
            start_with_windows: false,
            theme: default_theme(),
            pinned_version: None,
            custom_setup_path: String::new(),
            setup_silent_args: String::new(),
            large_file_cap_mb: default_cap(),
            language: default_lang(),
        }
    }
}

impl Settings {
    /// Extrai (owner, repo) da URL do GitHub.
    pub fn repo_parts(&self) -> Option<(String, String)> {
        let url = self.repo_url.trim().trim_end_matches('/');
        let rest = url
            .strip_prefix("https://github.com/")
            .or_else(|| url.strip_prefix("http://github.com/"))
            .or_else(|| url.strip_prefix("github.com/"))?;
        let rest = rest.trim_end_matches(".git");
        let mut it = rest.split('/');
        let owner = it.next()?.to_string();
        let repo = it.next()?.to_string();
        if owner.is_empty() || repo.is_empty() {
            return None;
        }
        Some((owner, repo))
    }

    pub fn large_cap_bytes(&self) -> u64 {
        self.large_file_cap_mb.max(16) * 1024 * 1024
    }
}
