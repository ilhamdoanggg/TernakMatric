---
description: Workflow menambah atau mengubah rumus murni di crate mod-calculation
---

Gunakan workflow ini untuk perubahan pada engine kalkulasi `backend/crates/mod-calculation/`. Rujukan: `AGENTS.md`, `docs/decisions/003-calculation-engine.md`, `.agents/skills/calculation-engine/SKILL.md`.

1. **Pastikan aturan kemurnian (`mod-calculation`)**:
   - Dilarang memakai SQLx, Axum, Tokio, network, env, `Utc::now()`, random, atau global mutable state.
   - Semua input — termasuk `calculated_at: DateTime<Utc>` dan parameter spesies — wajib dikirim sebagai argumen fungsi.
   - Dilarang hardcode konstanta universal spesies (density, feeding rate, dsb.) atau branching berdasarkan nama spesies; gunakan parameter dari `species_production_profiles.parameters` berversi.

2. **Gunakan tipe `Decimal` & validasi ketat**:
   - Gunakan `rust_decimal::Decimal`; dilarang `f64`/`f32`.
   - Validasi count/weight/days positif sebelum pembagian, gunakan `checked_*`, dan terapkan `round_dp` sesuai skala standar (ABW/ADG 3 desimal, SGR 4 desimal).
   - Jangan pernah panic; kembalikan `CalculationError`.

3. **Kembalikan `CalculationResult<T>` & kelola versi**:
   - Isi `value`, `unit`, `formula_code`, `formula_version`, `parameter_version`, `calculated_at`, dan `assumptions`.
   - Pakai `CalculationResult::estimate(...)` untuk hasil berbasis sampling/asumsi; `exact(...)` hanya jika seluruh input terukur.
   - Naikkan `formula_version` jika semantik rumus berubah; jangan mengubah hasil historis di `calculation_runs`.

4. **Tambahkan unit test deterministik di `backend/crates/mod-calculation/src/tests.rs`**:
   - Gunakan `fixed_time()` dan `rust_decimal_macros::dec!`.
   - Uji happy path, edge/error cases (pembagi nol, input negatif/nol), dan version metadata.

// turbo
5. **Verifikasi crate & workspace**:
```powershell
cargo fmt --manifest-path backend/Cargo.toml --all
cargo clippy --manifest-path backend/Cargo.toml -p mod-calculation --all-targets -- -D warnings
cargo test --manifest-path backend/Cargo.toml -p mod-calculation
```
Jika handler API ikut berubah, lanjutkan dengan `/verify`.
