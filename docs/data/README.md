# TernakMatric — Dokumentasi Schema & Kamus Data

**Status**: diselaraskan dengan schema terpasang · **Versi**: v2.0 · **Diselaraskan**: 2026-11-26

Direktori ini berisi **kamus data dan dokumentasi relasi database**. Seluruh SQL schema, seed global, trigger audit, dan kebijakan RLS yang benar-benar dipasang berada di [`../../backend/migrations/`](../../backend/migrations/) (`2026112401`–`2026112409`). Status audit migrasi tercatat di [`../notes/note-4-migration-audit.md`](../notes/note-4-migration-audit.md) dan [`../06-DATABASE-ERD.md`](../06-DATABASE-ERD.md).

## Isi direktori

| File | Peran |
| --- | --- |
| `DATA_DICTIONARY.md` | Inventaris tabel, cakupan (global/tenant), dan tujuan domain |
| `ERD.md` | Gambaran relasi dan keputusan domain |

## Cara menjalankan schema & keamanan RLS

1. Terapkan migrasi SQL (`backend/migrations/2026112401`–`2026112409`) dengan role migrator via Docker Compose.
2. Provisioning role aplikasi (`NOSUPERUSER NOBYPASSRLS`) dijalankan otomatis oleh [`../../deploy/db/init-app-role.sql`](../../deploy/db/init-app-role.sql).
3. Set `app.current_organization_id` dan `app.current_user_id` memakai `SET LOCAL` di setiap transaksi tenant (dilakukan otomatis oleh `OrganizationContext::begin_tx`).

```sql
BEGIN;
SET LOCAL app.current_organization_id = '00000000-0000-0000-0000-000000000001';
-- query tenant di sini
COMMIT;
```

Untuk menjalankan schema dan migrasi database lokal, gunakan:

```sh
docker compose up -d postgres
docker compose run --rm db-migrate
```

## Prinsip yang tidak berubah

- Model mencakup ikan, crustacean, mollusk, unggas, babi, kambing/domba, sapi/kerbau, kelinci, reptil, serangga, dan species custom tenant. Jangan menambah kolom spesifik species ke tabel generik.
- `users` dan `organizations` adalah entitas global; keduanya **sengaja tidak memakai RLS tenant** dan aksesnya dikontrol oleh membership/RBAC di backend (sedangkan `SELECT` `organization_members` pra-tenant dibatasi ke `user_id = current_user_id()`).
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

1. Policy/grant tambahan bila diperlukan untuk `users` dan `organizations`.
2. Workflow posting `draft -> posted` pada header inventory di domain layer (trigger immutable & guard `draft` sudah terpasang di migrasi).
3. Routine pemeliharaan projection `population_balances`/`inventory_balances` dengan row lock yang terverifikasi lewat uji integrasi DB.
4. Transaksi transfer unit dan model reservasi kapasitas.
5. Metadata storage lampiran dan policy aksesnya.
6. Siklus hidup paket/langganan dan usage metering.
7. Model akuntansi yang lebih lengkap bila dibutuhkan.
8. UI tenant untuk species custom dan profil produksi custom.
9. Eksekusi langsung `docker compose` + uji integrasi CI terhadap PostgreSQL live.
