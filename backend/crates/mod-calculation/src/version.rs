use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use utoipa::ToSchema;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum CalculationError {
    #[error("invalid positive input for {field}: {reason}")]
    InvalidInput { field: &'static str, reason: String },

    #[error("non-positive biomass gain ({gain_kg} kg); FCR cannot be computed")]
    NonPositiveBiomassGain { gain_kg: String },

    #[error("negative population balance ({balance})")]
    NegativePopulation { balance: i64 },

    #[error("division by zero for {context}")]
    DivisionByZero { context: &'static str },
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
pub struct CalculationAssumptions {
    pub is_estimate: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_sample_at: Option<DateTime<Utc>>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct CalculationResult<T> {
    pub value: T,
    pub unit: String,
    pub formula_code: String,
    pub formula_version: i32,
    pub parameter_version: Option<i32>,
    pub calculated_at: DateTime<Utc>,
    pub assumptions: CalculationAssumptions,
}

impl<T> CalculationResult<T> {
    pub fn exact(
        value: T,
        unit: impl Into<String>,
        formula_code: impl Into<String>,
        formula_version: i32,
        parameter_version: Option<i32>,
        calculated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            value,
            unit: unit.into(),
            formula_code: formula_code.into(),
            formula_version,
            parameter_version,
            calculated_at,
            assumptions: CalculationAssumptions {
                is_estimate: false,
                last_sample_at: None,
                notes: Vec::new(),
            },
        }
    }

    pub fn estimate(
        value: T,
        unit: impl Into<String>,
        formula_code: impl Into<String>,
        formula_version: i32,
        parameter_version: Option<i32>,
        calculated_at: DateTime<Utc>,
        last_sample_at: Option<DateTime<Utc>>,
        notes: Vec<String>,
    ) -> Self {
        Self {
            value,
            unit: unit.into(),
            formula_code: formula_code.into(),
            formula_version,
            parameter_version,
            calculated_at,
            assumptions: CalculationAssumptions {
                is_estimate: true,
                last_sample_at,
                notes,
            },
        }
    }
}
