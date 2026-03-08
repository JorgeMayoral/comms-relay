use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct MastodonStatus {
    pub id: String,
    pub url: String,
    pub uri: String,
    pub created_at: String,
}
