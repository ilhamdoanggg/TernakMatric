# AGENTS.md — TernakMatric

Kamu adalah coding agent di repository ini. Baca aturan ini sebelum mengubah kode, schema, atau dokumentasi. Jika aturan di sini bertentangan dengan kebiasaan umum, **ikuti aturan repo ini**.

---

## 1. Sumber kebenaran

| Area | Sumber kanonik |
| --- | --- |
| Produk, requirement, aturan bisnis | `docs/01-*` s.d. `docs/12-*`, `docs/decisions/001-003`, `docs/notes/note-1..3` |
| Nama tabel/kolom/schema | `backend/migrations/`, `docs/data/DATA_DICTIONARY.md`, `docs/06-DATABASE-ERD.md` |
| Schema yang benar-benar terpasang | `backend/migrations/` (`2026112401`–`2026112409`) |
| Utang teknis migrasi | `docs/notes/note-4-migration-audit.md` |
| Arsitektur & batas crate | `docs/07-SYSTEM-ARCHITECTURE.md`, `docs/09-REPOSITORY-STRUCTURE.md` |
| Kontrak API | Anotasi Utoipa di handler Rust → `packages/api-contracts/openapi/ternakmatric.v1.json` → `generated/typescript/schema.d.ts` → client `packages/api-contracts/src/index.ts` |
| Stack & pola web | `apps/web/` (Vue 3 SSR + TypeScript) dan `packages/api-contracts/README.md` |

Jangan mengarang nama tabel/kolom/satuan. Gunakan nama kanonik dari `docs/data/` dan migrasi di `backend/migrations/`.

Bahasa dokumentasi dan catatan: **Indonesia**. Identifier kode, nama file, dan pesan error stabil: **Inggris**.

---

## 2. Peta repository (ringkas)

```text
backend/
  migrations/              # SQLx/Postgres migrations (tergabung lengkap 2026112401–2026112409)
  crates/
    shared-kernel/         # tipe domain murni, AppError, ID newtype
    mod-calculation/       # engine kalkulasi murni, tanpa DB/HTTP/clock
    infra-db/              # PgPool, transaksi, RLS context, audit, outbox
    infra-auth/            # JWT, AuthPrincipal, RBAC, permission
    mod-tenant|enclosure|species|production|feeding|environment|health|finance
  bins/api-server/         # router Axum + ApiDoc + AppState
  bins/worker/             # snapshot harian, reminder vaksin, alert
docs/                      # seluruh produk, arsitektur, decisions, data/ (dictionary/ERD), & notes/
deploy/db/                 # runner migrasi + provisioning role aplikasi
packages/api-contracts/    # OpenAPI generated + client/tipe TypeScript
apps/web/                  # dashboard Vue 3 SSR + TypeScript (Vite 6, Tailwind)
apps/mobile/               # mobile offline-first (belum fokus utama)
```

`apps/web` adalah klien utama dan ikut diimplementasikan saat perubahan menyentuh kontrak API atau tampilan. `apps/mobile` belum menjadi fokus sampai ada keputusan terpisah.

---

## 3. Aturan mutlak

1. **Tenant isolation tidak bisa ditawar.**
   - Setiap query tenant wajib berada di transaksi `auth.context().begin_tx(state.pool())`.
   - `begin_tx` men-set `app.current_organization_id` dan `app.current_user_id` via `SET LOCAL`.
   - Query di bare `PgPool` **tidak punya tenant context**.
   - Tetap bind `auth.organization_id.0` secara eksplisit meskipun RLS aktif.
   - Jangan pernah mempercayai header `X-Organization-Id` langsung; `AuthPrincipal` sudah memverifikasi JWT + membership + RBAC.

2. **`mod-calculation` murni.**
   - Tidak boleh bergantung pada SQLx, Axum, Tokio, clock, network, env, atau global mutable state.
   - Semua input, termasuk `calculated_at: DateTime<Utc>`, harus menjadi parameter.
   - Dilarang hardcode density, stocking density, feeding rate, atau konstanta spesies universal. Ambil dari `species_production_profiles.parameters` berversi.

3. **Gunakan `Decimal`, bukan `f64`/`f32`, untuk nilai bisnis.**
   - Money, berat, laju, dan pengukuran memakai `rust_decimal::Decimal`.
   - Test memakai `rust_decimal_macros::dec!`.

