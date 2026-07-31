use std::collections::HashMap;

use crate::config::GitMindConfig;

use async_trait::async_trait;

pub const SYSTEM_PROMPT: &str = "You are a senior software engineer. Generate a conventional git commit message based on the provided diff. Rules: Maximum 128 characters title, explain WHY not WHAT, return ONLY the commit message (e.g., 'feat: add login').";

/// The context we send to the LLM
pub struct CommitContext {
    pub diff: String,
}

/// The trait that all LLM providers must implement
#[async_trait]
pub trait LlmProvider {
    async fn generate_commit(&self, context: &CommitContext) -> anyhow::Result<String>;
}

// This is the signature of a function that knows how to build a specific provider
pub type ProviderBuilder = Box<dyn Fn(&GitMindConfig) -> anyhow::Result<Box<dyn LlmProvider>>>;

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
        F: Fn(&GitMindConfig) -> anyhow::Result<Box<dyn LlmProvider>> + 'static,
    {
        self.builders.insert(name.to_string(), Box::new(builder));
    }
    /// Retrieve and build the active provider strategy based on the config
    pub fn get_active_provider(
        &self,
        config: &GitMindConfig,
    ) -> anyhow::Result<Box<dyn LlmProvider>> {
        // Find the builder associated with the active_provider string
        let builder = self.builders.get(&config.active_provider).ok_or_else(|| {
            anyhow::anyhow!(
                "No provider registered for strategy: {}",
                config.active_provider
            )
        })?;
        // Execute the builder function to instantiate the strategy
        builder(config)
    }
}
