use std::{collections::HashMap, sync::Arc};

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use comms::{payloads::PublicationRequest, publication::Publication};
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
    telemetry::init_tracing();
    let tracing_layer = telemetry::get_tracing_layer();

    Router::new()
        .route("/publications", get(list_publications))
        .route("/publications", post(post_publication))
        .route("/publications/{id}", get(get_publication))
        .with_state(Arc::new(state))
        .layer(tracing_layer)
}

async fn list_publications(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let publications = state.storage.lock().await.list();
    (StatusCode::OK, Json(publications))
}

async fn post_publication(
    State(state): State<Arc<AppState>>,
    Json(new_publication): Json<PublicationRequest>,
) -> impl IntoResponse {
    let new_publication = Publication::from(new_publication);
    state.storage.lock().await.add(new_publication.clone());
    (StatusCode::CREATED, Json(new_publication))
}

async fn get_publication(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Ulid>,
) -> impl IntoResponse {
    match state.storage.lock().await.get(id) {
        Some(publication) => (StatusCode::OK, Json(publication)).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}
