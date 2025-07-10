//! # dailylog
//!
//! A minimal journaling tool that provides a terminal-native experience for daily logging
//! with git synchronization capabilities.
//!
//! ## Features
//!
//! - Opens your `$EDITOR` to write journal entries
//! - Git commit style parsing: first line becomes title, body after blank line
//! - Saves entries with timestamps in markdown format to `YYYY-MM-DD.md` files
//! - Git sync support for backing up and sharing logs across devices
//! - Log summarization with statistics and consistency tracking
//! - Configurable via TOML configuration file
//!
//! ## Usage
//!
//! ```bash
//! # Create a new log entry
//! dailylog
//!
//! # View previous day's log
//! dailylog previous
//!
//! # Add to previous day's log
//! dailylog yesterday
//!
//! # Summarize past 7 days
//! dailylog summary
//!
//! # Git sync operations
//! dailylog sync
//! dailylog pull
//! dailylog push
//! ```

mod config;
mod display;
mod entry;
mod git;
mod summary;

use clap::{Parser, Subcommand};
use config::load_config;
use display::{add_to_previous_day_log, view_previous_day_log};
use entry::{append_to_log, get_log_file_path, open_editor};
use git::{auto_sync_if_enabled, git_pull, git_push, git_sync};
use std::fs;
use summary::summarize_logs;

/// Command-line interface for dailylog
#[derive(Parser)]
#[command(name = "dailylog")]
#[command(about = "A minimal journaling tool")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

/// Available subcommands for dailylog
#[derive(Subcommand)]
enum Commands {
    /// View the previous day's log entry
    Previous,
    /// Add to the previous day's log entry
    Yesterday,
    /// Summarize and review logs for the past X days
    Summary {
        /// Number of days to include in summary (default: 7)
        #[arg(short, long, default_value = "7")]
        days: u32,
    },
    /// Sync logs with git repository (pull then push)
    Sync,
    /// Pull latest logs from git repository
    Pull,
    /// Push logs to git repository
    Push,
}


/// Main entry point for the dailylog application.
///
/// Parses command-line arguments and dispatches to the appropriate functionality:
/// - No command: Create a new log entry for today
/// - Subcommands: Execute specific operations (view, append, summarize, git sync)
///
/// # Errors
///
/// Returns an error if:
/// - Configuration cannot be loaded
/// - Log directory cannot be created
/// - The requested operation fails
fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let config = load_config()?;
    fs::create_dir_all(&config.log_dir)?;

    match cli.command {
        Some(Commands::Previous) => {
            view_previous_day_log(&config.log_dir)?;
        }
        Some(Commands::Yesterday) => {
            add_to_previous_day_log(&config.log_dir)?;
            auto_sync_if_enabled(&config)?;
        }
        Some(Commands::Summary { days }) => {
            summarize_logs(&config.log_dir, days, &config)?;
        }
        Some(Commands::Sync) => {
            git_sync(&config)?;
        }
        Some(Commands::Pull) => {
            git_pull(&config.log_dir, &config.git_branch_name)?;
        }
        Some(Commands::Push) => {
            git_push(&config.log_dir, &config.git_branch_name)?;
        }
        None => {
            // Default behavior: create new log entry
            let log_path = get_log_file_path(&config.log_dir);
            let entry = open_editor()?;
            if !entry.trim().is_empty() {
                append_to_log(&log_path, &entry)?;
                println!("Log saved to {:?}", log_path);
                auto_sync_if_enabled(&config)?;
            } else {
                println!("No content written. Aborted.");
            }
        }
    }

    Ok(())
}
