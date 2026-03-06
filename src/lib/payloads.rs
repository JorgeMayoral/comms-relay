use serde::{Deserialize, Serialize};

use crate::publication::Publication;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewPublicationRequest {
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewPublicationResponse(Publication);

impl From<Publication> for NewPublicationResponse {
    fn from(value: Publication) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetPublicationResponse(Publication);

impl From<Publication> for GetPublicationResponse {
    fn from(value: Publication) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GetAllPublicationsResponse(Vec<Publication>);

impl From<Vec<Publication>> for GetAllPublicationsResponse {
    fn from(value: Vec<Publication>) -> Self {
        Self(value)
    }
}
