use anyhow::{Context, Result};
use clap::Parser;

use crate::cli::Cli;

mod actions;
mod cli;
mod config;
mod display;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    cli.run().await.context("run cli")
}
