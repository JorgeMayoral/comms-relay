use std::net::SocketAddr;

use anyhow::{Context, Result};

use crate::app::AppState;

mod app;
mod storage;

#[tokio::main]
async fn main() -> Result<()> {
    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .context("create TCP listener")?;
    let state = AppState::default();
    let app = app::app(state);
    axum::serve(listener, app).await.context("serve axum app")?;

    Ok(())
}