4. **ID memakai newtype dari `shared-kernel`.**
   - `OrganizationId`, `UserId`, `BatchId`, dll. didefinisikan dengan `define_id!`.
   - Bind `.0` hanya di boundary SQL; di domain/handler tetap newtype.

5. **Jangan akses tabel modul lain secara langsung.**
   - Jika butuh data dari modul lain, cari kontrak/API yang tersedia atau minta keputusan eksplisit.

6. **Generated artifacts tidak diedit manual.**
   - Jangan edit `packages/api-contracts/openapi/*.json` atau `generated/typescript/*` dengan tangan.
   - Sumber kebenaran adalah anotasi `utoipa::path` dan `ToSchema`.

7. **Migrasi yang sudah di-apply bersifat immutable.**
   - Jangan mengedit file migrasi lama. Tambah file baru dengan nomor berikutnya.
   - Migration terbaru saat ini: `2026112409_rls_triggers_hardening.sql`; berikutnya `2026112410_...`.
   - Sebelum menyentuh migrasi, baca `docs/notes/note-4-migration-audit.md`.

8. **Role aplikasi bukan superuser dan tidak punya `BYPASSRLS`.**
   - `deploy/db/init-app-role.sql` mengatur `NOSUPERUSER NOBYPASSRLS`.
   - Jangan menjalankan API/worker dengan role migrator.

9. **Gagal cepat di request path.**
   - Tidak boleh `unwrap`, `expect`, `panic!`, `println!`, atau `dbg!` di handler/service.
   - Pakai `tracing` untuk log dan `shared_kernel::AppError` → `ApiError`.

10. **Web SSR harus aman dan memakai kontrak API.**
    - `apps/web` mengonsumsi `@ternakmatric/api-contracts`; jangan mendefinisikan ulang tipe request/response secara manual.
    - Jangan akses `window`, `document`, atau `localStorage` saat SSR atau di top-level module; guard dengan `import.meta.env.SSR` atau pindahkan ke `onMounted`.
    - Hanya variabel `VITE_*` yang boleh terekspos ke browser. Secret server tidak boleh memakai prefix `VITE_`.

---

## 4. Perintah wajib

Jalankan dari root repository dengan PowerShell:

```powershell
cargo fmt --manifest-path backend/Cargo.toml --all
cargo clippy --manifest-path backend/Cargo.toml --workspace --all-targets -- -D warnings
cargo test --manifest-path backend/Cargo.toml --workspace
```

Fokus crate tertentu:

```powershell
cargo test --manifest-path backend/Cargo.toml -p mod-calculation
cargo clippy --manifest-path backend/Cargo.toml -p mod-health --all-targets -- -D warnings
```

Kontrak API setelah perubahan handler/DTO:

```powershell
npm --prefix packages/api-contracts run gen:openapi
npm --prefix packages/api-contracts run gen:ts
npm --prefix packages/api-contracts run typecheck
```

Web (Vue 3 SSR):

```powershell
npm --prefix apps/web install
npm --prefix apps/web run dev
npm --prefix apps/web run typecheck
npm --prefix apps/web run build
```

`dev` menjalankan SSR server di `http://localhost:3000` (override dengan `PORT`). `build` sudah mencakup `vue-tsc --noEmit` dan build client + server.

Script `preview` memakai sintaks env POSIX, jadi di PowerShell jalankan setelah build:

```powershell
$env:NODE_ENV = "production"
node apps/web/server.mjs
```

Web membaca `VITE_API_URL` di browser dan `API_INTERNAL_URL` di server SSR.

Database lokal:

```powershell
docker compose up -d postgres
docker compose run --rm db-migrate
```

`backend/Cargo.lock` **tracked**. Jangan hapus atau masukkan ke `.gitignore`. Dependency bersama dideklarasikan sekali di `[workspace.dependencies]` dan diwariskan dengan `dep.workspace = true`.

---

## 5. Dependency direction

Ikuti `docs/07-SYSTEM-ARCHITECTURE.md`:

- `shared-kernel`: tipe domain murni. Tidak bergantung pada `sqlx`, `axum`, atau I/O.
- `mod-calculation`: engine murni. Tidak bergantung pada DB, HTTP, clock, atau network.
- `infra-db`: `PgPool`, transaksi, RLS context, `AuditLogger`, `OutboxWriter`.
- `infra-auth`: JWT, extractor, RBAC. Boleh bergantung pada `infra-db`.
- `mod-*`: handler + SQL domain. Boleh memakai `shared-kernel`, `infra-db`, `infra-auth`, `mod-calculation`.
- `bins/api-server`: hanya merangkai router, state, dan `ApiDoc`.
- `bins/worker`: background job.

