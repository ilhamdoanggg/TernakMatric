# ADR 003 — Calculation Engine sebagai Domain Murni Berversi

- Status: Diterima
- Tanggal: 2026-11-24
- Terkait: `note-1.md` (calculation engine konsisten), `note-2.md` (business rules), `note-3.md` (versioning parameter & formula)

## Konteks

Perhitungan produksi ternak adalah nilai jual TernakMatric: kapasitas unit produksi (kolam, kandang, cage, hive), populasi, survival rate, ABW, biomassa, weight gain/ADG, egg rate, milk yield, FCR, estimasi output daging/telur/susu/fry, dan P&L. Berlaku untuk semua kategori spesies `fish | crustacean | mollusk | poultry | mammal | reptile | insect | other`. note-2 mensyaratkan semua kalkulasi terpisah dari controller/UI, mencatat versi formula dan parameter, serta tidak menggunakan density dan feeding rate universal. note-1 menekankan engine yang deterministik dan teruji.

## Keputusan

1. `mod-calculation` adalah crate Rust murni: **tanpa DB, tanpa HTTP, tanpa clock/network**. Semua input diterima eksplisit, termasuk tanggal dan parameter.
2. Satu modul per metrik: `unit_capacity`, `population`, `biomass`, `survival`, `mortality_rate`, `weight_gain`, `egg_rate`, `milk_yield`, `growth`, `fcr`, `output`, `finance`, plus `version`.
3. Setiap hasil kalkulasi membawa metadata:

   ```text
   value, unit, formula_version, parameter_version, calculated_at, assumptions
   ```

4. Density/okupansi unit, feeding rate, kurva pertumbuhan, target ADG, egg rate, milk yield, dan parameter lingkungan (air/udara/tanah/kandang) **tidak di-hardcode**. Sumber kanoniknya adalah `species_production_profiles` (`parameters` jsonb, `version`, `valid_from`/`valid_until`) dari `mod-species`, lalu dilewatkan sebagai input ke engine.
5. Estimasi dibedakan dari aktual: biomassa dan output forecast (daging, telur, susu, fry, offspring) adalah estimasi berbasis sampling terakhir + asumsi survival, bukan penimbangan seluruh populasi.
6. FCR memakai periode pengamatan konsisten dan memperhitungkan panen parsial, kematian, serta cull sesuai metode yang dipilih. Untuk kategori poultry/mammal dengan output telur atau susu, dipakai `egg_rate` dan `milk_yield` dari profil spesies sebagai metrik utama.
7. Persistensi hasil ada di luar engine, melalui `calculation_runs` dan `calculation_versions` yang dikelola `mod-analytics`. Engine tidak menulis ke database.
8. Satu-satunya angka satuan yang dipakai adalah `Decimal` dengan konversi unit eksplisit (gram/kg, m³, m², hari, liter, butir), bukan `f64` untuk nilai finansial. Hasil `calculation_runs` menyimpan `unit_id`, bukan kolom kolam lama.

## Konsekuensi

- `mod-calculation/src/tests.rs` berisi unit test untuk kasus nyata dan property-based test (misalnya populasi tidak negatif, FCR positif untuk gain positif).
- Seed data di `002_ternakmatric_seed.sql` harus mengisi species pada kategori `fish | crustacean | mollusk | poultry | mammal | reptile | insect | other`, unit types, production systems, dan `species_production_profiles` dengan `version`, `valid_from`/`valid_until`, serta sumber referensi.
- API analytics mengembalikan estimasi beserta `assumptions`, `formula_version`, dan tanggal sampling terakhir — bukan angka tunggal tanpa konteks.
- Perubahan rumus dibuat sebagai versi baru; hasil historis tidak dihitung ulang dengan rumus baru.
