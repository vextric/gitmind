use anyhow::Result;

pub const SYSTEM_PROMPT: &str = "You are a senior software engineer. Generate a conventional git commit message based on the provided diff. Rules: Maximum 128 characters title, explain WHY not WHAT, return ONLY the commit message (e.g., 'feat: add login').";

/// The context we send to the LLM
pub struct CommitContext {
    pub diff: String,
}

/// The trait that all LLM providers must implement
pub trait LlmProvider {
    async fn generate_commit(&self, context: &CommitContext) -> Result<String>;
}