Jangan menambahkan handler HTTP atau query SQL ke `shared-kernel` atau `mod-calculation`. Jangan buat cycle antar `mod-*`.

---

## 6. Menambah/mengubah endpoint Axum

Checklist urutan:

1. Taruh handler di `backend/crates/mod-<domain>/src/lib.rs`. Endpoint lintas modul/system (health, sync) ada di `bins/api-server`.
2. Request: `#[derive(Debug, Serialize, Deserialize, ToSchema)]`. Response DTO: `ToSchema`, tambah `sqlx::FromRow` hanya jika dibaca langsung dari query.
3. Command yang mengubah stok/ledger/saldo wajib punya `idempotency_key: Uuid`.
4. Handler generik over `S: AppBackendState`, pakai `AuthPrincipal`, panggil `auth.require(Permission::...)` sebelum bekerja.
5. Validasi input **sebelum** membuka transaksi; error → `AppError::Validation`.
6. Buka transaksi dengan `auth.context().begin_tx(...)`; bind `organization_id` eksplisit.
7. Untuk pengecekan kapasitas/saldo/uniqueness, lock baris induk dengan `FOR UPDATE`.
8. Tulis `AuditLogger::record` dan `OutboxWriter::append` di transaksi yang sama dengan perubahan domain.
9. Daftarkan route di `router<S>()`, lalu `.merge(...)` dan `nest(...)` `ApiDoc` di `bins/api-server/src/main.rs`.
10. Tambah path assertion di `test_openapi_spec_builds_with_all_module_paths`.
11. Regenerate OpenAPI + TypeScript, commit hasilnya.

Gunakan sintaks Axum 0.8 `{id}`, bukan `:id`.

Permission codes yang tersedia: `org.manage`, `farm.manage`, `species.manage`, `production.write`, `feeding.write`, `environment.write`, `health.write`, `finance.write`, `analytics.read`.

---

## 7. SQLx dan PostgreSQL

Pola transaksi tenant:

```rust
let mut tx = auth.context().begin_tx(state.pool()).await.map_err(ApiError)?;
let rows: Vec<SomeDto> = sqlx::query_as(
    r#"
    SELECT ...
    FROM some_table
    WHERE organization_id = $1 AND ...
    "#,
)
.bind(auth.organization_id.0)
.fetch_all(&mut *tx)
.await
.map_err(|e| ApiError(map_sqlx_err(e)))?;
tx.commit().await.map_err(|e| ApiError(map_sqlx_err(e)))?;
```

- `query_as` + `FromRow` untuk baris; `query_scalar` untuk nilai tunggal; `fetch_optional` untuk lookup yang boleh kosong.
- Row tidak ditemukan → `AppError::NotFound`, bukan `Option` sampai ke response.
- Filter dinamis mengikuti pola `($2::text IS NULL OR kolom = $2)`.
- Clamp pagination: `limit.clamp(1, 200)`.
- Commit eksplisit. Early return akan rollback dan itu disengaja.
- Transaksi singkat: tidak ada HTTP, file I/O, atau komputasi berat di dalamnya.

`infra_db::map_sqlx_err` adalah satu-satunya tempat mapping SQLSTATE:

| SQLSTATE | AppError |
| --- | --- |
| `23505` | `Conflict` |
| `23503`, `23514` | `Validation` |
| `RowNotFound` | `NotFound` |
| other | `Database` |

Jangan hand-roll SQLSTATE di handler.

Tidak ada `.sqlx/` cache karena query bersifat runtime. Jika memperkenalkan macro `query!`/`query_as!`:

```powershell
cargo sqlx prepare --workspace --manifest-path backend/Cargo.toml
```

lalu commit `.sqlx/` dan pastikan CI menjalankan `sqlx prepare --check`.

---

## 8. Migrasi database

Format nama: `YYYYMMDDNN_snake_case.sql`, contoh `2026112409_rls_triggers_hardening.sql`.

Untuk tabel tenant baru, sertakan:

