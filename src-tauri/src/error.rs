use serde::{Serialize, Serializer};

/// Erro unificado da aplicacao. Serializa como string para o frontend.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{0}")]
    Msg(String),
    #[error("Erro de E/S: {0}")]
    Io(#[from] std::io::Error),
    #[error("Erro de rede: {0}")]
    Http(#[from] reqwest::Error),
    #[error("JSON invalido: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Nenhuma versao encontrada na release.")]
    NoAsset,
    #[error("Caminho nao encontrado: {0}")]
    NotFound(String),
    #[error("{0}")]
    Permission(String),
}

impl AppError {
    pub fn msg(s: impl Into<String>) -> Self {
        AppError::Msg(s.into())
    }
}

impl From<String> for AppError {
    fn from(s: String) -> Self {
        AppError::Msg(s)
    }
}

impl From<&str> for AppError {
    fn from(s: &str) -> Self {
        AppError::Msg(s.to_string())
    }
}

impl From<walkdir::Error> for AppError {
    fn from(e: walkdir::Error) -> Self {
        AppError::Msg(format!("Falha ao varrer diretorio: {e}"))
    }
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
