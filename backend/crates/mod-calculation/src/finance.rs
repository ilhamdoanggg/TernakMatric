use crate::version::{CalculationError, CalculationResult};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CostBreakdown {
    pub stock_cost: Decimal,
    pub feed_cost: Decimal,
    pub health_and_vaccine_cost: Decimal,
    pub operational_cost: Decimal,
}

impl CostBreakdown {
    pub fn total(&self) -> Decimal {
        self.stock_cost + self.feed_cost + self.health_and_vaccine_cost + self.operational_cost
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum HppBasis {
    RealizedHarvestOnly,
    RealizedPlusStandingBiomass,
}

/// Computes HPP (Harga Pokok Produksi / Cost of Production) per kg live weight.
pub fn compute_hpp_per_kg_live(
    costs: &CostBreakdown,
    harvested_live_kg: Decimal,
    standing_biomass_kg: Decimal,
    basis: HppBasis,
    calculated_at: DateTime<Utc>,
) -> Result<CalculationResult<Decimal>, CalculationError> {
    let total_cost = costs.total();
    if total_cost < Decimal::ZERO {
        return Err(CalculationError::InvalidInput {
            field: "costs",
            reason: "total cost must be >= 0".into(),
        });
    }
    let denominator_kg = match basis {
        HppBasis::RealizedHarvestOnly => harvested_live_kg,
        HppBasis::RealizedPlusStandingBiomass => harvested_live_kg + standing_biomass_kg,
    };
    if denominator_kg <= Decimal::ZERO {
        return Err(CalculationError::DivisionByZero {
            context: "hpp_per_kg_live (live biomass <= 0)",
        });
    }
    let hpp = (total_cost / denominator_kg).round_dp(2);

    match basis {
        HppBasis::RealizedHarvestOnly => Ok(CalculationResult::exact(
            hpp,
            "currency/kg",
            "HPP_LIVE_WEIGHT",
            1,
            None,
            calculated_at,
        )),
        HppBasis::RealizedPlusStandingBiomass => Ok(CalculationResult::estimate(
            hpp,
            "currency/kg",
            "HPP_LIVE_WEIGHT",
            1,
            None,
            calculated_at,
            None,
            vec!["Includes estimated standing biomass alongside realized harvest weight.".into()],
        )),
    }
}

/// Computes HPP per non-meat output unit (e.g. per egg, per liter of milk, per fry).
pub fn compute_hpp_per_output_unit(
    costs: &CostBreakdown,
    output_quantity: Decimal,
    unit_label: &str,
    calculated_at: DateTime<Utc>,
) -> Result<CalculationResult<Decimal>, CalculationError> {
    if output_quantity <= Decimal::ZERO {
        return Err(CalculationError::DivisionByZero {
            context: "hpp_per_output_unit (output_quantity <= 0)",
        });
    }
    let hpp = (costs.total() / output_quantity).round_dp(2);
    Ok(CalculationResult::exact(
        hpp,
        format!("currency/{unit_label}"),
        "HPP_OUTPUT_UNIT",
        1,
        None,
        calculated_at,
    ))
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct BatchPnlSummary {
    pub total_cost: Decimal,
    pub total_revenue: Decimal,
    pub net_profit: Decimal,
    pub profit_margin_percent: Option<Decimal>,
}

pub fn compute_batch_pnl(
    costs: &CostBreakdown,
    total_revenue: Decimal,
    calculated_at: DateTime<Utc>,
) -> Result<CalculationResult<BatchPnlSummary>, CalculationError> {
    if total_revenue < Decimal::ZERO {
        return Err(CalculationError::InvalidInput {
            field: "total_revenue",
            reason: "revenue must be >= 0".into(),
        });
    }
    let total_cost = costs.total().round_dp(2);
    let rev = total_revenue.round_dp(2);
    let net_profit = (rev - total_cost).round_dp(2);
    let profit_margin_percent = if rev > Decimal::ZERO {
        Some(((net_profit / rev) * dec!(100)).round_dp(2))
    } else {
        None
    };

    Ok(CalculationResult::exact(
        BatchPnlSummary {
            total_cost,
            total_revenue: rev,
            net_profit,
            profit_margin_percent,
        },
        "currency",
        "PROFITABILITY",
        1,
        None,
        calculated_at,
    ))
}
