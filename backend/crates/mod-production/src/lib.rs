use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::{DateTime, NaiveDate, Utc};
use infra_auth::{ApiError, AppBackendState, AuthPrincipal, Permission};
use infra_db::{map_sqlx_err, AuditLogger, OutboxWriter};
use mod_calculation::{
    compute_abw_g, compute_adg_g_per_day, compute_biomass_kg, compute_sgr_percent_per_day,
    evaluate_enclosure_capacity, CalculationResult, CohortBiomassInput,
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use shared_kernel::error::ErrorEnvelope;
use shared_kernel::{AppError, MovementType, OutputType};
use std::collections::HashMap;
use utoipa::{OpenApi, ToSchema};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateBatchRequest {
    pub batch_code: String,
    pub species_id: Uuid,
    pub production_profile_id: Option<Uuid>,
    pub enclosure_id: Uuid,
    pub start_date: NaiveDate,
    pub target_end_date: Option<NaiveDate>,
    pub initial_count: i32,
    pub initial_avg_weight_g: Option<Decimal>,
    pub notes: Option<String>,
    pub idempotency_key: Uuid,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateBatchRequest {
    pub target_end_date: Option<NaiveDate>,
    pub status: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct BatchDto {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub batch_code: String,
    pub species_id: Uuid,
    pub production_profile_id: Option<Uuid>,
    pub start_date: NaiveDate,
    pub target_end_date: Option<NaiveDate>,
    pub initial_count: i32,
    pub initial_avg_weight_g: Option<Decimal>,
    pub status: String,
    pub notes: Option<String>,
    pub version: i32,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
    pub updated_at: DateTime<Utc>,
    pub updated_by: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct BatchSummaryQuery {
    pub status: Option<String>,
    pub limit: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct BatchDetailDto {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub batch_code: String,
    pub species_id: Uuid,
    pub production_profile_id: Option<Uuid>,
    pub start_date: NaiveDate,
    pub target_end_date: Option<NaiveDate>,
    pub initial_count: i32,
    pub initial_avg_weight_g: Option<Decimal>,
    pub status: String,
    pub notes: Option<String>,
    pub version: i32,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
    pub updated_at: DateTime<Utc>,
    pub updated_by: Option<Uuid>,
    pub active_enclosure_id: Option<Uuid>,
    pub current_population: i32,
    pub latest_abw_g: Option<Decimal>,
    pub latest_sampled_at: Option<DateTime<Utc>>,
    pub adg_g_per_day: Option<Decimal>,
    pub sgr_percent_per_day: Option<Decimal>,
    #[schema(value_type = Option<CalculationResult<String>>)]
    pub estimated_biomass: Option<CalculationResult<Decimal>>,
}

#[derive(sqlx::FromRow)]
struct BatchSummaryRow {
    id: Uuid,
    organization_id: Uuid,
    batch_code: String,
    species_id: Uuid,
    production_profile_id: Option<Uuid>,
    start_date: NaiveDate,
    target_end_date: Option<NaiveDate>,
    initial_count: i32,
    initial_avg_weight_g: Option<Decimal>,
    status: String,
    notes: Option<String>,
    version: i32,
    created_at: DateTime<Utc>,
    created_by: Option<Uuid>,
    updated_at: DateTime<Utc>,
    updated_by: Option<Uuid>,
    active_enclosure_id: Option<Uuid>,
    current_population: i32,
}

#[derive(sqlx::FromRow)]
struct BatchGrowthSampleRow {
    batch_id: Uuid,
    sampled_at: DateTime<Utc>,
    average_weight_g: Option<Decimal>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct GrowthSampleDto {
    pub id: Uuid,
    pub batch_id: Uuid,
    pub sampled_at: DateTime<Utc>,
    pub sample_count: i32,
    pub total_sample_weight_g: Decimal,
    pub average_weight_g: Option<Decimal>,
    pub average_length_cm: Option<Decimal>,
    pub method: Option<String>,
    pub notes: Option<String>,
    pub version: i32,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
    pub updated_at: DateTime<Utc>,
    pub updated_by: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct ProductionOutputDto {
    pub id: Uuid,
    pub batch_id: Uuid,
    pub stock_movement_id: Option<Uuid>,
    pub occurred_at: DateTime<Utc>,
    pub output_type: String,
    pub quantity: Decimal,
    pub unit: String,
    pub average_weight_g: Option<Decimal>,
    pub grade: Option<String>,
    pub notes: Option<String>,
    pub version: i32,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
    pub updated_at: DateTime<Utc>,
    pub updated_by: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct StockMovementRequest {
    pub movement_type: MovementType,
    pub quantity_delta: i32,
    pub occurred_at: DateTime<Utc>,
    pub reason: Option<String>,
    pub idempotency_key: Uuid,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct StockMovementResponse {
    pub id: Uuid,
    pub batch_id: Uuid,
    pub current_population: i32,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct RegisterTagRequest {
    pub tag_code: String,
    pub tag_type: Option<String>,
    pub sex: Option<String>,
    pub birth_date: Option<NaiveDate>,
    pub sire_tag_id: Option<Uuid>,
    pub dam_tag_id: Option<Uuid>,
    pub initial_weight_g: Option<Decimal>,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct IndividualTagDto {
    pub id: Uuid,
    pub batch_id: Uuid,
    pub tag_code: String,
    pub tag_type: String,
    pub sex: String,
    pub birth_date: Option<NaiveDate>,
    pub initial_weight_g: Option<Decimal>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
    pub updated_at: DateTime<Utc>,
    pub updated_by: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateGrowthSampleRequest {
    pub sampled_at: DateTime<Utc>,
    pub sample_count: i32,
    pub total_sample_weight_g: Decimal,
    pub average_length_cm: Option<Decimal>,
    pub method: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct GrowthSampleResponse {
    pub id: Uuid,
    pub abw_g: Decimal,
    pub adg_g_per_day: Option<Decimal>,
    pub sgr_percent_per_day: Option<Decimal>,
    pub estimated_biomass_kg: Option<Decimal>,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateOutputRequest {
    pub output_type: OutputType,
    pub quantity: Decimal,
    pub unit: String,
    pub headcount_removed: Option<i32>,
    pub average_weight_g: Option<Decimal>,
    pub grade: Option<String>,
    pub occurred_at: DateTime<Utc>,
    pub notes: Option<String>,
    pub idempotency_key: Uuid,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct OutputResponse {
    pub id: Uuid,
    pub batch_id: Uuid,
    pub output_type: OutputType,
    pub quantity: Decimal,
    pub unit: String,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
}

type GrowthSamplePoint = (DateTime<Utc>, Option<Decimal>);

fn assemble_batch_detail(
    row: BatchSummaryRow,
    samples: &[GrowthSamplePoint],
    calculated_at: DateTime<Utc>,
) -> Result<BatchDetailDto, ApiError> {
    let latest = samples.first();
    let prev = samples.get(1);
    let latest_sampled_at = latest.map(|s| s.0);
    let latest_abw_g = latest.and_then(|s| s.1);

    let (adg_g_per_day, sgr_percent_per_day) =
        match ((latest_sampled_at, latest_abw_g), prev.map(|p| (p.0, p.1))) {
            ((Some(cur_at), Some(cur_abw)), Some((prev_at, Some(prev_abw)))) => {
                let secs = (cur_at - prev_at).num_seconds();
                if secs > 0 {
                    let days = Decimal::from(secs) / Decimal::from(86_400);
                    let adg = compute_adg_g_per_day(prev_abw, cur_abw, days, None, calculated_at)
                        .map_err(|e| ApiError(AppError::Validation(e.to_string())))?
                        .value;
                    let sgr =
                        compute_sgr_percent_per_day(prev_abw, cur_abw, days, None, calculated_at)
                            .map_err(|e| ApiError(AppError::Validation(e.to_string())))?
                            .value;
                    (Some(adg), Some(sgr))
                } else {
                    (None, None)
                }
            }
            _ => (None, None),
        };

    let estimated_biomass = match latest_abw_g {
        Some(abw) => Some(
            compute_biomass_kg(
                &CohortBiomassInput {
                    estimated_population: row.current_population as i64,
                    average_body_weight_g: abw,
                    sampled_at: latest_sampled_at,
                },
                None,
                calculated_at,
            )
            .map_err(|e| ApiError(AppError::Validation(e.to_string())))?,
        ),
        None => None,
    };

    Ok(BatchDetailDto {
        id: row.id,
        organization_id: row.organization_id,
        batch_code: row.batch_code,
        species_id: row.species_id,
        production_profile_id: row.production_profile_id,
        start_date: row.start_date,
        target_end_date: row.target_end_date,
        initial_count: row.initial_count,
        initial_avg_weight_g: row.initial_avg_weight_g,
        status: row.status,
        notes: row.notes,
        version: row.version,
        created_at: row.created_at,
        created_by: row.created_by,
        updated_at: row.updated_at,
        updated_by: row.updated_by,
        active_enclosure_id: row.active_enclosure_id,
        current_population: row.current_population,
        latest_abw_g,
        latest_sampled_at,
        adg_g_per_day,
        sgr_percent_per_day,
        estimated_biomass,
    })
}

#[utoipa::path(
    get,
    path = "/api/v1/batches",
    responses((status = 200, body = [BatchDto])),
    tag = "Production"
)]
pub async fn list_batches<S: AppBackendState>(
    State(state): State<S>,
    auth: AuthPrincipal,
) -> Result<Json<Vec<BatchDto>>, ApiError> {
    let mut tx = auth
        .context()
        .begin_tx(state.pool())
        .await
        .map_err(ApiError)?;
    let rows: Vec<BatchDto> = sqlx::query_as(
        r#"
        SELECT id, organization_id, batch_code, species_id, production_profile_id,
               start_date, target_end_date, initial_count, initial_avg_weight_g, status, notes, version,
               created_at, created_by, updated_at, updated_by
        FROM production_batches
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
    path = "/api/v1/batches",
    request_body = CreateBatchRequest,
    responses((status = 201, body = BatchDto)),
    tag = "Production"
)]
pub async fn create_batch<S: AppBackendState>(
    State(state): State<S>,
    auth: AuthPrincipal,
    Json(req): Json<CreateBatchRequest>,
) -> Result<(axum::http::StatusCode, Json<BatchDto>), ApiError> {
    auth.require(Permission::ProductionWrite)
        .map_err(ApiError)?;
    if req.initial_count <= 0 {
        return Err(ApiError(AppError::Validation(
            "initial_count must be > 0".into(),
        )));
    }

    let mut tx = auth
        .context()
        .begin_tx(state.pool())
        .await
        .map_err(ApiError)?;

    let unit_cap: Option<(Option<Decimal>, Option<String>)> = sqlx::query_as(
        "SELECT capacity_value, capacity_unit FROM production_units WHERE organization_id = $1 AND id = $2 FOR UPDATE",
    )
    .bind(auth.organization_id.0)
    .bind(req.enclosure_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| ApiError(map_sqlx_err(e)))?;

    let Some((cap_value, cap_unit)) = unit_cap else {
        return Err(ApiError(AppError::NotFound("enclosure not found".into())));
    };

    if let (Some(max_cap), Some(unit)) = (cap_value, cap_unit) {
        if unit == "head" {
            let current_heads: i64 = sqlx::query_scalar(
                r#"
                SELECT COALESCE(SUM(pb.quantity), 0)::bigint
                FROM batch_unit_assignments bua
                JOIN population_balances pb
                  ON pb.organization_id = bua.organization_id AND pb.batch_id = bua.batch_id
                WHERE bua.organization_id = $1 AND bua.unit_id = $2 AND bua.released_at IS NULL
                "#,
            )
            .bind(auth.organization_id.0)
            .bind(req.enclosure_id)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| ApiError(map_sqlx_err(e)))?;

            let check = evaluate_enclosure_capacity(
                max_cap,
                Decimal::from(current_heads),
                Decimal::from(req.initial_count),
                None,
                Utc::now(),
            )
            .map_err(|e| ApiError(AppError::Validation(e.to_string())))?;

            if !check.value.fits {
                return Err(ApiError(AppError::CapacityExceeded {
                    message: "enclosure capacity exceeded across active batches".into(),
                    details: serde_json::to_value(check.value).unwrap_or_default(),
                }));
            }
        }
    }

    let batch: BatchDto = sqlx::query_as(
        r#"
        INSERT INTO production_batches (
            organization_id, batch_code, species_id, production_profile_id,
            start_date, target_end_date, initial_count, initial_avg_weight_g, status, notes, created_by
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'active', $9, $10)
        RETURNING id, organization_id, batch_code, species_id, production_profile_id,
                  start_date, target_end_date, initial_count, initial_avg_weight_g, status, notes, version,
                  created_at, created_by, updated_at, updated_by
        "#,
    )
    .bind(auth.organization_id.0)
    .bind(req.batch_code.trim())
    .bind(req.species_id)
    .bind(req.production_profile_id)
    .bind(req.start_date)
    .bind(req.target_end_date)
    .bind(req.initial_count)
    .bind(req.initial_avg_weight_g)
    .bind(&req.notes)
    .bind(auth.user_id.0)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| ApiError(map_sqlx_err(e)))?;

    sqlx::query(
        r#"
        INSERT INTO batch_unit_assignments (organization_id, batch_id, unit_id, assigned_at, allocated_count, created_by)
        VALUES ($1, $2, $3, now(), $4, $5)
        "#,
    )
    .bind(auth.organization_id.0)
    .bind(batch.id)
    .bind(req.enclosure_id)
    .bind(req.initial_count)
    .bind(auth.user_id.0)
    .execute(&mut *tx)
    .await
    .map_err(|e| ApiError(map_sqlx_err(e)))?;

    sqlx::query(
        r#"
        INSERT INTO stock_movements (
            organization_id, batch_id, movement_type, quantity_delta, occurred_at, idempotency_key, created_by
        )
        VALUES ($1, $2, 'initial_stock', $3, now(), $4, $5)
        "#,
    )
    .bind(auth.organization_id.0)
    .bind(batch.id)
    .bind(req.initial_count)
    .bind(req.idempotency_key)
    .bind(auth.user_id.0)
    .execute(&mut *tx)
    .await
    .map_err(|e| ApiError(map_sqlx_err(e)))?;

    sqlx::query(
        r#"
        INSERT INTO population_balances (organization_id, batch_id, quantity, updated_by)
        VALUES ($1, $2, $3, $4)
        "#,
    )
    .bind(auth.organization_id.0)
    .bind(batch.id)
    .bind(req.initial_count)
    .bind(auth.user_id.0)
    .execute(&mut *tx)
    .await
    .map_err(|e| ApiError(map_sqlx_err(e)))?;

    OutboxWriter::append(
        &mut tx,
        auth.organization_id,
        "production_batches",
        batch.id,
        "production_batches.created",
        serde_json::json!({ "batch_code": batch.batch_code, "initial_count": batch.initial_count }),
    )
    .await
    .map_err(ApiError)?;

    AuditLogger::record(
        &mut tx,
        auth.organization_id,
        Some(auth.user_id),
        "batch.created",
        "production_batches",
        Some(batch.id),
        None,
        Some(serde_json::to_value(&batch).unwrap_or_default()),
    )
    .await
    .map_err(ApiError)?;

    tx.commit().await.map_err(|e| ApiError(map_sqlx_err(e)))?;
    Ok((axum::http::StatusCode::CREATED, Json(batch)))
}

#[utoipa::path(
    get,
    path = "/api/v1/batches/summary",
    responses((status = 200, body = [BatchDetailDto])),
    tag = "Production"
)]
pub async fn list_batch_summaries<S: AppBackendState>(
    State(state): State<S>,
    auth: AuthPrincipal,
    Query(query): Query<BatchSummaryQuery>,
) -> Result<Json<Vec<BatchDetailDto>>, ApiError> {
    let limit = query.limit.unwrap_or(100).clamp(1, 200);
    let mut tx = auth
        .context()
        .begin_tx(state.pool())
        .await
        .map_err(ApiError)?;

    let batch_rows: Vec<BatchSummaryRow> = sqlx::query_as(
        r#"
        SELECT b.id, b.organization_id, b.batch_code, b.species_id, b.production_profile_id,
               b.start_date, b.target_end_date, b.initial_count, b.initial_avg_weight_g,
               b.status, b.notes, b.version, b.created_at, b.created_by, b.updated_at, b.updated_by,
               bua.unit_id AS active_enclosure_id,
               COALESCE(pb.quantity, b.initial_count) AS current_population
        FROM production_batches b
        LEFT JOIN batch_unit_assignments bua
          ON bua.organization_id = b.organization_id
         AND bua.batch_id = b.id
         AND bua.released_at IS NULL
        LEFT JOIN population_balances pb
          ON pb.organization_id = b.organization_id
         AND pb.batch_id = b.id
        WHERE b.organization_id = $1
          AND ($2::text IS NULL OR b.status = $2)
        ORDER BY b.created_at DESC
        LIMIT $3
        "#,
    )
    .bind(auth.organization_id.0)
    .bind(query.status.as_deref())
    .bind(limit)
    .fetch_all(&mut *tx)
    .await
    .map_err(|e| ApiError(map_sqlx_err(e)))?;

    let mut samples_by_batch: HashMap<Uuid, Vec<GrowthSamplePoint>> = HashMap::new();
    if !batch_rows.is_empty() {
        let batch_ids: Vec<Uuid> = batch_rows.iter().map(|b| b.id).collect();
        let sample_rows: Vec<BatchGrowthSampleRow> = sqlx::query_as(
            r#"
            SELECT batch_id, sampled_at, average_weight_g
            FROM (
                SELECT batch_id, sampled_at, average_weight_g,
                       ROW_NUMBER() OVER (PARTITION BY batch_id ORDER BY sampled_at DESC, created_at DESC) AS rn
                FROM growth_samples
                WHERE organization_id = $1 AND batch_id = ANY($2)
            ) ranked
            WHERE rn <= 2
            ORDER BY batch_id, rn ASC
            "#,
        )
        .bind(auth.organization_id.0)
        .bind(&batch_ids)
        .fetch_all(&mut *tx)
        .await
        .map_err(|e| ApiError(map_sqlx_err(e)))?;

        for s in sample_rows {
            samples_by_batch
                .entry(s.batch_id)
                .or_default()
                .push((s.sampled_at, s.average_weight_g));
        }
    }

    tx.commit().await.map_err(|e| ApiError(map_sqlx_err(e)))?;

    let now = Utc::now();
    let mut details = Vec::with_capacity(batch_rows.len());
    for row in batch_rows {
        let samples = samples_by_batch
            .get(&row.id)
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        details.push(assemble_batch_detail(row, samples, now)?);
    }
    Ok(Json(details))
}

#[utoipa::path(
    get,
    path = "/api/v1/batches/{id}",
    params(("id" = Uuid, Path, description = "Batch ID")),
    responses((status = 200, body = BatchDetailDto), (status = 404, body = ErrorEnvelope)),
    tag = "Production"
)]
pub async fn get_batch<S: AppBackendState>(
    State(state): State<S>,
    auth: AuthPrincipal,
    Path(id): Path<Uuid>,
) -> Result<Json<BatchDetailDto>, ApiError> {
    let mut tx = auth
        .context()
        .begin_tx(state.pool())
        .await
        .map_err(ApiError)?;

    let row: BatchSummaryRow = sqlx::query_as(
        r#"
        SELECT b.id, b.organization_id, b.batch_code, b.species_id, b.production_profile_id,
               b.start_date, b.target_end_date, b.initial_count, b.initial_avg_weight_g,
               b.status, b.notes, b.version, b.created_at, b.created_by, b.updated_at, b.updated_by,
               bua.unit_id AS active_enclosure_id,
               COALESCE(pb.quantity, b.initial_count) AS current_population
        FROM production_batches b
        LEFT JOIN batch_unit_assignments bua
          ON bua.organization_id = b.organization_id
         AND bua.batch_id = b.id
         AND bua.released_at IS NULL
        LEFT JOIN population_balances pb
          ON pb.organization_id = b.organization_id
         AND pb.batch_id = b.id
        WHERE b.organization_id = $1 AND b.id = $2
        "#,
    )
    .bind(auth.organization_id.0)
    .bind(id)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| ApiError(map_sqlx_err(e)))?;

    let samples: Vec<(DateTime<Utc>, Option<Decimal>)> = sqlx::query_as(
        r#"
        SELECT sampled_at, average_weight_g
        FROM growth_samples
        WHERE organization_id = $1 AND batch_id = $2
        ORDER BY sampled_at DESC, created_at DESC
        LIMIT 2
        "#,
    )
    .bind(auth.organization_id.0)
    .bind(id)
    .fetch_all(&mut *tx)
    .await
    .map_err(|e| ApiError(map_sqlx_err(e)))?;

    tx.commit().await.map_err(|e| ApiError(map_sqlx_err(e)))?;
    let detail = assemble_batch_detail(row, &samples, Utc::now())?;
    Ok(Json(detail))
}

#[utoipa::path(
    patch,
    path = "/api/v1/batches/{id}",
    params(("id" = Uuid, Path, description = "Batch ID")),
    request_body = UpdateBatchRequest,
    responses((status = 200, body = BatchDto)),
    tag = "Production"
)]
pub async fn update_batch<S: AppBackendState>(
    State(state): State<S>,
    auth: AuthPrincipal,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateBatchRequest>,
) -> Result<Json<BatchDto>, ApiError> {
    auth.require(Permission::ProductionWrite)
        .map_err(ApiError)?;
    let mut tx = auth
        .context()
        .begin_tx(state.pool())
        .await
        .map_err(ApiError)?;

    let before: BatchDto = sqlx::query_as(
        r#"
        SELECT id, organization_id, batch_code, species_id, production_profile_id,
               start_date, target_end_date, initial_count, initial_avg_weight_g, status, notes, version,
               created_at, created_by, updated_at, updated_by
        FROM production_batches WHERE organization_id = $1 AND id = $2 FOR UPDATE
        "#,
    )
    .bind(auth.organization_id.0)
    .bind(id)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| ApiError(map_sqlx_err(e)))?;

    let after: BatchDto = sqlx::query_as(
        r#"
        UPDATE production_batches
        SET target_end_date = COALESCE($3, target_end_date),
            status = COALESCE($4, status),
            notes = COALESCE($5, notes),
            version = version + 1,
            updated_by = $6
        WHERE organization_id = $1 AND id = $2
        RETURNING id, organization_id, batch_code, species_id, production_profile_id,
                  start_date, target_end_date, initial_count, initial_avg_weight_g, status, notes, version,
                  created_at, created_by, updated_at, updated_by
        "#,
    )
    .bind(auth.organization_id.0)
    .bind(id)
    .bind(req.target_end_date)
    .bind(&req.status)
    .bind(&req.notes)
    .bind(auth.user_id.0)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| ApiError(map_sqlx_err(e)))?;

    AuditLogger::record(
        &mut tx,
        auth.organization_id,
        Some(auth.user_id),
        "batch.updated",
        "production_batches",
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
    path = "/api/v1/batches/{id}/stock-movements",
    params(("id" = Uuid, Path, description = "Batch ID")),
    request_body = StockMovementRequest,
    responses((status = 201, body = StockMovementResponse)),
    tag = "Production"
)]
pub async fn record_stock_movement<S: AppBackendState>(
    State(state): State<S>,
    auth: AuthPrincipal,
    Path(batch_id): Path<Uuid>,
    Json(req): Json<StockMovementRequest>,
) -> Result<(axum::http::StatusCode, Json<StockMovementResponse>), ApiError> {
    auth.require(Permission::ProductionWrite)
        .map_err(ApiError)?;
    let mut tx = auth
        .context()
        .begin_tx(state.pool())
        .await
        .map_err(ApiError)?;

    let current_qty: i32 = sqlx::query_scalar(
        "SELECT quantity FROM population_balances WHERE organization_id = $1 AND batch_id = $2 FOR UPDATE",
    )
    .bind(auth.organization_id.0)
    .bind(batch_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| ApiError(map_sqlx_err(e)))?;

    let new_qty = if req.movement_type.affects_live_headcount() {
        let candidate = current_qty + req.quantity_delta;
        if candidate < 0 {
            return Err(ApiError(AppError::InsufficientBalance(format!(
                "population cannot become negative (current: {current_qty}, delta: {})",
                req.quantity_delta
            ))));
        }
        candidate
    } else {
        current_qty
    };

    let (movement_id, created_at, created_by): (Uuid, DateTime<Utc>, Option<Uuid>) = sqlx::query_as(
        r#"
        INSERT INTO stock_movements (
            organization_id, batch_id, movement_type, quantity_delta, occurred_at, reason, idempotency_key, created_by
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING id, created_at, created_by
        "#,
    )
    .bind(auth.organization_id.0)
    .bind(batch_id)
    .bind(req.movement_type.as_str())
    .bind(req.quantity_delta)
    .bind(req.occurred_at)
    .bind(&req.reason)
    .bind(req.idempotency_key)
    .bind(auth.user_id.0)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| ApiError(map_sqlx_err(e)))?;

    if req.movement_type.affects_live_headcount() {
        sqlx::query(
            "UPDATE population_balances SET quantity = $3, version = version + 1, updated_by = $4 WHERE organization_id = $1 AND batch_id = $2",
        )
        .bind(auth.organization_id.0)
        .bind(batch_id)
        .bind(new_qty)
        .bind(auth.user_id.0)
        .execute(&mut *tx)
        .await
        .map_err(|e| ApiError(map_sqlx_err(e)))?;
    }

    AuditLogger::record(
        &mut tx,
        auth.organization_id,
        Some(auth.user_id),
        "stock_movement.recorded",
        "stock_movements",
        Some(movement_id),
        Some(serde_json::json!({ "population": current_qty })),
        Some(serde_json::json!({ "population": new_qty, "delta": req.quantity_delta, "movement_type": req.movement_type.as_str() })),
    )
    .await
    .map_err(ApiError)?;

    tx.commit().await.map_err(|e| ApiError(map_sqlx_err(e)))?;
    Ok((
        axum::http::StatusCode::CREATED,
        Json(StockMovementResponse {
            id: movement_id,
            batch_id,
            current_population: new_qty,
            created_at,
            created_by,
        }),
    ))
}

#[utoipa::path(
    get,
    path = "/api/v1/batches/{id}/tags",
    params(("id" = Uuid, Path, description = "Batch ID")),
    responses((status = 200, body = [IndividualTagDto])),
    tag = "Production"
)]
pub async fn list_individual_tags<S: AppBackendState>(
    State(state): State<S>,
    auth: AuthPrincipal,
    Path(batch_id): Path<Uuid>,
) -> Result<Json<Vec<IndividualTagDto>>, ApiError> {
    let mut tx = auth
        .context()
        .begin_tx(state.pool())
        .await
        .map_err(ApiError)?;
    let rows: Vec<IndividualTagDto> = sqlx::query_as(
        r#"
        SELECT id, batch_id, tag_code, tag_type, sex, birth_date, initial_weight_g, status,
               created_at, created_by, updated_at, updated_by
        FROM individual_tags
        WHERE organization_id = $1 AND batch_id = $2
        ORDER BY created_at DESC
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
    path = "/api/v1/batches/{id}/tags",
    params(("id" = Uuid, Path, description = "Batch ID")),
    request_body = RegisterTagRequest,
    responses((status = 201, body = IndividualTagDto)),
    tag = "Production"
)]
pub async fn register_individual_tag<S: AppBackendState>(
    State(state): State<S>,
    auth: AuthPrincipal,
    Path(batch_id): Path<Uuid>,
    Json(req): Json<RegisterTagRequest>,
) -> Result<(axum::http::StatusCode, Json<IndividualTagDto>), ApiError> {
    auth.require(Permission::ProductionWrite)
        .map_err(ApiError)?;
    let mut tx = auth
        .context()
        .begin_tx(state.pool())
        .await
        .map_err(ApiError)?;
    let row: IndividualTagDto = sqlx::query_as(
        r#"
        INSERT INTO individual_tags (
            organization_id, batch_id, tag_code, tag_type, sex, birth_date,
            sire_tag_id, dam_tag_id, initial_weight_g, notes, created_by
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        RETURNING id, batch_id, tag_code, tag_type, sex, birth_date, initial_weight_g, status,
                  created_at, created_by, updated_at, updated_by
        "#,
    )
    .bind(auth.organization_id.0)
    .bind(batch_id)
    .bind(req.tag_code.trim())
    .bind(req.tag_type.as_deref().unwrap_or("ear_tag"))
    .bind(req.sex.as_deref().unwrap_or("unknown"))
    .bind(req.birth_date)
    .bind(req.sire_tag_id)
    .bind(req.dam_tag_id)
    .bind(req.initial_weight_g)
    .bind(&req.notes)
    .bind(auth.user_id.0)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| ApiError(map_sqlx_err(e)))?;
    tx.commit().await.map_err(|e| ApiError(map_sqlx_err(e)))?;
    Ok((axum::http::StatusCode::CREATED, Json(row)))
}

