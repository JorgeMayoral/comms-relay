use anyhow::{Context, Result};
use clap::Args;
use comms::payloads::GetPublicationResponse;
use ulid::Ulid;

#[derive(Debug, Args, Clone)]
pub struct GetArgs {
    id: Ulid,
}

impl GetArgs {
    pub async fn exec(&self) -> Result<()> {
        let url = format!("http://localhost:8000/publications/{}", self.id);
        let response = reqwest::get(url).await.context("get a publication")?;
        let data = response
            .json::<GetPublicationResponse>()
            .await
            .context("deserialize get publication response")?;
        let json = serde_json::to_string_pretty(&data).context("serialize response as JSON")?;
        println!("{json}");
        Ok(())
    }
}
