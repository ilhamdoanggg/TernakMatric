use axum::extract::{Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use infra_auth::{
    ApiError, AppBackendState, AuthPrincipal, JwtConfig, JwtService, RegisterDeviceRequest,
    RegisterDeviceResponse, SyncConflictRecord, SyncPullResponse, SyncPushRequest,
    SyncPushResponse, SyncService,
};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use utoipa::{OpenApi, ToSchema};

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub jwt: JwtService,
}

impl AppBackendState for AppState {
    fn pool(&self) -> &PgPool {
        &self.pool
    }

    fn jwt(&self) -> &JwtService {
        &self.jwt
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct HealthResponse {
    pub status: String,
    pub service: String,
}

#[derive(Debug, Deserialize)]
pub struct PullQuery {
    pub cursor: Option<i64>,
    pub limit: Option<i64>,
}

#[utoipa::path(
    get,
    path = "/health",
    responses((status = 200, body = HealthResponse)),
    tag = "System"
)]
async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".into(),
        service: "ternakmatric-api-server".into(),
    })
}

#[utoipa::path(
    post,
    path = "/api/v1/sync/devices",
    request_body = RegisterDeviceRequest,
    responses((status = 201, body = RegisterDeviceResponse)),
    tag = "Offline Sync"
)]
async fn sync_register_device(
    State(state): State<AppState>,
    auth: AuthPrincipal,
    Json(req): Json<RegisterDeviceRequest>,
) -> Result<(axum::http::StatusCode, Json<RegisterDeviceResponse>), ApiError> {
    let res = SyncService::register_device(&state.pool, &auth.context(), req)
        .await
        .map_err(ApiError)?;
    Ok((axum::http::StatusCode::CREATED, Json(res)))
}

#[utoipa::path(
    post,
    path = "/api/v1/sync/push",
    request_body = SyncPushRequest,
    responses((status = 200, body = SyncPushResponse)),
    tag = "Offline Sync"
)]
async fn sync_push(
    State(state): State<AppState>,
    auth: AuthPrincipal,
    Json(req): Json<SyncPushRequest>,
) -> Result<Json<SyncPushResponse>, ApiError> {
    let res = SyncService::push_commands(&state.pool, &auth.context(), req)
        .await
        .map_err(ApiError)?;
    Ok(Json(res))
}

#[utoipa::path(
    get,
    path = "/api/v1/sync/pull",
    responses((status = 200, body = SyncPullResponse)),
    tag = "Offline Sync"
)]
async fn sync_pull(
    State(state): State<AppState>,
    auth: AuthPrincipal,
    Query(q): Query<PullQuery>,
) -> Result<Json<SyncPullResponse>, ApiError> {
    let res = SyncService::pull_changes(
        &state.pool,
        &auth.context(),
        q.cursor.unwrap_or(0),
        q.limit.unwrap_or(100),
    )
    .await
    .map_err(ApiError)?;
    Ok(Json(res))
}

#[utoipa::path(
    get,
    path = "/api/v1/sync/conflicts",
    responses((status = 200, body = [SyncConflictRecord])),
    tag = "Offline Sync"
)]
async fn sync_conflicts(
    State(state): State<AppState>,
    auth: AuthPrincipal,
) -> Result<Json<Vec<SyncConflictRecord>>, ApiError> {
    let rows = SyncService::list_conflicts(&state.pool, &auth.context())
        .await
        .map_err(ApiError)?;
    Ok(Json(rows))
}

struct MergeModuleApis;

impl utoipa::Modify for MergeModuleApis {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        for module_api in [
            mod_tenant::TenantApi::openapi(),
            mod_enclosure::EnclosureApi::openapi(),
            mod_species::SpeciesApi::openapi(),
            mod_production::ProductionApi::openapi(),
            mod_feeding::FeedingApi::openapi(),
            mod_environment::EnvironmentApi::openapi(),
            mod_health::HealthApi::openapi(),
            mod_finance::FinanceApi::openapi(),
        ] {
            openapi.merge(module_api);
        }
    }
}

#[derive(OpenApi)]
#[openapi(
    info(
        title = "TernakMatric API",
        version = "1.0.0",
        description = "Multi-tenant Smart Animal Production Management Platform API"
    ),
    paths(health, sync_register_device, sync_push, sync_pull, sync_conflicts),
    components(schemas(
        HealthResponse, RegisterDeviceRequest, RegisterDeviceResponse,
        SyncPushRequest, SyncPushResponse, SyncPullResponse, SyncConflictRecord
    )),
    modifiers(&MergeModuleApis)
)]
pub struct ApiDoc;

