use anyhow::{Context, Ok, Result};
use comms::{
    payloads::{
        GetAllPublicationsResponse, GetPublicationResponse, NewPublicationRequest,
        NewPublicationResponse,
    },
    publication::Publication,
};
use ulid::Ulid;

pub async fn get_publication_by_id(id: &Ulid) -> Result<Publication> {
    let url = format!("http://localhost:8000/publications/{id}");
    let response = reqwest::get(url)
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

pub async fn get_all_publications() -> Result<Vec<Publication>> {
    let response = reqwest::get("http://localhost:8000/publications")
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

pub async fn post_net_publication(content: String) -> Result<Publication> {
    let client = reqwest::Client::new();
    let body = NewPublicationRequest { content };
    let response = client
        .post("http://localhost:8000/publications") // TODO: remove hardcoded url
        .bearer_auth("25df3e04-276b-4e9b-83b6-0534ad5ce451") // TODO: remove hardcoded token
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
