use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct BlueskyStatus {
    pub uri: String,
    pub url: String,
}
