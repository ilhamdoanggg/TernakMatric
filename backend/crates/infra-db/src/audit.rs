use crate::pool::map_sqlx_err;
use serde_json::Value;
use shared_kernel::{AppError, OrganizationId, UserId};
use sqlx::PgConnection;
use uuid::Uuid;

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
}
