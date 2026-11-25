use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use infra_auth::{ApiError, AppBackendState, AuthPrincipal, Permission};
use infra_db::{map_sqlx_err, AuditLogger};
use mod_calculation::{
    compute_enclosure_geometry, evaluate_enclosure_capacity, CalculationResult, CapacityEvaluation,
    EnclosureGeometry,
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use shared_kernel::AppError;
use utoipa::{OpenApi, ToSchema};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateEnclosureRequest {
    pub farm_id: Uuid,
    pub code: String,
    pub name: String,
    pub unit_type_id: Uuid,
    pub production_system_id: Option<Uuid>,
    pub geometry: Option<EnclosureGeometry>,
    pub dimensions: Option<serde_json::Value>,
    pub capacity_value: Option<Decimal>,
    pub capacity_unit: Option<String>,
    pub area_m2: Option<Decimal>,
    pub volume_m3: Option<Decimal>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateEnclosureRequest {
    pub name: Option<String>,
    pub capacity_value: Option<Decimal>,
    pub capacity_unit: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct EnclosureDto {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub farm_id: Uuid,
    pub code: String,
    pub name: String,
    pub unit_type_id: Uuid,
    pub production_system_id: Option<Uuid>,
    pub dimensions: serde_json::Value,
    pub capacity_value: Option<Decimal>,
    pub capacity_unit: Option<String>,
    pub area_m2: Option<Decimal>,
    pub volume_m3: Option<Decimal>,
    pub status: String,
    pub version: i32,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
    pub updated_at: DateTime<Utc>,
    pub updated_by: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CalculateCapacityRequest {
    pub requested_addition: Decimal,
    pub parameter_version: Option<i32>,
}

#[utoipa::path(
    get,
    path = "/api/v1/enclosures",
    responses((status = 200, body = [EnclosureDto])),
    tag = "Enclosure"
)]
pub async fn list_enclosures<S: AppBackendState>(
    State(state): State<S>,
    auth: AuthPrincipal,
) -> Result<Json<Vec<EnclosureDto>>, ApiError> {
    let mut tx = auth.context().begin_tx(state.pool()).await.map_err(ApiError)?;
    let rows: Vec<EnclosureDto> = sqlx::query_as(
        r#"
        SELECT id, organization_id, farm_id, code, name, unit_type_id, production_system_id,
               dimensions, capacity_value, capacity_unit, area_m2, volume_m3, status, version,
               created_at, created_by, updated_at, updated_by
        FROM production_units
        WHERE organization_id = $1
        ORDER BY created_at DESC
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
    path = "/api/v1/enclosures",
    request_body = CreateEnclosureRequest,
    responses((status = 201, body = EnclosureDto)),
    tag = "Enclosure"
)]
pub async fn create_enclosure<S: AppBackendState>(
    State(state): State<S>,
    auth: AuthPrincipal,
    Json(req): Json<CreateEnclosureRequest>,
) -> Result<(axum::http::StatusCode, Json<EnclosureDto>), ApiError> {
    auth.require(Permission::FarmManage).map_err(ApiError)?;

    let (derived_area, derived_volume, dims_json) = if let Some(geom) = &req.geometry {
        let calc = compute_enclosure_geometry(geom, Utc::now())
            .map_err(|e| ApiError(AppError::Validation(e.to_string())))?;
        let dims = serde_json::to_value(geom).unwrap_or_else(|_| serde_json::json!({}));
        (
            Some(calc.value.area_m2),
            calc.value.volume_m3,
            req.dimensions.unwrap_or(dims),
        )
    } else {
        (
            req.area_m2,
            req.volume_m3,
            req.dimensions.unwrap_or_else(|| serde_json::json!({})),
        )
    };

    let mut tx = auth.context().begin_tx(state.pool()).await.map_err(ApiError)?;
    let row: EnclosureDto = sqlx::query_as(
        r#"
        INSERT INTO production_units (
            organization_id, farm_id, code, name, unit_type_id, production_system_id,
            dimensions, capacity_value, capacity_unit, area_m2, volume_m3, created_by
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        RETURNING id, organization_id, farm_id, code, name, unit_type_id, production_system_id,
                  dimensions, capacity_value, capacity_unit, area_m2, volume_m3, status, version,
                  created_at, created_by, updated_at, updated_by
        "#,
    )
    .bind(auth.organization_id.0)
    .bind(req.farm_id)
    .bind(req.code.trim())
    .bind(req.name.trim())
    .bind(req.unit_type_id)
    .bind(req.production_system_id)
    .bind(dims_json)
    .bind(req.capacity_value)
    .bind(&req.capacity_unit)
    .bind(derived_area)
    .bind(derived_volume)
    .bind(auth.user_id.0)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| ApiError(map_sqlx_err(e)))?;

    AuditLogger::record(
        &mut tx,
        Some(auth.organization_id),
        Some(auth.user_id),
        "enclosure.created",
        "production_units",
        Some(row.id),
        None,
        Some(serde_json::to_value(&row).unwrap_or_default()),
    )
    .await
    .map_err(ApiError)?;

    tx.commit().await.map_err(|e| ApiError(map_sqlx_err(e)))?;
    Ok((axum::http::StatusCode::CREATED, Json(row)))
}

#[utoipa::path(
    get,
    path = "/api/v1/enclosures/{id}",
    params(("id" = Uuid, Path, description = "Enclosure ID")),
    responses((status = 200, body = EnclosureDto)),
    tag = "Enclosure"
)]
pub async fn get_enclosure<S: AppBackendState>(
    State(state): State<S>,
    auth: AuthPrincipal,
    Path(id): Path<Uuid>,
) -> Result<Json<EnclosureDto>, ApiError> {
    let mut tx = auth.context().begin_tx(state.pool()).await.map_err(ApiError)?;
    let row: EnclosureDto = sqlx::query_as(
        r#"
        SELECT id, organization_id, farm_id, code, name, unit_type_id, production_system_id,
               dimensions, capacity_value, capacity_unit, area_m2, volume_m3, status, version,
               created_at, created_by, updated_at, updated_by
        FROM production_units
        WHERE organization_id = $1 AND id = $2
        "#,
    )
    .bind(auth.organization_id.0)
    .bind(id)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| ApiError(map_sqlx_err(e)))?;
    tx.commit().await.map_err(|e| ApiError(map_sqlx_err(e)))?;
    Ok(Json(row))
}

