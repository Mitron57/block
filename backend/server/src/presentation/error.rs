use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

use crate::domain::DomainError;

#[derive(Debug)]
pub struct ApiError(pub DomainError);

impl From<DomainError> for ApiError {
    fn from(value: DomainError) -> Self {
        ApiError(value)
    }
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match &self.0 {
            DomainError::Forbidden => StatusCode::FORBIDDEN,
            DomainError::NotFound => StatusCode::NOT_FOUND,
            DomainError::Conflict(_) => StatusCode::CONFLICT,
            DomainError::InvalidInput(_) => StatusCode::UNPROCESSABLE_ENTITY,
            DomainError::Internal => StatusCode::INTERNAL_SERVER_ERROR,
        };
        let msg = self.0.to_string();
        (status, Json(ErrorBody { error: msg })).into_response()
    }
}
