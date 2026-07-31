use anyhow::{Context, Result};
use git2::{Repository, StatusOptions};
use std::path::PathBuf;

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

    /// Get a list of changed files (modified, added, deleted, untracked)
    pub fn get_changed_files(&self) -> Result<Vec<PathBuf>> {
        let mut options = StatusOptions::new();

        // We want to see untracked files, but skip ignored files (like target/ or node_modules/)
        options.include_untracked(true);
        options.exclude_submodules(true);

        let statuses = self.repo.statuses(Some(&mut options))?;
        let mut changed_files = Vec::new();

        for entry in statuses.iter() {
            let status = entry.status();

            // We are looking for any status that indicates a change
            let is_changed = status.intersects(
                git2::Status::INDEX_NEW
                    | git2::Status::INDEX_MODIFIED
                    | git2::Status::INDEX_DELETED
                    | git2::Status::INDEX_RENAMED
                    | git2::Status::WT_NEW
                    | git2::Status::WT_MODIFIED
                    | git2::Status::WT_DELETED
                    | git2::Status::WT_RENAMED,
            );

            if is_changed {
                if let Ok(path) = entry.path() {
                    changed_files.push(PathBuf::from(path));
                }
            }
        }

        Ok(changed_files)
    }
}
