use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

pub(crate) enum AppError {
    Internal(anyhow::Error),
    Unprocessable(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            Self::Internal(error) => {
                tracing::error!(error = %format_args!("{error:#}"), "internal server error");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
            Self::Unprocessable(error) => {
                tracing::error!(error = %format_args!("{error:#}"), "unprocessable entity error");
                StatusCode::UNPROCESSABLE_ENTITY.into_response()
            }
        }
    }
}

impl<E: Into<anyhow::Error>> From<E> for AppError {
    fn from(err: E) -> Self {
        AppError::Internal(err.into())
    }
}
