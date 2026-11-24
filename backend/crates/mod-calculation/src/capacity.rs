use crate::version::{CalculationError, CalculationResult};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "shape", rename_all = "snake_case")]
pub enum EnclosureGeometry {
    Rectangle {
        length_m: Decimal,
        width_m: Decimal,
        average_depth_m: Option<Decimal>,
    },
    Circle {
        diameter_m: Decimal,
        average_depth_m: Option<Decimal>,
    },
    AreaOnly {
        area_m2: Decimal,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GeometryMetrics {
    pub area_m2: Decimal,
    pub volume_m3: Option<Decimal>,
}

/// Computes `area_m2` and optional `volume_m3` from enclosure dimensions.
pub fn compute_enclosure_geometry(
    geometry: &EnclosureGeometry,
    calculated_at: DateTime<Utc>,
) -> Result<CalculationResult<GeometryMetrics>, CalculationError> {
    let pi = dec!(3.14159265358979323846);
    match geometry {
        EnclosureGeometry::Rectangle {
            length_m,
            width_m,
            average_depth_m,
        } => {
            if *length_m <= Decimal::ZERO || *width_m <= Decimal::ZERO {
                return Err(CalculationError::InvalidInput {
                    field: "dimensions",
                    reason: "length_m and width_m must be > 0".into(),
                });
            }
            let area = (*length_m * *width_m).round_dp(3);
            let volume = match average_depth_m {
                Some(d) if *d <= Decimal::ZERO => {
                    return Err(CalculationError::InvalidInput {
                        field: "average_depth_m",
                        reason: "depth must be > 0".into(),
                    });
                }
                Some(d) => Some((area * *d).round_dp(3)),
                None => None,
            };
            Ok(CalculationResult::exact(
                GeometryMetrics {
                    area_m2: area,
                    volume_m3: volume,
                },
                "m2/m3",
                "VOLUME_RECTANGLE",
                1,
                None,
                calculated_at,
            ))
        }
        EnclosureGeometry::Circle {
            diameter_m,
            average_depth_m,
        } => {
            if *diameter_m <= Decimal::ZERO {
                return Err(CalculationError::InvalidInput {
                    field: "diameter_m",
                    reason: "diameter_m must be > 0".into(),
                });
            }
            let radius = *diameter_m / dec!(2);
            let area = (pi * radius * radius).round_dp(3);
            let volume = match average_depth_m {
                Some(d) if *d <= Decimal::ZERO => {
                    return Err(CalculationError::InvalidInput {
                        field: "average_depth_m",
                        reason: "depth must be > 0".into(),
                    });
                }
                Some(d) => Some((area * *d).round_dp(3)),
                None => None,
            };
            Ok(CalculationResult::exact(
                GeometryMetrics {
                    area_m2: area,
                    volume_m3: volume,
                },
                "m2/m3",
                "VOLUME_CIRCLE",
                1,
                None,
                calculated_at,
            ))
        }
        EnclosureGeometry::AreaOnly { area_m2 } => {
            if *area_m2 <= Decimal::ZERO {
                return Err(CalculationError::InvalidInput {
                    field: "area_m2",
                    reason: "area_m2 must be > 0".into(),
                });
            }
            Ok(CalculationResult::exact(
                GeometryMetrics {
                    area_m2: area_m2.round_dp(3),
                    volume_m3: None,
                },
                "m2",
                "AREA_ONLY",
                1,
                None,
                calculated_at,
            ))
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CapacityEvaluation {
    pub max_capacity: Decimal,
    pub current_occupancy: Decimal,
    pub requested_addition: Decimal,
    pub projected_occupancy: Decimal,
    pub utilization_percent: Decimal,
    pub fits: bool,
}

/// Evaluates whether adding a cohort/batch fits within an enclosure's capacity
/// alongside all existing active batches (`multi-batch per enclosure`).
pub fn evaluate_enclosure_capacity(
    max_capacity: Decimal,
    existing_active_occupancy: Decimal,
    requested_addition: Decimal,
    parameter_version: Option<i32>,
    calculated_at: DateTime<Utc>,
) -> Result<CalculationResult<CapacityEvaluation>, CalculationError> {
    if max_capacity <= Decimal::ZERO {
        return Err(CalculationError::InvalidInput {
            field: "max_capacity",
            reason: "max_capacity must be > 0".into(),
        });
    }
    if existing_active_occupancy < Decimal::ZERO || requested_addition < Decimal::ZERO {
        return Err(CalculationError::InvalidInput {
            field: "occupancy",
            reason: "occupancy values must be >= 0".into(),
        });
    }
    let projected = existing_active_occupancy + requested_addition;
    let utilization = ((projected / max_capacity) * dec!(100)).round_dp(2);
    let fits = projected <= max_capacity;

    Ok(CalculationResult::exact(
        CapacityEvaluation {
            max_capacity,
            current_occupancy: existing_active_occupancy,
            requested_addition,
            projected_occupancy: projected,
            utilization_percent: utilization,
            fits,
        },
        "percent",
        "CAPACITY_UTILIZATION",
        1,
        parameter_version,
        calculated_at,
    ))
}
