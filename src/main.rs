use chrono::{Duration, Local, NaiveDate};
use clap::{Parser, Subcommand};
use dirs::home_dir;
use serde::Deserialize;
use std::{
    env,
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    process::Command,
};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

#[derive(Parser)]
#[command(name = "dailylog")]
#[command(about = "A minimal journaling tool")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

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
    /// Sync logs with git repository
    Sync,
    /// Pull latest logs from git repository
    Pull,
    /// Push logs to git repository
    Push,
}

#[derive(Deserialize, Default)]
struct Config {
    #[serde(default = "default_log_dir")]
    log_dir: String,
    git_repo: Option<String>,
    git_auto_sync: Option<bool>,

    #[serde(default = "default_branch")]
    git_branch_name: String,
}

fn default_log_dir() -> String {
    home_dir()
        .map(|path| path.join(".dailylog").to_string_lossy().into_owned())
        .unwrap_or_else(|| ".dailylog".to_string())
}

fn default_branch() -> String {
    "master".to_string()
}

fn load_config() -> anyhow::Result<Config> {
    let config_path = home_dir()
        .ok_or_else(|| anyhow::anyhow!("Cannot determine home directory"))?
        .join(".dailylog.toml");
    let config_str = fs::read_to_string(&config_path).unwrap_or_default();
    Ok(toml::from_str(&config_str).unwrap_or_default())
}

fn get_log_file_path(log_dir: &str) -> PathBuf {
    let date = Local::now().format("%Y-%m-%d").to_string();
    Path::new(log_dir).join(format!("{date}.md"))
}

fn get_previous_day_log_path(log_dir: &str) -> PathBuf {
    let yesterday = Local::now() - Duration::days(1);
    let date = yesterday.format("%Y-%m-%d").to_string();
    Path::new(log_dir).join(format!("{date}.md"))
}

fn get_log_file_path_for_date(log_dir: &str, date: NaiveDate) -> PathBuf {
    let date_str = date.format("%Y-%m-%d").to_string();
    Path::new(log_dir).join(format!("{date_str}.md"))
}

fn render_markdown_to_terminal(content: &str) -> anyhow::Result<()> {
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

fn view_previous_day_log(log_dir: &str) -> anyhow::Result<()> {
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

fn add_to_previous_day_log(log_dir: &str) -> anyhow::Result<()> {
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

fn open_editor() -> anyhow::Result<String> {
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

fn parse_entry(content: &str) -> (Option<String>, String) {
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

fn format_entry(title: Option<&str>, body: &str) -> String {
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

fn is_git_repo(log_dir: &str) -> bool {
    Path::new(log_dir).join(".git").exists()
}

fn run_git_command(log_dir: &str, args: &[&str]) -> anyhow::Result<()> {
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

fn init_git_repo(log_dir: &str, repo_url: &str, branch: &str) -> anyhow::Result<()> {
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

fn git_pull(log_dir: &str, branch: &str) -> anyhow::Result<()> {
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

fn git_push(log_dir: &str, branch: &str) -> anyhow::Result<()> {
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

fn git_sync(config: &Config) -> anyhow::Result<()> {
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

fn auto_sync_if_enabled(config: &Config) -> anyhow::Result<()> {
    if config.git_auto_sync.unwrap_or(false) && config.git_repo.is_some() {
        if let Err(e) = git_sync(config) {
            eprintln!("Warning: Auto-sync failed: {}", e);
        }
    }
    Ok(())
}

fn append_to_log(path: &Path, content: &str) -> anyhow::Result<()> {
    let (title, body) = parse_entry(content);
    let formatted_entry = format_entry(title.as_deref(), &body);

    if !formatted_entry.trim().is_empty() {
        let mut file = OpenOptions::new().create(true).append(true).open(path)?;
        writeln!(file, "{}", formatted_entry)?;
    }

    Ok(())
}

fn summarize_logs(log_dir: &str, days: u32) -> anyhow::Result<()> {
    let today = Local::now().date_naive();
    let mut total_entries = 0;
    let mut entries_by_day = Vec::new();

    // Print header
    let mut stdout = StandardStream::stdout(ColorChoice::Auto);
    stdout.set_color(ColorSpec::new().set_fg(Some(Color::Cyan)).set_bold(true))?;
    writeln!(stdout, "=== Log Summary for Past {} Days ===", days)?;
    stdout.reset()?;

    // Collect entries for each day
    for i in 0..days {
        let date = today - Duration::days(i as i64);
        let log_path = get_log_file_path_for_date(log_dir, date);

        if log_path.exists() {
            let content = fs::read_to_string(&log_path)?;
            if !content.trim().is_empty() {
                total_entries += 1;

                entries_by_day.push((date, content));
            }
        }
    }

    if entries_by_day.is_empty() {
        println!("No log entries found for the past {} days.", days);
        return Ok(());
    }

    // Print summary statistics
    stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green)).set_bold(true))?;
    writeln!(stdout, "\nSummary Statistics:")?;
    stdout.reset()?;
    println!("- Total days with entries: {}", total_entries);
    println!(
        "- Logging consistency: {:.1}% ({}/{} days)",
        (total_entries as f64 / days as f64) * 100.0,
        total_entries,
        days
    );

    // Show entries by day (most recent first)
    stdout.set_color(ColorSpec::new().set_fg(Some(Color::Yellow)).set_bold(true))?;
    writeln!(stdout, "\nDaily Entries:")?;
    stdout.reset()?;

    for (date, content) in entries_by_day {
        // Print date header
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Magenta)).set_bold(true))?;
        writeln!(stdout, "\n--- {} ---", date.format("%Y-%m-%d (%A)"),)?;
        stdout.reset()?;

        // Extract and show titles/headers from the content
        let titles = extract_entry_titles(&content);
        if !titles.is_empty() {
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::Blue)))?;
            for title in titles {
                println!("  - {}", title);
            }
            stdout.reset()?;
        } else {
            // If no clear titles, show first line or two
            let lines: Vec<&str> = content.lines().take(2).collect();
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)))?;
            for line in lines {
                if !line.trim().is_empty() {
                    println!("  {}", line.trim());
                    break;
                }
            }
            stdout.reset()?;
        }
    }

    // Print footer
    stdout.set_color(ColorSpec::new().set_fg(Some(Color::Cyan)).set_bold(true))?;
    writeln!(stdout, "\n=== End of Summary ===")?;
    stdout.reset()?;

    Ok(())
}

fn extract_entry_titles(content: &str) -> Vec<String> {
    let mut titles = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        // Look for markdown headers (## timestamp - title format)
        if trimmed.starts_with("## ") && trimmed.contains(" - ") {
            if let Some(title_part) = trimmed.split(" - ").nth(1) {
                titles.push(title_part.to_string());
            }
        }
        // Also look for other markdown headers
        else if trimmed.starts_with("# ") || trimmed.starts_with("### ") {
            titles.push(trimmed.trim_start_matches('#').trim().to_string());
        }
    }

    titles
}

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
            summarize_logs(&config.log_dir, days)?;
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
