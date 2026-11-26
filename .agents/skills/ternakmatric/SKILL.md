---
name: TernakMatric
description: Aturan kerja dan checklist untuk mengubah kode di repository TernakMatric (Rust Axum + SQLx/PostgreSQL, Vue 3 SSR). Gunakan sebelum mengubah handler/DTO, schema atau migrasi, calculation engine, kontrak API, atau apps/web.
---

# Skill: TernakMatric

Sumber kebenaran lengkap ada di `AGENTS.md` di root repo. Skill ini ringkasan operasional; jika ada pertentangan, `AGENTS.md` yang menang.

Dokumentasi dan catatan memakai **Bahasa Indonesia**. Identifier kode, nama file, dan pesan error memakai **Inggris**.

## Sebelum mengubah apa pun

1. Baca `AGENTS.md` dan dokumen area terkait di `docs/` (termasuk `docs/data/` dan `docs/notes/`).
2. Cek kamus data & ERD: `docs/data/DATA_DICTIONARY.md`, `docs/06-DATABASE-ERD.md`.
3. Cek schema & migrasi SQL yang benar-benar terpasang di `backend/migrations/`. Sebelum menyentuh migrasi, baca `docs/notes/note-4-migration-audit.md`.
4. Jangan mengarang nama tabel/kolom/satuan. Gunakan nama kanonik.

## Aturan yang paling sering dilanggar

1. **Tenant isolation tidak bisa ditawar.**
   - Semua query tenant di dalam `auth.context().begin_tx(state.pool())`.
   - Tetap bind `auth.organization_id.0` eksplisit meskipun RLS aktif.
   - Jangan percaya header `X-Organization-Id`; `AuthPrincipal` sudah memverifikasi JWT + membership + RBAC.
2. **`mod-calculation` murni.** Tanpa SQLx, Axum, Tokio, clock, network, env, atau global mutable state. Semua input (termasuk `calculated_at`) adalah parameter. Jangan hardcode density/feeding rate; ambil dari `species_production_profiles.parameters` berversi.
3. **`Decimal`, bukan `f64`/`f32`,** untuk money, berat, laju, dan pengukuran. Test memakai `dec!`.
4. **ID memakai newtype `shared-kernel`** (`OrganizationId`, `BatchId`, ...). Bind `.0` hanya di boundary SQL.
5. **Jangan akses tabel modul lain secara langsung.** Cari kontrak/API yang ada atau minta keputusan.
6. **Generated artifacts tidak diedit manual.** Sumber kebenaran adalah anotasi `utoipa::path` dan `ToSchema`. Regenerasi:
   ```sh
   npm --prefix packages/api-contracts run gen:openapi
   npm --prefix packages/api-contracts run gen:ts
   npm --prefix packages/api-contracts run typecheck
   ```
7. **Migrasi yang sudah di-apply immutable.** Jangan edit file lama. Migrasi terbaru `2026112409_rls_triggers_hardening.sql`; file berikutnya `2026112410_...`.
8. **Role aplikasi bukan superuser dan tidak punya `BYPASSRLS`.** Jangan menjalankan API/worker dengan role migrator.
9. **Gagal cepat di request path.** Tidak boleh `unwrap`, `expect`, `panic!`, `println!`, atau `dbg!` di handler/service. Pakai `tracing` dan `AppError` → `ApiError`.
10. **Web SSR aman dan memakai kontrak.** Tipe dari `@ternakmatric/api-contracts`; jangan definisikan ulang. Jangan akses `window`/`document`/`localStorage` saat render atau di top-level module. Hanya `VITE_*` yang boleh terekspos ke browser.

## Checklist endpoint Axum baru

1. Handler di `backend/crates/mod-<domain>/src/lib.rs`; endpoint lintas modul di `bins/api-server`.
2. Request `#[derive(Debug, Serialize, Deserialize, ToSchema)]`; response DTO `ToSchema` (+ `sqlx::FromRow` hanya jika dibaca langsung).
3. Command yang mengubah stok/ledger/saldo wajib punya `idempotency_key: Uuid`.
4. Handler generik over `S: AppBackendState`, pakai `AuthPrincipal`, panggil `auth.require(Permission::...)`.
5. Validasi input **sebelum** membuka transaksi; error → `AppError::Validation`.
6. Transaksi via `auth.context().begin_tx(...)`; bind `organization_id` eksplisit.
7. Untuk kapasitas/saldo/uniqueness, lock baris induk `FOR UPDATE`.
8. `AuditLogger::record` dan `OutboxWriter::append` di transaksi yang sama dengan perubahan domain.
9. Daftarkan route di `router<S>()`, merge + nest `ApiDoc` di `bins/api-server/src/main.rs`.
10. Tambah path assertion di `test_openapi_spec_builds_with_all_module_paths`.
11. Regenerasi OpenAPI + TypeScript, lalu commit hasilnya.

Gunakan sintaks Axum 0.8 `{id}`, bukan `:id`. Permission yang tersedia: `org.manage`, `farm.manage`, `species.manage`, `production.write`, `feeding.write`, `environment.write`, `health.write`, `finance.write`, `analytics.read`.

## Checklist migrasi baru

