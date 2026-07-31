use anyhow::{Context, Result};
use serde::Deserialize;
use std::fs;
use std::path::Path;

// This represents the entire TOML file
#[derive(Deserialize, Debug)]
pub struct GitMindConfig {
    // We will use this to determine which provider to load
    pub active_provider: String,

    // We use Option because a user might not have configured OpenAI if they only use Ollama
    pub ollama: Option<OllamaConfig>,
    pub avalai: Option<AvalAiConfig>,
}

#[derive(Deserialize, Debug)]
pub struct OllamaConfig {
    pub base_url: String,
    pub model: String,
}

#[derive(Deserialize, Debug)]
pub struct AvalAiConfig {
    pub base_url: String,
    pub api_key: String,
    pub model: String,
}

impl GitMindConfig {
    // We now accept the repository root path as an argument
    pub fn load(repo_root: &Path) -> Result<Self> {
        // Construct the path: <repo_root>/.gitmind.toml
        let config_path = repo_root.join(".gitmind.toml");

        // Read the file. If it doesn't exist, we provide a helpful error message.
        let config_str = fs::read_to_string(&config_path).context(format!(
            "Could not find or read config file at {:?}. Did you create a .gitmind.toml file?",
            config_path
        ))?;

        // Parse the TOML string into our struct just like before
        let config: GitMindConfig =
            toml::from_str(&config_str).context("Failed to parse .gitmind.toml config file")?;

        Ok(config)
    }
}
