use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::{DateTime, NaiveDate, Utc};
use infra_auth::{ApiError, AppBackendState, AuthPrincipal, Permission};
use infra_db::map_sqlx_err;
use mod_calculation::{compute_fcr, CalculationResult, FcrInput, FcrMode, FcrSummary};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use shared_kernel::{convert_mass_to_kg, AppError};
use utoipa::{OpenApi, ToSchema};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateWarehouseRequest {
    pub farm_id: Option<Uuid>,
    pub code: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct WarehouseDto {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub farm_id: Option<Uuid>,
    pub code: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateFeedProductRequest {
    pub sku: String,
    pub name: String,
    pub brand: Option<String>,
    pub pellet_size_mm: Option<Decimal>,
    pub protein_percent: Option<Decimal>,
    pub default_unit: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct FeedProductDto {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub sku: String,
    pub name: String,
    pub brand: Option<String>,
    pub pellet_size_mm: Option<Decimal>,
    pub protein_percent: Option<Decimal>,
    pub default_unit: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct InventoryLineInput {
    pub feed_product_id: Uuid,
    pub quantity: Decimal,
    pub unit: String,
    pub unit_cost: Option<Decimal>,
    pub lot_code: Option<String>,
    pub expiry_date: Option<NaiveDate>,
    pub batch_id: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateInventoryTxRequest {
    pub warehouse_id: Uuid,
    pub transaction_type: String,
    pub occurred_at: DateTime<Utc>,
    pub reference_code: Option<String>,
    pub notes: Option<String>,
    pub idempotency_key: Uuid,
    pub lines: Vec<InventoryLineInput>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct InventoryTxResponse {
    pub id: Uuid,
    pub transaction_type: String,
    pub lines_count: usize,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct RecordFeedingRequest {
    pub batch_id: Uuid,
    pub enclosure_id: Uuid,
    pub warehouse_id: Option<Uuid>,
    pub feed_product_id: Uuid,
    pub lot_code: Option<String>,
    pub fed_at: DateTime<Utc>,
    pub quantity: Decimal,
    pub unit: String,
    pub unit_cost: Option<Decimal>,
    pub feeding_method: Option<String>,
    pub notes: Option<String>,
    pub idempotency_key: Uuid,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct FeedingRecordResponse {
    pub id: Uuid,
    pub batch_id: Uuid,
    pub quantity_kg: Decimal,
}

#[derive(Debug, Deserialize)]
pub struct FcrQuery {
    pub mode: Option<FcrMode>,
}

#[utoipa::path(
    get,
    path = "/api/v1/warehouses",
    responses((status = 200, body = [WarehouseDto])),
    tag = "Feeding & Inventory"
)]
pub async fn list_warehouses<S: AppBackendState>(
    State(state): State<S>,
    auth: AuthPrincipal,
) -> Result<Json<Vec<WarehouseDto>>, ApiError> {
    let mut tx = auth.context().begin_tx(state.pool()).await.map_err(ApiError)?;
    let rows: Vec<WarehouseDto> = sqlx::query_as(
        "SELECT id, organization_id, farm_id, code, name, created_at FROM warehouses WHERE organization_id = $1 ORDER BY created_at DESC",
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
    path = "/api/v1/warehouses",
    request_body = CreateWarehouseRequest,
    responses((status = 201, body = WarehouseDto)),
    tag = "Feeding & Inventory"
)]
pub async fn create_warehouse<S: AppBackendState>(
    State(state): State<S>,
    auth: AuthPrincipal,
    Json(req): Json<CreateWarehouseRequest>,
) -> Result<(axum::http::StatusCode, Json<WarehouseDto>), ApiError> {
    auth.require(Permission::FeedingWrite).map_err(ApiError)?;
    let mut tx = auth.context().begin_tx(state.pool()).await.map_err(ApiError)?;
    let row: WarehouseDto = sqlx::query_as(
        r#"
        INSERT INTO warehouses (organization_id, farm_id, code, name)
        VALUES ($1, $2, $3, $4)
        RETURNING id, organization_id, farm_id, code, name, created_at
        "#,
    )
    .bind(auth.organization_id.0)
    .bind(req.farm_id)
    .bind(req.code.trim())
    .bind(req.name.trim())
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| ApiError(map_sqlx_err(e)))?;
    tx.commit().await.map_err(|e| ApiError(map_sqlx_err(e)))?;
    Ok((axum::http::StatusCode::CREATED, Json(row)))
}

#[utoipa::path(
    get,
    path = "/api/v1/feed-products",
    responses((status = 200, body = [FeedProductDto])),
    tag = "Feeding & Inventory"
)]
pub async fn list_feed_products<S: AppBackendState>(
    State(state): State<S>,
    auth: AuthPrincipal,
) -> Result<Json<Vec<FeedProductDto>>, ApiError> {
    let mut tx = auth.context().begin_tx(state.pool()).await.map_err(ApiError)?;
    let rows: Vec<FeedProductDto> = sqlx::query_as(
        r#"
        SELECT id, organization_id, sku, name, brand, pellet_size_mm, protein_percent, default_unit, is_active, created_at
        FROM feed_products
        WHERE organization_id = $1 AND is_active = true
        ORDER BY name ASC
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
    path = "/api/v1/feed-products",
    request_body = CreateFeedProductRequest,
    responses((status = 201, body = FeedProductDto)),
    tag = "Feeding & Inventory"
)]
pub async fn create_feed_product<S: AppBackendState>(
    State(state): State<S>,
    auth: AuthPrincipal,
    Json(req): Json<CreateFeedProductRequest>,
) -> Result<(axum::http::StatusCode, Json<FeedProductDto>), ApiError> {
    auth.require(Permission::FeedingWrite).map_err(ApiError)?;
    let mut tx = auth.context().begin_tx(state.pool()).await.map_err(ApiError)?;
    let row: FeedProductDto = sqlx::query_as(
        r#"
        INSERT INTO feed_products (organization_id, sku, name, brand, pellet_size_mm, protein_percent, default_unit)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id, organization_id, sku, name, brand, pellet_size_mm, protein_percent, default_unit, is_active, created_at
        "#,
    )
    .bind(auth.organization_id.0)
    .bind(req.sku.trim())
    .bind(req.name.trim())
    .bind(&req.brand)
    .bind(req.pellet_size_mm)
    .bind(req.protein_percent)
    .bind(req.default_unit.as_deref().unwrap_or("kg"))
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| ApiError(map_sqlx_err(e)))?;
    tx.commit().await.map_err(|e| ApiError(map_sqlx_err(e)))?;
    Ok((axum::http::StatusCode::CREATED, Json(row)))
}

#[utoipa::path(
    post,
    path = "/api/v1/inventory/transactions",
    request_body = CreateInventoryTxRequest,
    responses((status = 201, body = InventoryTxResponse)),
    tag = "Feeding & Inventory"
)]
pub async fn create_inventory_transaction<S: AppBackendState>(
    State(state): State<S>,
    auth: AuthPrincipal,
    Json(req): Json<CreateInventoryTxRequest>,
) -> Result<(axum::http::StatusCode, Json<InventoryTxResponse>), ApiError> {
    auth.require(Permission::FeedingWrite).map_err(ApiError)?;
    if req.lines.is_empty() {
        return Err(ApiError(AppError::Validation(
            "inventory transaction requires at least one line".into(),
        )));
    }

    let mut tx = auth.context().begin_tx(state.pool()).await.map_err(ApiError)?;
    let tx_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO inventory_transactions (
            organization_id, warehouse_id, transaction_type, status, occurred_at,
            reference_code, notes, idempotency_key, created_by
        )
        VALUES ($1, $2, $3, 'posted', $4, $5, $6, $7, $8)
        RETURNING id
        "#,
    )
    .bind(auth.organization_id.0)
    .bind(req.warehouse_id)
    .bind(&req.transaction_type)
    .bind(req.occurred_at)
    .bind(&req.reference_code)
    .bind(&req.notes)
    .bind(req.idempotency_key)
    .bind(auth.user_id.0)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| ApiError(map_sqlx_err(e)))?;

    for line in &req.lines {
        let lot = line.lot_code.as_deref().unwrap_or("DEFAULT");
        let custom_factor: Option<Decimal> = sqlx::query_scalar(
            r#"
            SELECT factor FROM feed_unit_conversions
            WHERE organization_id = $1 AND feed_product_id = $2 AND from_unit = $3 AND to_unit = 'kg'
            "#,
        )
        .bind(auth.organization_id.0)
        .bind(line.feed_product_id)
        .bind(&line.unit)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| ApiError(map_sqlx_err(e)))?;

        let qty_kg = convert_mass_to_kg(line.quantity, &line.unit, custom_factor).map_err(ApiError)?;
        let delta = match req.transaction_type.as_str() {
            "receipt" | "return" => qty_kg,
            "issue" | "transfer" => -qty_kg,
            "adjustment" => qty_kg,
            other => {
                return Err(ApiError(AppError::Validation(format!(
                    "unsupported transaction_type '{other}'"
                ))));
            }
        };

        let current_bal: Option<Decimal> = sqlx::query_scalar(
            r#"
            SELECT quantity FROM inventory_balances
            WHERE organization_id = $1 AND warehouse_id = $2 AND feed_product_id = $3 AND lot_code = $4
            FOR UPDATE
            "#,
        )
        .bind(auth.organization_id.0)
        .bind(req.warehouse_id)
        .bind(line.feed_product_id)
        .bind(lot)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| ApiError(map_sqlx_err(e)))?;

        let next_bal = current_bal.unwrap_or(Decimal::ZERO) + delta;
        if next_bal < Decimal::ZERO {
            return Err(ApiError(AppError::InsufficientBalance(format!(
                "warehouse inventory cannot go negative (available: {}, requested delta: {delta})",
                current_bal.unwrap_or(Decimal::ZERO)
            ))));
        }

        sqlx::query(
            r#"
            INSERT INTO inventory_transaction_lines (
                organization_id, transaction_id, feed_product_id, quantity, unit, unit_cost, lot_code, expiry_date, batch_id
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
        )
        .bind(auth.organization_id.0)
        .bind(tx_id)
        .bind(line.feed_product_id)
        .bind(line.quantity)
        .bind(&line.unit)
        .bind(line.unit_cost)
        .bind(lot)
        .bind(line.expiry_date)
        .bind(line.batch_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| ApiError(map_sqlx_err(e)))?;

        sqlx::query(
            r#"
            INSERT INTO inventory_balances (organization_id, warehouse_id, feed_product_id, lot_code, quantity)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (organization_id, warehouse_id, feed_product_id, lot_code)
            DO UPDATE SET quantity = EXCLUDED.quantity, version = inventory_balances.version + 1
            "#,
        )
        .bind(auth.organization_id.0)
        .bind(req.warehouse_id)
        .bind(line.feed_product_id)
        .bind(lot)
        .bind(next_bal)
        .execute(&mut *tx)
        .await
        .map_err(|e| ApiError(map_sqlx_err(e)))?;
    }

    tx.commit().await.map_err(|e| ApiError(map_sqlx_err(e)))?;
    Ok((
        axum::http::StatusCode::CREATED,
        Json(InventoryTxResponse {
            id: tx_id,
            transaction_type: req.transaction_type,
            lines_count: req.lines.len(),
        }),
    ))
}

