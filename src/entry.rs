//! Entry management and file operations.
//!
//! This module handles creating, parsing, and formatting journal entries.
//! It manages the git commit-style parsing (title on first line, body after blank line)
//! and file I/O operations for daily log files.

use chrono::{Duration, Local, NaiveDate};
use std::{
    env,
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    process::Command,
};

/// Returns the file path for today's log entry.
///
/// Generates a path in the format `{log_dir}/YYYY-MM-DD.md` for the current date.
///
/// # Arguments
///
/// * `log_dir` - The directory where log files are stored
///
/// # Example
///
/// ```rust
/// use dailylog::entry::get_log_file_path;
/// 
/// let path = get_log_file_path("/home/user/.dailylog");
/// // Returns something like "/home/user/.dailylog/2024-01-15.md"
/// ```
pub fn get_log_file_path(log_dir: &str) -> PathBuf {
    let date = Local::now().format("%Y-%m-%d").to_string();
    Path::new(log_dir).join(format!("{date}.md"))
}

/// Returns the file path for yesterday's log entry.
///
/// Generates a path in the format `{log_dir}/YYYY-MM-DD.md` for yesterday's date.
///
/// # Arguments
///
/// * `log_dir` - The directory where log files are stored
pub fn get_previous_day_log_path(log_dir: &str) -> PathBuf {
    let yesterday = Local::now() - Duration::days(1);
    let date = yesterday.format("%Y-%m-%d").to_string();
    Path::new(log_dir).join(format!("{date}.md"))
}

/// Returns the file path for a specific date's log entry.
///
/// Generates a path in the format `{log_dir}/YYYY-MM-DD.md` for the given date.
///
/// # Arguments
///
/// * `log_dir` - The directory where log files are stored
/// * `date` - The specific date for the log entry
pub fn get_log_file_path_for_date(log_dir: &str, date: NaiveDate) -> PathBuf {
    let date_str = date.format("%Y-%m-%d").to_string();
    Path::new(log_dir).join(format!("{date_str}.md"))
}

/// Opens the user's preferred editor to create a journal entry.
///
/// Creates a temporary file and launches the editor specified by the `$EDITOR`
/// environment variable (defaults to `vim` if not set). After the editor closes,
/// reads and returns the content that was written.
///
/// # Returns
///
/// The content written in the editor as a string.
///
/// # Errors
///
/// Returns an error if:
/// - The temporary file cannot be created
/// - The editor fails to launch
/// - The temporary file cannot be read after editing
///
/// # Example
///
/// ```rust
/// use dailylog::entry::open_editor;
///
/// let content = open_editor()?;
/// println!("User wrote: {}", content);
/// ```
pub fn open_editor() -> anyhow::Result<String> {
    let editor = env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());
    let mut temp_path = env::temp_dir();
    temp_path.push("dailylog.md");

    File::create(&temp_path)?;

    Command::new(editor)
        .arg(&temp_path)
        .status()
        .expect("Failed to launch editor");

    let mut contents = String::new();
    File::open(&temp_path)?.read_to_string(&mut contents)?;
    Ok(contents)
}

/// Opens the user's preferred editor with existing content pre-loaded.
///
/// Creates a temporary file with the provided content and launches the editor.
/// After the editor closes, reads and returns the modified content.
///
/// # Arguments
///
/// * `existing_content` - Content to pre-load in the editor
///
/// # Returns
///
/// The modified content from the editor as a string.
///
/// # Errors
///
/// Returns an error if:
/// - The temporary file cannot be created or written to
/// - The editor fails to launch
/// - The temporary file cannot be read after editing
pub fn open_editor_with_content(existing_content: &str) -> anyhow::Result<String> {
    let editor = env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());
    let mut temp_path = env::temp_dir();
    temp_path.push("dailylog.md");

    // Write existing content to temp file
    let mut file = File::create(&temp_path)?;
    file.write_all(existing_content.as_bytes())?;
    drop(file); // Ensure file is closed before opening in editor

    Command::new(editor)
        .arg(&temp_path)
        .status()
        .expect("Failed to launch editor");

    let mut contents = String::new();
    File::open(&temp_path)?.read_to_string(&mut contents)?;
    Ok(contents)
}

