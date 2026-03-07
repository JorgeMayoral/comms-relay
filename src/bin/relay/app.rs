use std::sync::Arc;

use anyhow::{Context, Result};
use axum::{
    Json, Router,
    extract::{FromRequestParts, Path, State},
    http::{StatusCode, header, request::Parts},
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
    pub api_token: String,
}

impl AppState {
    pub async fn new(db_url: &str, api_token: String) -> Result<Self> {
        let storage = PgStorage::create(db_url)
            .await
            .context("create postgres storage")?;
        Ok(Self { storage, api_token })
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
    _auth: BearerAuth,
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
        tracing::error!(error = %format_args!("{:#}", self.0), "internal server error");
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}

impl<E: Into<anyhow::Error>> From<E> for AppError {
    fn from(err: E) -> Self {
        AppError(err.into())
    }
}

struct BearerAuth;

impl FromRequestParts<Arc<AppState>> for BearerAuth {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        let unauthorized = || {
            (
                StatusCode::UNAUTHORIZED,
                [(header::WWW_AUTHENTICATE, "Bearer")],
            )
                .into_response()
        };

        let Some(value) = parts.headers.get(header::AUTHORIZATION) else {
            tracing::warn!("missing Authorization header");
            return Err(unauthorized());
        };
        let Ok(value) = value.to_str() else {
            tracing::warn!("Authorization header is not valid UTF-8");
            return Err(unauthorized());
        };
        let Some(token) = value.strip_prefix("Bearer ") else {
            tracing::warn!("Authorization header is not a Bearer token");
            return Err(unauthorized());
        };
        if token != state.api_token {
            tracing::warn!("invalid bearer token");
            return Err(unauthorized());
        }
        tracing::debug!("bearer auth ok");
        Ok(BearerAuth)
    }
}
