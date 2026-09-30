use serde::{Deserialize, Serialize};

/// Jogo da biblioteca local.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct Game {
    /// Identificador estavel, ex.: "steam:1091500" ou "manual:<uuid>".
    pub id: String,
    pub name: String,
    /// steam | epic | gog | xbox | manual
    pub source: String,
    pub app_id: Option<String>,
    pub install_dir: String,
    pub exe_path: Option<String>,
    /// Pasta onde o mod deve ser instalado (pode ser subpasta, ex.: bin\x64).
    pub mod_dir: Option<String>,
    pub cover_url: Option<String>,
    /// dx12 | vulkan | dx11 | unknown
    pub graphics_api: String,
    /// compatible | probable | untested | incompatible
    pub compat: String,
    pub compat_note: Option<String>,
    pub anti_cheat: Option<String>,
    pub fsr_detected: bool,
    pub mod_installed: bool,
    pub mod_version: Option<String>,
    pub last_checked: Option<String>,
    /// Mensagem curta de estado exibida no card (ex.: "Mod ausente: reaplique").
    pub status_message: Option<String>,
    /// Verdadeiro quando o manifesto existe mas arquivos do mod sumiram.
    pub divergence: bool,
    pub added_manually: bool,
}

/// Asset de uma release do GitHub.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ReleaseAsset {
    pub name: String,
    pub size: u64,
    pub digest: Option<String>,
    pub download_url: String,
    pub download_count: u64,
}

/// Release do GitHub normalizada.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct Release {
    pub tag: String,
    pub name: String,
    pub body: String,
    pub published_at: String,
    pub prerelease: bool,
    pub draft: bool,
    pub html_url: String,
    pub assets: Vec<ReleaseAsset>,
}

/// Release ja presente no cache local (baixada ou importada).
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct CachedRelease {
    pub tag: String,
    pub name: String,
    pub path: String,
    pub size: u64,
    pub sha256: Option<String>,
    /// github | custom
    pub source: String,
    pub downloaded_at: String,
    pub body: Option<String>,
    pub published_at: Option<String>,
}

/// Entrada da lista de compatibilidade (compat.json).
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct CompatEntry {
    pub name: String,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub app_ids: std::collections::BTreeMap<String, String>,
    #[serde(default = "default_api")]
    pub graphics_api: String,
    #[serde(default)]
    pub exe_subdir: String,
    #[serde(default)]
    pub exe_name: String,
    #[serde(default = "default_status")]
    pub status: String,
    #[serde(default)]
    pub fsr: bool,
    #[serde(default)]
    pub anti_cheat: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
}

fn default_api() -> String {
    "unknown".into()
}
fn default_status() -> String {
    "untested".into()
}

/// Arquivo registrado no manifesto/snapshot.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct FileRecord {
    pub path: String,
    pub size: u64,
    pub mtime: i64,
    /// Vazio quando o arquivo e grande demais para ser hasheado.
    pub sha256: String,
}

/// Arquivo pre-existente que foi sobrescrito ou removido pelo instalador.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ModifiedRecord {
    pub path: String,
    pub sha256: String,
    pub size: u64,
    pub mtime: i64,
    /// Caminho relativo do backup em AppData.
    pub backup_path: Option<String>,
    pub backed_up: bool,
}

/// Manifesto de instalacao reversivel, gravado na pasta do jogo e no AppData.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
    pub schema_version: u32,
    pub game_id: String,
    pub game_name: String,
    pub mod_version: String,
    pub setup_sha256: Option<String>,
    pub installed_at: String,
    pub mod_dir: String,
    pub backup_dir: String,
    pub created_files: Vec<FileRecord>,
    pub modified_files: Vec<ModifiedRecord>,
    pub deleted_files: Vec<ModifiedRecord>,
    pub created_dirs: Vec<String>,
    /// Arquivos grandes demais para backup: nao sao restauraveis se alterados.
    pub unbacked_files: Vec<String>,
}

/// Resultado da verificacao de integridade (reparar).
#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct RepairReport {
    pub ok: bool,
    pub missing: Vec<String>,
    pub modified: Vec<String>,
    pub checked: usize,
    pub message: String,
}

/// Informacoes da GPU detectada.
#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct GpuInfo {
    pub name: String,
    pub driver_version: String,
    /// rdna4 | rdna3 | rdna2 | other | unknown
    pub family: String,
    pub supported: bool,
    pub message: String,
}

/// Estado consolidado de um jogo (para a tela de detalhes).
#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct GameStatus {
    pub game_id: String,
    pub installed: bool,
    pub version: Option<String>,
    pub installed_at: Option<String>,
    pub files_count: usize,
    pub divergence: bool,
    pub game_running: bool,
    pub manifest_path: Option<String>,
    pub backup_dir: Option<String>,
    pub message: String,
}

/// Payload de progresso emitido por eventos para o frontend.
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Progress {
    pub game_id: Option<String>,
    pub stage: String,
    pub message: String,
    pub percent: Option<f64>,
}

/// Item de log exposto ao frontend.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LogEntry {
    pub time: String,
    pub level: String,
    pub message: String,
}

/// Resultado da checagem de releases.
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ReleaseFeed {
    pub releases: Vec<Release>,
    pub latest: Option<Release>,
    /// live | cache | offline
    pub origin: String,
    pub message: Option<String>,
}
