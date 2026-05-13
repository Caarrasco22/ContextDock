use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppSettings {
    pub root_projects_path: String,
    pub opencode_command: String,
    pub theme: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        let home = dirs_next().unwrap_or_else(|| PathBuf::from("."));
        let default_path = if cfg!(target_os = "windows") {
            home.join("Documents").join("Codex")
        } else {
            home.join("Projects")
        };

        Self {
            root_projects_path: default_path.to_string_lossy().to_string(),
            opencode_command: if cfg!(target_os = "windows") {
                "opencode.cmd".to_string()
            } else {
                "opencode".to_string()
            },
            theme: "dark".to_string(),
        }
    }
}

fn get_settings_path() -> PathBuf {
    let config_dir = dirs_next().unwrap_or_else(|| PathBuf::from("."));
    config_dir.join("contextdock-settings.json")
}

fn dirs_next() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        std::env::var("APPDATA").ok().map(PathBuf::from)
    }
    #[cfg(target_os = "macos")]
    {
        std::env::var("HOME").ok().map(|h| PathBuf::from(h).join("Library/Application Support"))
    }
    #[cfg(target_os = "linux")]
    {
        std::env::var("HOME").ok().map(|h| PathBuf::from(h).join(".config"))
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        None
    }
}

pub fn load_settings() -> Result<AppSettings, String> {
    let path = get_settings_path();

    if !path.exists() {
        return Ok(AppSettings::default());
    }

    let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    serde_json::from_str(&content).map_err(|e| e.to_string())
}

pub fn save_settings(settings: &AppSettings) -> Result<(), String> {
    let path = get_settings_path();

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let content = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    fs::write(&path, content).map_err(|e| e.to_string())
}