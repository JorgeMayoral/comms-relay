use anyhow::{Context, Result};
use clap::Args;

use crate::actions;

#[derive(Debug, Args, Clone)]
pub struct ListArgs;

impl ListArgs {
    pub async fn exec(&self, base_url: &str) -> Result<()> {
        let data = actions::get_all_publications(base_url)
            .await
            .context("list publications")?;
        let json = serde_json::to_string_pretty(&data).context("serialize publications as JSON")?;
        println!("{json}");
        Ok(())
    }
}