#[utoipa::path(
    get,
    path = "/api/v1/batches/{id}/growth-samples",
    params(("id" = Uuid, Path, description = "Batch ID")),
    responses((status = 200, body = [GrowthSampleDto])),
    tag = "Production"
)]
pub async fn list_growth_samples<S: AppBackendState>(
    State(state): State<S>,
    auth: AuthPrincipal,
    Path(batch_id): Path<Uuid>,
    Query(query): Query<PaginationQuery>,
) -> Result<Json<Vec<GrowthSampleDto>>, ApiError> {
    let limit = query.limit.unwrap_or(100).clamp(1, 200);
    let mut tx = auth
        .context()
        .begin_tx(state.pool())
        .await
        .map_err(ApiError)?;
    let rows: Vec<GrowthSampleDto> = sqlx::query_as(
        r#"
        SELECT id, batch_id, sampled_at, sample_count, total_sample_weight_g,
               average_weight_g, average_length_cm, method, notes, version,
               created_at, created_by, updated_at, updated_by
        FROM growth_samples
        WHERE organization_id = $1 AND batch_id = $2
        ORDER BY sampled_at DESC, created_at DESC
        LIMIT $3
        "#,
    )
    .bind(auth.organization_id.0)
    .bind(batch_id)
    .bind(limit)
    .fetch_all(&mut *tx)
    .await
    .map_err(|e| ApiError(map_sqlx_err(e)))?;
    tx.commit().await.map_err(|e| ApiError(map_sqlx_err(e)))?;
    Ok(Json(rows))
}

