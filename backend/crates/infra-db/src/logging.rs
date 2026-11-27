use crate::context::with_organization;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use shared_kernel::{AppError, OrganizationId};
use sqlx::PgPool;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct SystemLogArchiveDto {
    pub id: Uuid,
    pub organization_id: Option<Uuid>,
    pub log_date: NaiveDate,
    pub service_name: String,
    pub file_name: String,
    pub file_path: String,
    pub file_size_bytes: i64,
    pub line_count: i64,
    pub status: String,
    pub checksum_sha256: Option<String>,
    pub archived_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SystemLogContentDto {
    pub file_name: String,
    pub file_path: String,
    pub total_lines: usize,
    pub returned_lines: usize,
    pub is_truncated: bool,
    pub lines: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SyncLogsResponse {
    pub scanned_files: usize,
    pub synced_archives: usize,
    pub log_dir: String,
}

#[derive(Debug, Clone)]
pub struct RealtimeLogBroadcaster {
    sender: broadcast::Sender<String>,
}

impl RealtimeLogBroadcaster {
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }

    pub fn sender(&self) -> broadcast::Sender<String> {
        self.sender.clone()
    }

    pub fn publish(&self, line: &str) {
        let _ = self.sender.send(line.to_string());
    }
}

struct LogWriterInner {
    log_dir: PathBuf,
    current_date_stamp: Mutex<String>,
    current_file: Mutex<Option<File>>,
    broadcaster: RealtimeLogBroadcaster,
}

#[derive(Clone)]
pub struct DailyLogAppender {
    inner: Arc<LogWriterInner>,
}

impl DailyLogAppender {
    pub fn new(log_dir: impl Into<PathBuf>, broadcaster: RealtimeLogBroadcaster) -> Self {
        let dir = log_dir.into();
        let _ = fs::create_dir_all(&dir);
        Self {
            inner: Arc::new(LogWriterInner {
                log_dir: dir,
                current_date_stamp: Mutex::new(String::new()),
                current_file: Mutex::new(None),
                broadcaster,
            }),
        }
    }

    pub fn broadcaster(&self) -> RealtimeLogBroadcaster {
        self.inner.broadcaster.clone()
    }

    pub fn log_dir(&self) -> PathBuf {
        self.inner.log_dir.clone()
    }

    fn today_stamp() -> String {
        Utc::now().format("%d-%m-%Y").to_string()
    }

    fn get_or_rotate_file(&self, today: &str) -> Option<File> {
        let mut cached_date = self.inner.current_date_stamp.lock().ok()?;
        let mut cached_file = self.inner.current_file.lock().ok()?;

        if cached_file.is_none() || *cached_date != today {
            let file_name = format!("{today}.log");
            let target_path = self.inner.log_dir.join(&file_name);
            let _ = fs::create_dir_all(&self.inner.log_dir);
            match OpenOptions::new()
                .create(true)
                .append(true)
                .open(&target_path)
            {
                Ok(file) => {
                    *cached_date = today.to_string();
                    *cached_file = file.try_clone().ok();
                }
                Err(err) => {
                    eprintln!("Failed to open log file {}: {err}", target_path.display());
                    return None;
                }
            }
        }

        cached_file.as_ref().and_then(|f| f.try_clone().ok())
    }
}

impl Write for DailyLogAppender {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let _ = std::io::stdout().write_all(buf);

        let today = Self::today_stamp();
        if let Some(mut file) = self.get_or_rotate_file(&today) {
            let _ = file.write_all(buf);
            let _ = file.flush();
        }

        if let Ok(text) = std::str::from_utf8(buf) {
            let trimmed = text.trim_end();
            if !trimmed.is_empty() {
                self.inner.broadcaster.publish(trimmed);
            }
        }

        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        let _ = std::io::stdout().flush();
        if let Ok(mut guard) = self.inner.current_file.lock() {
            if let Some(ref mut file) = *guard {
                let _ = file.flush();
            }
        }
        Ok(())
    }
}

pub fn resolve_custom_log_dir() -> PathBuf {
    if let Ok(path) = std::env::var("APP_LOG_DIR") {
        if !path.trim().is_empty() {
            return PathBuf::from(path.trim());
        }
    }
    if let Ok(path) = std::env::var("LOG_DIR") {
        if !path.trim().is_empty() {
            return PathBuf::from(path.trim());
        }
    }
    PathBuf::from("logs")
}

pub fn parse_log_date_from_stem(file_name: &str) -> Option<NaiveDate> {
    let stem = file_name.strip_suffix(".log").unwrap_or(file_name);
    NaiveDate::parse_from_str(stem, "%d-%m-%Y").ok()
}

