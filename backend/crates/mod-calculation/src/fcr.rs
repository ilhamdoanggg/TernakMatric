use crate::version::{CalculationError, CalculationResult};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum FcrMode {
    /// Gain = Current Standing Biomass + Harvested Biomass - Initial Biomass - Added Stock Biomass
    Economic,
    /// Gain = Current Standing Biomass + Harvested Biomass + Mortality/Cull Biomass - Initial Biomass - Added Stock Biomass
    Biological,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct FcrInput {
    pub total_feed_kg: Decimal,
    pub initial_biomass_kg: Decimal,
    pub added_stock_biomass_kg: Decimal,
    pub current_standing_biomass_kg: Decimal,
    pub harvested_biomass_kg: Decimal,
    pub mortality_and_cull_biomass_kg: Decimal,
    pub mode: FcrMode,
    pub last_sample_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct FcrSummary {
    pub fcr: Decimal,
    pub biomass_gain_kg: Decimal,
    pub feed_consumed_kg: Decimal,
    pub mode: FcrMode,
}

pub fn compute_fcr(
    input: &FcrInput,
    parameter_version: Option<i32>,
    calculated_at: DateTime<Utc>,
) -> Result<CalculationResult<FcrSummary>, CalculationError> {
    if input.total_feed_kg < Decimal::ZERO {
        return Err(CalculationError::InvalidInput {
            field: "total_feed_kg",
            reason: "feed consumed must be >= 0".into(),
        });
    }

    let base_output = input.current_standing_biomass_kg + input.harvested_biomass_kg;
    let total_output = match input.mode {
        FcrMode::Economic => base_output,
        FcrMode::Biological => base_output + input.mortality_and_cull_biomass_kg,
    };
    let gain_kg = total_output - input.initial_biomass_kg - input.added_stock_biomass_kg;

    if gain_kg <= Decimal::ZERO {
        return Err(CalculationError::NonPositiveBiomassGain {
            gain_kg: gain_kg.to_string(),
        });
    }

    let fcr = (input.total_feed_kg / gain_kg).round_dp(3);
    let note = match input.mode {
        FcrMode::Economic => "Economic FCR excludes mortality/cull biomass from denominator gain.",
        FcrMode::Biological => "Biological FCR includes mortality/cull biomass in denominator gain.",
    };

    Ok(CalculationResult::estimate(
        FcrSummary {
            fcr,
            biomass_gain_kg: gain_kg.round_dp(3),
            feed_consumed_kg: input.total_feed_kg.round_dp(3),
            mode: input.mode,
        },
        "ratio",
        "FCR_GENERIC",
        1,
        parameter_version,
        calculated_at,
        input.last_sample_at,
        vec![note.into()],
    ))
}

pub fn compute_feeding_variance_percent(
    actual_feed_kg: Decimal,
    recommended_feed_kg: Decimal,
    parameter_version: Option<i32>,
    calculated_at: DateTime<Utc>,
) -> Result<CalculationResult<Decimal>, CalculationError> {
    if recommended_feed_kg <= Decimal::ZERO {
        return Err(CalculationError::DivisionByZero {
            context: "feeding variance (recommended_feed_kg <= 0)",
        });
    }
    let variance =
        (((actual_feed_kg - recommended_feed_kg) / recommended_feed_kg) * dec!(100)).round_dp(2);
    Ok(CalculationResult::exact(
        variance,
        "percent",
        "FEEDING_VARIANCE",
        1,
        parameter_version,
        calculated_at,
    ))
}
