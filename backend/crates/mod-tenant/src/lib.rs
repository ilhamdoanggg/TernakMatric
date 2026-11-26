use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::Argon2;
use axum::extract::{Path, Query, State};
use axum::routing::{get, patch, post};
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use infra_auth::{ApiError, AppBackendState, AuthPrincipal, AuthenticatedUser, Permission};
use infra_db::{map_sqlx_err, with_organization, with_user, AuditLogRow, AuditLogger};
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
    pub created_by: Option<Uuid>,
    pub updated_at: DateTime<Utc>,
    pub updated_by: Option<Uuid>,
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

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateFarmRequest {
    pub name: Option<String>,
    pub address: Option<String>,
    pub latitude: Option<Decimal>,
    pub longitude: Option<Decimal>,
    pub status: Option<String>,
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
    pub created_by: Option<Uuid>,
    pub updated_at: DateTime<Utc>,
    pub updated_by: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct AuditLogQuery {
    pub entity_type: Option<String>,
    pub entity_id: Option<Uuid>,
    pub limit: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AuditLogDto {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub actor_user_id: Option<Uuid>,
    pub actor_email: Option<String>,
    pub actor_display_name: Option<String>,
    pub action: String,
    pub entity_type: String,
    pub entity_id: Option<Uuid>,
    pub before_data: Option<serde_json::Value>,
    pub after_data: Option<serde_json::Value>,
    pub occurred_at: DateTime<Utc>,
}

impl From<AuditLogRow> for AuditLogDto {
    fn from(r: AuditLogRow) -> Self {
        Self {
            id: r.id,
            organization_id: r.organization_id,
            actor_user_id: r.actor_user_id,
            actor_email: r.actor_email,
            actor_display_name: r.actor_display_name,
            action: r.action,
            entity_type: r.entity_type,
            entity_id: r.entity_id,
            before_data: r.before_data,
            after_data: r.after_data,
            occurred_at: r.occurred_at,
        }
    }
}

fn hash_password_sync(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| AppError::Internal(format!("password hashing failed: {e}")))?;
    Ok(hash.to_string())
}

fn verify_password_sync(password: &str, stored_hash: &str) -> bool {
    let Ok(parsed_hash) = PasswordHash::new(stored_hash) else {
        return false;
    };
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
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
    let email = req.email.trim().to_lowercase();
    let display_name = req.display_name.trim().to_string();
    if email.is_empty()
        || display_name.is_empty()
        || req.password.len() < 8
        || req.password.len() > 1024
    {
        return Err(ApiError(AppError::Validation(
            "valid email, display_name, and password (8..=1024 chars) required".into(),
        )));
    }

    let plain_password = req.password;
    let password_hash = tokio::task::spawn_blocking(move || hash_password_sync(&plain_password))
        .await
        .map_err(|e| ApiError(AppError::Internal(format!("password task failed: {e}"))))?
        .map_err(ApiError)?;

    let user_id: Uuid = sqlx::query_scalar(
        "INSERT INTO users (email, display_name, password_hash) VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(&email)
    .bind(&display_name)
    .bind(password_hash)
    .fetch_one(state.pool())
    .await
    .map_err(|e| ApiError(map_sqlx_err(e)))?;

    let token = state
        .jwt()
        .issue_token(UserId(user_id), &email)
        .map_err(ApiError)?;
    Ok((
        axum::http::StatusCode::CREATED,
        Json(AuthTokenResponse {
            access_token: token,
            user_id,
            email,
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
    let email_input = req.email.trim().to_lowercase();
    if email_input.is_empty() || req.password.is_empty() || req.password.len() > 1024 {
        return Err(ApiError(AppError::Unauthorized(
            "invalid credentials".into(),
        )));
    }

    let row: Option<(Uuid, String, Option<String>)> = sqlx::query_as(
        "SELECT id, email, password_hash FROM users WHERE email = $1 AND status = 'active'",
    )
    .bind(&email_input)
    .fetch_optional(state.pool())
    .await
    .map_err(|e| ApiError(map_sqlx_err(e)))?;

    let Some((user_id, email, Some(stored_hash))) = row else {
        return Err(ApiError(AppError::Unauthorized(
            "invalid credentials".into(),
        )));
    };

    let plain_password = req.password;
    let valid =
        tokio::task::spawn_blocking(move || verify_password_sync(&plain_password, &stored_hash))
            .await
            .map_err(|e| {
                ApiError(AppError::Internal(format!(
                    "password verify task failed: {e}"
                )))
            })?;
    if !valid {
        return Err(ApiError(AppError::Unauthorized(
            "invalid credentials".into(),
        )));
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
    let mut tx = state
        .pool()
        .begin()
        .await
        .map_err(|e| ApiError(map_sqlx_err(e)))?;
    with_user(&mut *tx, user.user_id)
        .await
        .map_err(|e| ApiError(map_sqlx_err(e)))?;

    let rows: Vec<OrganizationDto> = sqlx::query_as(
        r#"
        SELECT o.id, o.name, o.slug, o.timezone, o.default_currency::text AS default_currency,
               o.status, o.created_at, o.created_by, o.updated_at, o.updated_by
        FROM organizations o
        JOIN organization_members m ON m.organization_id = o.id
        WHERE m.user_id = $1 AND m.status = 'active'
        ORDER BY o.created_at DESC
        "#,
    )
    .bind(user.user_id.0)
    .fetch_all(&mut *tx)
    .await
    .map_err(|e| ApiError(map_sqlx_err(e)))?;
    tx.commit().await.map_err(|e| ApiError(map_sqlx_err(e)))?;

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
    let mut tx = state
        .pool()
        .begin()
        .await
        .map_err(|e| ApiError(map_sqlx_err(e)))?;
    with_user(&mut *tx, user.user_id)
        .await
        .map_err(|e| ApiError(map_sqlx_err(e)))?;

    let org: OrganizationDto = sqlx::query_as(
        r#"
        INSERT INTO organizations (name, slug, timezone, default_currency, created_by)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, name, slug, timezone, default_currency::text AS default_currency,
                  status, created_at, created_by, updated_at, updated_by
        "#,
    )
    .bind(req.name.trim())
    .bind(req.slug.trim().to_lowercase())
    .bind(req.timezone.as_deref().unwrap_or("Asia/Jakarta"))
    .bind(req.default_currency.as_deref().unwrap_or("IDR"))
    .bind(user.user_id.0)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| ApiError(map_sqlx_err(e)))?;

    with_organization(&mut *tx, OrganizationId(org.id))
        .await
        .map_err(|e| ApiError(map_sqlx_err(e)))?;

    let member_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO organization_members (organization_id, user_id, status, joined_at, created_by)
        VALUES ($1, $2, 'active', now(), $2)
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
        OrganizationId(org.id),
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
    let mut tx = auth
        .context()
        .begin_tx(state.pool())
        .await
        .map_err(ApiError)?;
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

    let mut tx = auth
        .context()
        .begin_tx(state.pool())
        .await
        .map_err(ApiError)?;
    let target_user: (Uuid, String, String) =
        sqlx::query_as("SELECT id, email, display_name FROM users WHERE email = $1")
            .bind(req.user_email.trim().to_lowercase())
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| ApiError(map_sqlx_err(e)))?;

    let member_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO organization_members (organization_id, user_id, status, invited_by, joined_at, created_by)
        VALUES ($1, $2, 'active', $3, now(), $3)
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

    AuditLogger::record(
        &mut tx,
        auth.organization_id,
        Some(auth.user_id),
        "member.added",
        "organization_members",
        Some(member_id),
        None,
        Some(serde_json::json!({ "email": target_user.1, "role_code": req.role_code })),
    )
    .await
    .map_err(ApiError)?;

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
    let mut tx = auth
        .context()
        .begin_tx(state.pool())
        .await
        .map_err(ApiError)?;
    let farms: Vec<FarmDto> = sqlx::query_as(
        r#"
        SELECT id, organization_id, code, name, address, latitude, longitude, status, version,
               created_at, created_by, updated_at, updated_by
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
    let mut tx = auth
        .context()
        .begin_tx(state.pool())
        .await
        .map_err(ApiError)?;
    let farm: FarmDto = sqlx::query_as(
        r#"
        INSERT INTO farms (organization_id, code, name, address, latitude, longitude, created_by)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id, organization_id, code, name, address, latitude, longitude, status, version,
                  created_at, created_by, updated_at, updated_by
        "#,
    )
    .bind(auth.organization_id.0)
    .bind(req.code.trim())
    .bind(req.name.trim())
    .bind(&req.address)
    .bind(req.latitude)
    .bind(req.longitude)
    .bind(auth.user_id.0)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| ApiError(map_sqlx_err(e)))?;

    AuditLogger::record(
        &mut tx,
        auth.organization_id,
        Some(auth.user_id),
        "farm.created",
        "farms",
        Some(farm.id),
        None,
        Some(serde_json::to_value(&farm).unwrap_or_default()),
    )
    .await
    .map_err(ApiError)?;

    tx.commit().await.map_err(|e| ApiError(map_sqlx_err(e)))?;
    Ok((axum::http::StatusCode::CREATED, Json(farm)))
}

#[utoipa::path(
    patch,
    path = "/api/v1/farms/{id}",
    params(("id" = Uuid, Path, description = "Farm ID")),
    request_body = UpdateFarmRequest,
    responses((status = 200, body = FarmDto)),
    tag = "Tenant & Auth"
)]
pub async fn update_farm<S: AppBackendState>(
    State(state): State<S>,
    auth: AuthPrincipal,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateFarmRequest>,
) -> Result<Json<FarmDto>, ApiError> {
    auth.require(Permission::FarmManage).map_err(ApiError)?;
    let mut tx = auth
        .context()
        .begin_tx(state.pool())
        .await
        .map_err(ApiError)?;

    let before: FarmDto = sqlx::query_as(
        r#"
        SELECT id, organization_id, code, name, address, latitude, longitude, status, version,
               created_at, created_by, updated_at, updated_by
        FROM farms WHERE organization_id = $1 AND id = $2 FOR UPDATE
        "#,
    )
    .bind(auth.organization_id.0)
    .bind(id)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| ApiError(map_sqlx_err(e)))?;

    let after: FarmDto = sqlx::query_as(
        r#"
        UPDATE farms
        SET name = COALESCE($3, name),
            address = COALESCE($4, address),
            latitude = COALESCE($5, latitude),
            longitude = COALESCE($6, longitude),
            status = COALESCE($7, status),
            version = version + 1,
            updated_by = $8
        WHERE organization_id = $1 AND id = $2
        RETURNING id, organization_id, code, name, address, latitude, longitude, status, version,
                  created_at, created_by, updated_at, updated_by
        "#,
    )
    .bind(auth.organization_id.0)
    .bind(id)
    .bind(req.name.as_deref().map(str::trim))
    .bind(&req.address)
    .bind(req.latitude)
    .bind(req.longitude)
    .bind(&req.status)
    .bind(auth.user_id.0)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| ApiError(map_sqlx_err(e)))?;

    AuditLogger::record(
        &mut tx,
        auth.organization_id,
        Some(auth.user_id),
        "farm.updated",
        "farms",
        Some(id),
        Some(serde_json::to_value(&before).unwrap_or_default()),
        Some(serde_json::to_value(&after).unwrap_or_default()),
    )
    .await
    .map_err(ApiError)?;

    tx.commit().await.map_err(|e| ApiError(map_sqlx_err(e)))?;
    Ok(Json(after))
}

#[utoipa::path(
    get,
    path = "/api/v1/audit-logs",
    responses((status = 200, body = [AuditLogDto])),
    tag = "Tenant & Auth"
)]
pub async fn list_audit_logs<S: AppBackendState>(
    State(state): State<S>,
    auth: AuthPrincipal,
    Query(q): Query<AuditLogQuery>,
) -> Result<Json<Vec<AuditLogDto>>, ApiError> {
    auth.require(Permission::AnalyticsRead).map_err(ApiError)?;
    let mut tx = auth
        .context()
        .begin_tx(state.pool())
        .await
        .map_err(ApiError)?;
    let rows = AuditLogger::list_for_org(
        &mut tx,
        auth.organization_id,
        q.entity_type.as_deref(),
        q.entity_id,
        q.limit.unwrap_or(50),
    )
    .await
    .map_err(ApiError)?;
    tx.commit().await.map_err(|e| ApiError(map_sqlx_err(e)))?;
    Ok(Json(rows.into_iter().map(AuditLogDto::from).collect()))
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
        .route("/api/v1/farms/{id}", patch(update_farm::<S>))
        .route("/api/v1/audit-logs", get(list_audit_logs::<S>))
}

