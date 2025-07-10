//! Terminal display and markdown rendering.
//!
//! This module handles colorized output to the terminal, including
//! markdown rendering with syntax highlighting and styled display
//! of log entries.

use crate::entry::{get_previous_day_log_path, open_editor, append_to_log};
use chrono::{Duration, Local};
use std::fs;
use std::io::Write;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

/// Renders markdown content to the terminal with color highlighting.
///
/// Provides syntax highlighting for various markdown elements:
/// - H1 headers: bright blue and bold
/// - H2 headers: cyan and bold  
/// - H3 headers: green and bold
/// - List items: yellow bullets
/// - Code blocks: gray background
/// - Bold text: terminal bold formatting
///
/// # Arguments
///
/// * `content` - The markdown content to render
///
/// # Errors
///
/// Returns an error if writing to the terminal fails.
///
/// # Example
///
/// ```rust
/// use dailylog::display::render_markdown_to_terminal;
///
/// let markdown = "# Title\n\n- List item\n- Another item";
/// render_markdown_to_terminal(markdown)?;
/// ```
pub fn render_markdown_to_terminal(content: &str) -> anyhow::Result<()> {
    let mut stdout = StandardStream::stdout(ColorChoice::Auto);

    for line in content.lines() {
        if line.starts_with("# ") {
            // H1 headers - bright blue and bold
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::Blue)).set_bold(true))?;
            writeln!(stdout, "{}", line)?;
            stdout.reset()?;
        } else if line.starts_with("## ") {
            // H2 headers - cyan and bold
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::Cyan)).set_bold(true))?;
            writeln!(stdout, "{}", line)?;
            stdout.reset()?;
        } else if line.starts_with("### ") {
            // H3 headers - green and bold
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green)).set_bold(true))?;
            writeln!(stdout, "{}", line)?;
            stdout.reset()?;
        } else if line.starts_with("- ") || line.starts_with("* ") {
            // List items - yellow bullet
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::Yellow)))?;
            write!(stdout, "• ")?;
            stdout.reset()?;
            writeln!(stdout, "{}", &line[2..])?;
        } else if line.starts_with("```") {
            // Code blocks - gray background
            stdout.set_color(
                ColorSpec::new()
                    .set_bg(Some(Color::Black))
                    .set_fg(Some(Color::White)),
            )?;
            writeln!(stdout, "{}", line)?;
            stdout.reset()?;
        } else if line.trim().is_empty() {
            // Empty lines
            writeln!(stdout)?;
        } else {
            // Regular text - check for inline formatting
            let mut processed_line = line.to_string();

            // Handle **bold** text
            while let Some(start) = processed_line.find("**") {
                if let Some(end) = processed_line[start + 2..].find("**") {
                    let end = end + start + 2;
                    let before = &processed_line[..start];
                    let bold_text = &processed_line[start + 2..end];
                    let after = &processed_line[end + 2..];

                    write!(stdout, "{}", before)?;
                    stdout.set_color(ColorSpec::new().set_bold(true))?;
                    write!(stdout, "{}", bold_text)?;
                    stdout.reset()?;
                    processed_line = after.to_string();
                } else {
                    break;
                }
            }
            writeln!(stdout, "{}", processed_line)?;
        }
    }

    Ok(())
}

/// Displays the previous day's log entry with colorized output.
///
/// Reads and displays yesterday's log file with:
/// - Styled header and footer showing the date
/// - Markdown rendering with syntax highlighting
/// - Appropriate messages if the file doesn't exist or is empty
///
/// # Arguments
///
/// * `log_dir` - The directory containing log files
///
/// # Errors
///
/// Returns an error if:
/// - The log file cannot be read
/// - Terminal output fails
///
/// # Example
///
/// ```rust
/// use dailylog::display::view_previous_day_log;
///
/// view_previous_day_log("/path/to/logs")?;
/// ```
pub fn view_previous_day_log(log_dir: &str) -> anyhow::Result<()> {
    let log_path = get_previous_day_log_path(log_dir);

    if !log_path.exists() {
        println!("No log entry found for previous day: {:?}", log_path);
        return Ok(());
    }

    let content = fs::read_to_string(&log_path)?;
    if content.trim().is_empty() {
        println!("Previous day's log is empty: {:?}", log_path);
    } else {
        let yesterday = Local::now() - Duration::days(1);
        let date_str = yesterday.format("%Y-%m-%d").to_string();

        // Print header with styling
        let mut stdout = StandardStream::stdout(ColorChoice::Auto);
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Magenta)).set_bold(true))?;
        writeln!(stdout, "=== Log entry for {} ===", date_str)?;
        stdout.reset()?;

        // Render the content with markdown styling
        render_markdown_to_terminal(&content)?;

        // Print footer with styling
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Magenta)).set_bold(true))?;
        writeln!(stdout, "=== End of log entry ===")?;
        stdout.reset()?;
    }

    Ok(())
}

/// Adds a new entry to the previous day's log file.
///
/// This function:
/// 1. Shows existing content from yesterday's log (if any) with colorized display
/// 2. Opens the user's editor to write a new entry
/// 3. Appends the new entry to yesterday's log file
/// 4. Provides appropriate feedback about the operation
///
/// # Arguments
///
/// * `log_dir` - The directory containing log files
///
/// # Errors
///
/// Returns an error if:
/// - The existing log file cannot be read
/// - The editor fails to open
/// - The new entry cannot be written to the file
/// - Terminal output fails
///
/// # Example
///
/// ```rust
/// use dailylog::display::add_to_previous_day_log;
///
/// add_to_previous_day_log("/path/to/logs")?;
/// ```
pub fn add_to_previous_day_log(log_dir: &str) -> anyhow::Result<()> {
    let log_path = get_previous_day_log_path(log_dir);
    let yesterday = Local::now() - Duration::days(1);
    let date_str = yesterday.format("%Y-%m-%d").to_string();

    // Show existing content if available
    if log_path.exists() {
        let content = fs::read_to_string(&log_path)?;
        if !content.trim().is_empty() {
            println!("Existing entry for {}:", date_str);

            // Print header with styling
            let mut stdout = StandardStream::stdout(ColorChoice::Auto);
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::Magenta)).set_bold(true))?;
            writeln!(stdout, "=== Log entry for {} ===", date_str)?;
            stdout.reset()?;

            // Render the content with markdown styling
            render_markdown_to_terminal(&content)?;

            // Print footer with styling
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::Magenta)).set_bold(true))?;
            writeln!(stdout, "=== End of existing entry ===")?;
            stdout.reset()?;

            println!("\nAppending to yesterday's log...");
        } else {
            println!("Creating new entry for yesterday ({})", date_str);
        }
    } else {
        println!("Creating new entry for yesterday ({})", date_str);
    }

    // Open editor for new content
    let entry = open_editor()?;
    if !entry.trim().is_empty() {
        append_to_log(&log_path, &entry)?;
        println!("Log saved to {:?}", log_path);
    } else {
        println!("No content written. Aborted.");
    }

    Ok(())
}