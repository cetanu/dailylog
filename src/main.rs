use chrono::Local;
use dirs::home_dir;
use serde::Deserialize;
use std::{
    env,
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    process::Command,
};

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
    let config = load_config()?;
    fs::create_dir_all(&config.log_dir)?;
    let log_path = get_log_file_path(&config.log_dir);

    let entry = open_editor()?;
    if !entry.trim().is_empty() {
        append_to_log(&log_path, &entry)?;
        println!("Log saved to {:?}", log_path);
    } else {
        println!("No content written. Aborted.");
    }

    Ok(())
}
