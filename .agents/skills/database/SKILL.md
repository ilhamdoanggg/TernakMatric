---
name: Database
description: Pola PostgreSQL + SQLx TernakMatric — RLS tenant isolation, migrasi immutable, composite FK, audit trigger, ledger append-only, dan provisioning role. Gunakan saat menulis SQL, migrasi, query SQLx, atau mengubah schema.
---

# Skill: Database (TernakMatric)

Aturan repo lengkap ada di `AGENTS.md`. Skill `ternakmatric` memuat aturan lintas area; skill ini fokus pada PostgreSQL, migrasi, RLS, dan query SQLx.

## Sumber kebenaran

| Kebutuhan | File |
| --- | --- |
| Schema & seed kanonik | `backend/migrations/` (`2026112401`–`2026112409`) |
| Nama tabel/kolom/satuan | `docs/data/DATA_DICTIONARY.md` |
| ERD | `docs/06-DATABASE-ERD.md`, `docs/data/ERD.md` |
| Schema terpasang | `backend/migrations/` |
| Audit migrasi | `docs/notes/note-4-migration-audit.md` |
| Provisioning role | `deploy/db/init-app-role.sql` |
| Runner migrasi | `deploy/db/run-migrations.sh`, `infra-db::run_migrations()` |

Jangan mengarang nama tabel/kolom/satuan. Cek migrasi yang sudah ada sebelum menambah yang baru.

## RLS dan tenant isolation

- Setiap tabel tenant: `organization_id uuid NOT NULL REFERENCES organizations(id)`.
- Wajib `ENABLE ROW LEVEL SECURITY` **dan** `FORCE ROW LEVEL SECURITY`.
- Policy `tenant_isolation` memakai `current_organization_id()`.
- Role aplikasi: `NOSUPERUSER NOBYPASSRLS`. Jangan menjalankan API/worker dengan role migrator.
- Context di-set per transaksi oleh `auth.context().begin_tx(pool)`:
  - `SELECT set_config('app.current_organization_id', $1, true)`
  - `SELECT set_config('app.current_user_id', $1, true)`
- Meskipun RLS aktif, **tetap bind `auth.organization_id.0` eksplisit** di setiap query tenant.
- Query di bare `PgPool` tidak punya tenant context.
- Jangan mempercayai header `X-Organization-Id`; `AuthPrincipal` sudah memverifikasi JWT + membership + RBAC.

Contoh pola transaksi:

```rust
let mut tx = auth.context().begin_tx(state.pool()).await.map_err(ApiError)?;
let rows: Vec<ThingDto> = sqlx::query_as(
    r#"
    SELECT id, organization_id, name
    FROM things
    WHERE organization_id = $1 AND status = $2
    ORDER BY created_at DESC
    LIMIT $3
    "#,
)
.bind(auth.organization_id.0)
.bind(&status)
.bind(limit.clamp(1, 200))
.fetch_all(&mut *tx)
.await
.map_err(|e| ApiError(map_sqlx_err(e)))?;
tx.commit().await.map_err(|e| ApiError(map_sqlx_err(e)))?;
```

## Migrasi

Format nama: `YYYYMMDDNN_snake_case.sql`. Migrasi yang sudah di-apply **immutable**; jangan diedit. Migrasi terbaru saat ini `2026112409_rls_triggers_hardening.sql`; file berikutnya `2026112410_...`.

Tabel tenant baru wajib punya:

```sql
CREATE TABLE things (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id uuid NOT NULL REFERENCES organizations(id),
    name text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    created_by uuid,
    updated_by uuid,
    UNIQUE (id, organization_id)
);

ALTER TABLE things ENABLE ROW LEVEL SECURITY;
ALTER TABLE things FORCE ROW LEVEL SECURITY;

CREATE POLICY tenant_isolation ON things
    USING (organization_id = current_organization_id())
    WITH CHECK (organization_id = current_organization_id());

CREATE INDEX idx_things_org_created
    ON things (organization_id, created_at DESC);
```

Checklist tambahan:

- FK anak memakai composite: `(parent_id, organization_id) REFERENCES parent(id, organization_id)`.
- Trigger audit untuk `created_by`/`updated_by` (lihat `2026112409_rls_triggers_hardening.sql`).
- Referensi global-or-tenant memakai pola trigger `assert_global_or_tenant_ref` dari `2026112401` dan `2026112409_rls_triggers_hardening.sql`.
- Index query path umum selalu berprefix `organization_id`.
- Ledger append-only: `stock_movements`, `inventory_transaction_lines`, `audit_logs` (dan sesuai ADR 002 `inventory_transactions`, `production_outputs`). Koreksi memakai reversal/adjustment, bukan `UPDATE`/`DELETE`.
- Seed global `organization_id IS NULL` tidak boleh bergantung pada `BYPASSRLS` (`2026112408_seed_catalogs.sql` dijalankan sebelum `2026112409_rls_triggers_hardening.sql`).

### Dua jalur migrasi

- `deploy/db/run-migrations.sh` → `public.schema_migrations`
- `infra-db::run_migrations()` → `sqlx::migrate!()` → `_sqlx_migrations`

Jangan mengaktifkan keduanya tanpa keputusan. Baca `docs/notes/note-4-migration-audit.md` sebelum menyentuh migrasi atau provisioning.

### Grant role aplikasi

Jika mengubah provisioning role, perbarui juga `deploy/db/init-app-role.sql` dan tambahkan `ALTER DEFAULT PRIVILEGES` agar tabel baru tetap ter-grant.

## Tipe data

- ID: `uuid` (`gen_random_uuid()`), dipetakan ke newtype `shared-kernel`.
- Uang/berat/laju: `numeric` → `rust_decimal::Decimal`. Jangan `float`/`double precision`.
- Waktu: `timestamptz` → `DateTime<Utc>`. Tanggal: `date` → `NaiveDate`.
- JSON: `jsonb` → `serde_json::Value`/tipe `ToSchema`.
- Enum domain: cek tipe yang sudah ada di schema; jangan bikin nilai baru tanpa migrasi + dictionary.

## Query dan error

- `query_as` + `FromRow` untuk baris; `query_scalar` untuk nilai tunggal; `fetch_optional` untuk lookup yang boleh kosong.
- Row tidak ditemukan → `AppError::NotFound`.
- Filter dinamis: `($2::text IS NULL OR kolom = $2)`.
- Pagination: `limit.clamp(1, 200)`.
- Lock baris induk `FOR UPDATE` untuk cek kapasitas/saldo/uniqueness.
- Commit eksplisit. Early return rollback dan itu disengaja.
- Transaksi singkat: tanpa HTTP, file I/O, atau komputasi berat.
- Mapping SQLSTATE hanya lewat `infra_db::map_sqlx_err`:

| SQLSTATE | AppError |
| --- | --- |
| `23505` | `Conflict` |
| `23503`, `23514` | `Validation` |
| `RowNotFound` | `NotFound` |
| lainnya | `Database` |

## Verifikasi

Database lokal:

```sh
docker compose up -d postgres
docker compose run --rm db-migrate
```

Untuk perubahan schema, uji dari database kosong (bukan hanya database yang sudah termigrasi). Belum ada harness DB integration; jangan asumsikan `#[sqlx::test]`/Testcontainers tersedia.
