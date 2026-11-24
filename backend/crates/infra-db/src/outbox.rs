use crate::pool::map_sqlx_err;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use shared_kernel::{AppError, OrganizationId};
use sqlx::PgConnection;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct OutboxEventRow {
    pub id: i64,
    pub organization_id: Uuid,
    pub aggregate_type: String,
    pub aggregate_id: Uuid,
    pub event_type: String,
    pub payload: Value,
    pub created_at: DateTime<Utc>,
    pub published_at: Option<DateTime<Utc>>,
}

pub struct OutboxWriter;

impl OutboxWriter {
    pub async fn append(
        conn: &mut PgConnection,
        organization_id: OrganizationId,
        aggregate_type: &str,
        aggregate_id: Uuid,
        event_type: &str,
        payload: Value,
    ) -> Result<i64, AppError> {
        let id: i64 = sqlx::query_scalar(
            r#"
            INSERT INTO outbox_events (organization_id, aggregate_type, aggregate_id, event_type, payload)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id
            "#,
        )
        .bind(organization_id.0)
        .bind(aggregate_type)
        .bind(aggregate_id)
        .bind(event_type)
        .bind(payload)
        .fetch_one(conn)
        .await
        .map_err(map_sqlx_err)?;
        Ok(id)
    }
}
