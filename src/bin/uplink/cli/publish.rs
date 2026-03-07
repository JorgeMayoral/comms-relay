use anyhow::{Context, Result};
use clap::Args;

use crate::actions;

#[derive(Debug, Args, Clone)]
pub struct PublishArgs {
    content: String,
}

impl PublishArgs {
    pub async fn exec(&self) -> Result<()> {
        let data = actions::post_net_publication(self.content.clone())
            .await
            .context("publish content")?;
        let json = serde_json::to_string_pretty(&data).context("serialize publication as JSON")?;
        println!("{json}");
        Ok(())
    }
}
