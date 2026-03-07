use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

use crate::cli::{get::GetArgs, list::ListArgs, publish::PublishArgs};

mod get;
mod list;
mod publish;

#[derive(Debug, Parser, Clone)]
#[command(version, about, author)]
pub struct Cli {
    #[command(subcommand)]
    command: Command,
}

impl Cli {
    pub async fn run(&self) -> Result<()> {
        match &self.command {
            Command::Publish(args) => {
                args.exec().await.context("execute publish command")?;
            }
            Command::List(args) => {
                args.exec().await.context("execute list command")?;
            }
            Command::Get(args) => {
                args.exec().await.context("execute get command")?;
            }
        }

        Ok(())
    }
}

#[derive(Debug, Subcommand, Clone)]
enum Command {
    /// Creates a new publication
    Publish(PublishArgs),
    /// List all publications
    List(ListArgs),
    /// Get a single publication by ID
    Get(GetArgs),
}
