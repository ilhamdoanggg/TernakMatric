---
description: Workflow membuat migrasi PostgreSQL baru yang aman terhadap RLS, audit, dan provisioning role
---

Gunakan workflow ini saat menambah tabel, kolom, index, atau constraint baru di `backend/migrations/`. Rujukan: `AGENTS.md`, `docs/notes/note-4-migration-audit.md`, `.agents/skills/database/SKILL.md`.

1. **Audit schema yang sudah ada (jangan edit migrasi lama)**:
   - Migrasi yang sudah ada bersifat **immutable**. Migrasi terbaru saat ini `2026112409_rls_triggers_hardening.sql`.
   - Cek `docs/data/DATA_DICTIONARY.md`, `docs/06-DATABASE-ERD.md`, dan `backend/migrations/` agar nama tabel/kolom kanonik dan konsisten.
   - Buat file baru dengan format `backend/migrations/YYYYMMDDNN_snake_case.sql` (berikutnya `2026112410_...`).

2. **Terapkan checklist tabel tenant wajib**:
   - `organization_id uuid NOT NULL REFERENCES organizations(id)`
   - `UNIQUE (id, organization_id)` agar tabel anak bisa memakai composite FK `(parent_id, organization_id) REFERENCES parent(id, organization_id)`
   - `ENABLE ROW LEVEL SECURITY` dan `FORCE ROW LEVEL SECURITY`
   - Policy `tenant_isolation` memakai `current_organization_id()`
   - Kolom audit `created_at`, `updated_at`, `created_by`, `updated_by` + trigger audit
   - Index dengan prefix `organization_id` untuk query path umum
   - Referensi global-or-tenant memakai pola trigger `assert_global_or_tenant_ref`
   - Ledger (`stock_movements`, `inventory_transaction_lines`, `audit_logs`, serta `inventory_transactions`/`production_outputs` sesuai ADR 002) tetap append-only (koreksi via reversal/adjustment, bukan `UPDATE`/`DELETE`).

3. **Cek kompatibilitas role aplikasi & dua jalur migrasi**:
   - Pastikan tidak bergantung pada `BYPASSRLS`; periksa `deploy/db/init-app-role.sql` jika menambah objek atau schema baru agar role `ternakmatric_app` tetap punya hak akses.
   - Jangan mengubah dua jalur migrasi (`deploy/db/run-migrations.sh` vs `infra-db::run_migrations()`) tanpa keputusan eksplisit.

4. **Uji migrasi dari database kosong**:
   Jalankan workflow `/db-local` (atau reset volume lokal bila perlu menguji clean bootstrap) dan pastikan `docker compose run --rm db-migrate` selesai tanpa error.

5. **Selaraskan dokumentasi**:
   Perbarui `docs/data/DATA_DICTIONARY.md` dan `docs/06-DATABASE-ERD.md` jika menambah tabel/kolom/enum baru, lalu jalankan `cargo test --manifest-path backend/Cargo.toml --workspace`.
