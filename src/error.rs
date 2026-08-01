use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GitMindError {
    #[error("Could not find or read config file at {0}")]
    ConfigNotFound(PathBuf),

    #[error("Failed to parse .gitmind.toml config file: {0}")]
    ConfigParseError(#[from] toml::de::Error),

    #[error("Provider config section is missing: {0}")]
    MissingProviderConfig(String),

    #[error("LLM API returned an error: {0}")]
    LlmApiError(String),

    #[error("No provider registered for strategy: {0}")]
    ProviderNotFound(String),

    #[error("Git error: {0}")]
    GitError(#[from] git2::Error),

    #[error("Network error: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("UI error: {0}")]
    UiError(#[from] dialoguer::Error),

    #[error("Clipboard error: {0}")]
    ClipboardError(#[from] arboard::Error),

    #[error("Generic error: {0}")]
    Generic(String),
}

// Allow easy string conversions to Generic error
impl From<&str> for GitMindError {
    fn from(s: &str) -> Self {
        GitMindError::Generic(s.to_string())
    }
}

impl From<String> for GitMindError {
    fn from(s: String) -> Self {
        GitMindError::Generic(s)
    }
}