#[utoipa::path(
    post,
    path = "/api/v1/batches/{id}/growth-samples",
    params(("id" = Uuid, Path, description = "Batch ID")),
    request_body = CreateGrowthSampleRequest,
    responses((status = 201, body = GrowthSampleResponse)),
    tag = "Production"
)]
pub async fn record_growth_sample<S: AppBackendState>(
    State(state): State<S>,
    auth: AuthPrincipal,
    Path(batch_id): Path<Uuid>,
    Json(req): Json<CreateGrowthSampleRequest>,
) -> Result<(axum::http::StatusCode, Json<GrowthSampleResponse>), ApiError> {
    auth.require(Permission::ProductionWrite)
        .map_err(ApiError)?;
    let abw = compute_abw_g(
        req.total_sample_weight_g,
        req.sample_count as i64,
        Utc::now(),
    )
    .map_err(|e| ApiError(AppError::Validation(e.to_string())))?;

    let mut tx = auth
        .context()
        .begin_tx(state.pool())
        .await
        .map_err(ApiError)?;
    let prev: Option<(DateTime<Utc>, Option<Decimal>)> = sqlx::query_as(
        r#"
        SELECT sampled_at, average_weight_g
        FROM growth_samples
        WHERE organization_id = $1 AND batch_id = $2 AND sampled_at < $3
        ORDER BY sampled_at DESC
        LIMIT 1
        "#,
    )
    .bind(auth.organization_id.0)
    .bind(batch_id)
    .bind(req.sampled_at)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| ApiError(map_sqlx_err(e)))?;

    let (sample_id, created_at, created_by): (Uuid, DateTime<Utc>, Option<Uuid>) = sqlx::query_as(
        r#"
        INSERT INTO growth_samples (
            organization_id, batch_id, sampled_at, sample_count, total_sample_weight_g,
            average_weight_g, average_length_cm, method, notes, created_by
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        RETURNING id, created_at, created_by
        "#,
    )
    .bind(auth.organization_id.0)
    .bind(batch_id)
    .bind(req.sampled_at)
    .bind(req.sample_count)
    .bind(req.total_sample_weight_g)
    .bind(abw.value)
    .bind(req.average_length_cm)
    .bind(&req.method)
    .bind(&req.notes)
    .bind(auth.user_id.0)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| ApiError(map_sqlx_err(e)))?;

    let pop_opt: Option<i32> = sqlx::query_scalar(
        "SELECT quantity FROM population_balances WHERE organization_id = $1 AND batch_id = $2",
    )
    .bind(auth.organization_id.0)
    .bind(batch_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| ApiError(map_sqlx_err(e)))?;

    tx.commit().await.map_err(|e| ApiError(map_sqlx_err(e)))?;

    let (adg, sgr) = if let Some((prev_at, Some(prev_abw))) = prev {
        let secs = (req.sampled_at - prev_at).num_seconds();
        if secs > 0 {
            let days = Decimal::from(secs) / Decimal::from(86_400);
            let adg_res = compute_adg_g_per_day(prev_abw, abw.value, days, None, Utc::now()).ok();
            let sgr_res =
                compute_sgr_percent_per_day(prev_abw, abw.value, days, None, Utc::now()).ok();
            (adg_res.map(|r| r.value), sgr_res.map(|r| r.value))
        } else {
            (None, None)
        }
    } else {
        (None, None)
    };

    let estimated_biomass_kg = pop_opt.and_then(|pop| {
        compute_biomass_kg(
            &CohortBiomassInput {
                estimated_population: pop as i64,
                average_body_weight_g: abw.value,
                sampled_at: Some(req.sampled_at),
            },
            None,
            Utc::now(),
        )
        .ok()
        .map(|r| r.value)
    });

    Ok((
        axum::http::StatusCode::CREATED,
        Json(GrowthSampleResponse {
            id: sample_id,
            abw_g: abw.value,
            adg_g_per_day: adg,
            sgr_percent_per_day: sgr,
            estimated_biomass_kg,
            created_at,
            created_by,
        }),
    ))
}

