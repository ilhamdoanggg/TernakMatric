use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use infra_auth::{ApiError, AppBackendState, AuthPrincipal, AuthenticatedUser, Permission};
use infra_db::{map_sqlx_err, AuditLogger};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use shared_kernel::error::ErrorEnvelope;
use shared_kernel::{AppError, OrganizationId, UserId};
use utoipa::{OpenApi, ToSchema};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct RegisterRequest {
    pub email: String,
    pub display_name: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AuthTokenResponse {
    pub access_token: String,
    pub user_id: Uuid,
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateOrganizationRequest {
    pub name: String,
    pub slug: String,
    pub timezone: Option<String>,
    pub default_currency: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct OrganizationDto {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub timezone: String,
    pub default_currency: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct InviteMemberRequest {
    pub user_email: String,
    pub role_code: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct MemberDto {
    pub id: Uuid,
    pub user_id: Uuid,
    pub email: String,
    pub display_name: String,
    pub status: String,
    pub role_codes: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateFarmRequest {
    pub code: String,
    pub name: String,
    pub address: Option<String>,
    pub latitude: Option<Decimal>,
    pub longitude: Option<Decimal>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct FarmDto {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub address: Option<String>,
    pub latitude: Option<Decimal>,
    pub longitude: Option<Decimal>,
    pub status: String,
    pub version: i32,
    pub created_at: DateTime<Utc>,
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/register",
    request_body = RegisterRequest,
    responses((status = 201, body = AuthTokenResponse), (status = 409, body = ErrorEnvelope)),
    tag = "Tenant & Auth"
)]
pub async fn register<S: AppBackendState>(
    State(state): State<S>,
    Json(req): Json<RegisterRequest>,
) -> Result<(axum::http::StatusCode, Json<AuthTokenResponse>), ApiError> {
    if req.email.trim().is_empty() || req.password.len() < 8 {
        return Err(ApiError(AppError::Validation(
            "valid email and password (>= 8 chars) required".into(),
        )));
    }
    let password_hash = format!("plain_placeholder:{}", req.password);
    let user_id: Uuid = sqlx::query_scalar(
        "INSERT INTO users (email, display_name, password_hash) VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(req.email.trim().to_lowercase())
    .bind(req.display_name.trim())
    .bind(password_hash)
    .fetch_one(state.pool())
    .await
    .map_err(|e| ApiError(map_sqlx_err(e)))?;

    let token = state
        .jwt()
        .issue_token(UserId(user_id), &req.email)
        .map_err(ApiError)?;
    Ok((
        axum::http::StatusCode::CREATED,
        Json(AuthTokenResponse {
            access_token: token,
            user_id,
            email: req.email,
        }),
    ))
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/login",
    request_body = LoginRequest,
    responses((status = 200, body = AuthTokenResponse), (status = 401, body = ErrorEnvelope)),
    tag = "Tenant & Auth"
)]
pub async fn login<S: AppBackendState>(
    State(state): State<S>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<AuthTokenResponse>, ApiError> {
    let row: Option<(Uuid, String, Option<String>)> = sqlx::query_as(
        "SELECT id, email, password_hash FROM users WHERE email = $1 AND status = 'active'",
    )
    .bind(req.email.trim().to_lowercase())
    .fetch_optional(state.pool())
    .await
    .map_err(|e| ApiError(map_sqlx_err(e)))?;

    let Some((user_id, email, hash)) = row else {
        return Err(ApiError(AppError::Unauthorized("invalid credentials".into())));
    };
    let expected = format!("plain_placeholder:{}", req.password);
    if hash.as_deref() != Some(expected.as_str()) {
        return Err(ApiError(AppError::Unauthorized("invalid credentials".into())));
    }

    let token = state
        .jwt()
        .issue_token(UserId(user_id), &email)
        .map_err(ApiError)?;
    Ok(Json(AuthTokenResponse {
        access_token: token,
        user_id,
        email,
    }))
}

#[utoipa::path(
    get,
    path = "/api/v1/organizations",
    responses((status = 200, body = [OrganizationDto])),
    tag = "Tenant & Auth"
)]
pub async fn list_organizations<S: AppBackendState>(
    State(state): State<S>,
    user: AuthenticatedUser,
) -> Result<Json<Vec<OrganizationDto>>, ApiError> {
    let rows: Vec<OrganizationDto> = sqlx::query_as(
        r#"
        SELECT o.id, o.name, o.slug, o.timezone, o.default_currency::text AS default_currency, o.status, o.created_at
        FROM organizations o
        JOIN organization_members m ON m.organization_id = o.id
        WHERE m.user_id = $1 AND m.status = 'active'
        ORDER BY o.created_at DESC
        "#,
    )
    .bind(user.user_id.0)
    .fetch_all(state.pool())
    .await
    .map_err(|e| ApiError(map_sqlx_err(e)))?;

    Ok(Json(rows))
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations",
    request_body = CreateOrganizationRequest,
    responses((status = 201, body = OrganizationDto)),
    tag = "Tenant & Auth"
)]
pub async fn create_organization<S: AppBackendState>(
    State(state): State<S>,
    user: AuthenticatedUser,
    Json(req): Json<CreateOrganizationRequest>,
) -> Result<(axum::http::StatusCode, Json<OrganizationDto>), ApiError> {
    let mut tx = state.pool().begin().await.map_err(|e| ApiError(map_sqlx_err(e)))?;
    let org: OrganizationDto = sqlx::query_as(
        r#"
        INSERT INTO organizations (name, slug, timezone, default_currency)
        VALUES ($1, $2, $3, $4)
        RETURNING id, name, slug, timezone, default_currency::text AS default_currency, status, created_at
        "#,
    )
    .bind(req.name.trim())
    .bind(req.slug.trim().to_lowercase())
    .bind(req.timezone.as_deref().unwrap_or("Asia/Jakarta"))
    .bind(req.default_currency.as_deref().unwrap_or("IDR"))
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| ApiError(map_sqlx_err(e)))?;

    infra_db::context::with_organization(&mut *tx, OrganizationId(org.id))
        .await
        .map_err(|e| ApiError(map_sqlx_err(e)))?;

    let member_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO organization_members (organization_id, user_id, status, joined_at)
        VALUES ($1, $2, 'active', now())
        RETURNING id
        "#,
    )
    .bind(org.id)
    .bind(user.user_id.0)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| ApiError(map_sqlx_err(e)))?;

    sqlx::query(
        r#"
        INSERT INTO member_roles (organization_id, member_id, role_id)
        SELECT $1, $2, id FROM roles WHERE organization_id IS NULL AND code = 'ORG_OWNER'
        "#,
    )
    .bind(org.id)
    .bind(member_id)
    .execute(&mut *tx)
    .await
    .map_err(|e| ApiError(map_sqlx_err(e)))?;

    AuditLogger::record(
        &mut tx,
        Some(OrganizationId(org.id)),
        Some(user.user_id),
        "organization.created",
        "organizations",
        Some(org.id),
        None,
        Some(serde_json::json!({ "name": org.name, "slug": org.slug })),
    )
    .await
    .map_err(ApiError)?;

    tx.commit().await.map_err(|e| ApiError(map_sqlx_err(e)))?;
    Ok((axum::http::StatusCode::CREATED, Json(org)))
}

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{id}/members",
    params(("id" = Uuid, Path, description = "Organization ID")),
    responses((status = 200, body = [MemberDto])),
    tag = "Tenant & Auth"
)]
pub async fn list_members<S: AppBackendState>(
    State(state): State<S>,
    auth: AuthPrincipal,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<MemberDto>>, ApiError> {
    if id != auth.organization_id.0 {
        return Err(ApiError(AppError::Forbidden(
            "organization path mismatch".into(),
        )));
    }
    let mut tx = auth.context().begin_tx(state.pool()).await.map_err(ApiError)?;
    let rows: Vec<MemberDto> = sqlx::query_as(
        r#"
        SELECT m.id, u.id AS user_id, u.email, u.display_name, m.status,
               COALESCE(array_agg(r.code) FILTER (WHERE r.code IS NOT NULL), '{}') AS role_codes
        FROM organization_members m
        JOIN users u ON u.id = m.user_id
        LEFT JOIN member_roles mr ON mr.member_id = m.id AND mr.organization_id = m.organization_id
        LEFT JOIN roles r ON r.id = mr.role_id
        WHERE m.organization_id = $1
        GROUP BY m.id, u.id, u.email, u.display_name, m.status
        ORDER BY m.created_at ASC
        "#,
    )
    .bind(auth.organization_id.0)
    .fetch_all(&mut *tx)
    .await
    .map_err(|e| ApiError(map_sqlx_err(e)))?;
    tx.commit().await.map_err(|e| ApiError(map_sqlx_err(e)))?;
    Ok(Json(rows))
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{id}/members",
    params(("id" = Uuid, Path, description = "Organization ID")),
    request_body = InviteMemberRequest,
    responses((status = 201, body = MemberDto)),
    tag = "Tenant & Auth"
)]
pub async fn add_member<S: AppBackendState>(
    State(state): State<S>,
    auth: AuthPrincipal,
    Path(id): Path<Uuid>,
    Json(req): Json<InviteMemberRequest>,
) -> Result<(axum::http::StatusCode, Json<MemberDto>), ApiError> {
    auth.require(Permission::OrgManage).map_err(ApiError)?;
    if id != auth.organization_id.0 {
        return Err(ApiError(AppError::Forbidden(
            "organization path mismatch".into(),
        )));
    }

    let mut tx = auth.context().begin_tx(state.pool()).await.map_err(ApiError)?;
    let target_user: (Uuid, String, String) = sqlx::query_as(
        "SELECT id, email, display_name FROM users WHERE email = $1",
    )
    .bind(req.user_email.trim().to_lowercase())
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| ApiError(map_sqlx_err(e)))?;

    let member_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO organization_members (organization_id, user_id, status, invited_by, joined_at)
        VALUES ($1, $2, 'active', $3, now())
        RETURNING id
        "#,
    )
    .bind(auth.organization_id.0)
    .bind(target_user.0)
    .bind(auth.user_id.0)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| ApiError(map_sqlx_err(e)))?;

    sqlx::query(
        r#"
        INSERT INTO member_roles (organization_id, member_id, role_id)
        SELECT $1, $2, id FROM roles
        WHERE (organization_id IS NULL OR organization_id = $1) AND code = $3
        "#,
    )
    .bind(auth.organization_id.0)
    .bind(member_id)
    .bind(&req.role_code)
    .execute(&mut *tx)
    .await
    .map_err(|e| ApiError(map_sqlx_err(e)))?;

    tx.commit().await.map_err(|e| ApiError(map_sqlx_err(e)))?;

    Ok((
        axum::http::StatusCode::CREATED,
        Json(MemberDto {
            id: member_id,
            user_id: target_user.0,
            email: target_user.1,
            display_name: target_user.2,
            status: "active".into(),
            role_codes: vec![req.role_code],
        }),
    ))
}

