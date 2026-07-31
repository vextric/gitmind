use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::{
    config::GitMindConfig,
    llm::{CommitContext, LlmProvider, SYSTEM_PROMPT},
};

// --- OpenAI Implementation ---
/// A provider for OpenAI-compatible APIs
pub struct OpenAiProvider {
    client: Client,
    base_url: String,
    api_key: String,
    model: String,
}

impl OpenAiProvider {
    pub fn new(base_url: String, api_key: String, model: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
            api_key,
            model,
        }
    }
}

// --- Serde Structs for JSON Parsing ---
// #[derive(Serialize)] automatically writes the code to turn this struct into JSON.
#[derive(Serialize)]
struct OpenAiRequest<'a> {
    model: &'a str,
    messages: Vec<Message<'a>>,
}

#[derive(Serialize)]
struct Message<'a> {
    role: &'a str,
    content: &'a str,
}

// #[derive(Deserialize)] automatically writes the code to turn JSON into this struct.
#[derive(Deserialize)]
struct OpenAiResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: ResponseMessage,
}

#[derive(Deserialize)]
struct ResponseMessage {
    content: String,
}

#[async_trait]
impl LlmProvider for OpenAiProvider {
    async fn generate_commit(&self, context: &CommitContext) -> Result<String> {
        // Build the payload using our structs
        let request_body = OpenAiRequest {
            model: &self.model,
            messages: vec![
                Message {
                    role: "system",
                    content: SYSTEM_PROMPT,
                },
                Message {
                    role: "user",
                    content: &context.diff,
                },
            ],
        };

        match serde_json::to_string_pretty(&request_body) {
            Ok(json_string) => println!(
                "--- DEBUG: OpenAI Request Body ---\n{}\n----------------------------------",
                json_string
            ),
            Err(e) => println!("Failed to serialize request body for debugging: {}", e),
        }

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

        // Parse the JSON response into our OpenAiResponse struct
        let response_data: OpenAiResponse = response
            .json()
            .await
            .context("Failed to parse LLM response")?;

        // Extract the actual text string and clean it up
        let commit_message = response_data
            .choices
            .into_iter()
            .next()
            .context("LLM returned an empty choice list")?
            .message
            .content
            .trim()
            .to_string();

        Ok(commit_message)
    }
}

pub fn build_openai(config: &GitMindConfig) -> anyhow::Result<Box<dyn LlmProvider>> {
    let o_conf = config
        .openai
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("OpenAI config section is missing from .gitmind.toml"))?;

    let client = OpenAiProvider::new(
        o_conf.base_url.clone(),
        o_conf.api_key.clone(),
        o_conf.model.clone(),
    );
    Ok(Box::new(client))
}
