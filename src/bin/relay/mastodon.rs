use anyhow::{Context, Result};
use comms::mastodon::MastodonStatus;
use reqwest::multipart::Form;
use ulid::Ulid;

pub struct MastodonClient {
    client: reqwest::Client,
    instance_url: String,
    access_token: String,
}

impl MastodonClient {
    pub fn new(client: reqwest::Client, instance_url: String, access_token: String) -> Self {
        Self {
            client,
            instance_url,
            access_token,
        }
    }

    #[tracing::instrument(skip(self), err)]
    pub async fn post(&self, content: String) -> Result<MastodonStatus> {
        let idempotency_key = Ulid::new();
        let url = format!("{}/api/v1/statuses", self.instance_url);
        let form = Form::new().text("status", content);
        let response = self
            .client
            .post(url)
            .bearer_auth(&self.access_token)
            .header("Idempotency-Key", idempotency_key.to_string())
            .multipart(form)
            .send()
            .await
            .context("send request to Mastodon")?
            .error_for_status()
            .context("post status to Mastodon")?;
        let data: MastodonStatus = response
            .json()
            .await
            .context("deserialize Mastodon response")?;
        Ok(data)
    }
}