#[utoipa::path(
    get,
    path = "/api/v1/farms",
    responses((status = 200, body = [FarmDto])),
    tag = "Tenant & Auth"
)]
pub async fn list_farms<S: AppBackendState>(
    State(state): State<S>,
    auth: AuthPrincipal,
) -> Result<Json<Vec<FarmDto>>, ApiError> {
    let mut tx = auth.context().begin_tx(state.pool()).await.map_err(ApiError)?;
    let farms: Vec<FarmDto> = sqlx::query_as(
        r#"
        SELECT id, organization_id, code, name, address, latitude, longitude, status, version, created_at
        FROM farms
        WHERE organization_id = $1
        ORDER BY created_at DESC
        "#,
    )
    .bind(auth.organization_id.0)
    .fetch_all(&mut *tx)
    .await
    .map_err(|e| ApiError(map_sqlx_err(e)))?;
    tx.commit().await.map_err(|e| ApiError(map_sqlx_err(e)))?;
    Ok(Json(farms))
}

#[utoipa::path(
    post,
    path = "/api/v1/farms",
    request_body = CreateFarmRequest,
    responses((status = 201, body = FarmDto)),
    tag = "Tenant & Auth"
)]
pub async fn create_farm<S: AppBackendState>(
    State(state): State<S>,
    auth: AuthPrincipal,
    Json(req): Json<CreateFarmRequest>,
) -> Result<(axum::http::StatusCode, Json<FarmDto>), ApiError> {
    auth.require(Permission::FarmManage).map_err(ApiError)?;
    let mut tx = auth.context().begin_tx(state.pool()).await.map_err(ApiError)?;
    let farm: FarmDto = sqlx::query_as(
        r#"
        INSERT INTO farms (organization_id, code, name, address, latitude, longitude)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, organization_id, code, name, address, latitude, longitude, status, version, created_at
        "#,
    )
    .bind(auth.organization_id.0)
    .bind(req.code.trim())
    .bind(req.name.trim())
    .bind(&req.address)
    .bind(req.latitude)
    .bind(req.longitude)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| ApiError(map_sqlx_err(e)))?;
    tx.commit().await.map_err(|e| ApiError(map_sqlx_err(e)))?;
    Ok((axum::http::StatusCode::CREATED, Json(farm)))
}

pub fn router<S: AppBackendState>() -> Router<S> {
    Router::new()
        .route("/api/v1/auth/register", post(register::<S>))
        .route("/api/v1/auth/login", post(login::<S>))
        .route(
            "/api/v1/organizations",
            get(list_organizations::<S>).post(create_organization::<S>),
        )
        .route(
            "/api/v1/organizations/{id}/members",
            get(list_members::<S>).post(add_member::<S>),
        )
        .route("/api/v1/farms", get(list_farms::<S>).post(create_farm::<S>))
}

#[derive(OpenApi)]
#[openapi(
    paths(register, login, list_organizations, create_organization, list_members, add_member, list_farms, create_farm),
    components(schemas(
        RegisterRequest, LoginRequest, AuthTokenResponse,
        CreateOrganizationRequest, OrganizationDto, InviteMemberRequest, MemberDto,
        CreateFarmRequest, FarmDto
    ))
)]
pub struct TenantApi;
