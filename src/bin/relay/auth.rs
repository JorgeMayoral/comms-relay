use std::sync::Arc;

use axum::{
    extract::FromRequestParts,
    http::{StatusCode, header, request::Parts},
    response::IntoResponse,
};

use crate::app::AppState;

pub(crate) struct BearerAuth;

impl FromRequestParts<Arc<AppState>> for BearerAuth {
    type Rejection = axum::response::Response;

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
