use axum::extract::{Path as AxumPath, Query, State};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::routing::{get, post};
use axum::{Json, Router};
use futures_core::Stream;
use infra_auth::{
    ApiError, AppBackendState, AuthPrincipal, JwtConfig, JwtService, Permission,
    RegisterDeviceRequest, RegisterDeviceResponse, SyncConflictRecord, SyncPullResponse,
    SyncPushRequest, SyncPushResponse, SyncService,
};
use infra_db::{
    map_sqlx_err, resolve_custom_log_dir, sync_daily_logs_to_db, DailyLogAppender,
    RealtimeLogBroadcaster, SyncLogsResponse, SystemLogArchiveDto, SystemLogContentDto,
};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::pin::Pin;
use std::task::{Context, Poll};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use utoipa::{OpenApi, ToSchema};

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub jwt: JwtService,
    pub log_broadcaster: RealtimeLogBroadcaster,
    pub log_dir: PathBuf,
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

pub struct MpscLogStream {
    receiver: tokio::sync::mpsc::Receiver<String>,
}

impl MpscLogStream {
    pub fn new(receiver: tokio::sync::mpsc::Receiver<String>) -> Self {
        Self { receiver }
    }
}

impl Stream for MpscLogStream {
    type Item = Result<Event, Infallible>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.receiver.poll_recv(cx) {
            Poll::Ready(Some(line)) => Poll::Ready(Some(Ok(Event::default().data(line)))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ListLogsQuery {
    pub service_name: Option<String>,
    pub status: Option<String>,
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct LogContentQuery {
    pub limit: Option<usize>,
    pub tail: Option<bool>,
}

#[utoipa::path(
    get,
    path = "/api/v1/system/logs",
    params(
        ("service_name" = Option<String>, Query, description = "Filter by service name"),
        ("status" = Option<String>, Query, description = "Filter by status (active/archived)"),
        ("limit" = Option<i64>, Query, description = "Page limit (1..200)")
    ),
    responses((status = 200, body = [SystemLogArchiveDto])),
    tag = "System"
)]
async fn list_system_logs(
    State(state): State<AppState>,
    auth: AuthPrincipal,
    Query(q): Query<ListLogsQuery>,
) -> Result<Json<Vec<SystemLogArchiveDto>>, ApiError> {
    auth.require(Permission::OrgManage).map_err(ApiError)?;
    let limit = q.limit.unwrap_or(50).clamp(1, 200);

    let mut tx = auth
        .context()
        .begin_tx(&state.pool)
        .await
        .map_err(ApiError)?;
    let rows: Vec<SystemLogArchiveDto> = sqlx::query_as(
        r#"
        SELECT
            id, organization_id, log_date, service_name, file_name, file_path,
            file_size_bytes, line_count, status, checksum_sha256, archived_at, created_at, updated_at
        FROM system_log_archives
        WHERE ($1::uuid IS NULL OR organization_id IS NULL OR organization_id = $1)
          AND ($2::text IS NULL OR service_name = $2)
          AND ($3::text IS NULL OR status = $3)
        ORDER BY log_date DESC, created_at DESC
        LIMIT $4
        "#,
    )
    .bind(auth.organization_id.0)
    .bind(q.service_name)
    .bind(q.status)
    .bind(limit)
    .fetch_all(&mut *tx)
    .await
    .map_err(|e| ApiError(map_sqlx_err(e)))?;
    tx.commit().await.map_err(|e| ApiError(map_sqlx_err(e)))?;

    Ok(Json(rows))
}

#[utoipa::path(
    get,
    path = "/api/v1/system/logs/{file_name}/content",
    params(
        ("file_name" = String, Path, description = "Log file name (e.g. 27-11-2026.log)"),
        ("limit" = Option<usize>, Query, description = "Max lines to return (1..2000)"),
        ("tail" = Option<bool>, Query, description = "Return last N lines instead of first N lines")
    ),
    responses((status = 200, body = SystemLogContentDto)),
    tag = "System"
)]
async fn get_system_log_content(
    State(state): State<AppState>,
    auth: AuthPrincipal,
    AxumPath(file_name): AxumPath<String>,
    Query(q): Query<LogContentQuery>,
) -> Result<Json<SystemLogContentDto>, ApiError> {
    auth.require(Permission::OrgManage).map_err(ApiError)?;
    let res =
        infra_db::read_log_file_lines(&state.log_dir, &file_name, q.limit, q.tail.unwrap_or(true))
            .map_err(ApiError)?;
    Ok(Json(res))
}

