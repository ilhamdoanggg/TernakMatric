use infra_db::{map_sqlx_err, OrganizationContext};
use serde::{Deserialize, Serialize};
use shared_kernel::{AppError, OrganizationId, UserId};
use sqlx::PgPool;
use std::collections::HashSet;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum Permission {
    OrgManage,
    FarmManage,
    SpeciesManage,
    ProductionWrite,
    FeedingWrite,
    EnvironmentWrite,
    HealthWrite,
    FinanceWrite,
    AnalyticsRead,
}

impl Permission {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::OrgManage => "org.manage",
            Self::FarmManage => "farm.manage",
            Self::SpeciesManage => "species.manage",
            Self::ProductionWrite => "production.write",
            Self::FeedingWrite => "feeding.write",
            Self::EnvironmentWrite => "environment.write",
            Self::HealthWrite => "health.write",
            Self::FinanceWrite => "finance.write",
            Self::AnalyticsRead => "analytics.read",
        }
    }
}

#[derive(Debug, Clone)]
pub struct AuthPrincipal {
    pub user_id: UserId,
    pub organization_id: OrganizationId,
    pub roles: Vec<String>,
    pub permissions: HashSet<String>,
}

impl AuthPrincipal {
    pub fn context(&self) -> OrganizationContext {
        OrganizationContext::new(self.organization_id, self.user_id)
    }

    pub fn require(&self, permission: Permission) -> Result<(), AppError> {
        if self
            .roles
            .iter()
            .any(|r| r == "ORG_OWNER" || r == "ORG_ADMIN")
            || self.permissions.contains(permission.as_str())
        {
            Ok(())
        } else {
            Err(AppError::Forbidden(format!(
                "missing permission {}",
                permission.as_str()
            )))
        }
    }
}

/// Verifies active membership in `organization_id` and resolves RBAC role codes and permissions.
pub async fn verify_membership_and_permissions(
    pool: &PgPool,
    user_id: UserId,
    organization_id: OrganizationId,
) -> Result<AuthPrincipal, AppError> {
    let ctx = OrganizationContext::new(organization_id, user_id);
    let mut tx = ctx.begin_tx(pool).await?;

    let member_id: Option<Uuid> = sqlx::query_scalar(
        r#"
        SELECT id
        FROM organization_members
        WHERE organization_id = $1 AND user_id = $2 AND status = 'active'
        "#,
    )
    .bind(organization_id.0)
    .bind(user_id.0)
    .fetch_optional(&mut *tx)
    .await
    .map_err(map_sqlx_err)?;

    let Some(member_id) = member_id else {
        return Err(AppError::Forbidden(
            "user is not an active member of the requested organization".into(),
        ));
    };

    let roles: Vec<String> = sqlx::query_scalar(
        r#"
        SELECT r.code
        FROM member_roles mr
        JOIN roles r ON r.id = mr.role_id
        WHERE mr.organization_id = $1 AND mr.member_id = $2
        "#,
    )
    .bind(organization_id.0)
    .bind(member_id)
    .fetch_all(&mut *tx)
    .await
    .map_err(map_sqlx_err)?;

    let perms: Vec<String> = sqlx::query_scalar(
        r#"
        SELECT DISTINCT p.code
        FROM member_roles mr
        JOIN role_permissions rp ON rp.role_id = mr.role_id
        JOIN permissions p ON p.id = rp.permission_id
        WHERE mr.organization_id = $1 AND mr.member_id = $2
        "#,
    )
    .bind(organization_id.0)
    .bind(member_id)
    .fetch_all(&mut *tx)
    .await
    .map_err(map_sqlx_err)?;

    tx.commit().await.map_err(map_sqlx_err)?;

    Ok(AuthPrincipal {
        user_id,
        organization_id,
        roles,
        permissions: perms.into_iter().collect(),
    })
}
