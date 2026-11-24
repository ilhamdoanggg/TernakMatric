use crate::version::{CalculationError, CalculationResult};
use chrono::{DateTime, Utc};
use rust_decimal::{Decimal, MathematicalOps};
use rust_decimal_macros::dec;

/// Computes Average Body Weight (ABW) in grams per head.
pub fn compute_abw_g(
    total_sample_weight_g: Decimal,
    sample_count: i64,
    calculated_at: DateTime<Utc>,
) -> Result<CalculationResult<Decimal>, CalculationError> {
    if total_sample_weight_g <= Decimal::ZERO || sample_count <= 0 {
        return Err(CalculationError::InvalidInput {
            field: "growth_sample",
            reason: "total_sample_weight_g and sample_count must be > 0".into(),
        });
    }
    let abw = (total_sample_weight_g / Decimal::from(sample_count)).round_dp(3);
    Ok(CalculationResult::exact(
        abw,
        "g/head",
        "ABW",
        1,
        None,
        calculated_at,
    ))
}

/// Computes Average Daily Gain (ADG) in grams per day: `(final_abw_g - initial_abw_g) / days`.
pub fn compute_adg_g_per_day(
    initial_abw_g: Decimal,
    final_abw_g: Decimal,
    days: Decimal,
    parameter_version: Option<i32>,
    calculated_at: DateTime<Utc>,
) -> Result<CalculationResult<Decimal>, CalculationError> {
    if days <= Decimal::ZERO {
        return Err(CalculationError::InvalidInput {
            field: "days",
            reason: "observation days must be > 0".into(),
        });
    }
    let adg = ((final_abw_g - initial_abw_g) / days).round_dp(3);
    Ok(CalculationResult::exact(
        adg,
        "g/day",
        "WEIGHT_GAIN",
        1,
        parameter_version,
        calculated_at,
    ))
}

/// Computes Specific Growth Rate (SGR) in %/day: `((ln(final_abw_g) - ln(initial_abw_g)) / days) * 100`.
pub fn compute_sgr_percent_per_day(
    initial_abw_g: Decimal,
    final_abw_g: Decimal,
    days: Decimal,
    parameter_version: Option<i32>,
    calculated_at: DateTime<Utc>,
) -> Result<CalculationResult<Decimal>, CalculationError> {
    if initial_abw_g <= Decimal::ZERO || final_abw_g <= Decimal::ZERO {
        return Err(CalculationError::InvalidInput {
            field: "abw_g",
            reason: "initial and final weights must be > 0 for SGR".into(),
        });
    }
    if days <= Decimal::ZERO {
        return Err(CalculationError::InvalidInput {
            field: "days",
            reason: "observation days must be > 0".into(),
        });
    }
    let ln_init = initial_abw_g
        .checked_ln()
        .ok_or_else(|| CalculationError::InvalidInput {
            field: "initial_abw_g",
            reason: "failed to compute ln(initial_abw_g)".into(),
        })?;
    let ln_final = final_abw_g
        .checked_ln()
        .ok_or_else(|| CalculationError::InvalidInput {
            field: "final_abw_g",
            reason: "failed to compute ln(final_abw_g)".into(),
        })?;
    let sgr = (((ln_final - ln_init) / days) * dec!(100)).round_dp(4);

    Ok(CalculationResult::exact(
        sgr,
        "percent/day",
        "SGR",
        1,
        parameter_version,
        calculated_at,
    ))
}

/// Computes egg production rate (%) for poultry layers.
pub fn compute_egg_rate_percent(
    eggs_collected: i64,
    laying_population: i64,
    days: Decimal,
    parameter_version: Option<i32>,
    calculated_at: DateTime<Utc>,
) -> Result<CalculationResult<Decimal>, CalculationError> {
    if eggs_collected < 0 || laying_population <= 0 || days <= Decimal::ZERO {
        return Err(CalculationError::InvalidInput {
            field: "egg_rate_input",
            reason: "laying_population and days must be > 0 and eggs >= 0".into(),
        });
    }
    let rate = ((Decimal::from(eggs_collected) / (Decimal::from(laying_population) * days))
        * dec!(100))
    .round_dp(2);
    Ok(CalculationResult::exact(
        rate,
        "percent",
        "EGG_RATE",
        1,
        parameter_version,
        calculated_at,
    ))
}

/// Computes daily milk yield in liters/head/day for dairy mammals.
pub fn compute_milk_yield_per_head_day(
    milk_liters: Decimal,
    lactating_population: i64,
    days: Decimal,
    parameter_version: Option<i32>,
    calculated_at: DateTime<Utc>,
) -> Result<CalculationResult<Decimal>, CalculationError> {
    if milk_liters < Decimal::ZERO || lactating_population <= 0 || days <= Decimal::ZERO {
        return Err(CalculationError::InvalidInput {
            field: "milk_yield_input",
            reason: "lactating_population and days must be > 0 and milk_liters >= 0".into(),
        });
    }
    let yield_per_day =
        (milk_liters / (Decimal::from(lactating_population) * days)).round_dp(3);
    Ok(CalculationResult::exact(
        yield_per_day,
        "liter/head/day",
        "MILK_YIELD",
        1,
        parameter_version,
        calculated_at,
    ))
}
