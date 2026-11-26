# TernakMatric — Referensi Database (Baseline)

**Status**: baseline desain untuk review · **Versi**: v2.0 · **Diselaraskan**: 2026-11-26

Direktori ini adalah **sumber kebenaran untuk nama tabel, kolom, dan domain** pada tahap desain. Schema yang **benar-benar dipasang** berada di [`../backend/migrations/`](../backend/migrations/). Untuk status pemasangan dan selisihnya, baca [`../notes/note-4-migration-audit.md`](../notes/note-4-migration-audit.md) dan [`../docs/06-DATABASE-ERD.md`](../docs/06-DATABASE-ERD.md).

> **Baseline vs ekstensi**: `001_ternakmatric_schema.sql` dan `002_ternakmatric_seed.sql` tidak mencakup seluruh tabel/kolom di migrasi. Tambahan seperti `individual_tags`, `vaccination_schedules`, `biosecurity_logs`, `environment_readings.source`, `health_records.withdrawal_days`, tipe kalkulasi `sgr`/`hpp`, dan kolom audit `created_by`/`updated_at`/`updated_by` adalah **ekstensi yang diterapkan lewat migrasi**, bukan koreksi terhadap baseline ini.

## Isi direktori

| File | Peran |
| --- | --- |
| `001_ternakmatric_schema.sql` | Baseline schema PostgreSQL 16: tabel, constraint, index, trigger, dan policy RLS awal |
| `002_ternakmatric_seed.sql` | Data referensi global: katalog species, unit types, production systems, metrik lingkungan, paket, dan versi formula |
| `DATA_DICTIONARY.md` | Inventaris tabel, cakupan (global/tenant), dan tujuan domain |
| `ERD.md` | Gambaran relasi dan keputusan domain |

## Cara pakai (referensi review)

1. Tinjau schema bersama database/security engineer.
2. Terapkan `001_ternakmatric_schema.sql` dengan role migrator, lalu `002_ternakmatric_seed.sql`.
3. Buat role aplikasi terpisah, bukan superuser/owner, dan tanpa `BYPASSRLS`. Lihat [`../deploy/db/init-app-role.sql`](../deploy/db/init-app-role.sql).
4. Berikan hanya grant yang diperlukan.
5. Set `app.current_organization_id` memakai `SET LOCAL` di setiap transaksi tenant.

```sql
BEGIN;
SET LOCAL app.current_organization_id = '00000000-0000-0000-0000-000000000001';
-- query tenant di sini
COMMIT;
```

Untuk menjalankan schema yang dipakai aplikasi, gunakan:

```powershell
docker compose up -d postgres
docker compose run --rm db-migrate
```

## Prinsip yang tidak berubah

- Model mencakup ikan, crustacean, mollusk, unggas, babi, kambing/domba, sapi/kerbau, kelinci, reptil, serangga, dan species custom tenant. Jangan menambah kolom spesifik species ke tabel generik.
- `users` dan `organizations` adalah entitas global; keduanya **sengaja tidak memakai RLS tenant** sampai ada policy berbasis membership/GRANT.
- RLS adalah pertahanan berlapis: pengecekan otorisasi dan membership di backend tetap wajib.
- Owner tabel, superuser, dan role ber-`BYPASSRLS` dapat melewati policy. Jangan pakai salah satunya sebagai role runtime.
- `species`, `unit_types`, `production_systems`, `environment_metrics`, `roles`, `expense_categories`, dan `calculation_versions` memiliki baris global (`organization_id IS NULL`). Hak tulis aplikasi harus dibatasi.
- `species_production_profiles.parameters` adalah data berversi per species/strain/sistem. Nilai husbandry universal **tidak boleh** di-hardcode atau di-seed sebagai default.
- Ledger stok dan inventory bersifat append-only setelah posting. Koreksi memakai transaksi kompensasi, bukan `UPDATE`/`DELETE`.
- Sinkronisasi offline memerlukan UUID buatan klien, idempotency key, base version, resolusi konflik, dan validasi server. Jangan menerapkan update usang secara membabi buta.
- Kuantitas inventory dinormalisasi ke satuan kanonik per produk (disarankan kg untuk pakan). Transfer memerlukan pasangan efek ledger keluar/masuk dalam satu transaksi.
- Satu batch hanya memiliki satu assignment aktif. Transfer menutup assignment lama dan membuka yang baru dalam satu transaksi.
- `production_outputs` adalah ledger output generik (daging, telur, susu, fry, cull, offspring, atau lainnya) dan ditulis dalam transaksi yang sama dengan `stock_movements` terkait.
- `environment_readings.metric_id` merujuk katalog metrik global; `metric_value` dan `unit` disimpan eksplisit agar satu tabel dapat menampung air, udara, tanah, kandang, dan pakan.

## Item yang masih terbuka

Status terkini dari item berikut dicatat di [`../notes/note-4-migration-audit.md`](../notes/note-4-migration-audit.md):

1. Policy/grant berbasis membership untuk `users` dan `organizations`.
2. Workflow posting dan trigger immutable untuk header inventory serta output produksi.
3. Routine pemeliharaan projection `population_balances`/`inventory_balances` dengan row lock yang terverifikasi.
4. Transaksi transfer unit dan model reservasi kapasitas.
5. Metadata storage lampiran dan policy aksesnya.
6. Siklus hidup paket/langganan dan usage metering.
7. Model akuntansi yang lebih lengkap bila dibutuhkan.
8. UI tenant untuk species custom dan profil produksi custom.
9. Seed `role_permissions` dan urutan seed global terhadap RLS hardening.
