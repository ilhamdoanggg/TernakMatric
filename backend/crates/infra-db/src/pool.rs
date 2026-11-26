use shared_kernel::AppError;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::time::Duration;

/// Returns candidate `.env` filenames ordered from highest to lowest precedence
/// (`ENV_FILE`, `.env.<APP_ENV>`, `.env`).
pub fn env_file_candidates(env_file: Option<&str>, app_env: Option<&str>) -> Vec<String> {
    let mut files = Vec::new();
    if let Some(explicit) = env_file.map(str::trim).filter(|s| !s.is_empty()) {
        files.push(explicit.to_string());
    }
    if let Some(env_name) = app_env.map(str::trim).filter(|s| !s.is_empty()) {
        let candidate = format!(".env.{env_name}");
        if !files.contains(&candidate) {
            files.push(candidate);
        }
    }
    if !files.iter().any(|f| f == ".env") {
        files.push(".env".to_string());
    }
    files
}

/// Optionally loads `.env` files for local/non-container execution without
/// overwriting environment variables already set by the OS or container runtime.
pub fn load_env_files() {
    let explicit = std::env::var("ENV_FILE").ok();
    let app_env = std::env::var("APP_ENV").ok();
    for path in env_file_candidates(explicit.as_deref(), app_env.as_deref()) {
        dotenvy::from_filename(&path).ok();
    }
}

#[derive(Debug, Clone)]
pub struct DbConfig {
    pub database_url: String,
    pub max_connections: u32,
}

impl DbConfig {
    pub fn from_env() -> Result<Self, AppError> {
        let database_url = std::env::var("DATABASE_URL").map_err(|_| {
            AppError::Internal("DATABASE_URL environment variable is not set".into())
        })?;
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

#[cfg(test)]
mod tests {
    use super::env_file_candidates;

    #[test]
    fn test_env_file_candidates_precedence() {
        assert_eq!(env_file_candidates(None, None), vec![".env".to_string()]);
        assert_eq!(
            env_file_candidates(None, Some("staging")),
            vec![".env.staging".to_string(), ".env".to_string()]
        );
        assert_eq!(
            env_file_candidates(Some(" .env.custom "), Some("production")),
            vec![
                ".env.custom".to_string(),
                ".env.production".to_string(),
                ".env".to_string()
            ]
        );
    }
}
