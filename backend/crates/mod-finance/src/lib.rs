use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::{DateTime, NaiveDate, Utc};
use infra_auth::{ApiError, AppBackendState, AuthPrincipal, Permission};
use infra_db::{map_sqlx_err, AuditLogger};
use mod_calculation::{
    compute_batch_pnl, compute_hpp_per_kg_live, BatchPnlSummary, CalculationResult, CostBreakdown,
    HppBasis,
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use shared_kernel::AppError;
use utoipa::{OpenApi, ToSchema};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateExpenseRequest {
    pub batch_id: Option<Uuid>,
    pub farm_id: Option<Uuid>,
    pub category_id: Option<Uuid>,
    pub expense_date: NaiveDate,
    pub description: String,
    pub quantity: Decimal,
    pub unit_cost: Decimal,
    pub currency: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct ExpenseDto {
    pub id: Uuid,
    pub batch_id: Option<Uuid>,
    pub farm_id: Option<Uuid>,
    pub expense_date: NaiveDate,
    pub description: String,
    pub quantity: Decimal,
    pub unit_cost: Decimal,
    pub total_amount: Decimal,
    pub currency: String,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
    pub updated_at: DateTime<Utc>,
    pub updated_by: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SaleItemInput {
    pub production_output_id: Option<Uuid>,
    pub description: String,
    pub quantity: Decimal,
    pub unit: String,
    pub unit_price: Decimal,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateSaleRequest {
    pub batch_id: Uuid,
    pub sale_date: NaiveDate,
    pub buyer_name: Option<String>,
    pub currency: Option<String>,
    pub notes: Option<String>,
    pub items: Vec<SaleItemInput>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SaleResponse {
    pub id: Uuid,
    pub batch_id: Uuid,
    pub total_amount: Decimal,
    pub items_count: usize,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct BatchFinancialSummaryResponse {
    pub batch_id: Uuid,
    pub costs: CostBreakdown,
    pub pnl: CalculationResult<BatchPnlSummary>,
    #[schema(value_type = Option<CalculationResult<String>>)]
    pub hpp_per_kg_live: Option<CalculationResult<Decimal>>,
}

#[utoipa::path(
    post,
    path = "/api/v1/finance/expenses",
    request_body = CreateExpenseRequest,
    responses((status = 201, body = ExpenseDto)),
    tag = "Finance & HPP"
)]
pub async fn record_expense<S: AppBackendState>(
    State(state): State<S>,
    auth: AuthPrincipal,
    Json(req): Json<CreateExpenseRequest>,
) -> Result<(axum::http::StatusCode, Json<ExpenseDto>), ApiError> {
    auth.require(Permission::FinanceWrite).map_err(ApiError)?;
    let total_amount = (req.quantity * req.unit_cost).round_dp(2);
    let mut tx = auth
        .context()
        .begin_tx(state.pool())
        .await
        .map_err(ApiError)?;
    let row: ExpenseDto = sqlx::query_as(
        r#"
        INSERT INTO expenses (
            organization_id, batch_id, farm_id, category_id, expense_date,
            description, quantity, unit_cost, total_amount, currency, created_by
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        RETURNING id, batch_id, farm_id, expense_date, description, quantity, unit_cost, total_amount,
                  currency::text AS currency, created_at, created_by, updated_at, updated_by
        "#,
    )
    .bind(auth.organization_id.0)
    .bind(req.batch_id)
    .bind(req.farm_id)
    .bind(req.category_id)
    .bind(req.expense_date)
    .bind(req.description.trim())
    .bind(req.quantity)
    .bind(req.unit_cost)
    .bind(total_amount)
    .bind(req.currency.as_deref().unwrap_or("IDR"))
    .bind(auth.user_id.0)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| ApiError(map_sqlx_err(e)))?;

    AuditLogger::record(
        &mut tx,
        auth.organization_id,
        Some(auth.user_id),
        "expense.recorded",
        "expenses",
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
    path = "/api/v1/finance/sales",
    request_body = CreateSaleRequest,
    responses((status = 201, body = SaleResponse)),
    tag = "Finance & HPP"
)]
pub async fn record_sale<S: AppBackendState>(
    State(state): State<S>,
    auth: AuthPrincipal,
    Json(req): Json<CreateSaleRequest>,
) -> Result<(axum::http::StatusCode, Json<SaleResponse>), ApiError> {
    auth.require(Permission::FinanceWrite).map_err(ApiError)?;
    if req.items.is_empty() {
        return Err(ApiError(AppError::Validation(
            "sale must contain at least one item".into(),
        )));
    }
    let mut tx = auth
        .context()
        .begin_tx(state.pool())
        .await
        .map_err(ApiError)?;
    let (sale_id, created_at, created_by): (Uuid, DateTime<Utc>, Option<Uuid>) = sqlx::query_as(
        r#"
        INSERT INTO sales (organization_id, batch_id, sale_date, buyer_name, currency, notes, created_by)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id, created_at, created_by
        "#,
    )
    .bind(auth.organization_id.0)
    .bind(req.batch_id)
    .bind(req.sale_date)
    .bind(&req.buyer_name)
    .bind(req.currency.as_deref().unwrap_or("IDR"))
    .bind(&req.notes)
    .bind(auth.user_id.0)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| ApiError(map_sqlx_err(e)))?;

    let mut total_amount = Decimal::ZERO;
    for item in &req.items {
        let line_total = (item.quantity * item.unit_price).round_dp(2);
        total_amount += line_total;
        sqlx::query(
            r#"
            INSERT INTO sale_items (
                organization_id, sale_id, production_output_id, description, quantity, unit, unit_price, line_total
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(auth.organization_id.0)
        .bind(sale_id)
        .bind(item.production_output_id)
        .bind(item.description.trim())
        .bind(item.quantity)
        .bind(&item.unit)
        .bind(item.unit_price)
        .bind(line_total)
        .execute(&mut *tx)
        .await
        .map_err(|e| ApiError(map_sqlx_err(e)))?;
    }

    AuditLogger::record(
        &mut tx,
        auth.organization_id,
        Some(auth.user_id),
        "sale.recorded",
        "sales",
        Some(sale_id),
        None,
        Some(serde_json::json!({ "batch_id": req.batch_id, "total_amount": total_amount, "items_count": req.items.len() })),
    )
    .await
    .map_err(ApiError)?;

    tx.commit().await.map_err(|e| ApiError(map_sqlx_err(e)))?;
    Ok((
        axum::http::StatusCode::CREATED,
        Json(SaleResponse {
            id: sale_id,
            batch_id: req.batch_id,
            total_amount,
            items_count: req.items.len(),
            created_at,
            created_by,
        }),
    ))
}

#[utoipa::path(
    get,
    path = "/api/v1/batches/{id}/financial-summary",
    params(("id" = Uuid, Path, description = "Batch ID")),
    responses((status = 200, body = BatchFinancialSummaryResponse)),
    tag = "Finance & HPP"
)]
pub async fn get_batch_financial_summary<S: AppBackendState>(
    State(state): State<S>,
    auth: AuthPrincipal,
    Path(batch_id): Path<Uuid>,
) -> Result<Json<BatchFinancialSummaryResponse>, ApiError> {
    auth.require(Permission::AnalyticsRead).map_err(ApiError)?;
    let mut tx = auth
        .context()
        .begin_tx(state.pool())
        .await
        .map_err(ApiError)?;

    let feed_cost: Decimal = sqlx::query_scalar(
        "SELECT COALESCE(SUM(quantity * COALESCE(unit_cost, 0)), 0) FROM feeding_records WHERE organization_id = $1 AND batch_id = $2",
    )
    .bind(auth.organization_id.0)
    .bind(batch_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| ApiError(map_sqlx_err(e)))?;

    let direct_expenses: Decimal = sqlx::query_scalar(
        "SELECT COALESCE(SUM(total_amount), 0) FROM expenses WHERE organization_id = $1 AND batch_id = $2",
    )
    .bind(auth.organization_id.0)
    .bind(batch_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| ApiError(map_sqlx_err(e)))?;

    let revenue: Decimal = sqlx::query_scalar(
        r#"
        SELECT COALESCE(SUM(si.line_total), 0)
        FROM sale_items si
        JOIN sales s ON s.id = si.sale_id AND s.organization_id = si.organization_id
        WHERE s.organization_id = $1 AND s.batch_id = $2
        "#,
    )
    .bind(auth.organization_id.0)
    .bind(batch_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| ApiError(map_sqlx_err(e)))?;

    let harvested_live_kg: Decimal = sqlx::query_scalar(
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

    let standing_pop: i32 = sqlx::query_scalar(
        "SELECT quantity FROM population_balances WHERE organization_id = $1 AND batch_id = $2",
    )
    .bind(auth.organization_id.0)
    .bind(batch_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| ApiError(map_sqlx_err(e)))?
    .unwrap_or(0);

    let latest_abw_g: Decimal = sqlx::query_scalar(
        r#"
        SELECT COALESCE(average_weight_g, 0)
        FROM growth_samples
        WHERE organization_id = $1 AND batch_id = $2
        ORDER BY sampled_at DESC LIMIT 1
        "#,
    )
    .bind(auth.organization_id.0)
    .bind(batch_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| ApiError(map_sqlx_err(e)))?
    .unwrap_or(Decimal::ZERO);

    tx.commit().await.map_err(|e| ApiError(map_sqlx_err(e)))?;

    let costs = CostBreakdown {
        stock_cost: Decimal::ZERO,
        feed_cost: feed_cost.round_dp(2),
        health_and_vaccine_cost: Decimal::ZERO,
        operational_cost: direct_expenses.round_dp(2),
    };
    let now = Utc::now();
    let pnl = compute_batch_pnl(&costs, revenue, now)
        .map_err(|e| ApiError(AppError::Validation(e.to_string())))?;

    let standing_kg = (Decimal::from(standing_pop) * latest_abw_g) / Decimal::from(1000);
    let basis = if harvested_live_kg > Decimal::ZERO && standing_kg == Decimal::ZERO {
        HppBasis::RealizedHarvestOnly
    } else {
        HppBasis::RealizedPlusStandingBiomass
    };
    let hpp_per_kg_live =
        compute_hpp_per_kg_live(&costs, harvested_live_kg, standing_kg, basis, now).ok();

    Ok(Json(BatchFinancialSummaryResponse {
        batch_id,
        costs,
        pnl,
        hpp_per_kg_live,
    }))
}

pub fn router<S: AppBackendState>() -> Router<S> {
    Router::new()
        .route("/api/v1/finance/expenses", post(record_expense::<S>))
        .route("/api/v1/finance/sales", post(record_sale::<S>))
        .route(
            "/api/v1/batches/{id}/financial-summary",
            get(get_batch_financial_summary::<S>),
        )
}

#[derive(OpenApi)]
#[openapi(
    paths(record_expense, record_sale, get_batch_financial_summary),
    components(schemas(
        CreateExpenseRequest, ExpenseDto, SaleItemInput, CreateSaleRequest, SaleResponse,
        CostBreakdown, BatchPnlSummary, CalculationResult<BatchPnlSummary>,
        CalculationResult<String>, BatchFinancialSummaryResponse
    ))
)]
pub struct FinanceApi;
