use dirs::home_dir;
use serde::Deserialize;
use std::fs;

#[derive(Deserialize, Default)]
pub struct Config {
    #[serde(default = "default_log_dir")]
    pub log_dir: String,
    pub git_repo: Option<String>,
    pub git_auto_sync: Option<bool>,

    #[serde(default = "default_branch")]
    pub git_branch_name: String,

    #[serde(default = "default_summary_days")]
    pub summary_days: Vec<String>,
}

fn default_log_dir() -> String {
    home_dir()
        .map(|path| path.join(".dailylog").to_string_lossy().into_owned())
        .unwrap_or_else(|| ".dailylog".to_string())
}

fn default_branch() -> String {
    "master".to_string()
}

fn default_summary_days() -> Vec<String> {
    vec![
        "monday".to_string(),
        "tuesday".to_string(),
        "wednesday".to_string(),
        "thursday".to_string(),
        "friday".to_string(),
    ]
}

pub fn load_config() -> anyhow::Result<Config> {
    let config_path = home_dir()
        .ok_or_else(|| anyhow::anyhow!("Cannot determine home directory"))?
        .join(".dailylog.toml");
    let config_str = fs::read_to_string(&config_path).unwrap_or_default();
    Ok(toml::from_str(&config_str).unwrap_or_default())
}