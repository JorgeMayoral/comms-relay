use anyhow::{Context, Result};
use clap::Args;
use comms::payloads::{NewPublicationRequest, NewPublicationResponse};

#[derive(Debug, Args, Clone)]
pub struct PublishArgs {
    content: String,
}

impl PublishArgs {
    pub async fn exec(&self) -> Result<()> {
        let client = reqwest::Client::new();
        let body = NewPublicationRequest {
            content: self.content.clone(),
        };
        let response = client
            .post("http://localhost:8000/publications") // TODO: remove hardcoded url
            .bearer_auth("25df3e04-276b-4e9b-83b6-0534ad5ce451") // TODO: remove hardcoded token
            .json(&body)
            .send()
            .await
            .context("post new publication")?;
        let data = response
            .json::<NewPublicationResponse>()
            .await
            .context("deserialize new publication response")?;
        let json = serde_json::to_string_pretty(&data).context("serialize response as JSON")?;
        println!("{json}");
        Ok(())
    }
}