#[derive(OpenApi)]
#[openapi(
    paths(
        register,
        login,
        list_organizations,
        create_organization,
        list_members,
        add_member,
        list_farms,
        create_farm,
        update_farm,
        list_audit_logs
    ),
    components(schemas(
        RegisterRequest,
        LoginRequest,
        AuthTokenResponse,
        CreateOrganizationRequest,
        OrganizationDto,
        InviteMemberRequest,
        MemberDto,
        CreateFarmRequest,
        UpdateFarmRequest,
        FarmDto,
        AuditLogDto
    ))
)]
pub struct TenantApi;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_argon2id_hash_and_verify_roundtrip() {
        let hash1 = hash_password_sync("RahasiaTernak!2026").unwrap();
        let hash2 = hash_password_sync("RahasiaTernak!2026").unwrap();

        assert!(hash1.starts_with("$argon2id$"));
        assert_ne!(hash1, hash2, "random salt should produce unique hashes");
        assert!(verify_password_sync("RahasiaTernak!2026", &hash1));
        assert!(verify_password_sync("RahasiaTernak!2026", &hash2));
        assert!(!verify_password_sync("SalahPassword!2026", &hash1));
    }

    #[test]
    fn test_legacy_placeholder_and_malformed_hashes_rejected() {
        assert!(!verify_password_sync(
            "secret123",
            "plain_placeholder:secret123"
        ));
        assert!(!verify_password_sync("secret123", ""));
        assert!(!verify_password_sync("secret123", "not-a-valid-phc-hash"));
    }
}
