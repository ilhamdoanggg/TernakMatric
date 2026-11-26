# TernakMatric — Kamus Data (Inventaris Tabel)

Sumber kebenaran untuk tipe kolom dan constraint adalah SQL migrasi, bukan dokumen ini. Dokumen ini menjelaskan **tujuan dan kepemilikan** tiap tabel.

> **Status**: inventaris ini mengikuti baseline [`001_ternakmatric_schema.sql`](001_ternakmatric_schema.sql). Tabel yang hanya ada di migrasi—`individual_tags`, `vaccination_schedules`, `biosecurity_logs`—adalah ekstensi yang tercantum di [`../docs/06-DATABASE-ERD.md`](../docs/06-DATABASE-ERD.md). Untuk status schema terpasang, baca [`README.md`](README.md) dan [`../notes/note-4-migration-audit.md`](../notes/note-4-migration-audit.md).

## Identitas dan SaaS

| Tabel | Cakupan | Tujuan |
| --- | --- | --- |
| `users` | Global | Identitas login dan profil |
| `organizations` | Global | Batas tenant |
| `organization_members` | Tenant | Keanggotaan dan status undangan |
| `roles` | Global/tenant | Definisi peran sistem atau custom |
| `permissions` | Global | Katalog izin |
| `role_permissions` | Global/tenant | Pemetaan peran ke izin |
| `member_roles` | Tenant | Peran yang diberikan kepada anggota organisasi |
| `plans` | Global | Definisi dan batas paket langganan |
| `subscriptions` | Tenant | Siklus hidup langganan organisasi |

## Farm, unit produksi, dan species

| Tabel | Cakupan | Tujuan |
| --- | --- | --- |
| `farms` | Tenant | Lokasi/lapangan fisik |
| `unit_types` | Global | Katalog unit generik: kolam, kandang, cage, pasture, hive, dll. |
| `production_systems` | Global | Katalog sistem produksi generik |
| `production_units` | Tenant | Unit fisik pemeliharaan dengan kapasitas dan dimensi opsional |
| `species` | Global/tenant | Species bawaan dan custom beserta kategori |
| `species_production_profiles` | Tenant | Parameter berversi per species/sistem pada `parameters jsonb` |
| `growth_stages` | Tenant | Rentang berat/umur per profil produksi |
| `feeding_guidelines` | Tenant | Rekomendasi pakan per tahap pertumbuhan |

## Produksi

| Tabel | Cakupan | Tujuan |
| --- | --- | --- |
| `production_batches` | Tenant | Satu siklus produksi untuk species apa pun |
| `batch_unit_assignments` | Tenant | Okupansi batch di unit produksi dalam rentang waktu tertentu |
| `stock_movements` | Tenant | Ledger populasi ternak bersifat append-only |
| `population_balances` | Tenant | Proyeksi populasi yang dipelihara dalam transaksi |
| `mortality_records` | Tenant | Catatan detail kematian/cull |
| `growth_samples` | Tenant | Sampel jumlah, berat, dan panjang opsional |
| `environment_metrics` | Global | Katalog metrik: air, udara, tanah, kandang, pakan |
| `environment_readings` | Tenant | Pengukuran manual/IoT dengan satuan eksplisit |
| `health_records` | Tenant | Observasi, dugaan penyakit, dan perawatan |
| `feeding_records` | Tenant | Pakan aktual yang diberikan ke batch |

## Inventory

| Tabel | Cakupan | Tujuan |
| --- | --- | --- |
| `warehouses` | Tenant | Lokasi gudang/penyimpanan |
| `feed_products` | Tenant | SKU pakan dan atribut produk |
| `feed_unit_conversions` | Tenant | Faktor konversi ke satuan kanonik |
| `inventory_transactions` | Tenant | Header transaksi masuk, keluar, transfer, dan penyesuaian |
| `inventory_transaction_lines` | Tenant | Kuantitas, satuan, biaya, lot, dan referensi batch |
| `inventory_balances` | Tenant | Proyeksi saldo inventory yang dipelihara dalam transaksi |

## Output dan keuangan

| Tabel | Cakupan | Tujuan |
| --- | --- | --- |
| `production_outputs` | Tenant | Daging, telur, susu, fry, cull, offspring, dan output lainnya |
| `expense_categories` | Global/tenant | Katalog kategori biaya |
| `expenses` | Tenant | Catatan biaya |
| `sales` | Tenant | Header penjualan |
| `sale_items` | Tenant | Kuantitas, satuan, dan harga output yang terjual |
| `calculation_versions` | Global | Versi dan definisi rumus |
| `calculation_runs` | Tenant | Snapshot input/hasil untuk reproduktibilitas |
| `audit_logs` | Global/tenant | Aktor, aksi, entitas, dan snapshot sebelum/sesudah |

## Sinkronisasi offline

| Tabel | Cakupan | Tujuan |
| --- | --- | --- |
| `devices` | Tenant | Perangkat web/mobile yang terdaftar |
| `sync_operations` | Tenant | Antrean mutasi klien yang idempoten dan hasil pemrosesannya |
| `sync_conflicts` | Tenant | Konflik update usang dan riwayat resolusinya |
| `outbox_events` | Tenant | Change feed untuk respons sinkronisasi dan notifikasi |

## Semantik kolom penting

- Kuantitas selalu membawa satuan eksplisit: `quantity`, `unit`, `capacity_value`, `capacity_unit`, `metric_value`, `unit`.
- Nilai husbandry spesifik species berada di `species_production_profiles.parameters`, bukan di kolom generik.
- Waktu kejadian memakai `timestamptz`; tanggal bisnis seperti `start_date` memakai `date`.
- `organization_id` adalah batas keamanan, bukan sekadar kolom pelaporan.
- `base_version` pada operasi sinkronisasi adalah versi entitas terakhir yang dilihat klien; server wajib membandingkannya dengan versi server saat ini.
- `client_operation_id` dibuat di perangkat dan tetap sama saat retry.
- `input_snapshot` dan `result_snapshot` menyimpan input dan output kalkulasi secara persis.
- Ledger inventory, populasi, dan output dikoreksi dengan event reversal/adjustment, bukan `UPDATE`/`DELETE` destruktif.
- Resolusi konflik `client_wins` dilarang untuk entitas stok, inventory, output, dan keuangan.
