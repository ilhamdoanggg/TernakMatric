use crate::version::{CalculationError, CalculationResult};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CohortBiomassInput {
    pub estimated_population: i64,
    pub average_body_weight_g: Decimal,
    pub sampled_at: Option<DateTime<Utc>>,
}

/// Estimates total live biomass in kg from estimated headcount and latest sample ABW.
pub fn compute_biomass_kg(
    input: &CohortBiomassInput,
    parameter_version: Option<i32>,
    calculated_at: DateTime<Utc>,
) -> Result<CalculationResult<Decimal>, CalculationError> {
    if input.estimated_population < 0 || input.average_body_weight_g < Decimal::ZERO {
        return Err(CalculationError::InvalidInput {
            field: "biomass_input",
            reason: "population and average_body_weight_g must be >= 0".into(),
        });
    }
    let biomass = ((Decimal::from(input.estimated_population) * input.average_body_weight_g)
        / dec!(1000))
    .round_dp(3);

    Ok(CalculationResult::estimate(
        biomass,
        "kg",
        "BIOMASS_ESTIMATE",
        1,
        parameter_version,
        calculated_at,
        input.sampled_at,
        vec![
            "Estimated from ledger population and latest growth sample ABW; not a full-flock weigh-in.".into(),
        ],
    ))
}

/// Aggregates total active biomass across multiple cohorts sharing one enclosure.
pub fn compute_enclosure_total_biomass_kg(
    cohorts: &[CohortBiomassInput],
    parameter_version: Option<i32>,
    calculated_at: DateTime<Utc>,
) -> Result<CalculationResult<Decimal>, CalculationError> {
    let mut total = Decimal::ZERO;
    let mut latest_sample: Option<DateTime<Utc>> = None;
    for c in cohorts {
        let res = compute_biomass_kg(c, parameter_version, calculated_at)?;
        total += res.value;
        if let Some(s) = c.sampled_at {
            latest_sample = Some(match latest_sample {
                Some(prev) => prev.max(s),
                None => s,
            });
        }
    }
    Ok(CalculationResult::estimate(
        total.round_dp(3),
        "kg",
        "ENCLOSURE_BIOMASS_ESTIMATE",
        1,
        parameter_version,
        calculated_at,
        latest_sample,
        vec![format!(
            "Combined estimated biomass across {} active cohorts in enclosure.",
            cohorts.len()
        )],
    ))
}
