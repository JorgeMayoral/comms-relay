use anyhow::{Context, Ok, Result};
use comms::{
    payloads::{
        GetAllPublicationsResponse, GetPublicationResponse, NewPublicationRequest,
        NewPublicationResponse,
    },
    publication::Publication,
};
use ulid::Ulid;

pub async fn get_publication_by_id(base_url: &str, id: &Ulid) -> Result<Publication> {
    let url = format!("{base_url}/publications/{id}");
    let response = reqwest::get(&url)
        .await
        .context(format!("send GET /publications/{id} to relay"))?;
    let data = response
        .error_for_status()
        .context("check relay response status")?
        .json::<GetPublicationResponse>()
        .await
        .context("deserialize publication response")?;
    Ok(data.inner())
}

pub async fn get_all_publications(base_url: &str) -> Result<Vec<Publication>> {
    let url = format!("{base_url}/publications");
    let response = reqwest::get(&url)
        .await
        .context("send GET /publications to relay")?;
    let data = response
        .error_for_status()
        .context("check relay response status")?
        .json::<GetAllPublicationsResponse>()
        .await
        .context("deserialize publications list response")?;
    Ok(data.inner())
}

pub async fn post_new_publication(
    base_url: &str,
    token: &str,
    content: String,
) -> Result<Publication> {
    let client = reqwest::Client::new();
    let body = NewPublicationRequest { content };
    let url = format!("{base_url}/publications");
    let response = client
        .post(&url)
        .bearer_auth(token)
        .json(&body)
        .send()
        .await
        .context("send POST /publications to relay")?
        .error_for_status()
        .context("check relay response status")?;
    let data = response
        .json::<NewPublicationResponse>()
        .await
        .context("deserialize new publication response")?;
    Ok(data.inner())
}
