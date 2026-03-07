use anyhow::{Context, Result};
use clap::Args;
use owo_colors::{OwoColorize, Stream, Style};

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
        let saved_msg = format!("Config saved to {}", path.display());
        let success_style = Style::new().green().bold();
        println!(
            "{}",
            saved_msg.if_supports_color(Stream::Stdout, |t| t.style(success_style))
        );
        let url_label = "  url:  ".if_supports_color(Stream::Stdout, |t| t.dimmed());
        let url_val = config.url.as_deref().unwrap_or("(not set)");
        let url_style = if config.url.is_some() {
            Style::new().bold()
        } else {
            Style::new().dimmed()
        };
        println!(
            "{url_label} {}",
            url_val.if_supports_color(Stream::Stdout, |t| t.style(url_style))
        );
        let token_preview = config
            .token
            .as_deref()
            .map_or("(not set)".to_owned(), |t| format!("{}…", &t[..t.len().min(4)]));
        let token_label = "  token:".if_supports_color(Stream::Stdout, |t| t.dimmed());
        let token_style = if config.token.is_some() {
            Style::new().bold()
        } else {
            Style::new().dimmed()
        };
        println!(
            "{token_label} {}",
            token_preview.if_supports_color(Stream::Stdout, |t| t.style(token_style))
        );
        Ok(())
    }
}
