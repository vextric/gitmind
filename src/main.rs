mod git;

use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::git::GitEngine;

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
    let cli = Cli::parse();

    match &cli.command {
        Commands::Status => {
            println!("Analyzing repository...\n");

            let git_engine = GitEngine::new()?;
            let changed_files = git_engine.get_changed_files()?;

            if changed_files.is_empty() {
                println!("No changes detected. Working tree is clean.");
            } else {
                println!("Changed files:");
                for file in changed_files {
                    // .display() safely formats the file path for terminal output
                    // functionality
                    println!("  - {}", file.display());
                }
            }
        }
        Commands::Diff => {
            println!("Running diff...");
            // TODO: Hook up git2 diff here
        }
        Commands::Generate => {
            println!("Generating commit message...");
            // TODO: Hook up LLM here
        }
        Commands::Commit => {
            println!("Committing changes...");
            // TODO: Generate and run git commit
        }
    }

    Ok(())
}
