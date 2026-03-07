use std::{fs, path::PathBuf};

use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AppConfig {
    pub url: Option<String>,
    pub token: Option<String>,
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        let path = config_path()?;
        if !path.exists() {
            return Ok(Self::default());
        }
        let contents = fs::read_to_string(&path)
            .with_context(|| format!("read config file at {}", path.display()))?;
        toml::from_str(&contents)
            .with_context(|| format!("parse config file at {}", path.display()))
    }

    pub fn save(&self) -> Result<PathBuf> {
        let path = config_path()?;
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir)
                .with_context(|| format!("create config directory at {}", dir.display()))?;
        }
        let contents = toml::to_string(self).context("serialize config as TOML")?;
        fs::write(&path, contents)
            .with_context(|| format!("write config file at {}", path.display()))?;
        Ok(path)
    }
}

fn config_path() -> Result<PathBuf> {
    let dirs = ProjectDirs::from("dev", "yorch", "uplink")
        .context("determine platform config directory")?;
    Ok(dirs.config_dir().join("config.toml"))
}
