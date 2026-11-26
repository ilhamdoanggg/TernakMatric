use crate::error::ApiError;
use crate::jwt::JwtService;
use crate::rbac::{verify_membership_and_permissions, AuthPrincipal};
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use shared_kernel::{AppError, OrganizationId, UserId};
use sqlx::PgPool;
use uuid::Uuid;

pub trait AppBackendState: Clone + Send + Sync + 'static {
    fn pool(&self) -> &PgPool;
    fn jwt(&self) -> &JwtService;
}

#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user_id: UserId,
    pub email: String,
}

fn extract_bearer(parts: &Parts) -> Result<&str, ApiError> {
    let header = parts
        .headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| {
            ApiError(AppError::Unauthorized(
                "missing Authorization header".into(),
            ))
        })?;
    header
        .strip_prefix("Bearer ")
        .ok_or_else(|| ApiError(AppError::Unauthorized("expected Bearer token".into())))
}

impl<S: AppBackendState> FromRequestParts<S> for AuthenticatedUser {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let token = extract_bearer(parts)?;
        let claims = state.jwt().verify_token(token).map_err(ApiError)?;
        Ok(Self {
            user_id: UserId(claims.sub),
            email: claims.email,
        })
    }
}

impl<S: AppBackendState> FromRequestParts<S> for AuthPrincipal {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let user = AuthenticatedUser::from_request_parts(parts, state).await?;
        let org_raw = parts
            .headers
            .get("x-organization-id")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| {
                ApiError(AppError::Unauthorized(
                    "missing X-Organization-Id header".into(),
                ))
            })?;
        let org_uuid = Uuid::parse_str(org_raw).map_err(|_| {
            ApiError(AppError::Validation(
                "invalid X-Organization-Id UUID".into(),
            ))
        })?;

        verify_membership_and_permissions(state.pool(), user.user_id, OrganizationId(org_uuid))
            .await
            .map_err(ApiError)
    }
}
