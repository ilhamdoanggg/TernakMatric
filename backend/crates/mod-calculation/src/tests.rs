use crate::*;
use chrono::{TimeZone, Utc};
use rust_decimal_macros::dec;
use shared_kernel::MovementType;

fn fixed_time() -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 11, 24, 12, 0, 0).unwrap()
}

#[test]
fn test_enclosure_geometry_and_multi_batch_capacity() {
    let rect = compute_enclosure_geometry(
        &EnclosureGeometry::Rectangle {
            length_m: dec!(10),
            width_m: dec!(5),
            average_depth_m: Some(dec!(1.2)),
        },
        fixed_time(),
    )
    .unwrap();
    assert_eq!(rect.value.area_m2, dec!(50.000));
    assert_eq!(rect.value.volume_m3, Some(dec!(60.000)));

    let ok_cap =
        evaluate_enclosure_capacity(dec!(1500), dec!(1000), dec!(400), Some(1), fixed_time())
            .unwrap();
    assert!(ok_cap.value.fits);
    assert_eq!(ok_cap.value.utilization_percent, dec!(93.33));

    let overflow_cap =
        evaluate_enclosure_capacity(dec!(1500), dec!(1200), dec!(400), Some(1), fixed_time())
            .unwrap();
    assert!(!overflow_cap.value.fits);
}

#[test]
fn test_population_and_survival_across_partial_harvest_and_egg_collection() {
    let entries = vec![
        StockLedgerEntry {
            movement_type: MovementType::InitialStock,
            quantity_delta: 1000,
        },
        StockLedgerEntry {
            movement_type: MovementType::Mortality,
            quantity_delta: -30,
        },
        StockLedgerEntry {
            movement_type: MovementType::Harvest,
            quantity_delta: -200,
        },
        StockLedgerEntry {
            movement_type: MovementType::EggCollection,
            quantity_delta: 800, // must NOT affect headcount
        },
    ];
    let pop = compute_population_balance(&entries, fixed_time()).unwrap();
    assert_eq!(pop.value.current_alive, 770);
    assert_eq!(pop.value.total_stocked, 1000);
    assert_eq!(pop.value.total_harvested_or_sold, 200);

    let sr_cum = compute_survival_rate(
        &pop.value,
        SurvivalMode::CumulativeAccounted,
        Some(1),
        fixed_time(),
    )
    .unwrap();
    assert_eq!(sr_cum.value, dec!(97.00));
}

#[test]
fn test_growth_adg_sgr_poultry_and_dairy() {
    let abw = compute_abw_g(dec!(5000), 50, fixed_time()).unwrap();
    assert_eq!(abw.value, dec!(100.000));

    let adg = compute_adg_g_per_day(dec!(42), dec!(1842), dec!(30), Some(1), fixed_time()).unwrap();
    assert_eq!(adg.value, dec!(60.000)); // broiler 60 g/day

    let sgr =
        compute_sgr_percent_per_day(dec!(5), dec!(100), dec!(60), Some(1), fixed_time()).unwrap();
    assert!(sgr.value > dec!(4.9) && sgr.value < dec!(5.1)); // ln(20)/60*100 ~= 4.9929

    let egg_rate = compute_egg_rate_percent(450, 500, dec!(1), Some(1), fixed_time()).unwrap();
    assert_eq!(egg_rate.value, dec!(90.00));

    let milk =
        compute_milk_yield_per_head_day(dec!(360), 20, dec!(1), Some(1), fixed_time()).unwrap();
    assert_eq!(milk.value, dec!(18.000));
}

#[test]
fn test_fcr_economic_vs_biological_and_zero_gain_guard() {
    let fcr_econ = compute_fcr(
        &FcrInput {
            total_feed_kg: dec!(180),
            initial_biomass_kg: dec!(10),
            added_stock_biomass_kg: dec!(0),
            current_standing_biomass_kg: dec!(90),
            harvested_biomass_kg: dec!(40),
            mortality_and_cull_biomass_kg: dec!(5),
            mode: FcrMode::Economic,
            last_sample_at: Some(fixed_time()),
        },
        Some(1),
        fixed_time(),
    )
    .unwrap();
    // Gain = 90 + 40 - 10 = 120 kg -> FCR = 180 / 120 = 1.500
    assert_eq!(fcr_econ.value.fcr, dec!(1.500));

    let fcr_bio = compute_fcr(
        &FcrInput {
            total_feed_kg: dec!(180),
            initial_biomass_kg: dec!(10),
            added_stock_biomass_kg: dec!(0),
            current_standing_biomass_kg: dec!(90),
            harvested_biomass_kg: dec!(40),
            mortality_and_cull_biomass_kg: dec!(5),
            mode: FcrMode::Biological,
            last_sample_at: Some(fixed_time()),
        },
        Some(1),
        fixed_time(),
    )
    .unwrap();
    // Gain = 125 kg -> FCR = 180 / 125 = 1.440
    assert_eq!(fcr_bio.value.fcr, dec!(1.440));

    let err = compute_fcr(
        &FcrInput {
            total_feed_kg: dec!(50),
            initial_biomass_kg: dec!(100),
            added_stock_biomass_kg: dec!(0),
            current_standing_biomass_kg: dec!(80),
            harvested_biomass_kg: dec!(0),
            mortality_and_cull_biomass_kg: dec!(0),
            mode: FcrMode::Economic,
            last_sample_at: None,
        },
        None,
        fixed_time(),
    )
    .unwrap_err();
    assert!(matches!(
        err,
        CalculationError::NonPositiveBiomassGain { .. }
    ));
}

#[test]
fn test_hpp_per_kg_live_and_batch_pnl() {
    let costs = CostBreakdown {
        stock_cost: dec!(5000000),
        feed_cost: dec!(12000000),
        health_and_vaccine_cost: dec!(1000000),
        operational_cost: dec!(2000000),
    };
    let hpp = compute_hpp_per_kg_live(
        &costs,
        dec!(1000),
        dec!(0),
        HppBasis::RealizedHarvestOnly,
        fixed_time(),
    )
    .unwrap();
    assert_eq!(hpp.value, dec!(20000.00)); // Rp 20.000/kg hidup

    let pnl = compute_batch_pnl(&costs, dec!(26000000), fixed_time()).unwrap();
    assert_eq!(pnl.value.net_profit, dec!(6000000.00));
    assert_eq!(pnl.value.profit_margin_percent, Some(dec!(23.08)));
}
