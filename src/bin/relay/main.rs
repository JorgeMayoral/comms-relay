use std::net::SocketAddr;

use anyhow::{Context, Result};

use crate::app::AppState;

mod app;
mod auth;
mod error;
mod mastodon;
mod storage;
mod telemetry;

#[tokio::main]
async fn main() -> Result<()> {
    telemetry::init_tracing();

    let db_url = std::env::var("DATABASE_URL").context("get DATABASE_URL env variable")?;
    let api_token = std::env::var("RELAY_API_TOKEN").context("get RELAY_API_TOKEN env variable")?;
    let mastodon_access_token =
        std::env::var("MASTODON_ACCESS_TOKEN").context("get MASTODON_ACCESS_TOKEN env variable")?;
    let mastodon_instance_url = std::env::var("MASTODON_INSTANCE_URL")
        .context("get MASTODON_INSTANCE_URL env variable")?;

    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .context("create TCP listener")?;
    let state = AppState::new(&db_url, api_token, mastodon_access_token, mastodon_instance_url)
        .await
        .context("create app state")?;
    let app = app::app(state);
    tracing::info!("Server listening on http://{addr}");
    axum::serve(listener, app).await.context("serve axum app")?;

    Ok(())
}
