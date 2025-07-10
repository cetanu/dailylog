//! Configuration management for dailylog.
//!
//! This module handles loading and parsing the TOML configuration file,
//! providing sensible defaults for all settings.

use dirs::home_dir;
use serde::Deserialize;
use std::fs;

/// Application configuration loaded from `~/.dailylog.toml`.
///
/// All fields have sensible defaults and the configuration file is optional.
/// If the file doesn't exist or fields are missing, defaults will be used.
///
/// # Example Configuration
///
/// ```toml
/// # Directory where daily logs are stored
/// log_dir = "/path/to/your/logs"
///
/// # Git repository URL for syncing logs across devices
/// git_repo = "https://github.com/username/dailylogs.git"
///
/// # Enable automatic git sync after each log entry
/// git_auto_sync = true
///
/// # Git branch name to use
/// git_branch_name = "main"
///
/// # Days to include in summary statistics
/// summary_days = ["monday", "tuesday", "wednesday", "thursday", "friday"]
/// ```
#[derive(Deserialize, Default)]
pub struct Config {
    /// Directory where log files are stored (default: `~/.dailylog`)
    #[serde(default = "default_log_dir")]
    pub log_dir: String,
    
    /// Optional git repository URL for syncing logs
    pub git_repo: Option<String>,
    
    /// Whether to automatically sync with git after each entry (default: false)
    pub git_auto_sync: Option<bool>,

    /// Git branch name to use for syncing (default: "master")
    #[serde(default = "default_branch")]
    pub git_branch_name: String,

    /// Days of the week to include in summary statistics (default: Monday-Friday)
    #[serde(default = "default_summary_days")]
    pub summary_days: Vec<String>,
}

/// Returns the default log directory path.
///
/// Uses `~/.dailylog` if the home directory can be determined,
/// otherwise falls back to `.dailylog` in the current directory.
fn default_log_dir() -> String {
    home_dir()
        .map(|path| path.join(".dailylog").to_string_lossy().into_owned())
        .unwrap_or_else(|| ".dailylog".to_string())
}

/// Returns the default git branch name.
fn default_branch() -> String {
    "master".to_string()
}

/// Returns the default days to include in summary statistics.
///
/// Defaults to Monday through Friday (weekdays only).
fn default_summary_days() -> Vec<String> {
    vec![
        "monday".to_string(),
        "tuesday".to_string(),
        "wednesday".to_string(),
        "thursday".to_string(),
        "friday".to_string(),
    ]
}

/// Loads configuration from `~/.dailylog.toml`.
///
/// If the configuration file doesn't exist or cannot be parsed,
/// returns a default configuration with sensible defaults.
/// This ensures the application works out-of-the-box without
/// requiring any configuration.
///
/// # Errors
///
/// Returns an error only if the home directory cannot be determined.
/// All other errors (missing file, parse errors) are handled gracefully
/// by falling back to defaults.
///
/// # Example
///
/// ```rust
/// use dailylog::config::load_config;
///
/// let config = load_config()?;
/// println!("Log directory: {}", config.log_dir);
/// ```
pub fn load_config() -> anyhow::Result<Config> {
    let config_path = home_dir()
        .ok_or_else(|| anyhow::anyhow!("Cannot determine home directory"))?
        .join(".dailylog.toml");
    let config_str = fs::read_to_string(&config_path).unwrap_or_default();
    Ok(toml::from_str(&config_str).unwrap_or_default())
}