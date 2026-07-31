use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::{
    config::GitMindConfig,
    llm::{CommitContext, LlmProvider},
};

// --- Ollama Implementation ---
/// A provider for local Ollama instances
pub struct OllamaProvider {
    client: Client,
    host: String,
    model: String,
}

impl OllamaProvider {
    pub fn new(host: String, model: String) -> Self {
        Self {
            client: Client::new(),
            host,
            model,
        }
    }
}

// Serde Structs for Ollama JSON Parsing
#[derive(Serialize)]
struct OllamaRequest<'a> {
    model: &'a str,
    prompt: &'a str,
    stream: bool, // We set this to false to get the whole response at once
}

#[derive(Deserialize)]
struct OllamaResponse {
    response: String,
}

pub const SYSTEM_PROMPT: &str = r#"
    You are an expert developer summarizing code changes. You will be given a git diff. Your task is to write a conventional commit message for these changes.
    CRITICAL INSTRUCTIONS:
        - Output ONLY the raw commit message.
        - Do NOT output JSON.
        - Do NOT output tool or function calls.
        - Do not include any conversational filler (like 'Here is your commit message').
"#;

#[async_trait]
impl LlmProvider for OllamaProvider {
    async fn generate_commit(&self, context: &CommitContext) -> Result<String> {
        // Ollama's standard /api/generate endpoint takes a single combined prompt.
        // We use the format! macro to combine the system instructions and the code diff.
        let full_prompt = format!("{}\n\nCode Diff:\n{}", SYSTEM_PROMPT, context.diff);

        // Build the payload
        let request_body = OllamaRequest {
            model: &self.model,
            prompt: &full_prompt,
            stream: false,
        };

        // Ensure the host URL doesn't have a trailing slash before appending the path
        let url = format!("{}/api/generate", self.host.trim_end_matches('/'));

        // Make the HTTP POST request (No API key needed!)
        let response = self
            .client
            .post(&url)
            .json(&request_body)
            .send()
            .await
            .context("Failed to send request to Ollama")?;

        // Check for success
        if !response.status().is_success() {
            let error_text = response.text().await?;
            anyhow::bail!("Ollama API error: {}", error_text);
        }

        // Parse the JSON response
        let response_data: OllamaResponse = response
            .json()
            .await
            .context("Failed to parse Ollama response")?;

        // Return the string, trimming any excess whitespace or newlines
        Ok(response_data.response.trim().to_string())
    }
}

/// The specific builder function for the Ollama strategy
pub fn build_ollama(config: &GitMindConfig) -> anyhow::Result<Box<dyn LlmProvider>> {
    let o_conf = config
        .ollama
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Ollama config section is missing from .gitmind.toml"))?;

    // This is a test comment to test the app
    let client = OllamaProvider::new(o_conf.base_url.clone(), o_conf.model.clone());
    Ok(Box::new(client))
}
