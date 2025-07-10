use chrono::{Duration, Local, NaiveDate};
use std::{
    env,
    fs::{File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    process::Command,
};

pub fn get_log_file_path(log_dir: &str) -> PathBuf {
    let date = Local::now().format("%Y-%m-%d").to_string();
    Path::new(log_dir).join(format!("{date}.md"))
}

pub fn get_previous_day_log_path(log_dir: &str) -> PathBuf {
    let yesterday = Local::now() - Duration::days(1);
    let date = yesterday.format("%Y-%m-%d").to_string();
    Path::new(log_dir).join(format!("{date}.md"))
}

pub fn get_log_file_path_for_date(log_dir: &str, date: NaiveDate) -> PathBuf {
    let date_str = date.format("%Y-%m-%d").to_string();
    Path::new(log_dir).join(format!("{date_str}.md"))
}

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

pub fn append_to_log(path: &Path, content: &str) -> anyhow::Result<()> {
    let (title, body) = parse_entry(content);
    let formatted_entry = format_entry(title.as_deref(), &body);

    if !formatted_entry.trim().is_empty() {
        let mut file = OpenOptions::new().create(true).append(true).open(path)?;
        writeln!(file, "{}", formatted_entry)?;
    }

    Ok(())
}