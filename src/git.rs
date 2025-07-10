use crate::config::Config;
use chrono::Local;
use std::{path::Path, process::Command};

pub fn is_git_repo(log_dir: &str) -> bool {
    Path::new(log_dir).join(".git").exists()
}

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

pub fn auto_sync_if_enabled(config: &Config) -> anyhow::Result<()> {
    if config.git_auto_sync.unwrap_or(false) && config.git_repo.is_some() {
        if let Err(e) = git_sync(config) {
            eprintln!("Warning: Auto-sync failed: {}", e);
        }
    }
    Ok(())
}