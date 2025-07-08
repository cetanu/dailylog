use chrono::{Duration, Local};
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
    /// Sync logs with git repository
    Sync,
    /// Pull latest logs from git repository
    Pull,
    /// Push logs to git repository
    Push,
}

#[derive(Deserialize)]
struct Config {
    log_dir: String,
    git_repo: Option<String>,
    git_auto_sync: Option<bool>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            log_dir: home_dir()
                .ok_or_else(|| anyhow::anyhow!("Cannot determine home directory"))
                .unwrap()
                .join(".dailylog")
                .to_string_lossy()
                .into_owned(),
            git_repo: None,
            git_auto_sync: Some(false),
        }
    }
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
    Path::new(log_dir).join(format!("{date}.txt"))
}

fn get_previous_day_log_path(log_dir: &str) -> PathBuf {
    let yesterday = Local::now() - Duration::days(1);
    let date = yesterday.format("%Y-%m-%d").to_string();
    Path::new(log_dir).join(format!("{date}.txt"))
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
            stdout.set_color(ColorSpec::new().set_bg(Some(Color::Black)).set_fg(Some(Color::White)))?;
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
    temp_path.push("dailylog.tmp");

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

fn init_git_repo(log_dir: &str, repo_url: &str) -> anyhow::Result<()> {
    if is_git_repo(log_dir) {
        println!("Git repository already exists in {}", log_dir);
        return Ok(());
    }
    
    println!("Initializing git repository in {}", log_dir);
    run_git_command(log_dir, &["init"])?;
    run_git_command(log_dir, &["remote", "add", "origin", repo_url])?;
    
    // Try to pull existing logs
    if let Err(e) = run_git_command(log_dir, &["pull", "origin", "main"]) {
        println!("Note: Could not pull from remote (this is normal for new repos): {}", e);
        // Create initial commit
        run_git_command(log_dir, &["checkout", "-b", "main"])?;
    }
    
    Ok(())
}

fn git_pull(log_dir: &str) -> anyhow::Result<()> {
    if !is_git_repo(log_dir) {
        return Err(anyhow::anyhow!("Not a git repository. Use 'dailylog sync' to set up git sync first."));
    }
    
    println!("Pulling latest logs from git repository...");
    run_git_command(log_dir, &["pull", "origin", "main"])?;
    println!("Successfully pulled latest logs.");
    
    Ok(())
}

fn git_push(log_dir: &str) -> anyhow::Result<()> {
    if !is_git_repo(log_dir) {
        return Err(anyhow::anyhow!("Not a git repository. Use 'dailylog sync' to set up git sync first."));
    }
    
    // Add all log files
    run_git_command(log_dir, &["add", "*.txt"])?;
    
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
    run_git_command(log_dir, &["push", "origin", "main"])?;
    println!("Successfully pushed logs.");
    
    Ok(())
}

fn git_sync(config: &Config) -> anyhow::Result<()> {
    let repo_url = config.git_repo.as_ref()
        .ok_or_else(|| anyhow::anyhow!("No git repository configured. Please add 'git_repo = \"your-repo-url\"' to ~/.dailylog.toml"))?;
    
    if !is_git_repo(&config.log_dir) {
        init_git_repo(&config.log_dir, repo_url)?;
    }
    
    // Pull first, then push
    git_pull(&config.log_dir)?;
    git_push(&config.log_dir)?;
    
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
        Some(Commands::Sync) => {
            git_sync(&config)?;
        }
        Some(Commands::Pull) => {
            git_pull(&config.log_dir)?;
        }
        Some(Commands::Push) => {
            git_push(&config.log_dir)?;
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
