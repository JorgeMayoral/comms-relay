use serde::{Deserialize, Serialize};

use crate::publication::Publication;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewPublicationRequest {
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewPublicationResponse(Publication);

impl NewPublicationResponse {
    #[must_use]
    pub fn inner(self) -> Publication {
        self.0
    }
}

impl From<Publication> for NewPublicationResponse {
    fn from(value: Publication) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetPublicationResponse(Publication);

impl GetPublicationResponse {
    #[must_use]
    pub fn inner(self) -> Publication {
        self.0
    }
}

impl From<Publication> for GetPublicationResponse {
    fn from(value: Publication) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GetAllPublicationsResponse {
    publications: Vec<Publication>,
    page: i64,
    per_page: i64,
    total_results: i64,
    total_pages: i64,
}

impl GetAllPublicationsResponse {
    #[must_use]
    pub fn new(
        publications: Vec<Publication>,
        page: i64,
        per_page: i64,
        total_results: i64,
        total_pages: i64,
    ) -> Self {
        Self {
            publications,
            page,
            per_page,
            total_results,
            total_pages,
        }
    }

    #[must_use]
    pub fn publications(&self) -> Vec<Publication> {
        self.publications.clone()
    }

    #[must_use]
    pub fn page(&self) -> i64 {
        self.page
    }

    #[must_use]
    pub fn total_pages(&self) -> i64 {
        self.total_pages
    }

    #[must_use]
    pub fn total_results(&self) -> i64 {
        self.total_results
    }
}