#[utoipa::path(
    patch,
    path = "/api/v1/enclosures/{id}",
    params(("id" = Uuid, Path, description = "Enclosure ID")),
    request_body = UpdateEnclosureRequest,
    responses((status = 200, body = EnclosureDto)),
    tag = "Enclosure"
)]
pub async fn update_enclosure<S: AppBackendState>(
    State(state): State<S>,
    auth: AuthPrincipal,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateEnclosureRequest>,
) -> Result<Json<EnclosureDto>, ApiError> {
    auth.require(Permission::FarmManage).map_err(ApiError)?;
    let mut tx = auth.context().begin_tx(state.pool()).await.map_err(ApiError)?;

    let before: EnclosureDto = sqlx::query_as(
        r#"
        SELECT id, organization_id, farm_id, code, name, unit_type_id, production_system_id,
               dimensions, capacity_value, capacity_unit, area_m2, volume_m3, status, version,
               created_at, created_by, updated_at, updated_by
        FROM production_units WHERE organization_id = $1 AND id = $2 FOR UPDATE
        "#,
    )
    .bind(auth.organization_id.0)
    .bind(id)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| ApiError(map_sqlx_err(e)))?;

    let after: EnclosureDto = sqlx::query_as(
        r#"
        UPDATE production_units
        SET name = COALESCE($3, name),
            capacity_value = COALESCE($4, capacity_value),
            capacity_unit = COALESCE($5, capacity_unit),
            status = COALESCE($6, status),
            version = version + 1,
            updated_by = $7
        WHERE organization_id = $1 AND id = $2
        RETURNING id, organization_id, farm_id, code, name, unit_type_id, production_system_id,
                  dimensions, capacity_value, capacity_unit, area_m2, volume_m3, status, version,
                  created_at, created_by, updated_at, updated_by
        "#,
    )
    .bind(auth.organization_id.0)
    .bind(id)
    .bind(req.name.as_deref().map(str::trim))
    .bind(req.capacity_value)
    .bind(&req.capacity_unit)
    .bind(&req.status)
    .bind(auth.user_id.0)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| ApiError(map_sqlx_err(e)))?;

    AuditLogger::record(
        &mut tx,
        Some(auth.organization_id),
        Some(auth.user_id),
        "enclosure.updated",
        "production_units",
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
    post,
    path = "/api/v1/enclosures/{id}/calculate-capacity",
    params(("id" = Uuid, Path, description = "Enclosure ID")),
    request_body = CalculateCapacityRequest,
    responses((status = 200, body = CalculationResult<CapacityEvaluation>)),
    tag = "Enclosure"
)]
pub async fn calculate_capacity<S: AppBackendState>(
    State(state): State<S>,
    auth: AuthPrincipal,
    Path(id): Path<Uuid>,
    Json(req): Json<CalculateCapacityRequest>,
) -> Result<Json<CalculationResult<CapacityEvaluation>>, ApiError> {
    let mut tx = auth.context().begin_tx(state.pool()).await.map_err(ApiError)?;
    let cap_val: Option<Decimal> = sqlx::query_scalar(
        "SELECT capacity_value FROM production_units WHERE organization_id = $1 AND id = $2 FOR UPDATE",
    )
    .bind(auth.organization_id.0)
    .bind(id)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| ApiError(map_sqlx_err(e)))?;

    let max_capacity = cap_val.ok_or_else(|| {
        ApiError(AppError::Validation(
            "enclosure has no configured capacity_value".into(),
        ))
    })?;

    let existing_headcount: i64 = sqlx::query_scalar(
        r#"
        SELECT COALESCE(SUM(pb.quantity), 0)::bigint
        FROM batch_unit_assignments bua
        JOIN population_balances pb
          ON pb.organization_id = bua.organization_id AND pb.batch_id = bua.batch_id
        WHERE bua.organization_id = $1 AND bua.unit_id = $2 AND bua.released_at IS NULL
        "#,
    )
    .bind(auth.organization_id.0)
    .bind(id)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| ApiError(map_sqlx_err(e)))?;

    tx.commit().await.map_err(|e| ApiError(map_sqlx_err(e)))?;

    let eval = evaluate_enclosure_capacity(
        max_capacity,
        Decimal::from(existing_headcount),
        req.requested_addition,
        req.parameter_version,
        Utc::now(),
    )
    .map_err(|e| ApiError(AppError::Validation(e.to_string())))?;

    Ok(Json(eval))
}

pub fn router<S: AppBackendState>() -> Router<S> {
    Router::new()
        .route(
            "/api/v1/enclosures",
            get(list_enclosures::<S>).post(create_enclosure::<S>),
        )
        .route(
            "/api/v1/enclosures/{id}",
            get(get_enclosure::<S>).patch(update_enclosure::<S>),
        )
        .route(
            "/api/v1/enclosures/{id}/calculate-capacity",
            post(calculate_capacity::<S>),
        )
}

#[derive(OpenApi)]
#[openapi(
    paths(list_enclosures, create_enclosure, get_enclosure, update_enclosure, calculate_capacity),
    components(schemas(
        CreateEnclosureRequest, UpdateEnclosureRequest, EnclosureDto, CalculateCapacityRequest,
        EnclosureGeometry, CapacityEvaluation, CalculationResult<CapacityEvaluation>
    ))
)]
pub struct EnclosureApi;