pub fn calculate_simple_checksum(path: &Path) -> Option<String> {
    let mut file = File::open(path).ok()?;
    let mut hasher_val: u64 = 0xcbf29ce484222325;
    let mut buffer = [0u8; 8192];
    while let Ok(n) = file.read(&mut buffer) {
        if n == 0 {
            break;
        }
        for byte in &buffer[..n] {
            hasher_val ^= *byte as u64;
            hasher_val = hasher_val.wrapping_mul(0x100000001b3);
        }
    }
    Some(format!("{hasher_val:016x}"))
}

pub async fn sync_daily_logs_to_db(
    pool: &PgPool,
    log_dir: &Path,
    service_name: &str,
    organization_id: Option<Uuid>,
) -> Result<Vec<SystemLogArchiveDto>, AppError> {
    if !log_dir.exists() {
        fs::create_dir_all(log_dir).map_err(|e| AppError::Database(e.to_string()))?;
    }

    let today_date = Utc::now().date_naive();
    let mut synced = Vec::new();

    let entries = fs::read_dir(log_dir).map_err(|e| AppError::Database(e.to_string()))?;

    let mut tx = pool
        .begin()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    if let Some(org_id) = organization_id {
        let _ = with_organization(&mut *tx, OrganizationId(org_id)).await;
    }

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
            let file_name = path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or_default()
                .to_string();

            if !file_name.ends_with(".log") {
                continue;
            }

            let log_date = parse_log_date_from_stem(&file_name).unwrap_or(today_date);
            let metadata = entry
                .metadata()
                .map_err(|e| AppError::Database(e.to_string()))?;
            let file_size_bytes = metadata.len() as i64;

            let line_count = {
                let file = File::open(&path).map_err(|e| AppError::Database(e.to_string()))?;
                let reader = BufReader::new(file);
                reader.lines().count() as i64
            };

            let checksum = calculate_simple_checksum(&path);
            let status = if log_date >= today_date {
                "active"
            } else {
                "archived"
            };

            let row: SystemLogArchiveDto = sqlx::query_as(
                r#"
                INSERT INTO system_log_archives (
                    organization_id, log_date, service_name, file_name, file_path,
                    file_size_bytes, line_count, status, checksum_sha256, archived_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, now())
                ON CONFLICT (log_date, service_name, file_name)
                DO UPDATE SET
                    organization_id = COALESCE(EXCLUDED.organization_id, system_log_archives.organization_id),
                    file_path = EXCLUDED.file_path,
                    file_size_bytes = EXCLUDED.file_size_bytes,
                    line_count = EXCLUDED.line_count,
                    status = EXCLUDED.status,
                    checksum_sha256 = EXCLUDED.checksum_sha256,
                    archived_at = now(),
                    updated_at = now()
                RETURNING
                    id, organization_id, log_date, service_name, file_name, file_path,
                    file_size_bytes, line_count, status, checksum_sha256, archived_at, created_at, updated_at
                "#,
            )
            .bind(organization_id)
            .bind(log_date)
            .bind(service_name)
            .bind(&file_name)
            .bind(path.to_string_lossy().to_string())
            .bind(file_size_bytes)
            .bind(line_count)
            .bind(status)
            .bind(checksum)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

            synced.push(row);
        }
    }

    tx.commit()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(synced)
}

pub fn read_log_file_lines(
    log_dir: &Path,
    file_name: &str,
    limit: Option<usize>,
    tail: bool,
) -> Result<SystemLogContentDto, AppError> {
    if file_name.contains('/') || file_name.contains('\\') || file_name.contains("..") {
        return Err(AppError::Validation(
            "invalid file name path traversal".into(),
        ));
    }
    if !file_name.ends_with(".log") {
        return Err(AppError::Validation(
            "file must end with .log extension".into(),
        ));
    }

    let target_path = log_dir.join(file_name);
    if !target_path.exists() {
        return Err(AppError::NotFound(format!(
            "log file '{file_name}' not found"
        )));
    }

    let file = File::open(&target_path)
        .map_err(|e| AppError::Database(format!("failed to open log file: {e}")))?;
    let reader = BufReader::new(file);

    let mut all_lines = Vec::new();
    for line in reader.lines() {
        match line {
            Ok(l) => all_lines.push(l),
            Err(_) => all_lines.push("[binary or unreadable log chunk]".into()),
        }
    }

    let total_lines = all_lines.len();
    let max_lines = limit.unwrap_or(500).clamp(1, 2000);

    let (is_truncated, selected_lines) = if total_lines <= max_lines {
        (false, all_lines)
    } else if tail {
        let skip = total_lines.saturating_sub(max_lines);
        (true, all_lines.into_iter().skip(skip).collect())
    } else {
        (true, all_lines.into_iter().take(max_lines).collect())
    };

    Ok(SystemLogContentDto {
        file_name: file_name.to_string(),
        file_path: target_path.to_string_lossy().to_string(),
        total_lines,
        returned_lines: selected_lines.len(),
        is_truncated,
        lines: selected_lines,
    })
}
