use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use shared_kernel::error::ErrorEnvelope;
use shared_kernel::AppError;

#[derive(Debug)]
pub struct ApiError(pub AppError);

impl From<AppError> for ApiError {
    fn from(value: AppError) -> Self {
        Self(value)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = StatusCode::from_u16(self.0.status_code_u16())
            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        let details = match &self.0 {
            AppError::CapacityExceeded { details, .. } => Some(details.clone()),
            _ => None,
        };
        let body = ErrorEnvelope {
            code: self.0.code().to_string(),
            message: self.0.to_string(),
            details,
        };
        (status, Json(body)).into_response()
    }
}
