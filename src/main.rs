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
}

#[derive(Deserialize)]
struct Config {
    log_dir: String,
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

fn append_to_log(path: &Path, content: &str) -> anyhow::Result<()> {
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    writeln!(file, "{}", content)?;
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
        }
        None => {
            // Default behavior: create new log entry
            let log_path = get_log_file_path(&config.log_dir);
            let entry = open_editor()?;
            if !entry.trim().is_empty() {
                append_to_log(&log_path, &entry)?;
                println!("Log saved to {:?}", log_path);
            } else {
                println!("No content written. Aborted.");
            }
        }
    }

    Ok(())
}