#[utoipa::path(
    post,
    path = "/api/v1/system/logs/sync",
    responses((status = 200, body = SyncLogsResponse)),
    tag = "System"
)]
async fn sync_system_logs(
    State(state): State<AppState>,
    auth: AuthPrincipal,
) -> Result<Json<SyncLogsResponse>, ApiError> {
    auth.require(Permission::OrgManage).map_err(ApiError)?;
    let synced = sync_daily_logs_to_db(
        &state.pool,
        &state.log_dir,
        "api-server",
        Some(auth.organization_id.0),
    )
    .await
    .map_err(ApiError)?;

    Ok(Json(SyncLogsResponse {
        scanned_files: synced.len(),
        synced_archives: synced.len(),
        log_dir: state.log_dir.to_string_lossy().to_string(),
    }))
}

#[utoipa::path(
    get,
    path = "/api/v1/system/logs/stream",
    responses((status = 200, description = "Server-Sent Events text/event-stream with realtime log lines")),
    tag = "System"
)]
async fn stream_system_logs(
    State(state): State<AppState>,
    auth: AuthPrincipal,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, ApiError> {
    auth.require(Permission::OrgManage).map_err(ApiError)?;
    let (tx, rx) = tokio::sync::mpsc::channel::<String>(100);
    let mut b_rx = state.log_broadcaster.sender().subscribe();

    tokio::spawn(async move {
        loop {
            match b_rx.recv().await {
                Ok(line) => {
                    if tx.send(line).await.is_err() {
                        break;
                    }
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
                    let _ = tx
                        .send("[log buffer lagged - some lines dropped]".to_string())
                        .await;
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                    break;
                }
            }
        }
    });

    let stream = MpscLogStream::new(rx);
    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
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
    paths(
        health, sync_register_device, sync_push, sync_pull, sync_conflicts,
        list_system_logs, get_system_log_content, sync_system_logs, stream_system_logs
    ),
    components(schemas(
        HealthResponse, RegisterDeviceRequest, RegisterDeviceResponse,
        SyncPushRequest, SyncPushResponse, SyncPullResponse, SyncConflictRecord,
        SystemLogArchiveDto, SystemLogContentDto, SyncLogsResponse,
        ListLogsQuery, LogContentQuery
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
        .route("/api/v1/system/logs", get(list_system_logs))
        .route(
            "/api/v1/system/logs/{file_name}/content",
            get(get_system_log_content),
        )
        .route("/api/v1/system/logs/sync", post(sync_system_logs))
        .route("/api/v1/system/logs/stream", get(stream_system_logs))
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

    let log_dir = resolve_custom_log_dir();
    let broadcaster = RealtimeLogBroadcaster::new(500);
    let appender = DailyLogAppender::new(&log_dir, broadcaster.clone());

    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "api_server=info,tower_http=info".into()),
        )
        .with_writer(std::sync::Mutex::new(appender))
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
        log_broadcaster: broadcaster,
        log_dir,
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
        assert!(json.contains("\"/api/v1/system/logs\""));
        assert!(json.contains("\"/api/v1/system/logs/{file_name}/content\""));
        assert!(json.contains("\"/api/v1/system/logs/sync\""));
        assert!(json.contains("\"/api/v1/system/logs/stream\""));
    }
}
