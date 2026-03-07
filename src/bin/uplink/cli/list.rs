use anyhow::{Context, Result};
use clap::Args;

use crate::{actions, display};

#[derive(Debug, Args, Clone)]
pub struct ListArgs;

impl ListArgs {
    pub async fn exec(&self, base_url: &str, json: bool) -> Result<()> {
        let data = actions::get_all_publications(base_url)
            .await
            .context("list publications")?;
        if json {
            let out =
                serde_json::to_string_pretty(&data).context("serialize publications as JSON")?;
            println!("{out}");
        } else {
            display::print_publications(&data);
        }
        Ok(())
    }
}
