use crate::version::{CalculationError, CalculationResult};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use shared_kernel::MovementType;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct StockLedgerEntry {
    pub movement_type: MovementType,
    pub quantity_delta: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PopulationSummary {
    pub total_stocked: i64,
    pub total_mortality: i64,
    pub total_culled: i64,
    pub total_harvested_or_sold: i64,
    pub total_transferred_out: i64,
    pub current_alive: i64,
}

/// Computes current population and component totals from append-only `stock_movements`.
/// `EggCollection` and `MilkCollection` do not affect live animal headcount.
pub fn compute_population_balance(
    entries: &[StockLedgerEntry],
    calculated_at: DateTime<Utc>,
) -> Result<CalculationResult<PopulationSummary>, CalculationError> {
    let mut stocked: i64 = 0;
    let mut mortality: i64 = 0;
    let mut culled: i64 = 0;
    let mut harvested: i64 = 0;
    let mut transferred_out: i64 = 0;
    let mut balance: i64 = 0;

    for e in entries {
        if !e.movement_type.affects_live_headcount() {
            continue;
        }
        balance += e.quantity_delta;
        match e.movement_type {
            MovementType::InitialStock
            | MovementType::AdditionalStock
            | MovementType::Birth
            | MovementType::Purchase
            | MovementType::TransferIn => stocked += e.quantity_delta.max(0),
            MovementType::Mortality => mortality += (-e.quantity_delta).max(0),
            MovementType::Cull => culled += (-e.quantity_delta).max(0),
            MovementType::Harvest | MovementType::Sale => harvested += (-e.quantity_delta).max(0),
            MovementType::TransferOut => transferred_out += (-e.quantity_delta).max(0),
            MovementType::Adjustment | MovementType::Reversal => {}
            MovementType::EggCollection | MovementType::MilkCollection => {}
        }
    }

    if balance < 0 {
        return Err(CalculationError::NegativePopulation { balance });
    }

    Ok(CalculationResult::exact(
        PopulationSummary {
            total_stocked: stocked,
            total_mortality: mortality,
            total_culled: culled,
            total_harvested_or_sold: harvested,
            total_transferred_out: transferred_out,
            current_alive: balance,
        },
        "head",
        "POPULATION_BALANCE",
        1,
        None,
        calculated_at,
    ))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum SurvivalMode {
    /// Accounts for harvested & transferred-out stock as survived: (alive + harvested + transfer_out) / total_stocked * 100
    CumulativeAccounted,
    /// Remaining alive over net active denominator: alive / (total_stocked - harvested - transfer_out) * 100
    NetRemainingStock,
}

pub fn compute_survival_rate(
    summary: &PopulationSummary,
    mode: SurvivalMode,
    parameter_version: Option<i32>,
    calculated_at: DateTime<Utc>,
) -> Result<CalculationResult<Decimal>, CalculationError> {
    if summary.total_stocked <= 0 {
        return Err(CalculationError::DivisionByZero {
            context: "survival_rate (total_stocked <= 0)",
        });
    }

    let (num, den) = match mode {
        SurvivalMode::CumulativeAccounted => (
            summary.current_alive + summary.total_harvested_or_sold + summary.total_transferred_out,
            summary.total_stocked,
        ),
        SurvivalMode::NetRemainingStock => {
            let d = summary.total_stocked
                - summary.total_harvested_or_sold
                - summary.total_transferred_out;
            if d <= 0 {
                return Err(CalculationError::DivisionByZero {
                    context: "survival_rate net denominator <= 0",
                });
            }
            (summary.current_alive, d)
        }
    };

    let sr = ((Decimal::from(num) / Decimal::from(den)) * dec!(100)).round_dp(2);
    Ok(CalculationResult::exact(
        sr,
        "percent",
        "SURVIVAL_RATE",
        1,
        parameter_version,
        calculated_at,
    ))
}

pub fn compute_mortality_rate(
    summary: &PopulationSummary,
    parameter_version: Option<i32>,
    calculated_at: DateTime<Utc>,
) -> Result<CalculationResult<Decimal>, CalculationError> {
    if summary.total_stocked <= 0 {
        return Err(CalculationError::DivisionByZero {
            context: "mortality_rate (total_stocked <= 0)",
        });
    }
    let mr = ((Decimal::from(summary.total_mortality) / Decimal::from(summary.total_stocked))
        * dec!(100))
    .round_dp(2);
    Ok(CalculationResult::exact(
        mr,
        "percent",
        "MORTALITY_RATE",
        1,
        parameter_version,
        calculated_at,
    ))
}
