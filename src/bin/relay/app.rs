use std::sync::Arc;

use anyhow::{Context, Result};
use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use comms::{
    payloads::{
        GetAllPublicationsResponse, GetPublicationResponse, NewPublicationRequest,
        NewPublicationResponse,
    },
    publication::Publication,
};
use ulid::Ulid;

use crate::{storage::PgStorage, telemetry};

pub struct AppState {
    pub storage: PgStorage,
}

impl AppState {
    pub async fn new(db_url: &str) -> Result<Self> {
        let pg_storage = PgStorage::create(db_url)
            .await
            .context("create postgres storage")?;
        Ok(Self {
            storage: pg_storage,
        })
    }
}

pub fn app(state: AppState) -> Router {
    let tracing_layer = telemetry::get_tracing_layer();

    Router::new()
        .route("/publications", get(list_publications))
        .route("/publications", post(post_publication))
        .route("/publications/{id}", get(get_publication))
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
    Json(new_publication): Json<NewPublicationRequest>,
) -> axum::response::Result<(StatusCode, Json<NewPublicationResponse>), AppError> {
    let new_publication: Publication = new_publication.into();
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
) -> axum::response::Result<(StatusCode, Json<Option<GetPublicationResponse>>), AppError> {
    let publication = state
        .storage
        .get_publication(id)
        .await
        .context("get a publication from db")?;
    let status = if publication.is_some() {
        StatusCode::OK
    } else {
        StatusCode::NOT_FOUND
    };
    let response = publication.map(GetPublicationResponse::from);
    Ok((status, Json(response)))
}

struct AppError(anyhow::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        tracing::error!("internal server error: {:?}", self.0);
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}

impl<E: Into<anyhow::Error>> From<E> for AppError {
    fn from(err: E) -> Self {
        AppError(err.into())
    }
}