#[utoipa::path(
    get,
    path = "/api/v1/batches/{id}/outputs",
    params(("id" = Uuid, Path, description = "Batch ID")),
    responses((status = 200, body = [ProductionOutputDto])),
    tag = "Production"
)]
pub async fn list_outputs<S: AppBackendState>(
    State(state): State<S>,
    auth: AuthPrincipal,
    Path(batch_id): Path<Uuid>,
    Query(query): Query<PaginationQuery>,
) -> Result<Json<Vec<ProductionOutputDto>>, ApiError> {
    let limit = query.limit.unwrap_or(100).clamp(1, 200);
    let mut tx = auth
        .context()
        .begin_tx(state.pool())
        .await
        .map_err(ApiError)?;
    let rows: Vec<ProductionOutputDto> = sqlx::query_as(
        r#"
        SELECT id, batch_id, stock_movement_id, occurred_at, output_type,
               quantity, unit, average_weight_g, grade, notes, version,
               created_at, created_by, updated_at, updated_by
        FROM production_outputs
        WHERE organization_id = $1 AND batch_id = $2
        ORDER BY occurred_at DESC, created_at DESC
        LIMIT $3
        "#,
    )
    .bind(auth.organization_id.0)
    .bind(batch_id)
    .bind(limit)
    .fetch_all(&mut *tx)
    .await
    .map_err(|e| ApiError(map_sqlx_err(e)))?;
    tx.commit().await.map_err(|e| ApiError(map_sqlx_err(e)))?;
    Ok(Json(rows))
}