/// Parses entry content using git commit message style.
///
/// Follows the git commit convention:
/// - First line becomes the title
/// - Content after the first blank line becomes the body
/// - If no blank line is found, everything after the first line is treated as body
///
/// # Arguments
///
/// * `content` - The raw content from the editor
///
/// # Returns
///
/// A tuple of `(title, body)` where:
/// - `title` is `Some(String)` if a title was found, `None` otherwise
/// - `body` is the remaining content as a string
///
/// # Example
///
/// ```rust
/// use dailylog::entry::parse_entry;
///
/// let content = "Fixed authentication bug\n\nUpdated the login system to handle edge cases.";
/// let (title, body) = parse_entry(content);
/// 
/// assert_eq!(title, Some("Fixed authentication bug".to_string()));
/// assert_eq!(body, "Updated the login system to handle edge cases.");
/// ```
pub fn parse_entry(content: &str) -> (Option<String>, String) {
    let lines: Vec<&str> = content.lines().collect();

    if lines.is_empty() {
        return (None, String::new());
    }

    let title = lines[0].trim();
    if title.is_empty() {
        return (None, content.to_string());
    }

    // Find the first blank line
    let mut body_start = 1;
    for (i, line) in lines.iter().enumerate().skip(1) {
        if line.trim().is_empty() {
            body_start = i + 1;
            break;
        }
    }

    // If no blank line found, treat everything after first line as body
    if body_start == 1 && lines.len() > 1 {
        body_start = 1;
    }

    let body = if body_start < lines.len() {
        lines[body_start..].join("\n").trim().to_string()
    } else {
        String::new()
    };

    (Some(title.to_string()), body)
}

/// Formats a parsed entry into markdown with timestamp.
///
/// Creates a markdown-formatted entry with:
/// - A level 2 header with timestamp and title (if title exists)
/// - The body content below (if body exists)
///
/// # Arguments
///
/// * `title` - Optional title for the entry
/// * `body` - Body content of the entry
///
/// # Returns
///
/// A formatted markdown string ready to be written to a log file.
///
/// # Example
///
/// ```rust
/// use dailylog::entry::format_entry;
///
/// let formatted = format_entry(Some("Meeting notes"), "Discussed project timeline");
/// // Returns something like: "## 14:30 - Meeting notes\n\nDiscussed project timeline\n"
/// ```
pub fn format_entry(title: Option<&str>, body: &str) -> String {
    match title {
        Some(title) if !title.is_empty() => {
            let timestamp = Local::now().format("%H:%M").to_string();
            if body.is_empty() {
                format!("## {} - {}\n", timestamp, title)
            } else {
                format!("## {} - {}\n\n{}\n", timestamp, title, body)
            }
        }
        _ => {
            if body.is_empty() {
                String::new()
            } else {
                format!("{}\n", body)
            }
        }
    }
}

/// Appends a new entry to a log file.
///
/// Parses the content using git commit style, formats it with a timestamp,
/// and appends it to the specified log file. Creates the file if it doesn't exist.
///
/// # Arguments
///
/// * `path` - Path to the log file
/// * `content` - Raw content to parse and append
///
/// # Errors
///
/// Returns an error if the file cannot be opened or written to.
///
/// # Example
///
/// ```rust
/// use std::path::Path;
/// use dailylog::entry::append_to_log;
///
/// let content = "Fixed bug\n\nResolved the authentication issue.";
/// append_to_log(Path::new("2024-01-15.md"), content)?;
/// ```
pub fn append_to_log(path: &Path, content: &str) -> anyhow::Result<()> {
    let (title, body) = parse_entry(content);
    let formatted_entry = format_entry(title.as_deref(), &body);

    if !formatted_entry.trim().is_empty() {
        let mut file = OpenOptions::new().create(true).append(true).open(path)?;
        writeln!(file, "{}", formatted_entry)?;
    }

    Ok(())
}

/// Edits today's log file in-place using the user's preferred editor.
///
/// Reads the existing content of today's log file (if it exists), opens it in the editor,
/// and saves the modified content back to the file. If the log file doesn't exist,
/// starts with an empty file.
///
/// # Arguments
///
/// * `path` - Path to today's log file
///
/// # Errors
///
/// Returns an error if:
/// - The file cannot be read or written
/// - The editor fails to launch
/// - The temporary file operations fail
///
/// # Example
///
/// ```rust
/// use std::path::Path;
/// use dailylog::entry::edit_today_log;
///
/// edit_today_log(Path::new("2024-01-15.md"))?;
/// ```
pub fn edit_today_log(path: &Path) -> anyhow::Result<()> {
    // Read existing content if the file exists
    let existing_content = if path.exists() {
        fs::read_to_string(path)?
    } else {
        String::new()
    };

    // Open editor with existing content
    let new_content = open_editor_with_content(&existing_content)?;

    // Only write if content has changed or if it's not empty
    if new_content != existing_content && !new_content.trim().is_empty() {
        fs::write(path, new_content)?;
    } else if new_content.trim().is_empty() && path.exists() {
        // If user cleared all content, remove the file
        fs::remove_file(path)?;
        println!("Log file removed (content was empty)");
    }

    Ok(())
}