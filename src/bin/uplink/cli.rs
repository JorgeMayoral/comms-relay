use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

use crate::{
    cli::{config::ConfigArgs, get::GetArgs, list::ListArgs, publish::PublishArgs},
    config::AppConfig,
};

mod config;
mod get;
mod list;
mod publish;

#[derive(Debug, Parser, Clone)]
#[command(version, about, author)]
pub struct Cli {
    /// Relay server base URL (overrides config file and `RELAY_URL` env var)
    #[arg(long, env = "RELAY_URL", global = true)]
    url: Option<String>,

    /// Bearer token for authenticated requests (overrides config file and `RELAY_API_TOKEN` env var)
    #[arg(long, env = "RELAY_API_TOKEN", global = true)]
    token: Option<String>,

    /// Output raw JSON instead of formatted text
    #[arg(long, global = true)]
    json: bool,

    #[command(subcommand)]
    command: Command,
}

impl Cli {
    pub async fn run(&self) -> Result<()> {
        let file_config = AppConfig::load().context("load config file")?;
        let base_url = self
            .url
            .as_deref()
            .or(file_config.url.as_deref())
            .unwrap_or("http://localhost:8000");
        let token = self.token.as_deref().or(file_config.token.as_deref());

        match &self.command {
            Command::Config(args) => {
                args.exec().context("execute config command")?;
            }
            Command::Publish(args) => {
                let token = token.context(
                    "no token provided — run `uplink config --token <token>` or set RELAY_API_TOKEN",
                )?;
                args.exec(base_url, token, self.json)
                    .await
                    .context("execute publish command")?;
            }
            Command::List(args) => {
                args.exec(base_url, self.json)
                    .await
                    .context("execute list command")?;
            }
            Command::Get(args) => {
                args.exec(base_url, self.json)
                    .await
                    .context("execute get command")?;
            }
        }

        Ok(())
    }
}

#[derive(Debug, Subcommand, Clone)]
enum Command {
    /// Save relay URL and/or token to the local config file
    Config(ConfigArgs),
    /// Creates a new publication
    Publish(PublishArgs),
    /// List all publications
    List(ListArgs),
    /// Get a single publication by ID
    Get(GetArgs),
}
