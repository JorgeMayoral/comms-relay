use anyhow::{Context, Result};
use clap::Args;

use crate::config::AppConfig;

#[derive(Debug, Args, Clone)]
pub struct ConfigArgs {
    /// Relay server base URL
    #[arg(long)]
    url: Option<String>,

    /// Bearer token for authenticated requests
    #[arg(long)]
    token: Option<String>,
}

impl ConfigArgs {
    pub fn exec(&self) -> Result<()> {
        let mut config = AppConfig::load().context("load existing config")?;
        if let Some(url) = &self.url {
            config.url = Some(url.clone());
        }
        if let Some(token) = &self.token {
            config.token = Some(token.clone());
        }
        let path = config.save().context("save config")?;
        println!("Config saved to {}", path.display());
        println!("  url:   {}", config.url.as_deref().unwrap_or("(not set)"));
        println!(
            "  token: {}",
            config
                .token
                .as_deref()
                .map_or("(not set)".to_owned(), |t| format!(
                    "{}…",
                    &t[..t.len().min(4)]
                ))
        );
        Ok(())
    }
}
