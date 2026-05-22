use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LastActive {
    pub date: String,
    pub project_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub window_width: i32,
    pub window_height: i32,
    pub window_x: i32,
    pub window_y: i32,
    pub font_size: i32,
    pub last_active: Option<LastActive>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            window_width: 300,
            window_height: 500,
            window_x: 100,
            window_y: 100,
            font_size: 12,
            last_active: None,
        }
    }
}

impl Config {
    pub fn config_path() -> PathBuf {
        let mut p = config_base_dir();
        p.push("titrax");
        p.push("config.toml");
        p
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if let Ok(text) = fs::read_to_string(&path) {
            toml::from_str(&text).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    pub fn save(&self) {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(text) = toml::to_string(self) {
            let _ = fs::write(&path, text);
        }
    }
}

fn config_base_dir() -> PathBuf {
    if let Ok(val) = std::env::var("XDG_CONFIG_HOME") {
        PathBuf::from(val)
    } else {
        let mut home = home_dir();
        home.push(".config");
        home
    }
}

pub fn home_dir() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/tmp"))
}
