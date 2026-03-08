use std::sync::Arc;

use anyhow::{Context, Result};
use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get, post},
};
use comms::{
    payloads::{
        GetAllPublicationsResponse, GetPublicationResponse, NewPublicationRequest,
        NewPublicationResponse,
    },
    publication::Publication,
};
use ulid::Ulid;

use crate::{auth::BearerAuth, error::AppError, mastodon, storage::PgStorage, telemetry};

pub(crate) struct AppState {
    pub(crate) storage: PgStorage,
    pub(crate) api_token: String,
    pub(crate) mastodon_access_token: String,
    pub(crate) mastodon_instance_url: String,
}

impl AppState {
    pub(crate) async fn new(
        db_url: &str,
        api_token: String,
        mastodon_access_token: String,
        mastodon_instance_url: String,
    ) -> Result<Self> {
        let storage = PgStorage::create(db_url)
            .await
            .context("create postgres storage")?;
        Ok(Self {
            storage,
            api_token,
            mastodon_access_token,
            mastodon_instance_url,
        })
    }
}

pub fn app(state: AppState) -> Router {
    let tracing_layer = telemetry::get_tracing_layer();

    Router::new()
        .route("/publications", get(list_publications))
        .route("/publications", post(post_publication))
        .route("/publications/{id}", get(get_publication))
        .route("/publications/{id}", delete(delete_publication))
        .with_state(Arc::new(state))
        .layer(tracing_layer)
}

async fn list_publications(
    State(state): State<Arc<AppState>>,
) -> axum::response::Result<(StatusCode, Json<GetAllPublicationsResponse>), AppError> {
    let publications = state
        .storage
        .list_publications()
        .await
        .context("list all publications from db")?;
    Ok((
        StatusCode::OK,
        Json(GetAllPublicationsResponse::from(publications)),
    ))
}

async fn post_publication(
    State(state): State<Arc<AppState>>,
    _auth: BearerAuth,
    Json(new_publication): Json<NewPublicationRequest>,
) -> axum::response::Result<(StatusCode, Json<NewPublicationResponse>), AppError> {
    let mut new_publication: Publication = new_publication.into();
    match mastodon::MastodonClient::new(
        state.mastodon_instance_url.clone(),
        state.mastodon_access_token.clone(),
    )
    .post(new_publication.content().to_owned())
    .await
    {
        Ok(mastodon_response) => {
            new_publication.set_mastodon_id(mastodon_response.id);
            new_publication.set_mastodon_url(mastodon_response.url);
        }
        Err(error) => {
            tracing::error!(error = %format_args!("{:#}", error), "failed to post to Mastodon");
        }
    }

    state
        .storage
        .insert_publication(&new_publication)
        .await
        .context("insert new publication into db")?;
    Ok((
        StatusCode::CREATED,
        Json(NewPublicationResponse::from(new_publication)),
    ))
}

async fn get_publication(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Ulid>,
) -> axum::response::Result<Response, AppError> {
    let Some(publication) = state
        .storage
        .get_publication(id)
        .await
        .context("get a publication from db")?
    else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };
    Ok((
        StatusCode::OK,
        Json(GetPublicationResponse::from(publication)),
    )
        .into_response())
}

async fn delete_publication(
    State(state): State<Arc<AppState>>,
    _auth: BearerAuth,
    Path(id): Path<Ulid>,
) -> axum::response::Result<StatusCode, AppError> {
    let deleted = state
        .storage
        .delete_publication(id)
        .await
        .context("delete a publication from db")?;
    let status = if deleted {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    };
    Ok(status)
}
