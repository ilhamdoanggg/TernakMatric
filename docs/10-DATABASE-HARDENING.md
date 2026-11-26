# TernakMatric — Database Hardening Checklist

Lanjutan dari [`note-3.md`](../notes/note-3.md) bagian 7. Status checklist ini menjadi acuan *definition of done* untuk layer database pada `backend/migrations/` dan pengujian integrasi. Temuan yang masih menghalangi dituangkan di [`note-4-migration-audit.md`](../notes/note-4-migration-audit.md); "Terdefinisikan di SQL" berarti sudah ada di migrasi, belum terverifikasi oleh test otomatis.

| No | Item | Modul / artefak | Cara verifikasi | Status |
| --- | --- | --- | --- | --- |
| 1 | Partial unique index: satu `batch_unit_assignments` aktif per batch | `2026112403_init_production_ledger.sql` (`uq_one_active_assignment_per_batch`) | Test insert dua assignment aktif → gagal | Terdefinisikan di SQL |
| 2 | Composite tenant FK `(id, organization_id)` untuk seluruh relasi bisnis | Semua migration `2026112401`–`2026112407` | Test referensi silang tenant → gagal | Terdefinisikan di SQL |
| 3 | RLS aktif pada tabel bisnis, membership, dan role; app role bukan superuser/owner/BYPASSRLS | `2026112408_rls_triggers_hardening.sql` | Test query sebagai organisasi A tidak melihat data B | Terdefinisikan di SQL |
| 4 | Ledger append-only + reversal transaksi | `mod-production`, `mod-feeding`, `2026112408_rls_triggers_hardening.sql` | Test `UPDATE`/`DELETE` baris lama → gagal; reversal tidak mengubah saldo historis | Terdefinisikan di SQL |
| 5 | Inventory balance projection + rekonsiliasi | `mod-feeding`, `bins/worker` | Test saldo = sum ledger setelah concurrency | Siap uji integrasi DB |
| 6 | Validasi kapasitas unit produksi transaksional (total biomassa/okupansi semua batch aktif) | `mod-enclosure`, `mod-production` | Test dua batch aktif melebihi kapasitas → ditolak | Siap uji integrasi DB |
| 7 | Entity versioning + sync change-feed/cursor (`outbox_events`) | `infra-auth`, `infra-db`, `2026112407_init_offline_sync.sql` | Test update dengan `base_version` stale → conflict | Siap uji integrasi DB |
| 8 | Resolusi konflik offline (larangan `client_wins` pada ledger/keuangan) | `infra-auth`, `apps/mobile/src/offline`, `2026112407` | Test dua operator offline mengambil stok sama → satu rejected/conflict | Terdefinisikan di SQL & domain |
| 9 | Migration test otomatis & tenant-isolation test di CI | CI pipeline + PostgreSQL service | Job Postgres service hijau | Menunggu setup Postgres CI |
| 10 | Idempotency & concurrency test | `infra-auth`, `mod-production`, `mod-feeding` | Retry command dua kali → satu entitas | Siap uji integrasi DB |
| 11 | Semua constraint `CHECK` SQL yang dapat direpresentasikan (dimensi/area > 0, kuantitas > 0, nilai non-negatif, kategori spesies & `output_type` valid) | Migration `2026112401`–`2026112407` | Test constraint violation → gagal | Terdefinisikan di SQL |
| 12 | Aturan lintas baris (populasi, saldo, output ≤ stok) divalidasi di transaksi + row lock | Modul terkait (`SELECT ... FOR UPDATE`) | Test konkuren dua output → salah satu ditolak | Siap uji integrasi DB |
| 13 | Validasi parameter spesies per sistem produksi (`species_production_profiles.parameters`, `version`, `valid_from`/`valid_until`) | `mod-species`, `mod-calculation` | Test profil tanpa referensi/versi atau sudah expired → ditolak; hasil lama tidak berubah | Siap uji domain |

## Catatan

- Dua nilai yang **sengaja tidak ditetapkan**: kepadatan/okupansi unit produksi universal dan feeding rate universal. Keduanya harus berasal dari `species_production_profiles.parameters` berversi per spesies dan sistem produksi, dengan kategori `fish | crustacean | mollusk | poultry | mammal | reptile | insect | other` dan referensi yang bisa dipertanggungjawabkan.
- Nama kanonik schema mengikuti `data/`: `production_units`, `batch_unit_assignments`, `environment_readings`, `production_outputs`, `calculation_runs` (`unit_id`), bukan nama lama kolam/water quality/harvest.
- `calculation_versions` di `2026112409_seed_catalogs.sql` telah diperluas mencakup `SGR` (*Specific Growth Rate*) dan `HPP_LIVE_WEIGHT` (*Cost of Production per kg live weight*) selaras dengan `mod-calculation`.
- Kode-kode formula yang dipakai kode (`AREA_ONLY`, `CAPACITY_UTILIZATION`, `ENCLOSURE_BIOMASS_ESTIMATE`, `FEEDING_VARIANCE`, `HPP_OUTPUT_UNIT`, `EGG_OUTPUT_FORECAST`) belum semuanya tersedia di seed; lihat F-09 di `note-4`.
- `inventory_transactions` dan `production_outputs` belum memiliki trigger immutable; status dan usulannya ada di F-04 `note-4`.
