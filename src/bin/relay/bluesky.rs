use anyhow::{Context, Result};
use comms::bluesky::BlueskyStatus;
use jiff::Timestamp;
use serde::{Deserialize, Serialize};

pub struct BlueskyClient {
    client: reqwest::Client,
    instance_url: String,
    user_identifier: String,
    user_password: String,
}

#[derive(Deserialize)]
struct Session {
    did: String,
    #[serde(rename = "accessJwt")]
    access_jwt: String,
    handle: String,
}

#[derive(Serialize)]
struct CreateRecordRequest<'a> {
    repo: &'a str,
    collection: &'static str,
    record: PostRecord<'a>,
}

#[derive(Serialize)]
struct PostRecord<'a> {
    #[serde(rename = "$type")]
    type_: &'static str,
    text: &'a str,
    #[serde(rename = "createdAt")]
    created_at: String,
}

#[derive(Deserialize)]
struct CreateRecordResponse {
    uri: String,
}

impl BlueskyClient {
    pub fn new(
        client: reqwest::Client,
        instance_url: String,
        user_identifier: String,
        user_password: String,
    ) -> Self {
        Self {
            client,
            instance_url,
            user_identifier,
            user_password,
        }
    }

    #[tracing::instrument(skip(self), err)]
    pub async fn post(&self, content: String) -> Result<BlueskyStatus> {
        let session: Session = self
            .client
            .post(format!(
                "{}/xrpc/com.atproto.server.createSession",
                self.instance_url
            ))
            .json(&serde_json::json!({
                "identifier": self.user_identifier,
                "password": self.user_password,
            }))
            .send()
            .await
            .context("send Bluesky createSession request")?
            .error_for_status()
            .context("authenticate with Bluesky")?
            .json()
            .await
            .context("deserialize Bluesky session")?;

        let created_at = Timestamp::now().to_string();
        let body = CreateRecordRequest {
            repo: &session.did,
            collection: "app.bsky.feed.post",
            record: PostRecord {
                type_: "app.bsky.feed.post",
                text: &content,
                created_at,
            },
        };
        let record: CreateRecordResponse = self
            .client
            .post(format!(
                "{}/xrpc/com.atproto.repo.createRecord",
                self.instance_url
            ))
            .bearer_auth(&session.access_jwt)
            .json(&body)
            .send()
            .await
            .context("send Bluesky createRecord request")?
            .error_for_status()
            .context("create Bluesky post")?
            .json()
            .await
            .context("deserialize Bluesky createRecord response")?;

        let rkey = record
            .uri
            .rsplit('/')
            .next()
            .context("extract rkey from Bluesky AT URI")?;
        let url = format!("https://bsky.app/profile/{}/post/{rkey}", session.handle);

        Ok(BlueskyStatus {
            uri: record.uri,
            url,
        })
    }
}
