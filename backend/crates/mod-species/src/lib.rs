use axum::extract::{Path, Query, State};
use axum::routing::get;
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use infra_auth::{ApiError, AppBackendState, AuthPrincipal, Permission};
use infra_db::map_sqlx_err;
use serde::{Deserialize, Serialize};
use shared_kernel::SpeciesCategory;
use utoipa::{OpenApi, ToSchema};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct SpeciesFilter {
    pub category: Option<SpeciesCategory>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateSpeciesRequest {
    pub category: SpeciesCategory,
    pub common_name: String,
    pub scientific_name: Option<String>,
    pub code: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct SpeciesDto {
    pub id: Uuid,
    pub organization_id: Option<Uuid>,
    pub category: String,
    pub common_name: String,
    pub scientific_name: Option<String>,
    pub code: String,
    pub description: Option<String>,
    pub is_system: bool,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateProfileRequest {
    pub code: String,
    pub name: String,
    pub production_system_id: Option<Uuid>,
    pub parameters: serde_json::Value,
    pub version: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct SpeciesProfileDto {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub species_id: Uuid,
    pub code: String,
    pub name: String,
    pub production_system_id: Option<Uuid>,
    pub parameters: serde_json::Value,
    pub version: i32,
    pub is_active: bool,
    pub valid_from: DateTime<Utc>,
    pub valid_until: Option<DateTime<Utc>>,
}

#[utoipa::path(
    get,
    path = "/api/v1/species",
    responses((status = 200, body = [SpeciesDto])),
    tag = "Species"
)]
pub async fn list_species<S: AppBackendState>(
    State(state): State<S>,
    auth: AuthPrincipal,
    Query(filter): Query<SpeciesFilter>,
) -> Result<Json<Vec<SpeciesDto>>, ApiError> {
    let mut tx = auth.context().begin_tx(state.pool()).await.map_err(ApiError)?;
    let category_str = filter.category.map(|c| c.as_str().to_string());
    let rows: Vec<SpeciesDto> = sqlx::query_as(
        r#"
        SELECT id, organization_id, category, common_name, scientific_name, code, description, is_system, is_active, created_at
        FROM species
        WHERE (organization_id IS NULL OR organization_id = $1)
          AND ($2::text IS NULL OR category = $2)
          AND is_active = true
        ORDER BY is_system DESC, common_name ASC
        "#,
    )
    .bind(auth.organization_id.0)
    .bind(category_str)
    .fetch_all(&mut *tx)
    .await
    .map_err(|e| ApiError(map_sqlx_err(e)))?;
    tx.commit().await.map_err(|e| ApiError(map_sqlx_err(e)))?;
    Ok(Json(rows))
}

#[utoipa::path(
    post,
    path = "/api/v1/species",
    request_body = CreateSpeciesRequest,
    responses((status = 201, body = SpeciesDto)),
    tag = "Species"
)]
pub async fn create_custom_species<S: AppBackendState>(
    State(state): State<S>,
    auth: AuthPrincipal,
    Json(req): Json<CreateSpeciesRequest>,
) -> Result<(axum::http::StatusCode, Json<SpeciesDto>), ApiError> {
    auth.require(Permission::SpeciesManage).map_err(ApiError)?;
    let mut tx = auth.context().begin_tx(state.pool()).await.map_err(ApiError)?;
    let row: SpeciesDto = sqlx::query_as(
        r#"
        INSERT INTO species (organization_id, category, common_name, scientific_name, code, description, is_system)
        VALUES ($1, $2, $3, $4, $5, $6, false)
        RETURNING id, organization_id, category, common_name, scientific_name, code, description, is_system, is_active, created_at
        "#,
    )
    .bind(auth.organization_id.0)
    .bind(req.category.as_str())
    .bind(req.common_name.trim())
    .bind(&req.scientific_name)
    .bind(req.code.trim().to_uppercase())
    .bind(&req.description)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| ApiError(map_sqlx_err(e)))?;
    tx.commit().await.map_err(|e| ApiError(map_sqlx_err(e)))?;
    Ok((axum::http::StatusCode::CREATED, Json(row)))
}

#[utoipa::path(
    get,
    path = "/api/v1/species/{id}/profiles",
    params(("id" = Uuid, Path, description = "Species ID")),
    responses((status = 200, body = [SpeciesProfileDto])),
    tag = "Species"
)]
pub async fn list_profiles<S: AppBackendState>(
    State(state): State<S>,
    auth: AuthPrincipal,
    Path(species_id): Path<Uuid>,
) -> Result<Json<Vec<SpeciesProfileDto>>, ApiError> {
    let mut tx = auth.context().begin_tx(state.pool()).await.map_err(ApiError)?;
    let rows: Vec<SpeciesProfileDto> = sqlx::query_as(
        r#"
        SELECT id, organization_id, species_id, code, name, production_system_id, parameters, version, is_active, valid_from, valid_until
        FROM species_production_profiles
        WHERE organization_id = $1 AND species_id = $2
        ORDER BY code ASC, version DESC
        "#,
    )
    .bind(auth.organization_id.0)
    .bind(species_id)
    .fetch_all(&mut *tx)
    .await
    .map_err(|e| ApiError(map_sqlx_err(e)))?;
    tx.commit().await.map_err(|e| ApiError(map_sqlx_err(e)))?;
    Ok(Json(rows))
}

#[utoipa::path(
    post,
    path = "/api/v1/species/{id}/profiles",
    params(("id" = Uuid, Path, description = "Species ID")),
    request_body = CreateProfileRequest,
    responses((status = 201, body = SpeciesProfileDto)),
    tag = "Species"
)]
pub async fn create_profile<S: AppBackendState>(
    State(state): State<S>,
    auth: AuthPrincipal,
    Path(species_id): Path<Uuid>,
    Json(req): Json<CreateProfileRequest>,
) -> Result<(axum::http::StatusCode, Json<SpeciesProfileDto>), ApiError> {
    auth.require(Permission::SpeciesManage).map_err(ApiError)?;
    let mut tx = auth.context().begin_tx(state.pool()).await.map_err(ApiError)?;
    let row: SpeciesProfileDto = sqlx::query_as(
        r#"
        INSERT INTO species_production_profiles (
            organization_id, species_id, code, name, production_system_id, parameters, version
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id, organization_id, species_id, code, name, production_system_id, parameters, version, is_active, valid_from, valid_until
        "#,
    )
    .bind(auth.organization_id.0)
    .bind(species_id)
    .bind(req.code.trim().to_uppercase())
    .bind(req.name.trim())
    .bind(req.production_system_id)
    .bind(&req.parameters)
    .bind(req.version.unwrap_or(1))
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| ApiError(map_sqlx_err(e)))?;
    tx.commit().await.map_err(|e| ApiError(map_sqlx_err(e)))?;
    Ok((axum::http::StatusCode::CREATED, Json(row)))
}

pub fn router<S: AppBackendState>() -> Router<S> {
    Router::new()
        .route(
            "/api/v1/species",
            get(list_species::<S>).post(create_custom_species::<S>),
        )
        .route(
            "/api/v1/species/{id}/profiles",
            get(list_profiles::<S>).post(create_profile::<S>),
        )
}

#[derive(OpenApi)]
#[openapi(
    paths(list_species, create_custom_species, list_profiles, create_profile),
    components(schemas(CreateSpeciesRequest, SpeciesDto, CreateProfileRequest, SpeciesProfileDto))
)]
pub struct SpeciesApi;