#[utoipa::path(
    post,
    path = "/api/v1/feeding",
    request_body = RecordFeedingRequest,
    responses((status = 201, body = FeedingRecordResponse)),
    tag = "Feeding & Inventory"
)]
pub async fn record_feeding<S: AppBackendState>(
    State(state): State<S>,
    auth: AuthPrincipal,
    Json(req): Json<RecordFeedingRequest>,
) -> Result<(axum::http::StatusCode, Json<FeedingRecordResponse>), ApiError> {
    auth.require(Permission::FeedingWrite).map_err(ApiError)?;
    let mut tx = auth.context().begin_tx(state.pool()).await.map_err(ApiError)?;

    let custom_factor: Option<Decimal> = sqlx::query_scalar(
        r#"
        SELECT factor FROM feed_unit_conversions
        WHERE organization_id = $1 AND feed_product_id = $2 AND from_unit = $3 AND to_unit = 'kg'
        "#,
    )
    .bind(auth.organization_id.0)
    .bind(req.feed_product_id)
    .bind(&req.unit)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| ApiError(map_sqlx_err(e)))?;

    let qty_kg = convert_mass_to_kg(req.quantity, &req.unit, custom_factor).map_err(ApiError)?;
    let mut inv_line_id: Option<Uuid> = None;

    if let Some(wh_id) = req.warehouse_id {
        let lot = req.lot_code.as_deref().unwrap_or("DEFAULT");
        let bal: Decimal = sqlx::query_scalar(
            r#"
            SELECT quantity FROM inventory_balances
            WHERE organization_id = $1 AND warehouse_id = $2 AND feed_product_id = $3 AND lot_code = $4
            FOR UPDATE
            "#,
        )
        .bind(auth.organization_id.0)
        .bind(wh_id)
        .bind(req.feed_product_id)
        .bind(lot)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| ApiError(map_sqlx_err(e)))?
        .unwrap_or(Decimal::ZERO);

        if bal - qty_kg < Decimal::ZERO {
            return Err(ApiError(AppError::InsufficientBalance(format!(
                "insufficient feed stock in warehouse (available: {bal} kg, requested: {qty_kg} kg)"
            ))));
        }

        let inv_tx_id: Uuid = sqlx::query_scalar(
            r#"
            INSERT INTO inventory_transactions (
                organization_id, warehouse_id, transaction_type, status, occurred_at, notes, idempotency_key, created_by
            )
            VALUES ($1, $2, 'issue', 'posted', $3, $4, $5, $6)
            RETURNING id
            "#,
        )
        .bind(auth.organization_id.0)
        .bind(wh_id)
        .bind(req.fed_at)
        .bind(&req.notes)
        .bind(req.idempotency_key)
        .bind(auth.user_id.0)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| ApiError(map_sqlx_err(e)))?;

        let line_id: Uuid = sqlx::query_scalar(
            r#"
            INSERT INTO inventory_transaction_lines (
                organization_id, transaction_id, feed_product_id, quantity, unit, unit_cost, lot_code, batch_id
            )
            VALUES ($1, $2, $3, $4, 'kg', $5, $6, $7)
            RETURNING id
            "#,
        )
        .bind(auth.organization_id.0)
        .bind(inv_tx_id)
        .bind(req.feed_product_id)
        .bind(qty_kg)
        .bind(req.unit_cost)
        .bind(lot)
        .bind(req.batch_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| ApiError(map_sqlx_err(e)))?;

        sqlx::query(
            r#"
            UPDATE inventory_balances
            SET quantity = quantity - $5, version = version + 1
            WHERE organization_id = $1 AND warehouse_id = $2 AND feed_product_id = $3 AND lot_code = $4
            "#,
        )
        .bind(auth.organization_id.0)
        .bind(wh_id)
        .bind(req.feed_product_id)
        .bind(lot)
        .bind(qty_kg)
        .execute(&mut *tx)
        .await
        .map_err(|e| ApiError(map_sqlx_err(e)))?;

        inv_line_id = Some(line_id);
    }

    let feeding_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO feeding_records (
            organization_id, batch_id, unit_id, warehouse_id, feed_product_id, inventory_line_id,
            fed_at, quantity, unit, unit_cost, feeding_method, notes, idempotency_key, created_by
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
        RETURNING id
        "#,
    )
    .bind(auth.organization_id.0)
    .bind(req.batch_id)
    .bind(req.enclosure_id)
    .bind(req.warehouse_id)
    .bind(req.feed_product_id)
    .bind(inv_line_id)
    .bind(req.fed_at)
    .bind(req.quantity)
    .bind(&req.unit)
    .bind(req.unit_cost)
    .bind(&req.feeding_method)
    .bind(&req.notes)
    .bind(req.idempotency_key)
    .bind(auth.user_id.0)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| ApiError(map_sqlx_err(e)))?;

    tx.commit().await.map_err(|e| ApiError(map_sqlx_err(e)))?;
    Ok((
        axum::http::StatusCode::CREATED,
        Json(FeedingRecordResponse {
            id: feeding_id,
            batch_id: req.batch_id,
            quantity_kg: qty_kg,
        }),
    ))
}

