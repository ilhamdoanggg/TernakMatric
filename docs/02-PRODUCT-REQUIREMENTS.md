# 02 — Product Requirements Document (PRD): TernakMatric

**Dokumen**: `TM-DOC-02` · **Status**: Baseline v1.0 · **Tanggal**: 2026-11-24

## 1. Ringkasan Eksekutif

| Atribut | Keputusan |
| --- | --- |
| **Nama Produk** | TernakMatric |
| **Tipe Produk** | B2B SaaS Multi-Tenant |
| **Target Pengguna** | Peternak UMKM, peternakan menengah–besar, perusahaan integrasi, pengelola banyak farm |
| **Cakupan Spesies** | `fish`, `crustacean`, `mollusk`, `poultry`, `mammal`, `reptile`, `insect`, `other` |
| **Platform** | Web Dashboard (Vue 3 + TypeScript / Vite) & Mobile Field App (Flutter + SQLite) |
| **Backend & DB** | Rust Modular Monolith (Axum + SQLx + Utoipa) + PostgreSQL 16 (RLS Multi-Tenant) |
| **Strategi Rilis** | MVP (Operasional & Ledger Inti) → V1 (BI, Forecasting & Rekonsiliasi Lanjutan) → V2 (IoT & AI) |

---

## 2. Persona & Matriks RBAC Multi-Tenant

Satu `user` dapat tergabung dalam banyak `organizations` dengan peran berbeda di setiap organisasi. Karena itu peran disimpan pada level `organization_members` / `member_roles`, bukan global di `users`.

| Role | Deskripsi | Hak Akses Utama |
| --- | --- | --- |
| **Platform Super Admin** | Pengelola internal SaaS | Manajemen katalog global (`species`, `unit_types`, `plans`), monitoring sistem |
| **Organization Owner** | Pemilik bisnis / tenant | Penuh atas organisasi, langganan, staf, semua farm, dan laporan Laba/Rugi |
| **Organization Admin** | Admin operasional organisasi | Mengelola anggota, role, master spesies custom, gudang, dan konfigurasi farm |
| **Farm Manager** | Kepala lokasi / kandang / kolam | Membuat/mengatur enclosure, batch, transfer unit, jadwal vaksin, dan analisis produksi |
| **Farm Operator** | Pekerja lapangan / anak kandang | Input pakan, mortalitas, sampling pertumbuhan, kondisi lingkungan, dan output harian (offline/online) |
| **Finance Staff** | Bagian keuangan | Mencatat biaya operasional, pembelian stok, penjualan output, serta melihat HPP & P&L |
| **Viewer** | Investor / auditor / peninjau | Akses baca dashboard, grafik performa, dan laporan tanpa hak ubah |

---

## 3. Ruang Lingkup Produk per Tahap Rilis

### Phase A — MVP (Core Production, Inventory, Health & Finance)
1. **Tenant, Multi-Farm & RBAC (`mod-tenant`)**:
   - Registrasi, login JWT, manajemen organisasi, undangan anggota, role per tenant, dan multi-farm.
2. **Enclosure Management (`mod-enclosure`)**:
   - Master tipe unit (`unit_types`) & sistem produksi (`production_systems`).
   - Manajemen kandang, kolam, tambak, cage, hive, rak serangga (`production_units`), dimensi JSONB, `area_m2`, `volume_m3`, dan kapasitas (`head`, `kg`, `m3`, `m2`, dll.).
   - Validasi kapasitas gabungan untuk **beberapa batch aktif** dalam satu enclosure.
3. **Dynamic Species & Profiles (`mod-species`)**:
   - Master spesies bawaan sistem (`organization_id IS NULL`) + spesies custom milik tenant.
   - Profil produksi berversi (`species_production_profiles`), fase pertumbuhan (`growth_stages`), dan panduan pakan (`feeding_guidelines`).
