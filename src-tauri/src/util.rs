use std::path::Path;

use sha2::{Digest, Sha256};

/// SHA-256 de um arquivo, retornando hex minusculo. Vazio em caso de erro.
pub fn hash_file(path: &Path) -> String {
    let mut hasher = Sha256::new();
    let file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return String::new(),
    };
    let mut reader = std::io::BufReader::with_capacity(1024 * 1024, file);
    if std::io::copy(&mut reader, &mut hasher).is_err() {
        return String::new();
    }
    hex::encode(hasher.finalize())
}

pub fn hash_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

/// Normaliza nomes para comparacao (minusculo, sem pontuacao).
pub fn norm_name(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(|c| c.to_lowercase())
        .collect()
}

/// Formata bytes para exibicao.
pub fn human_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut v = bytes as f64;
    let mut i = 0;
    while v >= 1024.0 && i < UNITS.len() - 1 {
        v /= 1024.0;
        i += 1;
    }
    if i == 0 {
        format!("{bytes} B")
    } else {
        format!("{v:.1} {}", UNITS[i])
    }
}

/// Converte path para string com separador nativo do Windows.
pub fn to_win_string(p: &Path) -> String {
    p.to_string_lossy().to_string()
}

/// Compara dois caminhos ignorando maiusculas/minusculas e barras.
pub fn path_eq(a: &Path, b: &Path) -> bool {
    norm_path(a) == norm_path(b)
}

pub fn norm_path(p: &Path) -> String {
    p.to_string_lossy()
        .replace('/', "\\")
        .trim_end_matches('\\')
        .to_lowercase()
}
