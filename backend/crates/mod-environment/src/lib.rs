use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use infra_auth::{ApiError, AppBackendState, AuthPrincipal, Permission};
use infra_db::{map_sqlx_err, AuditLogger};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::{OpenApi, ToSchema};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct EnvironmentMetricDto {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub default_unit: String,
    pub category: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateReadingRequest {
    pub enclosure_id: Uuid,
    pub batch_id: Option<Uuid>,
    pub metric_id: Uuid,
    pub metric_value: Decimal,
    pub unit: String,
    pub measured_at: DateTime<Utc>,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct IotIngestRequest {
    pub enclosure_id: Uuid,
    pub metric_code: String,
    pub metric_value: Decimal,
    pub unit: String,
    pub measured_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct EnvironmentReadingDto {
    pub id: Uuid,
    pub unit_id: Uuid,
    pub batch_id: Option<Uuid>,
    pub metric_id: Uuid,
    pub metric_value: Decimal,
    pub unit: String,
    pub measured_at: DateTime<Utc>,
    pub source: String,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
    pub updated_at: DateTime<Utc>,
    pub updated_by: Option<Uuid>,
}

#[utoipa::path(
    get,
    path = "/api/v1/environment/metrics",
    responses((status = 200, body = [EnvironmentMetricDto])),
    tag = "Environment"
)]
pub async fn list_metrics<S: AppBackendState>(
    State(state): State<S>,
    _auth: AuthPrincipal,
) -> Result<Json<Vec<EnvironmentMetricDto>>, ApiError> {
    let rows: Vec<EnvironmentMetricDto> = sqlx::query_as(
        "SELECT id, code, name, default_unit, category FROM environment_metrics ORDER BY category, code",
    )
    .fetch_all(state.pool())
    .await
    .map_err(|e| ApiError(map_sqlx_err(e)))?;
    Ok(Json(rows))
}

#[utoipa::path(
    get,
    path = "/api/v1/enclosures/{id}/environment",
    params(("id" = Uuid, Path, description = "Enclosure ID")),
    responses((status = 200, body = [EnvironmentReadingDto])),
    tag = "Environment"
)]
pub async fn list_enclosure_readings<S: AppBackendState>(
    State(state): State<S>,
    auth: AuthPrincipal,
    Path(enclosure_id): Path<Uuid>,
) -> Result<Json<Vec<EnvironmentReadingDto>>, ApiError> {
    let mut tx = auth
        .context()
        .begin_tx(state.pool())
        .await
        .map_err(ApiError)?;
    let rows: Vec<EnvironmentReadingDto> = sqlx::query_as(
        r#"
        SELECT id, unit_id, batch_id, metric_id, metric_value, unit, measured_at, source, notes,
               created_at, created_by, updated_at, updated_by
        FROM environment_readings
        WHERE organization_id = $1 AND unit_id = $2
        ORDER BY measured_at DESC
        LIMIT 200
        "#,
    )
    .bind(auth.organization_id.0)
    .bind(enclosure_id)
    .fetch_all(&mut *tx)
    .await
    .map_err(|e| ApiError(map_sqlx_err(e)))?;
    tx.commit().await.map_err(|e| ApiError(map_sqlx_err(e)))?;
    Ok(Json(rows))
}

#[utoipa::path(
    post,
    path = "/api/v1/environment/readings",
    request_body = CreateReadingRequest,
    responses((status = 201, body = EnvironmentReadingDto)),
    tag = "Environment"
)]
pub async fn record_manual_reading<S: AppBackendState>(
    State(state): State<S>,
    auth: AuthPrincipal,
    Json(req): Json<CreateReadingRequest>,
) -> Result<(axum::http::StatusCode, Json<EnvironmentReadingDto>), ApiError> {
    auth.require(Permission::EnvironmentWrite)
        .map_err(ApiError)?;
    let mut tx = auth
        .context()
        .begin_tx(state.pool())
        .await
        .map_err(ApiError)?;
    let row: EnvironmentReadingDto = sqlx::query_as(
        r#"
        INSERT INTO environment_readings (
            organization_id, unit_id, batch_id, metric_id, metric_value, unit, measured_at, source, notes, created_by
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, 'manual', $8, $9)
        RETURNING id, unit_id, batch_id, metric_id, metric_value, unit, measured_at, source, notes,
                  created_at, created_by, updated_at, updated_by
        "#,
    )
    .bind(auth.organization_id.0)
    .bind(req.enclosure_id)
    .bind(req.batch_id)
    .bind(req.metric_id)
    .bind(req.metric_value)
    .bind(req.unit.trim())
    .bind(req.measured_at)
    .bind(&req.notes)
    .bind(auth.user_id.0)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| ApiError(map_sqlx_err(e)))?;

    AuditLogger::record(
        &mut tx,
        auth.organization_id,
        Some(auth.user_id),
        "environment_reading.recorded",
        "environment_readings",
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
    post,
    path = "/api/v1/environment/iot-ingest",
    request_body = IotIngestRequest,
    responses((status = 201, body = EnvironmentReadingDto)),
    tag = "Environment"
)]
pub async fn ingest_iot_reading<S: AppBackendState>(
    State(state): State<S>,
    auth: AuthPrincipal,
    Json(req): Json<IotIngestRequest>,
) -> Result<(axum::http::StatusCode, Json<EnvironmentReadingDto>), ApiError> {
    auth.require(Permission::EnvironmentWrite)
        .map_err(ApiError)?;
    let mut tx = auth
        .context()
        .begin_tx(state.pool())
        .await
        .map_err(ApiError)?;
    let metric_id: Uuid = sqlx::query_scalar("SELECT id FROM environment_metrics WHERE code = $1")
        .bind(req.metric_code.trim().to_uppercase())
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| ApiError(map_sqlx_err(e)))?;

    let row: EnvironmentReadingDto = sqlx::query_as(
        r#"
        INSERT INTO environment_readings (
            organization_id, unit_id, metric_id, metric_value, unit, measured_at, source, created_by
        )
        VALUES ($1, $2, $3, $4, $5, $6, 'iot_sensor', $7)
        RETURNING id, unit_id, batch_id, metric_id, metric_value, unit, measured_at, source, notes,
                  created_at, created_by, updated_at, updated_by
        "#,
    )
    .bind(auth.organization_id.0)
    .bind(req.enclosure_id)
    .bind(metric_id)
    .bind(req.metric_value)
    .bind(req.unit.trim())
    .bind(req.measured_at)
    .bind(auth.user_id.0)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| ApiError(map_sqlx_err(e)))?;
    tx.commit().await.map_err(|e| ApiError(map_sqlx_err(e)))?;
    Ok((axum::http::StatusCode::CREATED, Json(row)))
}

pub fn router<S: AppBackendState>() -> Router<S> {
    Router::new()
        .route("/api/v1/environment/metrics", get(list_metrics::<S>))
        .route(
            "/api/v1/enclosures/{id}/environment",
            get(list_enclosure_readings::<S>),
        )
        .route(
            "/api/v1/environment/readings",
            post(record_manual_reading::<S>),
        )
        .route(
            "/api/v1/environment/iot-ingest",
            post(ingest_iot_reading::<S>),
        )
}

#[derive(OpenApi)]
#[openapi(
    paths(
        list_metrics,
        list_enclosure_readings,
        record_manual_reading,
        ingest_iot_reading
    ),
    components(schemas(
        EnvironmentMetricDto,
        CreateReadingRequest,
        IotIngestRequest,
        EnvironmentReadingDto
    ))
)]
pub struct EnvironmentApi;
