use anyhow::{Context, Result};
use clap::Args;
use ulid::Ulid;

use crate::actions;

#[derive(Debug, Args, Clone)]
pub struct GetArgs {
    id: Ulid,
}

impl GetArgs {
    pub async fn exec(&self, base_url: &str) -> Result<()> {
        let data = actions::get_publication_by_id(base_url, &self.id)
            .await
            .context(format!("get publication {}", self.id))?;
        let json = serde_json::to_string_pretty(&data).context("serialize publication as JSON")?;
        println!("{json}");
        Ok(())
    }
}
