use crate::error::GitMindError;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tracing::debug;

// This represents the entire TOML file
#[derive(Deserialize, Debug)]
pub struct GitMindConfig {
    // We will use this to determine which provider to load
    pub active_provider: String,

    pub project: Option<ProjectConfig>,

    // We use Option because a user might not have configured AvalAi f they only use Ollama
    pub ollama: Option<OllamaConfig>,
    pub avalai: Option<AvalAiConfig>,
    pub cli: Option<HashMap<String, String>>,

    pub system_prompt: String,
}

#[derive(Deserialize, Debug)]
pub struct ProjectConfig {
    pub languages: Option<Vec<String>>,
    pub additional_info: Option<String>,
    pub author_name: Option<String>,
    pub ignored_extensions: Option<Vec<String>>,
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
    pub fn load(repo_root: &Path) -> Result<Self, GitMindError> {
        // Construct the path: <repo_root>/.gitmind.toml
        let config_path = repo_root.join(".gitmind.toml");
        debug!("Looking for configuration file at {:?}", config_path);

        // Read the file. If it doesn't exist, we provide a helpful error message.
        let config_str = fs::read_to_string(&config_path)
            .map_err(|_| GitMindError::ConfigNotFound(config_path.clone()))?;

        // Parse the TOML string into our struct just like before
        let config: GitMindConfig = toml::from_str(&config_str)?;

        debug!("Successfully loaded and parsed .gitmind.toml");

        Ok(config)
    }

    // Giving the LLM some more context about the project
    pub fn get_system_prompt(&self) -> String {
        let mut system_prompt = self.system_prompt.clone();

        if let Some(project) = &self.project {
            if let Some(langs) = &project.languages {
                system_prompt.push_str(&format!(
                    "The project uses these languages: {}. ",
                    langs.join(", ")
                ));
            }
            if let Some(author) = &project.author_name {
                system_prompt.push_str(&format!("The author's name is {}. ", author));
            }
            if let Some(info) = &project.additional_info {
                system_prompt.push_str(&format!("Additional context: {}. ", info));
            }
        }

        system_prompt
    }
}
