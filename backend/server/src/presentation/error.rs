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

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;
    use axum::response::IntoResponse;

    use crate::domain::DomainError;

    use super::ApiError;

    fn status(err: DomainError) -> StatusCode {
        ApiError(err).into_response().status()
    }

    #[test]
    fn maps_domain_errors_to_http_status() {
        assert_eq!(status(DomainError::Forbidden), StatusCode::FORBIDDEN);
        assert_eq!(status(DomainError::NotFound), StatusCode::NOT_FOUND);
        assert_eq!(
            status(DomainError::Conflict("dup".into())),
            StatusCode::CONFLICT
        );
        assert_eq!(
            status(DomainError::InvalidInput("bad".into())),
            StatusCode::UNPROCESSABLE_ENTITY
        );
        assert_eq!(status(DomainError::Internal), StatusCode::INTERNAL_SERVER_ERROR);
    }

}
