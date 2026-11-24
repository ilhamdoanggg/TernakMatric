use serde::Serialize;
use serde_json::Value;
use thiserror::Error;
use utoipa::ToSchema;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("validation error: {0}")]
    Validation(String),

    #[error("authentication required: {0}")]
    Unauthorized(String),

    #[error("forbidden: {0}")]
    Forbidden(String),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("conflict: {0}")]
    Conflict(String),

    #[error("capacity exceeded: {message}")]
    CapacityExceeded { message: String, details: Value },

    #[error("insufficient balance: {0}")]
    InsufficientBalance(String),

    #[error("database error: {0}")]
    Database(String),

    #[error("internal error: {0}")]
    Internal(String),
}

impl AppError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Validation(_) => "VALIDATION_ERROR",
            Self::Unauthorized(_) => "UNAUTHORIZED",
            Self::Forbidden(_) => "FORBIDDEN",
            Self::NotFound(_) => "NOT_FOUND",
            Self::Conflict(_) => "CONFLICT",
            Self::CapacityExceeded { .. } => "CAPACITY_EXCEEDED",
            Self::InsufficientBalance(_) => "INSUFFICIENT_BALANCE",
            Self::Database(_) => "DATABASE_ERROR",
            Self::Internal(_) => "INTERNAL_ERROR",
        }
    }

    pub fn status_code_u16(&self) -> u16 {
        match self {
            Self::Validation(_) => 400,
            Self::Unauthorized(_) => 401,
            Self::Forbidden(_) => 403,
            Self::NotFound(_) => 404,
            Self::Conflict(_) => 409,
            Self::CapacityExceeded { .. } => 422,
            Self::InsufficientBalance(_) => 422,
            Self::Database(_) | Self::Internal(_) => 500,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorEnvelope {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Value>,
}
