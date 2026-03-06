use jiff::{Timestamp, Zoned, tz::TimeZone};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::payloads::NewPublicationRequest;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Publication {
    id: Ulid,
    content: String,
    pub_date: Zoned,
    mastodon_id: Option<String>,
    mastodon_url: Option<String>,
    bluesky_id: Option<String>,
    bluesky_url: Option<String>,
}

impl Publication {
    #[must_use]
    pub fn new(
        id: Ulid,
        content: String,
        pub_date: Zoned,
        mastodon_id: Option<String>,
        mastodon_url: Option<String>,
        bluesky_id: Option<String>,
        bluesky_url: Option<String>,
    ) -> Self {
        Self {
            id,
            content,
            pub_date,
            mastodon_id,
            mastodon_url,
            bluesky_id,
            bluesky_url,
        }
    }

    #[must_use]
    pub fn id(&self) -> &Ulid {
        &self.id
    }

    #[must_use]
    pub fn content(&self) -> &str {
        &self.content
    }

    #[must_use]
    pub fn pub_date(&self) -> &Zoned {
        &self.pub_date
    }

    #[must_use]
    pub fn mastodon_id(&self) -> Option<&str> {
        self.mastodon_id.as_deref()
    }

    #[must_use]
    pub fn mastodon_url(&self) -> Option<&str> {
        self.mastodon_url.as_deref()
    }

    #[must_use]
    pub fn bluesky_id(&self) -> Option<&str> {
        self.bluesky_id.as_deref()
    }

    #[must_use]
    pub fn bluesky_url(&self) -> Option<&str> {
        self.bluesky_url.as_deref()
    }
}

impl From<NewPublicationRequest> for Publication {
    fn from(value: NewPublicationRequest) -> Self {
        let id = Ulid::new();
        let pub_date = Timestamp::now().to_zoned(TimeZone::UTC);
        Self::new(id, value.content, pub_date, None, None, None, None)
    }
}
