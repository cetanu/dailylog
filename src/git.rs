//! Git repository operations and synchronization.
//!
//! This module provides functionality for managing git repositories,
//! including initialization, pulling, pushing, and automatic syncing
//! of daily log files across devices.

use crate::config::Config;
use chrono::Local;
use std::{path::Path, process::Command};

/// Checks if a directory is a git repository.
///
/// Determines if the specified directory contains a `.git` subdirectory,
/// indicating it's a git repository.
///
/// # Arguments
///
/// * `log_dir` - The directory to check
///
/// # Returns
///
/// `true` if the directory is a git repository, `false` otherwise.
///
/// # Example
///
/// ```rust
/// use dailylog::git::is_git_repo;
///
/// if is_git_repo("/path/to/logs") {
///     println!("Directory is a git repository");
/// }
/// ```
pub fn is_git_repo(log_dir: &str) -> bool {
    Path::new(log_dir).join(".git").exists()
}

/// Executes a git command in the specified directory.
///
/// Runs a git command with the given arguments in the log directory,
/// capturing output and checking for success.
///
/// # Arguments
///
/// * `log_dir` - The directory to run the git command in
/// * `args` - Command line arguments to pass to git
///
/// # Errors
///
/// Returns an error if:
/// - The git command fails to execute
/// - The git command returns a non-zero exit status
///
/// # Example
///
/// ```rust
/// use dailylog::git::run_git_command;
///
/// run_git_command("/path/to/logs", &["status", "--porcelain"])?;
/// ```
pub fn run_git_command(log_dir: &str, args: &[&str]) -> anyhow::Result<()> {
    let output = Command::new("git")
        .args(args)
        .current_dir(log_dir)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("Git command failed: {}", stderr));
    }

    Ok(())
}

/// Initializes a git repository in the log directory.
///
/// Sets up a new git repository with the specified remote URL and branch.
/// If the repository already exists, this function does nothing.
/// Attempts to pull existing logs from the remote, and if that fails
/// (which is normal for new repositories), creates the specified branch.
///
/// # Arguments
///
/// * `log_dir` - The directory to initialize as a git repository
/// * `repo_url` - The remote repository URL to add as origin
/// * `branch` - The branch name to use
///
/// # Errors
///
/// Returns an error if any git commands fail during initialization.
///
/// # Example
///
/// ```rust
/// use dailylog::git::init_git_repo;
///
/// init_git_repo(
///     "/path/to/logs",
///     "https://github.com/user/dailylogs.git",
///     "main"
/// )?;
/// ```
pub fn init_git_repo(log_dir: &str, repo_url: &str, branch: &str) -> anyhow::Result<()> {
    if is_git_repo(log_dir) {
        println!("Git repository already exists in {}", log_dir);
        return Ok(());
    }

    println!("Initializing git repository in {}", log_dir);
    run_git_command(log_dir, &["init"])?;
    run_git_command(log_dir, &["remote", "add", "origin", repo_url])?;

    // Try to pull existing logs
    if let Err(e) = run_git_command(log_dir, &["pull", "origin", branch]) {
        println!(
            "Note: Could not pull from remote (this is normal for new repos): {}",
            e
        );
        // Create initial commit
        run_git_command(log_dir, &["checkout", "-b", branch])?;
    }

    Ok(())
}

/// Pulls the latest logs from the remote git repository.
///
/// Downloads and merges changes from the remote repository to keep
/// the local logs synchronized.
///
/// # Arguments
///
/// * `log_dir` - The log directory (must be a git repository)
/// * `branch` - The branch to pull from
///
/// # Errors
///
/// Returns an error if:
/// - The directory is not a git repository
/// - The pull operation fails
///
/// # Example
///
/// ```rust
/// use dailylog::git::git_pull;
///
/// git_pull("/path/to/logs", "main")?;
/// ```
pub fn git_pull(log_dir: &str, branch: &str) -> anyhow::Result<()> {
    if !is_git_repo(log_dir) {
        return Err(anyhow::anyhow!(
            "Not a git repository. Use 'dailylog sync' to set up git sync first."
        ));
    }

    println!("Pulling latest logs from git repository...");
    run_git_command(log_dir, &["pull", "origin", branch])?;
    println!("Successfully pulled latest logs.");

    Ok(())
}

