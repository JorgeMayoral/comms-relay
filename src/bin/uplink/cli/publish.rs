use anyhow::{Context, Result};
use clap::Args;

use crate::{actions, display};

#[derive(Debug, Args, Clone)]
pub struct PublishArgs {
    content: String,
}

impl PublishArgs {
    pub async fn exec(&self, base_url: &str, token: &str, json: bool) -> Result<()> {
        let data = actions::post_net_publication(base_url, token, self.content.clone())
            .await
            .context("publish content")?;
        if json {
            let out =
                serde_json::to_string_pretty(&data).context("serialize publication as JSON")?;
            println!("{out}");
        } else {
            display::print_publish_success(&data);
        }
        Ok(())
    }
}