#[utoipa::path(
    post,
    path = "/api/v1/batches/{id}/outputs",
    params(("id" = Uuid, Path, description = "Batch ID")),
    request_body = CreateOutputRequest,
    responses((status = 201, body = OutputResponse)),
    tag = "Production"
)]
pub async fn record_output<S: AppBackendState>(
    State(state): State<S>,
    auth: AuthPrincipal,
    Path(batch_id): Path<Uuid>,
    Json(req): Json<CreateOutputRequest>,
) -> Result<(axum::http::StatusCode, Json<OutputResponse>), ApiError> {
    auth.require(Permission::ProductionWrite)
        .map_err(ApiError)?;
    let mut tx = auth
        .context()
        .begin_tx(state.pool())
        .await
        .map_err(ApiError)?;

    let mut movement_id: Option<Uuid> = None;
    if req.output_type.reduces_population_when_harvested() {
        let removed = req.headcount_removed.unwrap_or(0);
        if removed > 0 {
            let current_qty: i32 = sqlx::query_scalar(
                "SELECT quantity FROM population_balances WHERE organization_id = $1 AND batch_id = $2 FOR UPDATE",
            )
            .bind(auth.organization_id.0)
            .bind(batch_id)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| ApiError(map_sqlx_err(e)))?;

            if current_qty - removed < 0 {
                return Err(ApiError(AppError::InsufficientBalance(format!(
                    "harvest headcount {removed} exceeds available population {current_qty}"
                ))));
            }

            let mid: Uuid = sqlx::query_scalar(
                r#"
                INSERT INTO stock_movements (
                    organization_id, batch_id, movement_type, quantity_delta, occurred_at, idempotency_key, created_by
                )
                VALUES ($1, $2, 'harvest', $3, $4, $5, $6)
                RETURNING id
                "#,
            )
            .bind(auth.organization_id.0)
            .bind(batch_id)
            .bind(-removed)
            .bind(req.occurred_at)
            .bind(req.idempotency_key)
            .bind(auth.user_id.0)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| ApiError(map_sqlx_err(e)))?;

            sqlx::query(
                "UPDATE population_balances SET quantity = quantity - $3, version = version + 1, updated_by = $4 WHERE organization_id = $1 AND batch_id = $2",
            )
            .bind(auth.organization_id.0)
            .bind(batch_id)
            .bind(removed)
            .bind(auth.user_id.0)
            .execute(&mut *tx)
            .await
            .map_err(|e| ApiError(map_sqlx_err(e)))?;

            movement_id = Some(mid);
        }
    }

    let output_type_str = match req.output_type {
        OutputType::Meat => "meat",
        OutputType::Egg => "egg",
        OutputType::Milk => "milk",
        OutputType::Fry => "fry",
        OutputType::Cull => "cull",
        OutputType::Offspring => "offspring",
        OutputType::Other => "other",
    };

    let (out_id, created_at, created_by): (Uuid, DateTime<Utc>, Option<Uuid>) = sqlx::query_as(
        r#"
        INSERT INTO production_outputs (
            organization_id, batch_id, stock_movement_id, occurred_at, output_type,
            quantity, unit, average_weight_g, grade, notes, created_by
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        RETURNING id, created_at, created_by
        "#,
    )
    .bind(auth.organization_id.0)
    .bind(batch_id)
    .bind(movement_id)
    .bind(req.occurred_at)
    .bind(output_type_str)
    .bind(req.quantity)
    .bind(&req.unit)
    .bind(req.average_weight_g)
    .bind(&req.grade)
    .bind(&req.notes)
    .bind(auth.user_id.0)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| ApiError(map_sqlx_err(e)))?;

    AuditLogger::record(
        &mut tx,
        auth.organization_id,
        Some(auth.user_id),
        "output.recorded",
        "production_outputs",
        Some(out_id),
        None,
        Some(serde_json::json!({ "batch_id": batch_id, "output_type": output_type_str, "quantity": req.quantity, "unit": req.unit })),
    )
    .await
    .map_err(ApiError)?;

    tx.commit().await.map_err(|e| ApiError(map_sqlx_err(e)))?;
    Ok((
        axum::http::StatusCode::CREATED,
        Json(OutputResponse {
            id: out_id,
            batch_id,
            output_type: req.output_type,
            quantity: req.quantity,
            unit: req.unit,
            created_at,
            created_by,
        }),
    ))
}

