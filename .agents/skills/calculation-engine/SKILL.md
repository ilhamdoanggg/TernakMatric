---
name: Calculation Engine
description: Use when adding or changing formulas in the TernakMatric mod-calculation crate, including purity rules, CalculationResult and versioning, Decimal rounding, input validation, species-profile parameters, and calculation tests.
---

# Calculation Engine (`mod-calculation`)

Production calculations are the core value of TernakMatric. The crate is deliberately pure and versioned; see `docs/decisions/003-calculation-engine.md`.

## Purity rules

`mod-calculation` must not use:

- SQLx, Axum, Tokio, or any network I/O,
- the system clock (`Utc::now()`), random numbers, or environment variables,
- global mutable state or hidden configuration.

Every input is a parameter, including `calculated_at: DateTime<Utc>`. This makes results reproducible and testable.

## Result shape

Every calculation returns `Result<CalculationResult<T>, CalculationError>`:

```rust
pub fn compute_abw_g(
    total_sample_weight_g: Decimal,
    sample_count: i64,
    calculated_at: DateTime<Utc>,
) -> Result<CalculationResult<Decimal>, CalculationError> {
    // validate, compute, round
    Ok(CalculationResult::exact(
        abw,
        "g/head",
        "ABW",
        1,
        None,
        calculated_at,
    ))
}
```

`CalculationResult` carries:

- `value` and `unit`,
- `formula_code` and `formula_version`,
- `parameter_version` (the `species_production_profiles.parameters` version when profile parameters are used),
- `calculated_at`,
- `assumptions` (`is_estimate`, `last_sample_at`, `notes`).

Use `CalculationResult::estimate(...)` and record assumptions when an input is sampled or estimated. Use `exact(...)` only when every input is measured.

## Versioning

- `formula_version` is an integer per formula code. Bump it whenever the semantics of the formula change; do not silently change results that historical `calculation_runs` already recorded under version 1.
- Pass the profile parameter version through as `parameter_version`.
- New formula: add a module file, `pub mod` and `pub use` it in `lib.rs`, assign a stable `formula_code`, and add tests.
- Never branch on species names. Species- and system-specific behavior belongs in versioned profile parameters, which are passed in explicitly.

## Decimal and numeric rules

- Use `rust_decimal::Decimal`; never `f64` or `f32`.
- Validate before dividing:
  - positive weights, counts, and days,
  - no division by zero,
  - population must not go negative.
- Return `CalculationError::InvalidInput { field, reason }`, `DivisionByZero`, `NonPositiveBiomassGain`, or `NegativePopulation` as appropriate. Never panic.
- Use checked math for transcendental functions (`checked_ln`, checked division) and propagate errors.
- Round explicitly with `round_dp(n)` and keep the established scale:
  - ABW and ADG: 3 decimals,
  - SGR: 4 decimals,
  - currency and percentages: follow the existing module and tests.
- Document the formula in a doc comment, including its unit and equation.

## No universal defaults

Universal density, stocking density, and feeding-rate constants are explicitly forbidden. Those values come from versioned `species_production_profiles.parameters`, selected by the caller and referenced by `parameter_version`. If a formula needs such a parameter, accept it as an argument; do not hardcode a default.

## Errors

`CalculationError` lives in `version.rs`. Add a variant only for a distinct failure mode a caller must handle. Keep `#[derive(Debug, Error, Clone, PartialEq, Eq)]` and stable messages.

## Tests

Add tests to `backend/crates/mod-calculation/src/tests.rs`:

- one happy path per formula,
- boundary and invalid-input cases (zero days, zero weight, negative population),
- exact `Decimal` assertions with the expected scale,
- approximate assertions only for irrational results (ln-based SGR),
- a fixed timestamp helper instead of the real clock.

```rust
#[test]
fn test_growth_adg_sgr_poultry_and_dairy() {
    let adg = compute_adg_g_per_day(dec!(42), dec!(1842), dec!(30), Some(1), fixed_time()).unwrap();
    assert_eq!(adg.value, dec!(60.000));
}
```

## Verification

```powershell
cargo test --manifest-path backend/Cargo.toml -p mod-calculation
cargo clippy --manifest-path backend/Cargo.toml -p mod-calculation --all-targets -- -D warnings
```

If a handler consumes the formula, also update its mapping from `CalculationError` to `AppError::Validation` and its API tests.