async fn openapi_json() -> Json<utoipa::openapi::OpenApi> {
    Json(ApiDoc::openapi())
}

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/api-docs/openapi.json", get(openapi_json))
        .route("/api/v1/sync/devices", post(sync_register_device))
        .route("/api/v1/sync/push", post(sync_push))
        .route("/api/v1/sync/pull", get(sync_pull))
        .route("/api/v1/sync/conflicts", get(sync_conflicts))
        .merge(mod_tenant::router::<AppState>())
        .merge(mod_enclosure::router::<AppState>())
        .merge(mod_species::router::<AppState>())
        .merge(mod_production::router::<AppState>())
        .merge(mod_feeding::router::<AppState>())
        .merge(mod_environment::router::<AppState>())
        .merge(mod_health::router::<AppState>())
        .merge(mod_finance::router::<AppState>())
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    infra_db::load_env_files();

    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "api_server=info,tower_http=info".into()),
        )
        .init();

    let args: Vec<String> = std::env::args().collect();
    if let Some(pos) = args.iter().position(|a| a == "--export-openapi") {
        let out_path = args
            .get(pos + 1)
            .map(|s| s.as_str())
            .unwrap_or("../packages/api-contracts/openapi/ternakmatric.v1.json");
        let spec = serde_json::to_string_pretty(&ApiDoc::openapi())?;
        if let Some(parent) = std::path::Path::new(out_path).parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(out_path, spec)?;
        println!("Exported OpenAPI specification to {out_path}");
        return Ok(());
    }

    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://ternakmatric:ternakmatric@localhost:5432/ternakmatric".into()
    });
    let pool = PgPoolOptions::new()
        .max_connections(16)
        .connect_lazy(&db_url)?;

    let state = AppState {
        pool,
        jwt: JwtService::new(JwtConfig::from_env()),
    };

    let addr: SocketAddr = std::env::var("BIND_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:8080".into())
        .parse()?;
    tracing::info!("TernakMatric API server listening on http://{addr}");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, build_router(state)).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openapi_spec_builds_with_all_module_paths() {
        let spec = ApiDoc::openapi();
        let json = serde_json::to_string(&spec).unwrap();
        assert!(!json.contains("\"//"));
        assert!(json.contains("\"/api/v1/unit-types\""));
        assert!(json.contains("\"/api/v1/production-systems\""));
        assert!(json.contains("\"/api/v1/enclosures\""));
        assert!(json.contains("\"/api/v1/batches\""));
        assert!(json.contains("\"/api/v1/batches/summary\""));
        assert!(json.contains("\"/api/v1/batches/{id}\""));
        assert!(json.contains("\"get_batch\""));
        assert!(json.contains("\"/api/v1/batches/{id}/growth-samples\""));
        assert!(json.contains("\"list_growth_samples\""));
        assert!(json.contains("\"/api/v1/batches/{id}/outputs\""));
        assert!(json.contains("\"list_outputs\""));
        assert!(json.contains("\"/api/v1/inventory/balances\""));
        assert!(json.contains("\"/api/v1/feeding\""));
        assert!(json.contains("\"/api/v1/environment/readings\""));
        assert!(json.contains("\"/api/v1/batches/{id}/vaccinations\""));
        assert!(json.contains("\"/api/v1/batches/{id}/health-records\""));
        assert!(json.contains("\"list_health_records\""));
        assert!(json.contains("\"/api/v1/batches/{id}/mortalities\""));
        assert!(json.contains("\"list_mortalities\""));
        assert!(json.contains("\"/api/v1/finance/expense-categories\""));
        assert!(json.contains("\"/api/v1/finance/expenses\""));
        assert!(json.contains("\"list_expenses\""));
        assert!(json.contains("\"/api/v1/finance/sales\""));
        assert!(json.contains("\"list_sales\""));
        assert!(json.contains("\"/api/v1/batches/{id}/financial-summary\""));
        assert!(json.contains("\"/api/v1/sync/push\""));
        assert!(json.contains("\"/api/v1/audit-logs\""));
        assert!(json.contains("\"/api/v1/farms/{id}\""));
    }
}
