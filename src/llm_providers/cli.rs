use async_trait::async_trait;
use std::process::{Command, Stdio};
use tracing::{debug, error};

use crate::{
    config::GitMindConfig,
    error::GitMindError,
    llm::{CommitContext, LlmProvider},
};

/// A generic provider that shells out to any command-line tool defined in the config.
pub struct GenericCliProvider {
    command_string: String,
    system_prompt: String,
}

impl GenericCliProvider {
    pub fn new(command_string: String, system_prompt: String) -> Self {
        Self {
            command_string,
            system_prompt,
        }
    }
}

/// A simple helper function to parse a command string into arguments,
/// respecting double quotes. This allows paths with spaces.
fn parse_command_string(cmd: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current_arg = String::new();
    let mut in_quotes = false;

    for c in cmd.chars() {
        match c {
            '"' => {
                in_quotes = !in_quotes; // Toggle quotes
            }
            ' ' if !in_quotes => {
                // If we hit a space outside of quotes, we've finished an argument
                if !current_arg.is_empty() {
                    args.push(current_arg.clone());
                    current_arg.clear();
                }
            }
            _ => {
                // Add the character to our current argument
                current_arg.push(c);
            }
        }
    }
    // Push the last argument if there is one
    if !current_arg.is_empty() {
        args.push(current_arg);
    }
    args
}

#[async_trait]
impl LlmProvider for GenericCliProvider {
    async fn generate_commit(&self, context: &CommitContext) -> Result<String, GitMindError> {
        let full_prompt = format!("{}\n\nCode Diff:\n{}", self.system_prompt, context.diff);

        debug!("Parsing command string: {}", self.command_string);
        let parsed_args = parse_command_string(&self.command_string);

        if parsed_args.is_empty() {
            return Err(GitMindError::Generic("CLI command string is empty".into()));
        }

        // The first part is the executable path
        let executable = &parsed_args[0];

        // The rest are the arguments, but we need to substitute `$prompt` with our actual prompt.
        let mut final_args = Vec::new();
        for arg in parsed_args.iter().skip(1) {
            if arg.contains("$prompt") {
                // Replace $prompt with the massive string containing our system prompt and diff
                final_args.push(arg.replace("$prompt", &full_prompt));
            } else {
                final_args.push(arg.clone());
            }
        }

        debug!("Invoking generic CLI command: {}", executable);

        // Execute the command!
        let output = Command::new(executable)
            .args(&final_args) // Pass our processed arguments safely
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| {
                error!("Failed to start process '{}': {}", executable, e);
                GitMindError::Generic(format!("Failed to execute generic cli: {}", e))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("CLI failed with exit code: {}", output.status);
            error!("Stderr: {}", stderr);
            return Err(GitMindError::Generic(format!("CLI tool error: {}", stderr)));
        }

        let response = String::from_utf8(output.stdout)
            .map_err(|e| GitMindError::Generic(format!("Invalid UTF-8 in stdout: {}", e)))?;

        Ok(response.trim().to_string())
    }
}

pub fn build_cli(config: &GitMindConfig) -> Result<Box<dyn LlmProvider>, GitMindError> {
    // The active provider will be something like "cli:gemini" or just "cli".
    // Let's extract the name after the colon.
    let parts: Vec<&str> = config.active_provider.split(':').collect();

    // If they didn't specify a sub-name, we error out.
    let cli_name = parts.get(1).ok_or_else(|| {
        GitMindError::Generic("No CLI name specified. Use active_provider = 'cli:name'".into())
    })?;

    // Try to get the [cli] block
    let cli_map = config
        .cli
        .as_ref()
        .ok_or_else(|| GitMindError::MissingProviderConfig("cli".into()))?;

    // Find the specific command string for this cli_name
    let command_string = cli_map.get(*cli_name).ok_or_else(|| {
        GitMindError::Generic(format!("No configuration found for cli '{}'", cli_name))
    })?;

    let client = GenericCliProvider::new(command_string.clone(), config.get_system_prompt());
    Ok(Box::new(client))
}
