use chrono::Utc;
use infra_db::with_organization;
use mod_calculation::{compute_biomass_kg, CohortBiomassInput};
use rust_decimal::Decimal;
use shared_kernel::OrganizationId;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::time::Duration;
use uuid::Uuid;

async fn run_daily_snapshots(pool: &PgPool) -> Result<usize, sqlx::Error> {
    let org_ids: Vec<Uuid> =
        sqlx::query_scalar("SELECT id FROM organizations WHERE status = 'active'")
            .fetch_all(pool)
            .await?;

    let mut snapshots_written = 0usize;
    let calc_version_id: Option<Uuid> = sqlx::query_scalar(
        "SELECT id FROM calculation_versions WHERE code = 'BIOMASS_ESTIMATE' AND version = 1",
    )
    .fetch_optional(pool)
    .await?;

    let Some(version_id) = calc_version_id else {
        return Ok(0);
    };

    for org_id in org_ids {
        let mut tx = pool.begin().await?;
        with_organization(&mut *tx, OrganizationId(org_id)).await?;

        let batches: Vec<(Uuid, i32, Option<Decimal>)> = sqlx::query_as(
            r#"
            SELECT b.id, COALESCE(pb.quantity, b.initial_count),
                   (SELECT gs.average_weight_g FROM growth_samples gs
                    WHERE gs.organization_id = b.organization_id AND gs.batch_id = b.id
                    ORDER BY gs.sampled_at DESC LIMIT 1)
            FROM production_batches b
            LEFT JOIN population_balances pb
              ON pb.organization_id = b.organization_id AND pb.batch_id = b.id
            WHERE b.organization_id = $1 AND b.status = 'active'
            "#,
        )
        .bind(org_id)
        .fetch_all(&mut *tx)
        .await?;

        for (batch_id, pop, abw_opt) in batches {
            let Some(abw_g) = abw_opt else { continue };
            if let Ok(res) = compute_biomass_kg(
                &CohortBiomassInput {
                    estimated_population: pop as i64,
                    average_body_weight_g: abw_g,
                    sampled_at: None,
                },
                None,
                Utc::now(),
            ) {
                sqlx::query(
                    r#"
                    INSERT INTO calculation_runs (
                        organization_id, batch_id, calculation_version_id, calculation_type,
                        formula_version, input_snapshot, result_snapshot, assumptions
                    )
                    VALUES ($1, $2, $3, 'biomass', $4, $5, $6, $7)
                    "#,
                )
                .bind(org_id)
                .bind(batch_id)
                .bind(version_id)
                .bind(res.formula_version)
                .bind(serde_json::json!({ "population": pop, "abw_g": abw_g }))
                .bind(serde_json::json!({ "biomass_kg": res.value, "unit": res.unit }))
                .bind(serde_json::to_value(&res.assumptions).unwrap_or_default())
                .execute(&mut *tx)
                .await?;
                snapshots_written += 1;
            }
        }
        tx.commit().await?;
    }
    Ok(snapshots_written)
}

async fn run_vaccination_reminders_and_alerts(pool: &PgPool) -> Result<(), sqlx::Error> {
    let org_ids: Vec<Uuid> =
        sqlx::query_scalar("SELECT id FROM organizations WHERE status = 'active'")
            .fetch_all(pool)
            .await?;

    for org_id in org_ids {
        let mut tx = pool.begin().await?;
        with_organization(&mut *tx, OrganizationId(org_id)).await?;

        // Mark overdue vaccination schedules
        let overdue: Vec<(Uuid, Uuid, String)> = sqlx::query_as(
            r#"
            UPDATE vaccination_schedules
            SET status = 'overdue'
            WHERE organization_id = $1 AND status = 'scheduled' AND scheduled_date < CURRENT_DATE
            RETURNING id, batch_id, vaccine_name
            "#,
        )
        .bind(org_id)
        .fetch_all(&mut *tx)
        .await?;

        for (sched_id, batch_id, vaccine_name) in overdue {
            sqlx::query(
                r#"
                INSERT INTO outbox_events (organization_id, aggregate_type, aggregate_id, event_type, payload)
                VALUES ($1, 'vaccination_schedules', $2, 'alert.vaccination_overdue', $3)
                "#,
            )
            .bind(org_id)
            .bind(sched_id)
            .bind(serde_json::json!({ "batch_id": batch_id, "vaccine_name": vaccine_name }))
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(std::env::var("RUST_LOG").unwrap_or_else(|_| "worker=info".into()))
        .init();

    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://ternakmatric:ternakmatric@localhost:5432/ternakmatric".into());
    let pool = PgPoolOptions::new()
        .max_connections(8)
        .connect_lazy(&db_url)?;

    tracing::info!("TernakMatric background worker started (daily snapshot, vaccine reminder, alert engine)");

    let mut tick = tokio::time::interval(Duration::from_secs(300));
    loop {
        tick.tick().await;
        if let Err(err) = run_daily_snapshots(&pool).await {
            tracing::warn!(error = %err, "daily snapshot cycle skipped/failed");
        }
        if let Err(err) = run_vaccination_reminders_and_alerts(&pool).await {
            tracing::warn!(error = %err, "vaccine reminder & alert cycle skipped/failed");
        }
    }
}
