use shared_kernel::AppError;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct DbConfig {
    pub database_url: String,
    pub max_connections: u32,
}

impl DbConfig {
    pub fn from_env() -> Result<Self, AppError> {
        let database_url = std::env::var("DATABASE_URL")
            .map_err(|_| AppError::Internal("DATABASE_URL environment variable is not set".into()))?;
        let max_connections = std::env::var("DB_MAX_CONNECTIONS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(16);
        Ok(Self {
            database_url,
            max_connections,
        })
    }
}

pub async fn create_pool(cfg: &DbConfig) -> Result<PgPool, AppError> {
    PgPoolOptions::new()
        .max_connections(cfg.max_connections)
        .acquire_timeout(Duration::from_secs(5))
        .connect(&cfg.database_url)
        .await
        .map_err(|e| AppError::Database(e.to_string()))
}

pub async fn run_migrations(pool: &PgPool) -> Result<(), AppError> {
    sqlx::migrate!("../../migrations")
        .run(pool)
        .await
        .map_err(|e| AppError::Database(format!("migration failed: {e}")))
}

pub fn map_sqlx_err(err: sqlx::Error) -> AppError {
    if let Some(db_err) = err.as_database_error() {
        let code = db_err.code().map(|c| c.to_string()).unwrap_or_default();
        return match code.as_str() {
            "23505" => AppError::Conflict(db_err.message().to_string()),
            "23503" | "23514" => AppError::Validation(db_err.message().to_string()),
            _ => AppError::Database(db_err.message().to_string()),
        };
    }
    match err {
        sqlx::Error::RowNotFound => AppError::NotFound("record not found".into()),
        other => AppError::Database(other.to_string()),
    }
}