Format `YYYYMMDDNN_snake_case.sql`. Untuk tabel tenant baru sertakan:

- `organization_id uuid NOT NULL REFERENCES organizations(id)`
- `UNIQUE (id, organization_id)` dan composite FK anak `(parent_id, organization_id)`
- `ENABLE ROW LEVEL SECURITY` + `FORCE ROW LEVEL SECURITY`
- policy `tenant_isolation` memakai `current_organization_id()`
- kolom audit `created_at`, `updated_at`, `created_by`, `updated_by` + trigger
- index berprefix `organization_id`

Ledger append-only: `stock_movements`, `inventory_transaction_lines`, `audit_logs` (dan sesuai ADR 002 `inventory_transactions`, `production_outputs`). Koreksi memakai reversal/adjustment, bukan `UPDATE`/`DELETE`.

Ada dua jalur migrasi yang coexistence: `deploy/db/run-migrations.sh` (`public.schema_migrations`) dan `infra-db::run_migrations()` (`_sqlx_migrations`). Jangan aktifkan keduanya tanpa keputusan.

## Pola SQLx

- `query_as` + `FromRow` untuk baris; `query_scalar` untuk nilai tunggal; `fetch_optional` untuk lookup yang boleh kosong.
- Row tidak ditemukan → `AppError::NotFound`.
- Filter dinamis: `($2::text IS NULL OR kolom = $2)`.
- Pagination: `limit.clamp(1, 200)`.
- Commit eksplisit. Early return rollback dan itu disengaja.
- Transaksi singkat: tanpa HTTP, file I/O, atau komputasi berat.
- Mapping SQLSTATE hanya lewat `infra_db::map_sqlx_err` (`23505`→`Conflict`, `23503`/`23514`→`Validation`, `RowNotFound`→`NotFound`, lainnya→`Database`).

## Calculation engine

- Kembalikan `Result<CalculationResult<T>, CalculationError>` dengan `value`, `unit`, `formula_code`, `formula_version`, `parameter_version`, `calculated_at`, `assumptions`.
- `estimate(...)` untuk hasil berbasis sampling; `exact(...)` hanya jika semua input terukur.
- `formula_version` naik jika semantik rumus berubah; jangan ubah hasil lama di `calculation_runs`.
- Jangan branch berdasarkan nama species.
- Validasi pembagi sebelum pembagian; gunakan `checked_*` dan `round_dp` (ABW/ADG 3 desimal, SGR 4 desimal).

## Web (`apps/web`)

- Vue 3.5 SSR + TypeScript strict, Vue Router 4, Vite 6, Tailwind 3, `server.mjs` tanpa framework.
- `<script setup lang="ts">` + Composition API. Tanpa `any`.
- Semua HTTP lewat `createWebApiClient()` di `src/lib/api.ts`; jangan `fetch` mentah dan jangan hardcode base URL.
- Token/`organization_id` lewat `getToken`/`getOrganizationId`; client dibuat per request (jangan module scope saat SSR).
- `Decimal` dikirim sebagai string. Jangan `parseFloat` untuk aritmetika bisnis; format dengan `Intl.NumberFormat("id-ID")`.
- Tailwind utility-first, tema gelap `slate`/`emerald`. Tanpa UI kit/state library baru.
- Data yang harus tampil di HTML server diambil via `onServerPrefetch` atau `entry-server.ts`, bukan `onMounted`.
- Data mock hanya untuk scaffold; jangan tambah mock bila endpoint sudah ada. `DashboardView` dan `SpeciesOverviewTable` masih mock.
- Aksesibilitas: elemen semantik, label form, `alt`, tabel/navigasi bisa dioperasikan keyboard.

## Verifikasi (definition of done)

```sh
cargo fmt --manifest-path backend/Cargo.toml --all
cargo clippy --manifest-path backend/Cargo.toml --workspace --all-targets -- -D warnings
cargo test --manifest-path backend/Cargo.toml --workspace
```

Untuk handler/DTO juga regenerasi kontrak dan typecheck:

```sh
npm --prefix packages/api-contracts run gen:openapi
npm --prefix packages/api-contracts run gen:ts
npm --prefix packages/api-contracts run typecheck
```

Untuk `apps/web`:

```sh
npm --prefix apps/web run typecheck
npm --prefix apps/web run build
```

Untuk perubahan SSR, jalankan `npm --prefix apps/web run dev` dan pastikan halaman ter-render dari server (view source atau `Invoke-WebRequest`/`curl`), bukan hanya setelah hydration.

Database lokal: `docker compose up -d postgres` lalu `docker compose run --rm db-migrate`.

## Jangan lakukan

- Jangan edit `packages/api-contracts/openapi/*.json` atau `generated/typescript/*` dengan tangan.
- Jangan hapus `backend/Cargo.lock` atau memasukkannya ke `.gitignore`.
- Jangan menambah handler HTTP atau query SQL ke `shared-kernel`/`mod-calculation`.
- Jangan membuat cycle antar `mod-*`.
- Jangan menambah Vitest/Playwright atau test runner web tanpa keputusan.
- Jangan melemahkan assert yang gagal; perbaiki kode produksi atau laporkan.
- Jangan menaruh secret di variabel `VITE_*`.
