use crate::pool::map_sqlx_err;
use shared_kernel::{AppError, OrganizationId, UserId};
use sqlx::{PgPool, Postgres, Transaction};

/// Verified tenant context derived from authenticated user membership.
/// Never constructed directly from untrusted client headers without DB verification.
#[derive(Debug, Clone, Copy)]
pub struct OrganizationContext {
    pub organization_id: OrganizationId,
    pub user_id: UserId,
}

impl OrganizationContext {
    pub fn new(organization_id: OrganizationId, user_id: UserId) -> Self {
        Self {
            organization_id,
            user_id,
        }
    }

    /// Opens a PostgreSQL transaction and sets `app.current_organization_id` via `SET LOCAL`
    /// so RLS policies apply safely even with pooled connections.
    pub async fn begin_tx<'a>(
        &self,
        pool: &'a PgPool,
    ) -> Result<Transaction<'a, Postgres>, AppError> {
        let mut tx = pool.begin().await.map_err(map_sqlx_err)?;
        with_organization(&mut *tx, self.organization_id)
            .await
            .map_err(map_sqlx_err)?;
        Ok(tx)
    }
}

pub async fn with_organization<'e, E>(
    executor: E,
    org_id: OrganizationId,
) -> Result<(), sqlx::Error>
where
    E: sqlx::PgExecutor<'e>,
{
    sqlx::query("SELECT set_config('app.current_organization_id', $1, true)")
        .bind(org_id.0.to_string())
        .execute(executor)
        .await?;
    Ok(())
}
