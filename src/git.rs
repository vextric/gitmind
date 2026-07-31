use anyhow::{Context, Result};
use git2::{DiffFormat, DiffOptions, Repository, StatusOptions};
use std::{
    path::{Path, PathBuf},
    process::Command,
};
use tracing::debug;

#[derive(Debug, PartialEq)] // This allows us to print it and compare it
pub enum FileStatus {
    Staged,
    Changed,
    Untracked,
}

#[derive(Debug)]
pub struct GitFile {
    pub path: PathBuf,
    pub status: FileStatus,
}

pub struct GitEngine {
    repo: Repository,
}

impl GitEngine {
    /// Initialize a new GitEngine by opening the repository in the current directory
    pub fn new() -> Result<Self> {
        // We look for a git repository in the current directory (".")
        let repo = Repository::open(".")
            .context("Failed to open git repository. Are you running this inside a git folder?")?;

        Ok(Self { repo })
    }

    /// Returns the path to the root of the git repository
    pub fn get_repo_root(&self) -> Option<&Path> {
        // workdir() returns the path to the working directory of the repository.
        // It returns None if this is a "bare" repository, which we don't need to worry about right now.
        self.repo.workdir()
    }

    /// Get a list of changed files (modified, added, deleted, untracked)
    pub fn get_changed_files(&self) -> Result<Vec<GitFile>> {
        let mut options = StatusOptions::new();

        // We want to see untracked files, but skip ignored files (like target/ or node_modules/)
        options.include_untracked(true);
        options.exclude_submodules(true);

        let statuses = self.repo.statuses(Some(&mut options))?;
        let mut changed_files = Vec::new();

        for entry in statuses.iter() {
            let status = entry.status();

            // We are looking for any status that indicates a change
            // A file can technically be both Staged and Changed in Git
            //      - but this if/else if chain simplifies it by prioritizing Staged
            let file_status = if status.intersects(
                git2::Status::INDEX_NEW
                    | git2::Status::INDEX_MODIFIED
                    | git2::Status::INDEX_DELETED
                    | git2::Status::INDEX_RENAMED,
            ) {
                Some(FileStatus::Staged)
            } else if status.intersects(git2::Status::WT_NEW) {
                Some(FileStatus::Untracked)
            } else if status.intersects(
                git2::Status::WT_MODIFIED | git2::Status::WT_DELETED | git2::Status::WT_RENAMED,
            ) {
                Some(FileStatus::Changed)
            } else {
                None // Not a status we care about
            };

            // If we found a valid status and a valid path, push it to our vector!
            if let (Some(fs), Ok(path)) = (file_status, entry.path()) {
                changed_files.push(GitFile {
                    path: PathBuf::from(path),
                    status: fs,
                });
            }
        }

        debug!("Found {} changed files", changed_files.len());
        Ok(changed_files)
    }

    /// Get the diff of all changes (staged and unstaged) against HEAD
    pub fn get_diff(&self, ignored_exts: &[String]) -> Result<String> {
        debug!("Generating git diff against HEAD...");
        let mut diff_opts = DiffOptions::new();

        // Get the current HEAD (the latest commit) and turn it into a "Tree"
        let head = self
            .repo
            .head()
            .context("Failed to get HEAD. Is there an initial commit?")?;
        let head_tree = head.peel_to_tree().context("Failed to peel HEAD to tree")?;

        // Generate the diff between the HEAD tree and the current working directory
        let diff = self
            .repo
            .diff_tree_to_workdir_with_index(Some(&head_tree), Some(&mut diff_opts))
            .context("Failed to generate diff")?;

        let mut diff_text = String::new();

        // Format the diff into a standard patch (similar to `git diff`)
        diff.print(DiffFormat::Patch, |delta, _hunk, line| {
            let should_skip = delta
                .new_file()
                .path()
                .and_then(|p| p.extension()) // Get the OsStr extension
                .and_then(|ext| ext.to_str()) // Safely convert OsStr to standard &str
                .map(|ext_str| {
                    // Check if our config's `ignored_exts` array contains this extension
                    ignored_exts.iter().any(|ignored| ignored == ext_str)
                })
                .unwrap_or(false);

            if should_skip {
                return true;
            }

            let origin = line.origin();

            // Add the +, -, or space prefix to the line
            let prefix = match origin {
                '+' | '-' | ' ' => origin.to_string(),
                _ => String::new(),
            };

            // Convert the raw bytes to a UTF-8 string and push it to our diff_text
            if let Ok(content) = std::str::from_utf8(line.content()) {
                diff_text.push_str(&format!("{}{}", prefix, content));
            }

            true // Return true to continue iterating through the diff lines
        })
        .context("Failed to print diff")?;

        Ok(diff_text)
    }

    /// Execute the actual git commit
    pub fn commit(&self, message: &str) -> Result<()> {
        debug!("Executing 'git commit -a -m ...'");
        // While we COULD use git2 to create the commit, using std::process::Command
        // to shell out to the git executable is often better for the final commit.
        // This ensures the user's pre-commit hooks, GPG signing, and global git
        // config are all executed correctly automatically!

        let status = Command::new("git")
            .args(["commit", "-a", "-m", message]) // -a stages all modified/deleted files
            .status()
            .context("Failed to execute git commit command")?;

        if status.success() {
            Ok(())
        } else {
            anyhow::bail!("Git commit failed or was aborted.");
        }
    }
}
