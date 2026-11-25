use crate::pool::map_sqlx_err;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use shared_kernel::{AppError, OrganizationId, UserId};
use sqlx::PgConnection;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AuditLogRow {
    pub id: Uuid,
    pub organization_id: Option<Uuid>,
    pub actor_user_id: Option<Uuid>,
    pub actor_email: Option<String>,
    pub actor_display_name: Option<String>,
    pub action: String,
    pub entity_type: String,
    pub entity_id: Option<Uuid>,
    pub before_data: Option<Value>,
    pub after_data: Option<Value>,
    pub occurred_at: DateTime<Utc>,
}

pub struct AuditLogger;

impl AuditLogger {
    pub async fn record(
        conn: &mut PgConnection,
        organization_id: Option<OrganizationId>,
        actor_user_id: Option<UserId>,
        action: &str,
        entity_type: &str,
        entity_id: Option<Uuid>,
        before_data: Option<Value>,
        after_data: Option<Value>,
    ) -> Result<(), AppError> {
        sqlx::query(
            r#"
            INSERT INTO audit_logs (
                organization_id, actor_user_id, action, entity_type, entity_id, before_data, after_data
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(organization_id.map(|o| o.0))
        .bind(actor_user_id.map(|u| u.0))
        .bind(action)
        .bind(entity_type)
        .bind(entity_id)
        .bind(before_data)
        .bind(after_data)
        .execute(conn)
        .await
        .map_err(map_sqlx_err)?;
        Ok(())
    }

    pub async fn list_for_org(
        conn: &mut PgConnection,
        organization_id: OrganizationId,
        entity_type: Option<&str>,
        entity_id: Option<Uuid>,
        limit: i64,
    ) -> Result<Vec<AuditLogRow>, AppError> {
        let rows: Vec<AuditLogRow> = sqlx::query_as(
            r#"
            SELECT a.id, a.organization_id, a.actor_user_id,
                   u.email AS actor_email, u.display_name AS actor_display_name,
                   a.action, a.entity_type, a.entity_id,
                   a.before_data, a.after_data, a.occurred_at
            FROM audit_logs a
            LEFT JOIN users u ON u.id = a.actor_user_id
            WHERE a.organization_id = $1
              AND ($2::text IS NULL OR a.entity_type = $2)
              AND ($3::uuid IS NULL OR a.entity_id = $3)
            ORDER BY a.occurred_at DESC
            LIMIT $4
            "#,
        )
        .bind(organization_id.0)
        .bind(entity_type)
        .bind(entity_id)
        .bind(limit.clamp(1, 200))
        .fetch_all(conn)
        .await
        .map_err(map_sqlx_err)?;
        Ok(rows)
    }
}
