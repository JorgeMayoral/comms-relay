use serde::{Deserialize, Serialize};

use crate::payloads::PublicationRequest;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Publication {
    id: String,
    content: String,
    pub_date: String,
    mastodon_url: Option<String>,
    bluesky_url: Option<String>,
}

impl Publication {
    #[must_use]
    pub fn new(
        id: String,
        content: String,
        pub_date: String,
        mastodon_url: Option<String>,
        bluesky_url: Option<String>,
    ) -> Self {
        Self {
            id,
            content,
            pub_date,
            mastodon_url,
            bluesky_url,
        }
    }

    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }
}

impl From<PublicationRequest> for Publication {
    fn from(value: PublicationRequest) -> Self {
        Self::new(
            "some_id".to_owned(),
            value.content,
            "some_date".to_owned(),
            None,
            None,
        )
    }
}
