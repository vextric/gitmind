use std::collections::HashMap;
use tracing::debug;

use crate::{config::GitMindConfig, error::GitMindError};

use async_trait::async_trait;

/// The context we send to the LLM
pub struct CommitContext {
    pub diff: String,
}

/// The trait that all LLM providers must implement
#[async_trait]
pub trait LlmProvider {
    async fn generate_commit(&self, context: &CommitContext) -> Result<String, GitMindError>;
}

// This is the signature of a function that knows how to build a specific provider
pub type ProviderBuilder = Box<dyn Fn(&GitMindConfig) -> Result<Box<dyn LlmProvider>, GitMindError>>;

pub struct ProviderRegistry {
    builders: HashMap<String, ProviderBuilder>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self {
            builders: HashMap::new(),
        }
    }

    /// Register a new provider strategy dynamically
    pub fn register<F>(&mut self, name: &str, builder: F)
    where
        // F is any function/closure that matches our builder signature
        F: Fn(&GitMindConfig) -> Result<Box<dyn LlmProvider>, GitMindError> + 'static,
    {
        debug!("Registering provider strategy: {}", name);
        self.builders.insert(name.to_string(), Box::new(builder));
    }
    /// Retrieve and build the active provider strategy based on the config
    pub fn get_active_provider(
        &self,
        config: &GitMindConfig,
    ) -> Result<Box<dyn LlmProvider>, GitMindError> {
        debug!("Retrieving provider strategy for active provider: {}", config.active_provider);
        // Split the provider string to handle variants (e.g., "cli:gemini")
        // We only care about the first part ("cli") to find the builder.
        let provider_base = config.active_provider.split(':').next().unwrap_or(&config.active_provider);
        let builder = self.builders.get(provider_base).ok_or_else(|| {
            GitMindError::ProviderNotFound(config.active_provider.clone())
        })?;
        // Execute the builder function to instantiate the strategy
        builder(config)
    }
}
