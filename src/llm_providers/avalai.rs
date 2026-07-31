use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::{
    config::GitMindConfig,
    llm::{CommitContext, LlmProvider},
};

// --- AvalAI Implementation ---
/// A provider for AvalAI-compatible APIs
pub struct AvalAiProvider {
    client: Client,
    base_url: String,
    api_key: String,
    model: String,
    prompt: String,
}

impl AvalAiProvider {
    pub fn new(base_url: String, api_key: String, model: String, prompt: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
            api_key,
            model,
            prompt,
        }
    }
}

// --- Serde Structs for JSON Parsing ---
// #[derive(Serialize)] automatically writes the code to turn this struct into JSON.
#[derive(Serialize)]
struct AvalAiRequest<'a> {
    model: &'a str,
    instructions: &'a str,
    input: &'a str,
}

// #[derive(Deserialize)] automatically writes the code to turn JSON into this struct.
#[derive(Deserialize, Debug)]
struct AvalAiResponse {
    output: Vec<Output>,
}

#[derive(Deserialize, Debug)]
struct Output {
    content: Vec<Content>,
}

#[derive(Deserialize, Debug)]
struct Content {
    text: String,
}

#[async_trait]
impl LlmProvider for AvalAiProvider {
    async fn generate_commit(&self, context: &CommitContext) -> Result<String> {
        // Build the payload using our structs
        let request_body = AvalAiRequest {
            model: &self.model,
            instructions: &self.prompt,
            input: &context.diff,
        };

        // Make the HTTP POST request
        let response = self
            .client
            .post(&self.base_url)
            .bearer_auth(&self.api_key)
            .json(&request_body)
            .send()
            .await
            .context("Failed to send request to LLM")?;

        // Ensure the request succeeded (status code 200)
        if !response.status().is_success() {
            let error_text = response.text().await?;
            anyhow::bail!("LLM API error: {}", error_text);
        }

        // Parse the JSON response into our AvalAiResponse struct
        let response_data: AvalAiResponse = response
            .json()
            .await
            .context("Failed to parse LLM response")?;

        // Extract the actual text string and clean it up
        let commit_message = response_data
            .output
            .into_iter()
            .next()
            .context("LLM returned an empty output list")?
            .content[0]
            .text
            .trim()
            .to_string();

        Ok(commit_message)
    }
}

pub fn build_avalai(config: &GitMindConfig) -> anyhow::Result<Box<dyn LlmProvider>> {
    let o_conf = config
        .avalai
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("AvalAI config section is missing from .gitmind.toml"))?;

    let client = AvalAiProvider::new(
        o_conf.base_url.clone(),
        o_conf.api_key.clone(),
        o_conf.model.clone(),
        config.get_system_prompt(),
    );
    Ok(Box::new(client))
}
