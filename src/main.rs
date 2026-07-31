use anyhow::Result;
use clap::{Parser, Subcommand};

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
            println!("Running status...");
            // TODO: Hook up git2 status here
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
