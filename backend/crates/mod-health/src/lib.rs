use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::{DateTime, NaiveDate, Utc};
use infra_auth::{ApiError, AppBackendState, AuthPrincipal, Permission};
use infra_db::map_sqlx_err;
use serde::{Deserialize, Serialize};
use shared_kernel::AppError;
use utoipa::{OpenApi, ToSchema};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateVaccinationScheduleRequest {
    pub vaccine_name: String,
    pub target_disease: Option<String>,
    pub route: String,
    pub scheduled_date: NaiveDate,
    pub dosage: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct VaccinationScheduleDto {
    pub id: Uuid,
    pub batch_id: Uuid,
    pub vaccine_name: String,
    pub target_disease: Option<String>,
    pub route: String,
    pub scheduled_date: NaiveDate,
    pub dosage: Option<String>,
    pub status: String,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateHealthRecordRequest {
    pub observed_at: DateTime<Utc>,
    pub category: String,
    pub description: String,
    pub treatment_product: Option<String>,
    pub dosage: Option<String>,
    pub withdrawal_days: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct HealthRecordDto {
    pub id: Uuid,
    pub batch_id: Uuid,
    pub observed_at: DateTime<Utc>,
    pub category: String,
    pub description: String,
    pub treatment_product: Option<String>,
    pub dosage: Option<String>,
    pub withdrawal_days: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct RecordMortalityRequest {
    pub quantity: i32,
    pub occurred_at: DateTime<Utc>,
    pub suspected_cause: Option<String>,
    pub notes: Option<String>,
    pub idempotency_key: Uuid,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct MortalityResponse {
    pub id: Uuid,
    pub stock_movement_id: Uuid,
    pub remaining_population: i32,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateBiosecurityLogRequest {
    pub enclosure_id: Option<Uuid>,
    pub activity_type: String,
    pub performed_at: DateTime<Utc>,
    pub agent_used: Option<String>,
    pub concentration: Option<String>,
    pub actor_name: Option<String>,
    pub passed: Option<bool>,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct BiosecurityLogDto {
    pub id: Uuid,
    pub farm_id: Uuid,
    pub unit_id: Option<Uuid>,
    pub activity_type: String,
    pub performed_at: DateTime<Utc>,
    pub passed: bool,
}

#[utoipa::path(
    get,
    path = "/api/v1/batches/{id}/vaccinations",
    params(("id" = Uuid, Path, description = "Batch ID")),
    responses((status = 200, body = [VaccinationScheduleDto])),
    tag = "Health & Biosecurity"
)]
pub async fn list_vaccinations<S: AppBackendState>(
    State(state): State<S>,
    auth: AuthPrincipal,
    Path(batch_id): Path<Uuid>,
) -> Result<Json<Vec<VaccinationScheduleDto>>, ApiError> {
    let mut tx = auth.context().begin_tx(state.pool()).await.map_err(ApiError)?;
    let rows: Vec<VaccinationScheduleDto> = sqlx::query_as(
        r#"
        SELECT id, batch_id, vaccine_name, target_disease, route, scheduled_date, dosage, status, completed_at
        FROM vaccination_schedules
        WHERE organization_id = $1 AND batch_id = $2
        ORDER BY scheduled_date ASC
        "#,
    )
    .bind(auth.organization_id.0)
    .bind(batch_id)
    .fetch_all(&mut *tx)
    .await
    .map_err(|e| ApiError(map_sqlx_err(e)))?;
    tx.commit().await.map_err(|e| ApiError(map_sqlx_err(e)))?;
    Ok(Json(rows))
}

#[utoipa::path(
    post,
    path = "/api/v1/batches/{id}/vaccinations",
    params(("id" = Uuid, Path, description = "Batch ID")),
    request_body = CreateVaccinationScheduleRequest,
    responses((status = 201, body = VaccinationScheduleDto)),
    tag = "Health & Biosecurity"
)]
pub async fn schedule_vaccination<S: AppBackendState>(
    State(state): State<S>,
    auth: AuthPrincipal,
    Path(batch_id): Path<Uuid>,
    Json(req): Json<CreateVaccinationScheduleRequest>,
) -> Result<(axum::http::StatusCode, Json<VaccinationScheduleDto>), ApiError> {
    auth.require(Permission::HealthWrite).map_err(ApiError)?;
    let mut tx = auth.context().begin_tx(state.pool()).await.map_err(ApiError)?;
    let row: VaccinationScheduleDto = sqlx::query_as(
        r#"
        INSERT INTO vaccination_schedules (
            organization_id, batch_id, vaccine_name, target_disease, route, scheduled_date, dosage, notes, created_by
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING id, batch_id, vaccine_name, target_disease, route, scheduled_date, dosage, status, completed_at
        "#,
    )
    .bind(auth.organization_id.0)
    .bind(batch_id)
    .bind(req.vaccine_name.trim())
    .bind(&req.target_disease)
    .bind(&req.route)
    .bind(req.scheduled_date)
    .bind(&req.dosage)
    .bind(&req.notes)
    .bind(auth.user_id.0)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| ApiError(map_sqlx_err(e)))?;
    tx.commit().await.map_err(|e| ApiError(map_sqlx_err(e)))?;
    Ok((axum::http::StatusCode::CREATED, Json(row)))
}

#[utoipa::path(
    post,
    path = "/api/v1/batches/{id}/health-records",
    params(("id" = Uuid, Path, description = "Batch ID")),
    request_body = CreateHealthRecordRequest,
    responses((status = 201, body = HealthRecordDto)),
    tag = "Health & Biosecurity"
)]
pub async fn record_health_event<S: AppBackendState>(
    State(state): State<S>,
    auth: AuthPrincipal,
    Path(batch_id): Path<Uuid>,
    Json(req): Json<CreateHealthRecordRequest>,
) -> Result<(axum::http::StatusCode, Json<HealthRecordDto>), ApiError> {
    auth.require(Permission::HealthWrite).map_err(ApiError)?;
    let mut tx = auth.context().begin_tx(state.pool()).await.map_err(ApiError)?;
    let row: HealthRecordDto = sqlx::query_as(
        r#"
        INSERT INTO health_records (
            organization_id, batch_id, observed_at, category, description, treatment_product, dosage, withdrawal_days, created_by
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING id, batch_id, observed_at, category, description, treatment_product, dosage, withdrawal_days
        "#,
    )
    .bind(auth.organization_id.0)
    .bind(batch_id)
    .bind(req.observed_at)
    .bind(&req.category)
    .bind(&req.description)
    .bind(&req.treatment_product)
    .bind(&req.dosage)
    .bind(req.withdrawal_days)
    .bind(auth.user_id.0)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| ApiError(map_sqlx_err(e)))?;
    tx.commit().await.map_err(|e| ApiError(map_sqlx_err(e)))?;
    Ok((axum::http::StatusCode::CREATED, Json(row)))
}

#[utoipa::path(
    post,
    path = "/api/v1/batches/{id}/mortalities",
    params(("id" = Uuid, Path, description = "Batch ID")),
    request_body = RecordMortalityRequest,
    responses((status = 201, body = MortalityResponse)),
    tag = "Health & Biosecurity"
)]
pub async fn record_mortality<S: AppBackendState>(
    State(state): State<S>,
    auth: AuthPrincipal,
    Path(batch_id): Path<Uuid>,
    Json(req): Json<RecordMortalityRequest>,
) -> Result<(axum::http::StatusCode, Json<MortalityResponse>), ApiError> {
    auth.require(Permission::HealthWrite).map_err(ApiError)?;
    if req.quantity <= 0 {
        return Err(ApiError(AppError::Validation(
            "mortality quantity must be > 0".into(),
        )));
    }
    let mut tx = auth.context().begin_tx(state.pool()).await.map_err(ApiError)?;

    let current_pop: i32 = sqlx::query_scalar(
        "SELECT quantity FROM population_balances WHERE organization_id = $1 AND batch_id = $2 FOR UPDATE",
    )
    .bind(auth.organization_id.0)
    .bind(batch_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| ApiError(map_sqlx_err(e)))?;

    let remaining = current_pop - req.quantity;
    if remaining < 0 {
        return Err(ApiError(AppError::InsufficientBalance(format!(
            "mortality {} exceeds available population {}",
            req.quantity, current_pop
        ))));
    }

    let movement_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO stock_movements (
            organization_id, batch_id, movement_type, quantity_delta, occurred_at, reason, idempotency_key, created_by
        )
        VALUES ($1, $2, 'mortality', $3, $4, $5, $6, $7)
        RETURNING id
        "#,
    )
    .bind(auth.organization_id.0)
    .bind(batch_id)
    .bind(-req.quantity)
    .bind(req.occurred_at)
    .bind(&req.suspected_cause)
    .bind(req.idempotency_key)
    .bind(auth.user_id.0)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| ApiError(map_sqlx_err(e)))?;

    sqlx::query(
        "UPDATE population_balances SET quantity = $3, version = version + 1 WHERE organization_id = $1 AND batch_id = $2",
    )
    .bind(auth.organization_id.0)
    .bind(batch_id)
    .bind(remaining)
    .execute(&mut *tx)
    .await
    .map_err(|e| ApiError(map_sqlx_err(e)))?;

    let mortality_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO mortality_records (
            organization_id, batch_id, stock_movement_id, quantity, occurred_at, suspected_cause, notes, created_by
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING id
        "#,
    )
    .bind(auth.organization_id.0)
    .bind(batch_id)
    .bind(movement_id)
    .bind(req.quantity)
    .bind(req.occurred_at)
    .bind(&req.suspected_cause)
    .bind(&req.notes)
    .bind(auth.user_id.0)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| ApiError(map_sqlx_err(e)))?;

    tx.commit().await.map_err(|e| ApiError(map_sqlx_err(e)))?;
    Ok((
        axum::http::StatusCode::CREATED,
        Json(MortalityResponse {
            id: mortality_id,
            stock_movement_id: movement_id,
            remaining_population: remaining,
        }),
    ))
}

