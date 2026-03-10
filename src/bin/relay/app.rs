use std::sync::Arc;

use anyhow::{Context, Result};
use axum::{
    Json, Router,
    extract::{Path, Query, State},
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
use serde::Deserialize;
use ulid::Ulid;

use crate::{
    auth::BearerAuth, bluesky::BlueskyClient, error::AppError, mastodon::MastodonClient,
    storage::PgStorage, telemetry,
};

pub(crate) struct AppState {
    pub(crate) storage: PgStorage,
    pub(crate) api_token: String,
    pub(crate) mastodon_client: MastodonClient,
    pub(crate) bluesky_client: BlueskyClient,
}

impl AppState {
    pub(crate) async fn new(
        db_url: &str,
        api_token: String,
        mastodon_access_token: String,
        mastodon_instance_url: String,
        bluesky_instance_url: String,
        bluesky_identifier: String,
        bluesky_app_password: String,
    ) -> Result<Self> {
        let storage = PgStorage::create(db_url)
            .await
            .context("create postgres storage")?;

        let http_client = reqwest::ClientBuilder::new()
            .build()
            .context("build reqwest HTTP client")?;

        let mastodon_client = MastodonClient::new(
            http_client.clone(),
            mastodon_instance_url,
            mastodon_access_token,
        );
        let bluesky_client = BlueskyClient::new(
            http_client,
            bluesky_instance_url,
            bluesky_identifier,
            bluesky_app_password,
        );

        Ok(Self {
            storage,
            api_token,
            mastodon_client,
            bluesky_client,
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

#[derive(Debug, Deserialize)]
struct Pagination {
    page: Option<i64>,
    per_page: Option<i64>,
}

async fn list_publications(
    State(state): State<Arc<AppState>>,
    Query(Pagination { page, per_page }): Query<Pagination>,
) -> axum::response::Result<(StatusCode, Json<GetAllPublicationsResponse>), AppError> {
    let page = page.unwrap_or(1);
    let per_page = per_page.unwrap_or(100);
    if page < 1 {
        return Err(AppError::Unprocessable("`page` must be >= 1".into()));
    }
    if !(1..=500).contains(&per_page) {
        return Err(AppError::Unprocessable(
            "`per_page` must be between 1 and 500".into(),
        ));
    }
    let offset = per_page * (page - 1);
    let publications = state
        .storage
        .list_publications(per_page, offset)
        .await
        .context("list all publications from db")?;
    let total_results = state
        .storage
        .count_publications()
        .await
        .context("count publications")?;
    let total_pages = (total_results + per_page - 1) / per_page;
    let response =
        GetAllPublicationsResponse::new(publications, page, per_page, total_results, total_pages);
    Ok((StatusCode::OK, Json(response)))
}

async fn post_publication(
    State(state): State<Arc<AppState>>,
    _auth: BearerAuth,
    Json(new_publication): Json<NewPublicationRequest>,
) -> axum::response::Result<(StatusCode, Json<NewPublicationResponse>), AppError> {
    let mut new_publication: Publication = new_publication.into();
    match state
        .mastodon_client
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

    match state
        .bluesky_client
        .post(new_publication.content().to_owned())
        .await
    {
        Ok(bluesky_response) => {
            new_publication.set_bluesky_id(bluesky_response.uri);
            new_publication.set_bluesky_url(bluesky_response.url);
        }
        Err(error) => {
            tracing::error!(error = %format_args!("{:#}", error), "failed to post to Bluesky");
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
