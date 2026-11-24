use chrono::{DateTime, Utc};
use infra_db::{map_sqlx_err, OrganizationContext, OutboxWriter};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use shared_kernel::{AppError, DeviceId};
use sqlx::PgPool;
use utoipa::ToSchema;
use uuid::Uuid;

pub const LEDGER_AND_FINANCE_ENTITIES: &[&str] = &[
    "stock_movements",
    "inventory_transactions",
    "inventory_transaction_lines",
    "expenses",
    "sales",
    "production_outputs",
    "feeding_records",
    "mortality_records",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum SyncOperationType {
    Create,
    Update,
    Delete,
}

impl SyncOperationType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Create => "create",
            Self::Update => "update",
            Self::Delete => "delete",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum SyncStatus {
    Pending,
    Applied,
    Rejected,
    Conflict,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ConflictResolution {
    ServerWins,
    ClientWins,
    Merged,
    Manual,
}

impl ConflictResolution {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ServerWins => "server_wins",
            Self::ClientWins => "client_wins",
            Self::Merged => "merged",
            Self::Manual => "manual",
        }
    }
}

/// Enforces ADR 002: `client_wins` is forbidden for population, inventory, output, and financial ledgers.
pub fn validate_conflict_resolution(
    entity_type: &str,
    resolution: ConflictResolution,
) -> Result<(), AppError> {
    if resolution == ConflictResolution::ClientWins
        && LEDGER_AND_FINANCE_ENTITIES.contains(&entity_type)
    {
        return Err(AppError::Validation(format!(
            "client_wins resolution is forbidden for ledger/finance entity '{entity_type}'"
        )));
    }
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RegisterDeviceRequest {
    pub device_label: Option<String>,
    pub platform: String,
    pub device_public_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RegisterDeviceResponse {
    pub id: Uuid,
    pub device_public_id: String,
    pub platform: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SyncCommand {
    pub client_operation_id: Uuid,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub operation_type: SyncOperationType,
    pub base_version: Option<i32>,
    pub server_current_version: Option<i32>,
    pub device_occurred_at: DateTime<Utc>,
    pub payload: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SyncPushRequest {
    pub device_id: DeviceId,
    pub commands: Vec<SyncCommand>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SyncPushItemResult {
    pub client_operation_id: Uuid,
    pub entity_id: Uuid,
    pub status: SyncStatus,
    pub deduplicated: bool,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SyncPushResponse {
    pub results: Vec<SyncPushItemResult>,
    pub latest_cursor: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct OutboxChangeEvent {
    pub cursor: i64,
    pub aggregate_type: String,
    pub aggregate_id: Uuid,
    pub event_type: String,
    pub payload: Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SyncPullResponse {
    pub events: Vec<OutboxChangeEvent>,
    pub next_cursor: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct SyncConflictRecord {
    pub id: Uuid,
    pub sync_operation_id: Uuid,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub conflict_type: String,
    pub server_snapshot: Option<Value>,
    pub client_snapshot: Option<Value>,
    pub resolution: Option<String>,
    pub created_at: DateTime<Utc>,
}

pub struct SyncService;

impl SyncService {
    pub async fn register_device(
        pool: &PgPool,
        ctx: &OrganizationContext,
        req: RegisterDeviceRequest,
    ) -> Result<RegisterDeviceResponse, AppError> {
        let mut tx = ctx.begin_tx(pool).await?;
        let row: (Uuid, DateTime<Utc>) = sqlx::query_as(
            r#"
            INSERT INTO devices (organization_id, user_id, device_label, platform, device_public_id, last_seen_at)
            VALUES ($1, $2, $3, $4, $5, now())
            ON CONFLICT (organization_id, device_public_id)
            DO UPDATE SET last_seen_at = now(), device_label = COALESCE(EXCLUDED.device_label, devices.device_label)
            RETURNING id, created_at
            "#,
        )
        .bind(ctx.organization_id.0)
        .bind(ctx.user_id.0)
        .bind(&req.device_label)
        .bind(&req.platform)
        .bind(&req.device_public_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(map_sqlx_err)?;
        tx.commit().await.map_err(map_sqlx_err)?;

        Ok(RegisterDeviceResponse {
            id: row.0,
            device_public_id: req.device_public_id,
            platform: req.platform,
            created_at: row.1,
        })
    }

    /// Processes offline commands idempotently using `(organization_id, device_id, client_operation_id)`
    /// and detects stale `base_version` conflicts without silently overwriting server state.
    pub async fn push_commands(
        pool: &PgPool,
        ctx: &OrganizationContext,
        req: SyncPushRequest,
    ) -> Result<SyncPushResponse, AppError> {
        let mut tx = ctx.begin_tx(pool).await?;
        let mut results = Vec::with_capacity(req.commands.len());

        for cmd in req.commands {
            let existing: Option<(Uuid, String)> = sqlx::query_as(
                r#"
                SELECT id, status
                FROM sync_operations
                WHERE organization_id = $1 AND device_id = $2 AND client_operation_id = $3
                "#,
            )
            .bind(ctx.organization_id.0)
            .bind(req.device_id.0)
            .bind(cmd.client_operation_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(map_sqlx_err)?;

            if let Some((_op_id, existing_status)) = existing {
                let status = match existing_status.as_str() {
                    "applied" => SyncStatus::Applied,
                    "conflict" => SyncStatus::Conflict,
                    "rejected" => SyncStatus::Rejected,
                    _ => SyncStatus::Pending,
                };
                results.push(SyncPushItemResult {
                    client_operation_id: cmd.client_operation_id,
                    entity_id: cmd.entity_id,
                    status,
                    deduplicated: true,
                    message: Some("operation already processed (idempotent replay)".into()),
                });
                continue;
            }

            let is_stale_conflict = match (cmd.base_version, cmd.server_current_version) {
                (Some(base), Some(current)) => base < current,
                _ => false,
            };

            let status = if is_stale_conflict {
                SyncStatus::Conflict
            } else {
                SyncStatus::Applied
            };
            let status_str = if is_stale_conflict { "conflict" } else { "applied" };

            let sync_op_id: Uuid = sqlx::query_scalar(
                r#"
                INSERT INTO sync_operations (
                    organization_id, device_id, client_operation_id, entity_type, entity_id,
                    operation_type, base_version, device_occurred_at, payload, status, processed_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, now())
                RETURNING id
                "#,
            )
            .bind(ctx.organization_id.0)
            .bind(req.device_id.0)
            .bind(cmd.client_operation_id)
            .bind(&cmd.entity_type)
            .bind(cmd.entity_id)
            .bind(cmd.operation_type.as_str())
            .bind(cmd.base_version)
            .bind(cmd.device_occurred_at)
            .bind(&cmd.payload)
            .bind(status_str)
            .fetch_one(&mut *tx)
            .await
            .map_err(map_sqlx_err)?;

            if is_stale_conflict {
                sqlx::query(
                    r#"
                    INSERT INTO sync_conflicts (
                        organization_id, sync_operation_id, entity_type, entity_id,
                        conflict_type, client_snapshot
                    )
                    VALUES ($1, $2, $3, $4, 'stale_base_version', $5)
                    "#,
                )
                .bind(ctx.organization_id.0)
                .bind(sync_op_id)
                .bind(&cmd.entity_type)
                .bind(cmd.entity_id)
                .bind(&cmd.payload)
                .execute(&mut *tx)
                .await
                .map_err(map_sqlx_err)?;
            } else {
                OutboxWriter::append(
                    &mut tx,
                    ctx.organization_id,
                    &cmd.entity_type,
                    cmd.entity_id,
                    &format!("{}.{}", cmd.entity_type, cmd.operation_type.as_str()),
                    cmd.payload.clone(),
                )
                .await?;
            }

            results.push(SyncPushItemResult {
                client_operation_id: cmd.client_operation_id,
                entity_id: cmd.entity_id,
                status,
                deduplicated: false,
                message: if is_stale_conflict {
                    Some("stale base_version conflict recorded".into())
                } else {
                    None
                },
            });
        }

        let latest_cursor: i64 = sqlx::query_scalar(
            "SELECT COALESCE(MAX(id), 0) FROM outbox_events WHERE organization_id = $1",
        )
        .bind(ctx.organization_id.0)
        .fetch_one(&mut *tx)
        .await
        .map_err(map_sqlx_err)?;

        tx.commit().await.map_err(map_sqlx_err)?;

        Ok(SyncPushResponse {
            results,
            latest_cursor,
        })
    }

    pub async fn pull_changes(
        pool: &PgPool,
        ctx: &OrganizationContext,
        after_cursor: i64,
        limit: i64,
    ) -> Result<SyncPullResponse, AppError> {
        let mut tx = ctx.begin_tx(pool).await?;
        let rows: Vec<infra_db::OutboxEventRow> = sqlx::query_as(
            r#"
            SELECT id, organization_id, aggregate_type, aggregate_id, event_type, payload, created_at, published_at
            FROM outbox_events
            WHERE organization_id = $1 AND id > $2
            ORDER BY id ASC
            LIMIT $3
            "#,
        )
        .bind(ctx.organization_id.0)
        .bind(after_cursor)
        .bind(limit.clamp(1, 500))
        .fetch_all(&mut *tx)
        .await
        .map_err(map_sqlx_err)?;
        tx.commit().await.map_err(map_sqlx_err)?;

        let next_cursor = rows.last().map(|r| r.id).unwrap_or(after_cursor);
        let events = rows
            .into_iter()
            .map(|r| OutboxChangeEvent {
                cursor: r.id,
                aggregate_type: r.aggregate_type,
                aggregate_id: r.aggregate_id,
                event_type: r.event_type,
                payload: r.payload,
                created_at: r.created_at,
            })
            .collect();

        Ok(SyncPullResponse {
            events,
            next_cursor,
        })
    }

    pub async fn list_conflicts(
        pool: &PgPool,
        ctx: &OrganizationContext,
    ) -> Result<Vec<SyncConflictRecord>, AppError> {
        let mut tx = ctx.begin_tx(pool).await?;
        let rows: Vec<SyncConflictRecord> = sqlx::query_as(
            r#"
            SELECT id, sync_operation_id, entity_type, entity_id, conflict_type,
                   server_snapshot, client_snapshot, resolution, created_at
            FROM sync_conflicts
            WHERE organization_id = $1 AND resolved_at IS NULL
            ORDER BY created_at DESC
            "#,
        )
        .bind(ctx.organization_id.0)
        .fetch_all(&mut *tx)
        .await
        .map_err(map_sqlx_err)?;
        tx.commit().await.map_err(map_sqlx_err)?;
        Ok(rows)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_wins_forbidden_on_ledgers() {
        for entity in LEDGER_AND_FINANCE_ENTITIES {
            let err = validate_conflict_resolution(entity, ConflictResolution::ClientWins);
            assert!(err.is_err(), "expected error for {entity}");
            assert!(validate_conflict_resolution(entity, ConflictResolution::ServerWins).is_ok());
        }
        assert!(validate_conflict_resolution("farms", ConflictResolution::ClientWins).is_ok());
    }
}
