use std::{collections::HashMap, sync::Arc};

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
};
use comms::{
    payloads::{
        GetAllPublicationsResponse, GetPublicationResponse, NewPublicationRequest,
        NewPublicationResponse,
    },
    publication::Publication,
};
use tokio::sync::Mutex;
use ulid::Ulid;

use crate::{storage::PublicationStorage, telemetry};

pub struct AppState {
    pub storage: Mutex<Box<dyn PublicationStorage + Send + Sync>>,
}

impl Default for AppState {
    fn default() -> Self {
        let map = HashMap::new();
        let boxed_map = Box::new(map);
        Self {
            storage: Mutex::new(boxed_map),
        }
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
) -> (StatusCode, Json<GetAllPublicationsResponse>) {
    let publications = state.storage.lock().await.list();
    (
        StatusCode::OK,
        Json(GetAllPublicationsResponse::from(publications)),
    )
}

async fn post_publication(
    State(state): State<Arc<AppState>>,
    Json(new_publication): Json<NewPublicationRequest>,
) -> (StatusCode, Json<NewPublicationResponse>) {
    let new_publication: Publication = new_publication.into();
    state.storage.lock().await.add(new_publication.clone());
    (
        StatusCode::CREATED,
        Json(NewPublicationResponse::from(new_publication)),
    )
}

async fn get_publication(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Ulid>,
) -> (StatusCode, Json<Option<GetPublicationResponse>>) {
    match state.storage.lock().await.get(id) {
        Some(publication) => (
            StatusCode::OK,
            Json(Some(GetPublicationResponse::from(publication.to_owned()))),
        ),
        None => (StatusCode::NOT_FOUND, Json(None)),
    }
}
