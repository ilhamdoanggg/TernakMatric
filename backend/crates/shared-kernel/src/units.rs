use crate::AppError;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

/// Converts standard mass quantities (`kg`, `g`, `ton`, or explicit custom conversion factor) to kg.
pub fn convert_mass_to_kg(
    quantity: Decimal,
    unit: &str,
    custom_to_kg_factor: Option<Decimal>,
) -> Result<Decimal, AppError> {
    if quantity < Decimal::ZERO {
        return Err(AppError::Validation("quantity must be non-negative".into()));
    }
    match unit.to_ascii_lowercase().as_str() {
        "kg" => Ok(quantity),
        "g" => Ok(quantity / dec!(1000)),
        "ton" | "tonne" => Ok(quantity * dec!(1000)),
        _ => match custom_to_kg_factor {
            Some(factor) if factor > Decimal::ZERO => Ok(quantity * factor),
            _ => Err(AppError::Validation(format!(
                "missing conversion factor to kg for unit '{unit}'"
            ))),
        },
    }
}

/// Converts liquid volumes (`liter`, `l`, `ml`) to liters.
pub fn convert_volume_to_liters(quantity: Decimal, unit: &str) -> Result<Decimal, AppError> {
    if quantity < Decimal::ZERO {
        return Err(AppError::Validation("volume must be non-negative".into()));
    }
    match unit.to_ascii_lowercase().as_str() {
        "liter" | "l" => Ok(quantity),
        "ml" => Ok(quantity / dec!(1000)),
        _ => Err(AppError::Validation(format!(
            "unsupported liquid volume unit '{unit}'"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MovementType;

    #[test]
    fn test_mass_and_volume_conversions() {
        assert_eq!(convert_mass_to_kg(dec!(500), "g", None).unwrap(), dec!(0.5));
        assert_eq!(
            convert_mass_to_kg(dec!(2), "bag", Some(dec!(30))).unwrap(),
            dec!(60)
        );
        assert!(convert_mass_to_kg(dec!(2), "bag", None).is_err());
        assert_eq!(
            convert_volume_to_liters(dec!(2500), "ml").unwrap(),
            dec!(2.5)
        );
    }

    #[test]
    fn test_egg_and_milk_do_not_reduce_headcount() {
        assert!(!MovementType::EggCollection.affects_live_headcount());
        assert!(!MovementType::MilkCollection.affects_live_headcount());
        assert!(MovementType::Mortality.affects_live_headcount());
        assert!(MovementType::Harvest.affects_live_headcount());
    }
}
