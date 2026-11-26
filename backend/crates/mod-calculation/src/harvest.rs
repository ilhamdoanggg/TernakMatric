use crate::version::{CalculationError, CalculationResult};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MeatHarvestForecastInput {
    pub current_alive_population: i64,
    pub current_abw_g: Decimal,
    pub expected_adg_g_per_day: Decimal,
    pub days_until_harvest: i64,
    pub assumed_survival_rate_percent: Decimal,
    pub last_sample_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MeatHarvestForecast {
    pub projected_population: i64,
    pub projected_abw_g: Decimal,
    pub projected_harvest_kg: Decimal,
}

pub fn forecast_meat_harvest(
    input: &MeatHarvestForecastInput,
    parameter_version: Option<i32>,
    calculated_at: DateTime<Utc>,
) -> Result<CalculationResult<MeatHarvestForecast>, CalculationError> {
    if input.current_alive_population < 0
        || input.current_abw_g <= Decimal::ZERO
        || input.days_until_harvest < 0
    {
        return Err(CalculationError::InvalidInput {
            field: "harvest_forecast",
            reason: "population >= 0, abw > 0, and days >= 0 required".into(),
        });
    }
    if input.assumed_survival_rate_percent <= Decimal::ZERO
        || input.assumed_survival_rate_percent > dec!(100)
    {
        return Err(CalculationError::InvalidInput {
            field: "assumed_survival_rate_percent",
            reason: "assumed_survival_rate_percent must be within (0, 100]".into(),
        });
    }

    let projected_pop_dec = (Decimal::from(input.current_alive_population)
        * (input.assumed_survival_rate_percent / dec!(100)))
    .round_dp(0);
    let projected_population: i64 = projected_pop_dec.to_string().parse().unwrap_or(0);

    let projected_abw_g = (input.current_abw_g
        + input.expected_adg_g_per_day * Decimal::from(input.days_until_harvest))
    .round_dp(3);
    let projected_harvest_kg =
        ((Decimal::from(projected_population) * projected_abw_g) / dec!(1000)).round_dp(3);

    Ok(CalculationResult::estimate(
        MeatHarvestForecast {
            projected_population,
            projected_abw_g,
            projected_harvest_kg,
        },
        "kg",
        "HARVEST_FORECAST",
        1,
        parameter_version,
        calculated_at,
        input.last_sample_at,
        vec![format!(
            "Assumes {}% survival over remaining {} days and ADG {} g/day from species profile.",
            input.assumed_survival_rate_percent,
            input.days_until_harvest,
            input.expected_adg_g_per_day
        )],
    ))
}

pub fn forecast_egg_output(
    laying_population: i64,
    expected_egg_rate_percent: Decimal,
    days: i64,
    parameter_version: Option<i32>,
    calculated_at: DateTime<Utc>,
) -> Result<CalculationResult<Decimal>, CalculationError> {
    if laying_population < 0 || expected_egg_rate_percent < Decimal::ZERO || days < 0 {
        return Err(CalculationError::InvalidInput {
            field: "egg_forecast",
            reason: "population, rate, and days must be >= 0".into(),
        });
    }
    let total_eggs = (Decimal::from(laying_population)
        * (expected_egg_rate_percent / dec!(100))
        * Decimal::from(days))
    .round_dp(0);
    Ok(CalculationResult::estimate(
        total_eggs,
        "egg",
        "EGG_OUTPUT_FORECAST",
        1,
        parameter_version,
        calculated_at,
        None,
        vec![format!(
            "Forecast assumes {expected_egg_rate_percent}% hen-day rate over {days} days."
        )],
    ))
}
