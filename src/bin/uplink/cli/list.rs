use anyhow::{Context, Result};
use clap::Args;
use comms::payloads::GetAllPublicationsResponse;

#[derive(Debug, Args, Clone)]
pub struct ListArgs;

impl ListArgs {
    pub async fn exec(&self) -> Result<()> {
        let response = reqwest::get("http://localhost:8000/publications")
            .await
            .context("list all publications")?;
        let data = response
            .json::<GetAllPublicationsResponse>()
            .await
            .context("deserialize all publications response")?;
        let json = serde_json::to_string_pretty(&data).context("serialize response as JSON")?;
        println!("{json}");
        Ok(())
    }
}