pub fn router<S: AppBackendState>() -> Router<S> {
    Router::new()
        .route(
            "/api/v1/batches",
            get(list_batches::<S>).post(create_batch::<S>),
        )
        .route("/api/v1/batches/summary", get(list_batch_summaries::<S>))
        .route(
            "/api/v1/batches/{id}",
            get(get_batch::<S>).patch(update_batch::<S>),
        )
        .route(
            "/api/v1/batches/{id}/stock-movements",
            post(record_stock_movement::<S>),
        )
        .route(
            "/api/v1/batches/{id}/tags",
            get(list_individual_tags::<S>).post(register_individual_tag::<S>),
        )
        .route(
            "/api/v1/batches/{id}/growth-samples",
            get(list_growth_samples::<S>).post(record_growth_sample::<S>),
        )
        .route(
            "/api/v1/batches/{id}/outputs",
            get(list_outputs::<S>).post(record_output::<S>),
        )
}

#[derive(OpenApi)]
#[openapi(
    paths(
        list_batches,
        create_batch,
        list_batch_summaries,
        get_batch,
        update_batch,
        record_stock_movement,
        list_individual_tags,
        register_individual_tag,
        list_growth_samples,
        record_growth_sample,
        list_outputs,
        record_output
    ),
    components(schemas(
        CreateBatchRequest,
        UpdateBatchRequest,
        BatchDto,
        BatchDetailDto,
        CalculationResult<String>,
        StockMovementRequest,
        StockMovementResponse,
        RegisterTagRequest,
        IndividualTagDto,
        GrowthSampleDto,
        CreateGrowthSampleRequest,
        GrowthSampleResponse,
        ProductionOutputDto,
        CreateOutputRequest,
        OutputResponse,
        ErrorEnvelope
    ))
)]
pub struct ProductionApi;