- `organization_id uuid NOT NULL REFERENCES organizations(id)`
- `UNIQUE (id, organization_id)` agar anak bisa memakai composite FK
- FK anak `(parent_id, organization_id) REFERENCES parent(id, organization_id)`
- `ENABLE ROW LEVEL SECURITY` + `FORCE ROW LEVEL SECURITY`
- policy `tenant_isolation` memakai `current_organization_id()`
- audit columns `created_at`, `updated_at`, `created_by`, `updated_by` + trigger audit
- index dengan prefix `organization_id` untuk query path umum

Referensi global-or-tenant memakai pola trigger `assert_global_or_tenant_ref` dari `2026112401` dan `2026112409_rls_triggers_hardening.sql`.

Ledger bersifat append-only: `stock_movements`, `inventory_transaction_lines`, `audit_logs` (dan sesuai ADR 002 juga `inventory_transactions`, `production_outputs`). Koreksi memakai reversal/adjustment, bukan `UPDATE`/`DELETE`.

Seed global `organization_id IS NULL` tidak boleh bergantung pada `BYPASSRLS` (`2026112408_seed_catalogs.sql` dijalankan sebelum `2026112409_rls_triggers_hardening.sql`); lihat `docs/notes/note-4-migration-audit.md`.

Ada dua jalur migrasi yang masih coexistence:

- `deploy/db/run-migrations.sh` → `public.schema_migrations`
- `infra-db::run_migrations()` → `sqlx::migrate!()` → `_sqlx_migrations`

Jangan mengaktifkan keduanya tanpa keputusan. Jika mengubah provisioning role, perbarui juga `deploy/db/init-app-role.sql` dan tambahkan `ALTER DEFAULT PRIVILEGES` agar tabel baru tetap ter-grant.

---

## 9. Calculation engine

Setiap fungsi mengembalikan `Result<CalculationResult<T>, CalculationError>` dan membawa:

- `value`, `unit`
- `formula_code`, `formula_version`
- `parameter_version` dari profil berversi
- `calculated_at`
- `assumptions` (`is_estimate`, `last_sample_at`, `notes`)

Gunakan `CalculationResult::estimate(...)` untuk hasil berbasis sampling/asumsi; `exact(...)` hanya jika semua input terukur.

Versioning:

- `formula_version` naik jika semantik rumus berubah.
- Jangan mengubah hasil yang sudah tersimpan di `calculation_runs` dengan rumus lama.
- Jangan branch berdasarkan nama species; semua perilaku spesies lewat parameter profil berversi.

Validasi sebelum pembagian: count/weight/days positif, tidak dibagi nol, populasi tidak negatif. Gunakan `checked_*` dan `round_dp` sesuai skala yang sudah dipakai modul (ABW/ADG 3 desimal, SGR 4 desimal).

---

## 10. Web (`apps/web`)

Stack: Vue 3.5 SSR + TypeScript strict, Vue Router 4, Vite 6, Tailwind CSS 3, dan server SSR Node tanpa framework (`server.mjs`).

### Struktur

```text
apps/web/
  server.mjs                 # HTTP server; Vite middleware di dev, dist/ di produksi
  src/main.ts                # createSSRApp + factory router (satu instance per request)
  src/entry-client.ts        # hydration di browser
  src/entry-server.ts        # renderToString + preload links
  src/router/index.ts        # memory history saat SSR, web history di browser
  src/lib/api.ts             # createWebApiClient()
  src/components/            # komponen UI reusable
  src/features/<feature>/    # komponen spesifik fitur
  src/views/                 # halaman level route
```

View baru didaftarkan di `src/router/index.ts`. Komponen yang hanya dipakai satu fitur diletakkan di `src/features/<feature>/`, bukan di `src/components/`.

### Aturan implementasi

1. Gunakan SFC `<script setup lang="ts">` dan Composition API. Jangan memakai Options API untuk kode baru.
2. TypeScript strict: jangan `any`, gunakan `import type`, dan ambil tipe API dari `@ternakmatric/api-contracts`. Jangan mendefinisikan ulang bentuk request/response secara manual.
3. Semua HTTP lewat `createWebApiClient()` di `src/lib/api.ts`. Jangan memanggil `fetch` mentah dari komponen dan jangan hardcode base URL.
   - Browser: `VITE_API_URL`.
   - SSR: `API_INTERNAL_URL` (hanya server).
   - Fallback lokal: `http://localhost:8080`.