#[utoipa::path(
    get,
    path = "/api/v1/batches/{id}/fcr",
    params(("id" = Uuid, Path, description = "Batch ID")),
    responses((status = 200, body = CalculationResult<FcrSummary>)),
    tag = "Feeding & Inventory"
)]
pub async fn get_batch_fcr<S: AppBackendState>(
    State(state): State<S>,
    auth: AuthPrincipal,
    Path(batch_id): Path<Uuid>,
    Query(query): Query<FcrQuery>,
) -> Result<Json<CalculationResult<FcrSummary>>, ApiError> {
    let mut tx = auth.context().begin_tx(state.pool()).await.map_err(ApiError)?;
    let total_feed_kg: Decimal = sqlx::query_scalar(
        r#"
        SELECT COALESCE(SUM(
            CASE WHEN unit = 'g' THEN quantity / 1000 ELSE quantity END
        ), 0)
        FROM feeding_records
        WHERE organization_id = $1 AND batch_id = $2
        "#,
    )
    .bind(auth.organization_id.0)
    .bind(batch_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| ApiError(map_sqlx_err(e)))?;

    let (init_count, init_abw): (i32, Option<Decimal>) = sqlx::query_as(
        "SELECT initial_count, initial_avg_weight_g FROM production_batches WHERE organization_id = $1 AND id = $2",
    )
    .bind(auth.organization_id.0)
    .bind(batch_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| ApiError(map_sqlx_err(e)))?;

    let alive_pop: i32 = sqlx::query_scalar(
        "SELECT quantity FROM population_balances WHERE organization_id = $1 AND batch_id = $2",
    )
    .bind(auth.organization_id.0)
    .bind(batch_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| ApiError(map_sqlx_err(e)))?
    .unwrap_or(init_count);

    let latest_sample: Option<(DateTime<Utc>, Option<Decimal>)> = sqlx::query_as(
        r#"
        SELECT sampled_at, average_weight_g FROM growth_samples
        WHERE organization_id = $1 AND batch_id = $2
        ORDER BY sampled_at DESC LIMIT 1
        "#,
    )
    .bind(auth.organization_id.0)
    .bind(batch_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| ApiError(map_sqlx_err(e)))?;

    let harvested_kg: Decimal = sqlx::query_scalar(
        r#"
        SELECT COALESCE(SUM(CASE WHEN unit = 'g' THEN quantity / 1000 ELSE quantity END), 0)
        FROM production_outputs
        WHERE organization_id = $1 AND batch_id = $2 AND output_type IN ('meat','fry','cull') AND unit IN ('kg','g')
        "#,
    )
    .bind(auth.organization_id.0)
    .bind(batch_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| ApiError(map_sqlx_err(e)))?;

    tx.commit().await.map_err(|e| ApiError(map_sqlx_err(e)))?;

    let initial_biomass_kg =
        (Decimal::from(init_count) * init_abw.unwrap_or(Decimal::ZERO)) / Decimal::from(1000);
    let current_abw_g = latest_sample
        .as_ref()
        .and_then(|s| s.1)
        .unwrap_or(init_abw.unwrap_or(Decimal::ZERO));
    let current_standing_biomass_kg =
        (Decimal::from(alive_pop) * current_abw_g) / Decimal::from(1000);

    let res = compute_fcr(
        &FcrInput {
            total_feed_kg,
            initial_biomass_kg,
            added_stock_biomass_kg: Decimal::ZERO,
            current_standing_biomass_kg,
            harvested_biomass_kg: harvested_kg,
            mortality_and_cull_biomass_kg: Decimal::ZERO,
            mode: query.mode.unwrap_or(FcrMode::Economic),
            last_sample_at: latest_sample.map(|s| s.0),
        },
        None,
        Utc::now(),
    )
    .map_err(|e| ApiError(AppError::Validation(e.to_string())))?;

    Ok(Json(res))
}

pub fn router<S: AppBackendState>() -> Router<S> {
    Router::new()
        .route(
            "/api/v1/warehouses",
            get(list_warehouses::<S>).post(create_warehouse::<S>),
        )
        .route(
            "/api/v1/feed-products",
            get(list_feed_products::<S>).post(create_feed_product::<S>),
        )
        .route(
            "/api/v1/inventory/transactions",
            post(create_inventory_transaction::<S>),
        )
        .route("/api/v1/feeding", post(record_feeding::<S>))
        .route("/api/v1/batches/{id}/fcr", get(get_batch_fcr::<S>))
}

#[derive(OpenApi)]
#[openapi(
    paths(list_warehouses, create_warehouse, list_feed_products, create_feed_product, create_inventory_transaction, record_feeding, get_batch_fcr),
    components(schemas(
        CreateWarehouseRequest, WarehouseDto, CreateFeedProductRequest, FeedProductDto,
        InventoryLineInput, CreateInventoryTxRequest, InventoryTxResponse,
        RecordFeedingRequest, FeedingRecordResponse, FcrSummary, CalculationResult<FcrSummary>
    ))
)]
pub struct FeedingApi;
