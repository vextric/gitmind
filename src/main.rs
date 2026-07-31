mod config;
mod git;
mod llm;
mod llm_providers;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

use crate::{git::GitEngine, llm::CommitContext};

/// 🧠 GitMind: An AI-powered Git commit assistant
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Show the current git status
    Status,
    /// Show the current diff
    Diff,
    /// Generate a commit message without committing
    Generate,
    /// Generate and execute the commit
    Commit,
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut registry = crate::llm::ProviderRegistry::new();

    // Register all known strategies here
    registry.register("ollama", crate::llm_providers::ollama::build_ollama);
    registry.register("openai", crate::llm_providers::openai::build_openai);

    let cli = Cli::parse();

    match &cli.command {
        Commands::Status => {
            println!("Analyzing repository...\n");

            let git_engine = GitEngine::new()?;
            let changed_files = git_engine.get_changed_files()?;

            if changed_files.is_empty() {
                println!("No changes detected. Working tree is clean.");
            } else {
                // TODO: Mark the untracked files and don't show them as changed
                println!("Changed files:");

                for file in changed_files {
                    // Rust's match statement makes formatting based on enums incredibly easy
                    let prefix = match file.status {
                        crate::git::FileStatus::Staged => "[STAGED]",
                        crate::git::FileStatus::Changed => "[MODIFIED]",
                        crate::git::FileStatus::Untracked => "[UNTRACKED]",
                    };

                    // .display() safely formats the file path for terminal output
                    println!("  {:12} {}", prefix, file.path.display());
                }
            }
        }
        Commands::Diff => {
            let git_engine = GitEngine::new()?;
            let diff = git_engine.get_diff()?;

            if diff.is_empty() {
                println!("No changes to diff.");
            } else {
                println!("{}", diff);
            }
        }
        Commands::Generate => {
            println!("Analyzing diff and generating message...\n");

            let git_engine = GitEngine::new()?;
            let diff = git_engine.get_diff()?;

            if diff.is_empty() {
                println!("No changes detected. Nothing to commit.");
                return Ok(());
            }

            let repo_root = git_engine
                .get_repo_root()
                .context("Could not determine repository root")?;
            let config = config::GitMindConfig::load(repo_root)?;

            // Ask the registry for the strategy
            let provider_strategy = registry.get_active_provider(&config)?;

            let context = CommitContext { diff };
            let message = provider_strategy.generate_commit(&context).await?;

            println!("✨ Suggested Commit Message:\n");
            println!("{}", message);
            println!("\n(Use 'gitmind commit' to apply this, once implemented)");
        }
        Commands::Commit => {
            let git_engine = GitEngine::new()?;

            // For now, since we don't have the LLM hooked up,
            // we will just use a hardcoded test message.
            let test_message = "feat(gitmind): test automatic commit via CLI";

            println!("Committing with message: '{}'", test_message);
            git_engine.commit(test_message)?;
            println!("Commit successful!");
        }
    }

    Ok(())
}