#[utoipa::path(
    post,
    path = "/api/v1/farms/{id}/biosecurity-logs",
    params(("id" = Uuid, Path, description = "Farm ID")),
    request_body = CreateBiosecurityLogRequest,
    responses((status = 201, body = BiosecurityLogDto)),
    tag = "Health & Biosecurity"
)]
pub async fn record_biosecurity_log<S: AppBackendState>(
    State(state): State<S>,
    auth: AuthPrincipal,
    Path(farm_id): Path<Uuid>,
    Json(req): Json<CreateBiosecurityLogRequest>,
) -> Result<(axum::http::StatusCode, Json<BiosecurityLogDto>), ApiError> {
    auth.require(Permission::HealthWrite).map_err(ApiError)?;
    let mut tx = auth.context().begin_tx(state.pool()).await.map_err(ApiError)?;
    let row: BiosecurityLogDto = sqlx::query_as(
        r#"
        INSERT INTO biosecurity_logs (
            organization_id, farm_id, unit_id, activity_type, performed_at,
            agent_used, concentration, actor_name, passed, notes, created_by
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        RETURNING id, farm_id, unit_id, activity_type, performed_at, passed
        "#,
    )
    .bind(auth.organization_id.0)
    .bind(farm_id)
    .bind(req.enclosure_id)
    .bind(&req.activity_type)
    .bind(req.performed_at)
    .bind(&req.agent_used)
    .bind(&req.concentration)
    .bind(&req.actor_name)
    .bind(req.passed.unwrap_or(true))
    .bind(&req.notes)
    .bind(auth.user_id.0)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| ApiError(map_sqlx_err(e)))?;
    tx.commit().await.map_err(|e| ApiError(map_sqlx_err(e)))?;
    Ok((axum::http::StatusCode::CREATED, Json(row)))
}

pub fn router<S: AppBackendState>() -> Router<S> {
    Router::new()
        .route(
            "/api/v1/batches/{id}/vaccinations",
            get(list_vaccinations::<S>).post(schedule_vaccination::<S>),
        )
        .route(
            "/api/v1/batches/{id}/health-records",
            post(record_health_event::<S>),
        )
        .route(
            "/api/v1/batches/{id}/mortalities",
            post(record_mortality::<S>),
        )
        .route(
            "/api/v1/farms/{id}/biosecurity-logs",
            post(record_biosecurity_log::<S>),
        )
}

#[derive(OpenApi)]
#[openapi(
    paths(list_vaccinations, schedule_vaccination, record_health_event, record_mortality, record_biosecurity_log),
    components(schemas(
        CreateVaccinationScheduleRequest, VaccinationScheduleDto,
        CreateHealthRecordRequest, HealthRecordDto,
        RecordMortalityRequest, MortalityResponse,
        CreateBiosecurityLogRequest, BiosecurityLogDto
    ))
)]
pub struct HealthApi;
