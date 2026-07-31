mod config;
mod git;
mod llm;
mod llm_providers;

use anyhow::{Context, Result};
use arboard::Clipboard;
use clap::{Parser, Subcommand};
use dialoguer::{Input, Select, theme::ColorfulTheme};

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
    Commit {
        /// The commit message to use
        #[arg(short, long)] // This tells clap to expect `-m` or `--message`
        message: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let mut registry = crate::llm::ProviderRegistry::new();

    // Register all known strategies here
    registry.register("ollama", crate::llm_providers::ollama::build_ollama);
    registry.register("avalai", crate::llm_providers::avalai::build_avalai);

    let git_engine = GitEngine::new()?;

    let repo_root = git_engine
        .get_repo_root()
        .context("Could not determine repository root")?;
    let config = config::GitMindConfig::load(repo_root)?;

    // Safely extract the ignored extensions slice, defaulting to empty [] if it's missing
    let ignored_exts = config
        .project
        .as_ref()
        .and_then(|p| p.ignored_extensions.as_deref())
        .unwrap_or_default();

    match &cli.command {
        Commands::Status => {
            println!("Analyzing repository...\n");

            let changed_files = git_engine.get_changed_files()?;

            if changed_files.is_empty() {
                println!("No changes detected. Working tree is clean.");
            } else {
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
            let diff = git_engine.get_diff(ignored_exts)?;

            if diff.is_empty() {
                println!("No changes to diff.");
            } else {
                println!("{}", diff);
            }
        }
        Commands::Generate => {
            println!("Analyzing diff and generating message...\n");

            let diff = git_engine.get_diff(ignored_exts)?;

            if diff.is_empty() {
                println!("No changes detected. Nothing to commit.");
                return Ok(());
            }

            // Ask the registry for the strategy
            let provider_strategy = registry.get_active_provider(&config)?;

            let context = CommitContext { diff };
            let message = provider_strategy
                .generate_commit(&context)
                .await?
                .replace("```", "")
                .replace("commit\n", "")
                .replace("text\n", "")
                .trim()
                .to_string();

            // println!("✨ Suggested Commit Message:\n");
            // println!("{}", message);
            // println!("\n(Use 'gitmind commit' to apply this, once implemented)");

            println!(
                "\nGenerated Commit Message:\n------------------------\n{}\n------------------------\n",
                message
            );
            // We make the message mutable so the user can edit it if they choose to
            let mut final_message = message;
            let options = &["Commit now", "Edit message", "Copy to clipboard", "Cancel"];

            // This loop keeps asking until the user commits or cancels
            loop {
                // Show the interactive menu
                let selection = Select::with_theme(&ColorfulTheme::default())
                    .with_prompt("What would you like to do?")
                    .default(0) // Default to the first option
                    .items(&options[..])
                    .interact()?; // Wait for user input

                match selection {
                    0 => {
                        // "Commit now"
                        println!("Committing...");
                        git_engine.commit(&final_message)?;
                        println!("Commit successful!");
                        break;
                    }
                    1 => {
                        // "Edit message"
                        // Use dialoguer's Input to let them type a new message,
                        // prepopulated with the LLM's message!
                        final_message = Input::with_theme(&ColorfulTheme::default())
                            .with_prompt("Edit message")
                            .default(final_message)
                            .interact_text()?;
                        println!("\nUpdated message to:\n{}\n", final_message);
                        // The loop repeats so they can choose to commit it now!
                    }
                    2 => {
                        // "Copy to clipboard"
                        let mut clipboard =
                            Clipboard::new().context("Failed to access clipboard")?;
                        clipboard
                            .set_text(&final_message)
                            .context("Failed to copy to clipboard")?;
                        println!("Copied to clipboard!");
                        break;
                    }
                    3 | _ => {
                        // "Cancel" or escape
                        println!("Aborted.");
                        break;
                    }
                }
            }
        }
        Commands::Commit { message } => {
            println!("Committing with message: '{}'", message);

            // Pass the message provided by the user in the CLI!
            git_engine.commit(&message)?;

            println!("Commit successful!");
        }
    }

    Ok(())
}
