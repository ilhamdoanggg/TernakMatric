# TernakMatric — Database Hardening Checklist

Lanjutan dari [`note-3.md`](notes/note-3.md) bagian 7. Status checklist ini menjadi acuan *definition of done* untuk layer database pada `backend/migrations/` dan pengujian integrasi. Riwayat audit & status akhir dituangkan di [`note-4-migration-audit.md`](notes/note-4-migration-audit.md); "Terdefinisikan di SQL" berarti sudah ada di migrasi (`2026112401`–`2026112409`), menunggu verifikasi terhadap PostgreSQL live.

| No | Item | Modul / artefak | Cara verifikasi | Status |
| --- | --- | --- | --- | --- |
| 1 | Partial unique index: satu `batch_unit_assignments` aktif per batch | `2026112403_init_production_ledger.sql` (`uq_one_active_assignment_per_batch`) | Test insert dua assignment aktif → gagal | Terdefinisikan di SQL |
| 2 | Composite tenant FK `(id, organization_id)` untuk seluruh relasi bisnis | Migration `2026112401`–`2026112407`, `2026112409` | Test referensi silang tenant (termasuk `reversal_of` & `assert_global_or_tenant_ref`) → gagal | Terdefinisikan di SQL |
| 3 | RLS aktif pada tabel bisnis, membership, dan role; app role bukan superuser/owner/BYPASSRLS | `2026112408`, `2026112409`, `deploy/db/init-app-role.sql` | Test query sebagai organisasi A tidak melihat data B | Terdefinisikan di SQL & role script |
| 4 | Ledger append-only + reversal transaksi | `mod-production`, `mod-feeding`, `2026112409`, `init-app-role.sql` | Test `UPDATE`/`DELETE` baris lama → gagal; transisi `draft` terjaga trigger | Terdefinisikan di SQL & role script |
| 5 | Inventory balance projection + rekonsiliasi | `mod-feeding`, `bins/worker` | Test saldo = sum ledger setelah concurrency | Siap uji integrasi DB |
| 6 | Validasi kapasitas unit produksi transaksional (total biomassa/okupansi semua batch aktif) | `mod-enclosure`, `mod-production` | Test dua batch aktif melebihi kapasitas → ditolak | Siap uji integrasi DB |
| 7 | Entity versioning + sync change-feed/cursor (`outbox_events`) | `infra-auth`, `infra-db`, `2026112407`, `2026112409` | Test update dengan `base_version` stale → conflict | Siap uji integrasi DB |
| 8 | Resolusi konflik offline (larangan `client_wins` pada ledger/keuangan/projection) | `infra-auth`, `apps/mobile/src/offline`, `2026112407` | Test dua operator offline mengambil stok sama → satu rejected/conflict | Terdefinisikan di SQL & domain |
| 9 | Migration test otomatis & tenant-isolation test di CI | CI pipeline + PostgreSQL service | Job Postgres service hijau | Menunggu setup Postgres CI |
| 10 | Idempotency & concurrency test | `infra-auth`, `mod-production`, `mod-feeding` | Retry command dua kali → satu entitas | Siap uji integrasi DB |
| 11 | Semua constraint `CHECK` SQL yang dapat direpresentasikan (dimensi/area > 0, kuantitas > 0, nilai non-negatif, kategori spesies & `output_type` valid) | Migration `2026112401`–`2026112407` | Test constraint violation → gagal | Terdefinisikan di SQL |
| 12 | Aturan lintas baris (populasi, saldo, output ≤ stok) divalidasi di transaksi + row lock | Modul terkait (`SELECT ... FOR UPDATE`) | Test konkuren dua output → salah satu ditolak | Siap uji integrasi DB |
| 13 | Validasi parameter spesies per sistem produksi (`species_production_profiles.parameters`, `version`, `valid_from`/`valid_until`) | `mod-species`, `mod-calculation` | Test profil tanpa referensi/versi atau sudah expired → ditolak; hasil lama tidak berubah | Siap uji domain |

## Catatan

- Dua nilai yang **sengaja tidak ditetapkan**: kepadatan/okupansi unit produksi universal dan feeding rate universal. Keduanya harus berasal dari `species_production_profiles.parameters` berversi per spesies dan sistem produksi, dengan kategori `fish | crustacean | mollusk | poultry | mammal | reptile | insect | other` dan referensi yang bisa dipertanggungjawabkan.
- Nama kanonik schema mengikuti `docs/data/DATA_DICTIONARY.md` & `backend/migrations/`: `production_units`, `batch_unit_assignments`, `environment_readings`, `production_outputs`, `calculation_runs` (`unit_id`), bukan nama lama kolam/water quality/harvest.
- `calculation_versions` di `2026112408_seed_catalogs.sql` kini mencakup 23 definisi formula termasuk seluruh kode formula aktif di `mod-calculation`: `ABW`, `AREA_ONLY`, `CAPACITY_UTILIZATION`, `ENCLOSURE_BIOMASS_ESTIMATE`, `FEEDING_VARIANCE`, `HPP_LIVE_WEIGHT`, `HPP_OUTPUT_UNIT`, `EGG_OUTPUT_FORECAST`, dan `SGR`.
- `production_outputs` dilindungi trigger immutable `deny_ledger_mutation()` + `REVOKE UPDATE, DELETE`; header `inventory_transactions` memakai trigger `deny_posted_inventory_tx_mutation()` + `REVOKE DELETE` sehingga hanya status `draft` yang boleh bertransisi ke `posted`/`voided`, sedangkan baris `posted`/`voided` bersifat append-only (`2026112409`).
- Checklist eksekusi PostgreSQL & CI lanjutan (non-superuser fresh migration, RBAC `role_permissions`, worker `updated_by` NULL, retry sync) dipantau di bagian 7 [`note-4-migration-audit.md`](notes/note-4-migration-audit.md).