/// Pushes local log changes to the remote git repository.
///
/// Adds all markdown files, creates a commit with a timestamp,
/// and pushes to the remote repository. If there are no changes,
/// the operation completes without creating a commit.
///
/// # Arguments
///
/// * `log_dir` - The log directory (must be a git repository)
/// * `branch` - The branch to push to
///
/// # Errors
///
/// Returns an error if:
/// - The directory is not a git repository
/// - Any git operations fail
///
/// # Example
///
/// ```rust
/// use dailylog::git::git_push;
///
/// git_push("/path/to/logs", "main")?;
/// ```
pub fn git_push(log_dir: &str, branch: &str) -> anyhow::Result<()> {
    if !is_git_repo(log_dir) {
        return Err(anyhow::anyhow!(
            "Not a git repository. Use 'dailylog sync' to set up git sync first."
        ));
    }

    // Add all log files
    run_git_command(log_dir, &["add", "*.md"])?;

    // Check if there are changes to commit
    let status_output = Command::new("git")
        .args(&["status", "--porcelain"])
        .current_dir(log_dir)
        .output()?;

    if status_output.stdout.is_empty() {
        println!("No changes to push.");
        return Ok(());
    }

    // Commit with timestamp
    let commit_msg = format!("Update logs - {}", Local::now().format("%Y-%m-%d %H:%M"));
    run_git_command(log_dir, &["commit", "-m", &commit_msg])?;

    println!("Pushing logs to git repository...");
    run_git_command(log_dir, &["push", "origin", branch])?;
    println!("Successfully pushed logs.");

    Ok(())
}

/// Performs a full git synchronization (pull then push).
///
/// This is the main sync operation that:
/// 1. Initializes the git repository if it doesn't exist
/// 2. Pulls the latest changes from the remote
/// 3. Pushes any local changes to the remote
///
/// # Arguments
///
/// * `config` - Application configuration containing git settings
///
/// # Errors
///
/// Returns an error if:
/// - No git repository is configured
/// - Repository initialization fails
/// - Pull or push operations fail
///
/// # Example
///
/// ```rust
/// use dailylog::git::git_sync;
/// use dailylog::config::load_config;
///
/// let config = load_config()?;
/// git_sync(&config)?;
/// ```
pub fn git_sync(config: &Config) -> anyhow::Result<()> {
    let repo_url = config.git_repo.as_ref()
        .ok_or_else(|| anyhow::anyhow!("No git repository configured. Please add 'git_repo = \"your-repo-url\"' to ~/.dailylog.toml"))?;

    if !is_git_repo(&config.log_dir) {
        init_git_repo(&config.log_dir, repo_url, &config.git_branch_name)?;
    }

    // Pull first, then push
    git_pull(&config.log_dir, &config.git_branch_name)?;
    git_push(&config.log_dir, &config.git_branch_name)?;

    Ok(())
}

/// Automatically syncs with git if auto-sync is enabled in configuration.
///
/// Checks the configuration to see if automatic git synchronization is enabled,
/// and if so, performs a full sync operation. If sync fails, prints a warning
/// but doesn't return an error (to avoid interrupting the main workflow).
///
/// # Arguments
///
/// * `config` - Application configuration
///
/// # Example
///
/// ```rust
/// use dailylog::git::auto_sync_if_enabled;
/// use dailylog::config::load_config;
///
/// let config = load_config()?;
/// auto_sync_if_enabled(&config)?; // Only syncs if enabled in config
/// ```
pub fn auto_sync_if_enabled(config: &Config) -> anyhow::Result<()> {
    if config.git_auto_sync.unwrap_or(false) && config.git_repo.is_some() {
        if let Err(e) = git_sync(config) {
            eprintln!("Warning: Auto-sync failed: {}", e);
        }
    }
    Ok(())
}