4. **Production Batch, Ledger & Individual Tags (`mod-production`)**:
   - Siklus/cohort (`production_batches`), penempatan ke unit (`batch_unit_assignments` — 1 assignment aktif per batch).
   - Ledger populasi *append-only* (`stock_movements`) + saldo terprojeksi (`population_balances`).
   - Pelacakan individu (`individual_tags`) seperti eartag sapi/kambing/babi atau RFID yang tertaut ke batch.
   - Sampling pertumbuhan (`growth_samples`) dan pencatatan output (`production_outputs`: daging, telur, susu, fry, cull, anakan).
5. **Feeding & Feed Inventory (`mod-feeding`)**:
   - Gudang (`warehouses`), SKU pakan/ransum (`feed_products`), konversi unit (`feed_unit_conversions`).
   - Transaksi gudang *append-only* (`receipt`, `issue`, `transfer`, `adjustment`, `return`) + saldo (`inventory_balances`).
   - Pencatatan pakan harian (`feeding_records`) terhubung ke pengeluaran gudang + kalkulator FCR.
6. **Environment Monitoring (`mod-environment`)**:
   - Katalog metrik lingkungan (`environment_metrics`: air, udara, tanah, kandang, pakan).
   - Pencatatan manual (`environment_readings`) dan kontrak webhook terstruktur siap IoT.
7. **Health, Vaccination & Biosecurity (`mod-health`)**:
   - Jadwal vaksinasi terencana (`vaccination_schedules`) dan pengingat otomatis dari `worker`.
   - Riwayat observasi penyakit, pengobatan (`health_records`), mortalitas terperinci (`mortality_records`), serta log biosecurity (`biosecurity_logs`: sanitasi, karantina, lalu lintas tamu/kendaraan).
8. **Finance, HPP & P&L (`mod-finance`)**:
   - Kategori biaya, pencatatan biaya langsung/operasional (`expenses`), penjualan berbasis output (`sales`, `sale_items`).
   - Perhitungan HPP per kg hidup (atau per unit output seperti butir telur/liter susu) dan Laba/Rugi per batch.
9. **Offline-First Sync (`infra-auth` + `apps/mobile`)**:
   - Penyimpanan lokal SQLite pada mobile, antrean *command* dengan `client_operation_id` (idempotency) dan `base_version` (deteksi konflik, tanpa `client_wins` untuk ledger/finance).

### Phase B — V1 (Business Intelligence & Forecasting)
- Snapshot harian otomatis (`worker`) dan persistensi histori kalkulasi (`calculation_runs`).
- Prediksi tanggal panen dan estimasi output (daging, telur, susu, fry) berbasis kurva pertumbuhan dan asumsi survival.
- Komparasi performa antar-enclosure, antar-farm, dan antar-siklus.
- Rekonsiliasi periodik ledger vs saldo proyeksi.

### Phase C — V2 (Smart Farming & IoT Automation)
- Ingesti sensor IoT real-time (DO, suhu air, pH, NH3 kandang, kelembapan) + mesin alert ambang batas di `worker`.
- Integrasi *auto-feeder* dan rekomendasi pakan/panen berbasis data historis.

---

## 4. Kebutuhan Non-Fungsional (NFR)

| Area | Target & Ketentuan |
| --- | --- |
| **Isolasi Tenant** | Pertahanan berlapis: verifikasi membership di backend + composite FK `(organization_id, id)` + PostgreSQL RLS (`SET LOCAL app.current_organization_id`) |
| **Integritas Data** | Semua mutasi stok/gudang/keuangan dijalankan dalam transaksi DB dengan *row locking* (`SELECT ... FOR UPDATE`) |
| **Auditabilitas** | Ledger bersifat *append-only*; setiap perubahan penting dicatat di `audit_logs`; hasil kalkulasi menyimpan versi formula & parameter |
| **Kinerja** | Latensi API p95 < 500 ms untuk transaksi harian dan pembacaan dashboard standar |
| **Ketersediaan** | Target awal 99,5% untuk API server |
| **Offline Resilience** | Mobile tetap beroperasi penuh saat putus koneksi; retry aman tanpa duplikasi data berkat `idempotency_key` |
| **Kontrak API** | *Single source of truth* dari Rust `utoipa` OpenAPI → generated TypeScript client untuk Web & Mobile |
