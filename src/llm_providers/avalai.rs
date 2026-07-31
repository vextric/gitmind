use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{debug, error};

use crate::{
    config::GitMindConfig,
    error::GitMindError,
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
    async fn generate_commit(&self, context: &CommitContext) -> Result<String, GitMindError> {
        // Build the payload using our structs
        let request_body = AvalAiRequest {
            model: &self.model,
            instructions: &self.prompt,
            input: &context.diff,
        };

        debug!("Sending request to AvalAI URL: {}", self.base_url);
        if let Ok(json_str) = serde_json::to_string_pretty(&request_body) {
            debug!("AvalAI Request Body:\n{}", json_str);
        }

        // Make the HTTP POST request
        let response = self
            .client
            .post(&self.base_url)
            .bearer_auth(&self.api_key)
            .json(&request_body)
            .send()
            .await?;

        // Ensure the request succeeded (status code 200)
        if !response.status().is_success() {
            let error_text = response.text().await?;
            error!("AvalAI API returned an error: {}", error_text);
            return Err(GitMindError::LlmApiError(error_text));
        }

        // Parse the JSON response into our AvalAiResponse struct
        let response_data: AvalAiResponse = response.json().await?;

        // Extract the actual text string and clean it up
        let commit_message = response_data
            .output
            .into_iter()
            .next()
            .ok_or_else(|| GitMindError::Generic("LLM returned an empty output list".into()))?
            .content
            .into_iter()
            .next()
            .ok_or_else(|| GitMindError::Generic("LLM returned an empty content list".into()))?
            .text
            .trim()
            .to_string();

        Ok(commit_message)
    }
}

pub fn build_avalai(config: &GitMindConfig) -> Result<Box<dyn LlmProvider>, GitMindError> {
    let o_conf = config
        .avalai
        .as_ref()
        .ok_or_else(|| GitMindError::MissingProviderConfig("avalai".into()))?;

    let client = AvalAiProvider::new(
        o_conf.base_url.clone(),
        o_conf.api_key.clone(),
        o_conf.model.clone(),
        config.get_system_prompt(),
    );
    Ok(Box::new(client))
}