4. Token dan `organization_id` dikirim lewat `getToken`/`getOrganizationId` (header `Authorization` dan `X-Organization-Id`), bukan query string. Jangan menyimpan token di module scope saat SSR; buat client per request.
5. `Decimal` dari backend dikirim sebagai string. Jangan `parseFloat` untuk aritmetika bisnis dan jangan menghitung ulang FCR/ADG/HPP di browser; tampilkan dengan `Intl.NumberFormat("id-ID")`.
6. Format uang, angka, dan tanggal memakai locale `id-ID`. Jangan hardcode pemisah ribuan atau simbol mata uang.
7. Tailwind utility-first; pertahankan tema gelap `slate`/`emerald` yang sudah ada. Jangan menambah UI kit, state library, atau framework CSS baru tanpa keputusan eksplisit.
8. Data mock hanya untuk scaffold. Fitur yang diklaim selesai harus memakai API; jangan menambah mock baru bila endpoint sudah tersedia. `DashboardView` dan `SpeciesOverviewTable` saat ini masih memakai data contoh dan harus diganti saat fitur di-wire.
9. SSR-safe:
   - Jangan akses `window`, `document`, `localStorage`, atau `navigator` di top-level module atau saat render.
   - Guard dengan `import.meta.env.SSR` atau pindahkan ke `onMounted`.
   - Jangan memakai `Date.now()` atau nilai acak saat render agar tidak terjadi hydration mismatch.
   - Data yang harus tampil di HTML server diambil lewat `onServerPrefetch` atau di `entry-server.ts`, bukan `onMounted`.
10. Jangan menaruh secret di variabel `VITE_*`; semua `VITE_*` terekspos ke browser.
11. Aksesibilitas: gunakan elemen semantik, label form, `alt` pada gambar, dan pastikan tabel serta navigasi bisa dioperasikan dengan keyboard.

### Verifikasi web

```powershell
npm --prefix apps/web run typecheck
npm --prefix apps/web run build
npm --prefix apps/web run dev
```

Belum ada test runner web. Verifikasi minimal adalah `typecheck` dan `build`; untuk perubahan SSR, jalankan `dev` dan pastikan halaman ter-render dari server (view source atau `Invoke-WebRequest`), bukan hanya setelah hydration.

---

## 11. Testing

- Test kalkulasi: `backend/crates/mod-calculation/src/tests.rs`.
- Test kontrak OpenAPI: modul test di bawah `backend/bins/api-server/src/main.rs`.
- Web: belum ada test runner. Verifikasi minimal `npm --prefix apps/web run typecheck` dan `npm --prefix apps/web run build`; jangan menambah Vitest/Playwright tanpa keputusan.
- Belum ada harness database integration. Jangan asumsikan `#[sqlx::test]` atau Testcontainers tersedia. Untuk test DB, koordinasikan dulu dan ikuti pola: start Postgres via compose, jalankan migrasi, baca `DATABASE_URL`, skip test jika tidak ada, isolasi data per test, uji negative case tenant.
- `unwrap`/`expect` boleh di test, tidak boleh di handler.
- Test deterministik: `fixed_time()`, `dec!`, tanpa sleep/network.
- Jangan melemahkan assert yang gagal; perbaiki kode produksinya atau laporkan ketidaksesuaian.

---

## 12. Definition of done

Sebelum mengatakan selesai:

- [ ] `cargo fmt --all` bersih
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` hijau
- [ ] `cargo test --workspace` hijau
- [ ] Untuk handler/DTO: OpenAPI + TypeScript regenerated, `packages/api-contracts` dan `apps/web` typecheck hijau
- [ ] Untuk perubahan tenant: query berada di `begin_tx` dan bind `organization_id` eksplisit
- [ ] Audit/outbox ditulis di transaksi yang sama dengan perubahan domain
- [ ] Untuk perubahan schema: migrasi baru, bukan edit lama; diuji dari database kosong
- [ ] Untuk perubahan formula: `formula_version` dan test diperbarui
- [ ] Untuk perubahan `apps/web`: `npm --prefix apps/web run typecheck` dan `npm --prefix apps/web run build` hijau
- [ ] Untuk route/halaman web baru: render SSR berhasil dan tidak ada akses `window`/`document`/`localStorage` tanpa guard `import.meta.env.SSR`
- [ ] Tidak ada mock data baru untuk fitur yang sudah memiliki endpoint API
- [ ] Dokumentasi terkait (`docs/`, termasuk `docs/data/` & `docs/notes/`) diselaraskan bila nama/semantik berubah
