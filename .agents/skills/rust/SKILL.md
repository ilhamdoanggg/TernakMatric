---
name: Rust
description: Panduan backend TernakMatric (Rust, Axum 0.8, SQLx 0.8, Utoipa 5, Tokio, rust_decimal). Gunakan sebelum mengubah handler, crate, auth/tenant, audit/outbox, OpenAPI, calculation engine, worker, atau test di backend/.
---

# Skill: Rust (TernakMatric backend)

Sumber kebenaran: `AGENTS.md`. Skill `ternakmatric` memuat aturan lintas area; skill `database` memuat detail PostgreSQL/RLS/migrasi. Skill ini fokus pada cara menulis kode Rust yang konsisten dengan `backend/`.

## 1. Peta workspace

Workspace root: `backend/Cargo.toml` (edition 2021, resolver 2). Semua dependency bersama dideklarasikan di `[workspace.dependencies]` dan diwariskan dengan `dep.workspace = true`; jangan menulis versi berbeda di crate anggota. `backend/Cargo.lock` **tracked** — jangan hapus atau ignore.

| Path | Tanggung jawab | Catatan dependensi |
| --- | --- | --- |
| `crates/shared-kernel` | Tipe domain murni, `AppError`, ID newtype, enum domain, konversi satuan | Tanpa `sqlx`/`axum`/I/O. Boleh `chrono`, `serde`, `utoipa`, `rust_decimal` |
| `crates/infra-db` | `PgPool`, `OrganizationContext`, `begin_tx`, audit, outbox, `map_sqlx_err`, runner migrasi | Boleh `shared-kernel`, `sqlx` |
| `crates/infra-auth` | JWT, extractor `AuthPrincipal`/`AuthenticatedUser`, RBAC, `SyncService` | Boleh `infra-db`, `axum` |
| `crates/mod-calculation` | Engine kalkulasi murni | Tanpa DB/HTTP/Tokio/clock/network/env. Boleh `chrono` untuk tipe waktu |
| `crates/mod-tenant` | Auth register/login, organizations, members, farms, audit log | `mod-*` lain tidak boleh diimpor |
| `crates/mod-enclosure` | Farms, production units, kapasitas | — |
| `crates/mod-species` | Species, production profiles, parameter berversi | — |
| `crates/mod-production` | Batch, stock movement, individual tag, growth sample, output | — |
| `crates/mod-feeding` | Warehouse, feed product, inventory transaction, feeding record | — |
| `crates/mod-environment` | Environment metrics dan readings | — |
| `crates/mod-health` | Vaccination, mortality, health record | — |
| `crates/mod-finance` | Expense, sale, ringkasan finansial/HPP | — |
| `bins/api-server` | `AppState`, system/sync routes, `ApiDoc`, router wiring, export OpenAPI | Hanya merangkai; logika domain di `mod-*` |
| `bins/worker` | Snapshot harian, reminder vaksin, alert | Job background, tanpa HTTP |

Aturan:

- Tidak ada `mod-*` yang bergantung pada `mod-*` lain. Jangan membuat cycle.
- Jangan menambah handler HTTP atau query SQL ke `shared-kernel`/`mod-calculation`.
- Jangan menambah akses baru ke tabel milik modul lain. Beberapa query lintas domain sudah ada (mis. `population_balances` dan `production_units` dipakai lintas modul); perlakukan sebagai technical debt dan jangan memperluasnya tanpa keputusan.
- `bins/api-server` tidak menjalankan migrasi saat start; migrasi dijalankan lewat `deploy/db/run-migrations.sh` atau `infra-db::run_migrations()`.

## 2. Perintah verifikasi

Dari root repo:

```sh
cargo fmt --manifest-path backend/Cargo.toml --all
cargo clippy --manifest-path backend/Cargo.toml --workspace --all-targets -- -D warnings
cargo test --manifest-path backend/Cargo.toml --workspace
```

Fokus crate:

```sh
cargo test --manifest-path backend/Cargo.toml -p mod-calculation
cargo clippy --manifest-path backend/Cargo.toml -p mod-health --all-targets -- -D warnings
```

Setelah perubahan handler/DTO/route, regenerasi kontrak:

```sh
npm --prefix packages/api-contracts run gen:openapi
npm --prefix packages/api-contracts run gen:ts
npm --prefix packages/api-contracts run typecheck
```

`gen:openapi` menjalankan `cargo run -p api-server -- --export-openapi openapi/ternakmatric.v1.json`. Jangan mengedit `packages/api-contracts/openapi/*.json` atau `generated/typescript/*` dengan tangan.

Query SQL memakai runtime API (`query_as`, `query_scalar`), bukan macro `query!`, sehingga tidak ada cache `.sqlx/`. Jika nanti memperkenalkan macro:

```sh
cargo sqlx prepare --workspace --manifest-path backend/Cargo.toml
```

lalu commit `.sqlx/` dan pastikan CI menjalankan `sqlx prepare --check`.

## 3. Tipe domain

- **Uang, berat, laju, pengukuran:** `rust_decimal::Decimal`. Jangan `f64`/`f32`. Test memakai `rust_decimal_macros::dec!`.
- **Decimal di API** dikirim sebagai string; jangan `parseFloat` di klien dan jangan hitung ulang nilai bisnis.
- **ID:** newtype `shared-kernel` via `define_id!`: `OrganizationId`, `UserId`, `FarmId`, `EnclosureId`, `SpeciesId`, `ProfileId`, `BatchId`, `IndividualTagId`, `WarehouseId`, `FeedProductId`, `DeviceId`, `OperationId`. Tersedia `new()`, `from_uuid()`, `into_inner()`, `as_uuid()`, dan `serde(transparent)`.
  - Domain/service memakai newtype; bind `.0` hanya di boundary SQL.
  - DTO HTTP yang sudah ada umumnya memakai `Uuid`; konversi di boundary, jangan bocorkan newtype ke kontrak baru tanpa alasan.
- **Enum domain** ada di `shared-kernel/src/domain.rs`: `SpeciesCategory`, `UnitHabitat`, `CapacityUnit`, `BatchStatus`, `MovementType`, `OutputType`, `EnvironmentCategory`. Semua `serde(rename_all = "snake_case")`; gunakan enum, bukan string mentah, untuk nilai yang sudah dimodelkan.
- **Waktu:** `chrono::{DateTime<Utc>, NaiveDate}`. `mod-calculation` menerima `calculated_at` sebagai parameter dan tidak memanggil `Utc::now()`.
- **Satuan:** gunakan `convert_mass_to_kg` / `convert_volume_to_liters` dari `shared-kernel`. Faktor konversi feed berasal dari tabel `feed_unit_conversions`, jangan hardcode.
- **Validasi input** sebelum transaksi: trim string, positivity count/weight/days, `quantity >= 0`, dan `initial_count > 0`. Error → `AppError::Validation`.

## 4. Error handling

`shared_kernel::AppError` adalah error domain tunggal. `infra_auth::ApiError(pub AppError)` membungkusnya dan mengimplementasikan `IntoResponse` menjadi `ErrorEnvelope { code, message, details }`; `details` hanya diisi untuk `CapacityExceeded`.

| Varian | HTTP | Kode |
| --- | --- | --- |
| `Validation` | 400 | `VALIDATION_ERROR` |
| `Unauthorized` | 401 | `UNAUTHORIZED` |
| `Forbidden` | 403 | `FORBIDDEN` |
| `NotFound` | 404 | `NOT_FOUND` |
| `Conflict` | 409 | `CONFLICT` |
| `CapacityExceeded { message, details }` | 422 | `CAPACITY_EXCEEDED` |
| `InsufficientBalance` | 422 | `INSUFFICIENT_BALANCE` |
| `Database` / `Internal` | 500 | `DATABASE_ERROR` / `INTERNAL_ERROR` |

Aturan:

- Mapping SQLSTATE hanya lewat `infra_db::map_sqlx_err`:

  | SQLSTATE | AppError |
  | --- | --- |
  | `23505` | `Conflict` |
  | `23503`, `23514` | `Validation` |
  | `RowNotFound` | `NotFound` |
  | lainnya | `Database` |

- Pola query: `.map_err(|e| ApiError(map_sqlx_err(e)))?`. Jangan hand-roll SQLSTATE.
- `fetch_one` yang gagal karena row tidak ada otomatis menjadi `NotFound` lewat `map_sqlx_err`. Jika butuh pesan kontekstual, pakai `fetch_optional` lalu `AppError::NotFound("...")`.
- Jangan `unwrap`, `expect`, `panic!`, `println!`, atau `dbg!` di handler/service. Gunakan `tracing` (mis. `tracing::warn!(error = %err, "...")`).
- `serde_json::to_value(...).unwrap_or_default()` boleh untuk payload audit/outbox, bukan untuk data bisnis.
- `AppError::Forbidden` untuk path/tenant mismatch; `Validation` untuk input; `InsufficientBalance` untuk stok/populasi negatif; `CapacityExceeded` dengan `details` JSON untuk kapasitas.
- Early return di dalam transaksi akan rollback; itu perilaku yang disengaja.

## 5. Auth, tenant, dan state

- `AppBackendState` (di `infra-auth/src/extractor.rs`) mewajibkan `pool() -> &PgPool` dan `jwt() -> &JwtService`. `AppState` di `bins/api-server` mengimplementasikannya.
- Handler domain di `mod-*` generik: `pub async fn foo<S: AppBackendState>(...)`. Handler system/sync di `bins/api-server` memakai `AppState` konkret.
- `AuthPrincipal` = JWT + header `X-Organization-Id` + verifikasi membership aktif + RBAC. Jangan percaya header mentah; extractor sudah memverifikasi.
- Panggil `auth.require(Permission::...)` **sebelum** bekerja. `ORG_OWNER`/`ORG_ADMIN` bypass pemeriksaan permission.
- Transaksi tenant: `auth.context().begin_tx(state.pool()).await` → set `app.current_organization_id` dan `app.current_user_id` via `SET LOCAL`. Tetap bind `auth.organization_id.0` eksplisit.
- Untuk path yang memuat organization id (mis. `/organizations/{id}/members`), bandingkan dengan `auth.organization_id.0` dan kembalikan `Forbidden` bila beda.

Permission:

| Varian | Kode |
| --- | --- |
| `OrgManage` | `org.manage` |
| `FarmManage` | `farm.manage` |
| `SpeciesManage` | `species.manage` |
| `ProductionWrite` | `production.write` |
| `FeedingWrite` | `feeding.write` |
| `EnvironmentWrite` | `environment.write` |
| `HealthWrite` | `health.write` |
| `FinanceWrite` | `finance.write` |
| `AnalyticsRead` | `analytics.read` |

### Eksepsi yang disengaja

| Flow | Extractor | Pool |
| --- | --- | --- |
| `POST /auth/register`, `POST /auth/login` | Tanpa extractor auth | Bare `state.pool()`; belum ada organisasi |
| `GET /organizations`, `POST /organizations` | `AuthenticatedUser` (JWT saja) | Bare pool; bootstrap/pra-pemilihan organisasi |
| `GET /environment/metrics` | `AuthPrincipal` | Bare pool; `environment_metrics` adalah katalog global tanpa `organization_id` |

- `create_organization` memakai `pool.begin()` + `with_user`, insert organization, lalu `with_organization` di transaksi yang sama sebelum insert member/role/audit.
- Jangan meniru bare pool untuk tabel tenant. Bare pool hanya untuk katalog global dan flow bootstrap tanpa organisasi.
- Worker tidak memakai `AuthPrincipal`. Pola worker: ambil daftar organization aktif dari pool, lalu per organisasi `pool.begin()` + `with_organization(&mut *tx, OrganizationId(org_id))`. Worker hanya men-set organization, bukan user; jika butuh actor, panggil `with_user` secara eksplisit.

## 6. Resep handler

```rust
use axum::extract::State;
use axum::Json;
use infra_auth::{ApiError, AppBackendState, AuthPrincipal, Permission};
use infra_db::{map_sqlx_err, AuditLogger, OutboxWriter};
use shared_kernel::error::ErrorEnvelope;
use shared_kernel::AppError;

#[utoipa::path(
    post,
    path = "/api/v1/things",
    request_body = CreateThingRequest,
    responses(
        (status = 201, body = ThingDto),
        (status = 409, body = ErrorEnvelope)
    ),
    tag = "Things"
)]
pub async fn create_thing<S: AppBackendState>(
    State(state): State<S>,
    auth: AuthPrincipal,
    Json(req): Json<CreateThingRequest>,
) -> Result<(axum::http::StatusCode, Json<ThingDto>), ApiError> {
    auth.require(Permission::ProductionWrite).map_err(ApiError)?;

    // 1. Validasi sebelum transaksi.
    if req.name.trim().is_empty() {
        return Err(ApiError(AppError::Validation(
            "name must not be empty".into(),
        )));
    }

    // 2. Transaksi tenant.
    let mut tx = auth
        .context()
        .begin_tx(state.pool())
        .await
        .map_err(ApiError)?;

    // 3. Bind organization_id eksplisit.
    let row: ThingDto = sqlx::query_as(
        r#"
        INSERT INTO things (organization_id, name, created_by)
        VALUES ($1, $2, $3)
        RETURNING id, organization_id, name, created_at
        "#,
    )
    .bind(auth.organization_id.0)
    .bind(req.name.trim())
    .bind(auth.user_id.0)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| ApiError(map_sqlx_err(e)))?;

    // 4. Audit/outbox di transaksi yang sama.
    OutboxWriter::append(
        &mut tx,
        auth.organization_id,
        "things",
        row.id,
        "things.created",
        serde_json::json!({ "name": row.name }),
    )
    .await
    .map_err(ApiError)?;

    AuditLogger::record(
        &mut tx,
        auth.organization_id,
        Some(auth.user_id),
        "thing.created",
        "things",
        Some(row.id),
        None,
        Some(serde_json::to_value(&row).unwrap_or_default()),
    )
    .await
    .map_err(ApiError)?;

    // 5. Commit eksplisit.
    tx.commit().await.map_err(|e| ApiError(map_sqlx_err(e)))?;
    Ok((axum::http::StatusCode::CREATED, Json(row)))
}
```

Detail yang mudah terlewat:

- Urutan extractor: `State`, `AuthPrincipal`/`AuthenticatedUser`, `Path`/`Query`, lalu `Json` (body-consuming) paling akhir.
- Kembalikan `(StatusCode::CREATED, Json(dto))` untuk create; `Json(dto)` untuk 200.
- Route Axum 0.8 memakai `{id}`, bukan `:id`.
- Request `#[derive(Debug, Serialize, Deserialize, ToSchema)]`. Response DTO `ToSchema`; tambah `sqlx::FromRow` hanya jika dibaca langsung dari query.
- Command yang mengubah stok/ledger/saldo wajib punya `idempotency_key: Uuid` (standar AGENTS). Beberapa handler finance lama belum memilikinya; jangan menambah command baru tanpa idempotency.
- Cek kapasitas/saldo/uniqueness: lock baris induk dengan `FOR UPDATE` (contoh: `production_units`, `population_balances`, `inventory_balances`, `vaccination_schedules`).
- Update memakai pola: `SELECT ... FOR UPDATE` (simpan `before`) → `UPDATE ... version = version + 1 ... RETURNING ...` (simpan `after`) → audit dengan `before` dan `after`.
- Jangan menelan error kalkulasi dengan `.ok()` pada command baru; map ke `AppError::Validation` atau error domain yang sesuai.

## 7. Pola SQLx

- `query_as` + `FromRow` untuk baris; `query_scalar` untuk nilai tunggal; `fetch_optional` untuk lookup yang boleh kosong; `fetch_one` untuk yang harus ada.
- Setiap query tenant: `WHERE organization_id = $1` dan bind `auth.organization_id.0`.
- Filter dinamis: `($2::text IS NULL OR kolom = $2)`, `($3::uuid IS NULL OR entity_id = $3)`.
- Pagination: `limit.clamp(1, 200)` untuk endpoint biasa; sync pull memakai `clamp(1, 500)`.
- Cast yang sering dipakai: `currency::text`, `COALESCE(SUM(...), 0)::bigint`, `COALESCE(pb.quantity, b.initial_count)`.
- Transaksi singkat: tanpa HTTP, file I/O, atau komputasi berat di dalamnya. Kalkulasi murni boleh sebelum/sesudah transaksi; jangan menahan lock saat menghitung.
- Commit eksplisit. Jangan lupa `tx.commit()` di semua jalur sukses.
- Ledger append-only: `stock_movements`, `inventory_transaction_lines`, `audit_logs` (dan sesuai ADR 002 `inventory_transactions`, `production_outputs`). Koreksi memakai reversal/adjustment, bukan `UPDATE`/`DELETE`.
- Untuk mutasi stok: insert movement + update balance (`version = version + 1`) dalam transaksi yang sama. Jangan pernah menulis balance tanpa movement.
- Idempotency: andalkan unique constraint `idempotency_key`; SQLSTATE `23505` dipetakan ke `Conflict`. Untuk sync offline, dedup memakai `(organization_id, device_id, client_operation_id)`.
- `&mut *tx` menghasilkan `&mut PgConnection`; `AuditLogger`/`OutboxWriter` menerima `&mut PgConnection`.

## 8. Audit dan outbox

Signature aktual:

```rust
AuditLogger::record(
    conn: &mut PgConnection,
    organization_id: OrganizationId,
    actor_user_id: Option<UserId>,
    action: &str,          // "batch.created", "farm.updated", ...
    entity_type: &str,     // nama tabel: "production_batches", "farms", ...
    entity_id: Option<Uuid>,
    before_data: Option<Value>,
    after_data: Option<Value>,
) -> Result<(), AppError>;

OutboxWriter::append(
    conn: &mut PgConnection,
    organization_id: OrganizationId,
    aggregate_type: &str,
    aggregate_id: Uuid,
    event_type: &str,      // "production_batches.created", ...
    payload: Value,
) -> Result<i64, AppError>;
```

- Tulis audit dan outbox di transaksi yang sama dengan perubahan domain.
- Standar kode baru: audit ditulis untuk setiap aksi user; `actor_user_id` = `Some(auth.user_id)`. Untuk aksi sistem/worker, gunakan `None` atau `with_user` jika actor diketahui. Beberapa handler lama belum menulis audit; jangan menambah utang baru.
- Create: `before_data = None`, `after_data = Some(snapshot)`. Update: isi keduanya.
- Outbox dipakai untuk event yang perlu disinkronkan/dikonsumsi worker. Implementasi saat ini memakai outbox di `create_batch` dan `SyncService::push_commands`; jangan menambah event tanpa consumer yang jelas.
- Konvensi penamaan: action `entity.verb` (`batch.created`, `expense.recorded`), event type `aggregate.verb` (`production_batches.created`), payload JSON ringkas yang cukup untuk consumer.

## 9. Router, OpenAPI, dan wiring

Setiap `mod-*` mengekspos:

```rust
pub fn router<S: AppBackendState>() -> Router<S> {
    Router::new()
        .route("/api/v1/things", get(list_things::<S>).post(create_thing::<S>))
        .route("/api/v1/things/{id}", patch(update_thing::<S>))
}

#[derive(OpenApi)]
#[openapi(
    paths(list_things, create_thing, update_thing),
    components(schemas(CreateThingRequest, UpdateThingRequest, ThingDto))
)]
pub struct ThingsApi;
```

- `paths(...)` harus memuat semua handler; `components(schemas(...))` harus memuat semua DTO, termasuk tipe generik seperti `CalculationResult<CapacityEvaluation>`.
- `bins/api-server` menggabungkan lewat `MergeModuleApis` (`mod_tenant::TenantApi::openapi()`, dst.) dan `.merge(mod_x::router::<AppState>())` di `build_router`.
- Modul baru: tambah member di `backend/Cargo.toml`, dependency `Cargo.toml`, `router`, `XxxApi`, merge di `MergeModuleApis`, merge router, dan assertion path di `test_openapi_spec_builds_with_all_module_paths`.
- Endpoint system/sync (health, devices, push/pull/conflicts) berada di `bins/api-server`; sync memakai `SyncService` dari `infra-auth`.
- Tag OpenAPI yang dipakai: `System`, `Tenant & Auth`, `Enclosure`, `Species`, `Production`, `Feeding & Inventory`, `Environment`, `Health & Biosecurity`, `Finance & HPP`, `Offline Sync`.
- Sertakan `ErrorEnvelope` di `responses(...)` untuk error yang relevan (mis. 409/422); tidak semua handler lama melakukannya, tapi handler baru sebaiknya.
- `AuditLogQuery` dan query struct lain cukup `Deserialize` jika tidak muncul di schema.

## 10. Calculation engine

- Purity: tanpa `sqlx`, `axum`, Tokio, clock, network, env, atau global mutable state. `chrono` hanya untuk tipe `DateTime<Utc>`; waktu selalu parameter `calculated_at`.
- Signature: `Result<CalculationResult<T>, CalculationError>`.
- `CalculationResult<T>` membawa `value`, `unit`, `formula_code`, `formula_version`, `parameter_version`, `calculated_at`, `assumptions { is_estimate, last_sample_at, notes }`.
- `estimate(...)` untuk hasil berbasis sampling/asumsi; `exact(...)` hanya jika semua input terukur.
- Jangan branch berdasarkan nama species. Perilaku spesies lewat parameter profil berversi (`species_production_profiles.parameters`); `parameter_version` diteruskan ke hasil.
- `formula_version` naik jika semantik rumus berubah. Jangan mengubah hasil lama di `calculation_runs`.
- Gunakan `checked_*` dan `round_dp`. Jangan bulatkan ulang di handler.

Peta formula saat ini (`formula_version = 1`):

| Fungsi | `formula_code` | Unit | Pembulatan | Jenis |
| --- | --- | --- | --- | --- |
| `compute_abw_g` | `ABW` | `g/head` | 3 | exact |
| `compute_adg_g_per_day` | `WEIGHT_GAIN` | `g/day` | 3 | exact |
| `compute_sgr_percent_per_day` | `SGR` | `percent/day` | 4 | exact |
| `compute_egg_rate_percent` | `EGG_RATE` | `percent` | 2 | exact |
| `compute_milk_yield_per_head_day` | `MILK_YIELD` | `liter/head/day` | 3 | exact |
| `compute_biomass_kg` | `BIOMASS_ESTIMATE` | `kg` | 3 | estimate |
| `compute_enclosure_total_biomass_kg` | `ENCLOSURE_BIOMASS_ESTIMATE` | `kg` | 3 | estimate |
| `compute_enclosure_geometry` | `VOLUME_RECTANGLE` / `VOLUME_CIRCLE` / `AREA_ONLY` | `m2/m3` atau `m2` | 3 | exact |
| `evaluate_enclosure_capacity` | `CAPACITY_UTILIZATION` | `percent` | 2 | exact |
| `compute_population_balance` | `POPULATION_BALANCE` | `head` | integer | exact |
| `compute_survival_rate` | `SURVIVAL_RATE` | `percent` | 2 | exact |
| `compute_mortality_rate` | `MORTALITY_RATE` | `percent` | 2 | exact |
| `compute_fcr` | `FCR_GENERIC` | `ratio` | 3 | estimate |
| `compute_feeding_variance_percent` | `FEEDING_VARIANCE` | `percent` | 2 | exact |
| `compute_hpp_per_kg_live` | `HPP_LIVE_WEIGHT` | `currency/kg` | 2 | exact, atau estimate bila termasuk standing biomass |
| `compute_hpp_per_output_unit` | `HPP_OUTPUT_UNIT` | `currency/{unit}` | 2 | exact |
| `compute_batch_pnl` | `PROFITABILITY` | `currency` | 2 | exact |
| `forecast_meat_harvest` | `HARVEST_FORECAST` | `kg` | populasi 0, ABW/kg 3 | estimate |
| `forecast_egg_output` | `EGG_OUTPUT_FORECAST` | `egg` | 0 | estimate |

Guard yang wajib: count/weight/days positif, `total_stocked > 0` untuk rate, denominator tidak nol, populasi tidak negatif, survival rate `(0, 100]`, dan `gain_kg > 0` untuk FCR. `MovementType::EggCollection`/`MilkCollection` tidak mengurangi headcount (`affects_live_headcount()`).

## 11. Worker

`bins/worker` berjalan tiap 300 detik dan menjalankan:

1. `run_daily_snapshots`: ambil organization aktif, per organisasi set context, hitung biomassa, insert `calculation_runs` dengan `calculation_version` `BIOMASS_ESTIMATE` v1.
2. `run_vaccination_reminders_and_alerts`: tandai `vaccination_schedules` yang lewat tanggal menjadi `overdue`, lalu insert `outbox_events` dengan event `alert.vaccination_overdue`.

Aturan worker:

- Jangan memakai `AuthPrincipal`; gunakan `pool.begin()` + `with_organization`.
- Jangan memanggil handler HTTP; panggil service/fungsi murni langsung.
- Error dikembalikan sebagai `sqlx::Error` dan di-log `tracing::warn!`; satu siklus gagal tidak boleh mematikan loop.
- Untuk job baru, pikirkan idempotensi/duplikasi (unique constraint, tanggal, atau cursor) karena worker bisa berjalan berulang.
- Worker tidak menjalankan migrasi.

## 12. Test

Lokasi test yang ada:

| File | Isi |
| --- | --- |
| `crates/mod-calculation/src/tests.rs` | Kalkulasi, `fixed_time()`, `dec!`, edge case |
| `crates/shared-kernel/src/units.rs` | Konversi massa/volume, semantik movement |
| `crates/infra-db/src/pool.rs` | Precedence file `.env` |
| `crates/infra-auth/src/jwt.rs` | Issue/verify JWT |
| `crates/infra-auth/src/sync.rs` | Larangan `client_wins` untuk ledger/finance |
| `bins/api-server/src/main.rs` | `ApiDoc` memuat semua path |

- Test deterministik: `fixed_time()`, `dec!`, tanpa sleep/network.
- `unwrap`/`expect` boleh di test, tidak boleh di handler.
- Belum ada harness DB integration. Jangan mengasumsikan `#[sqlx::test]`/Testcontainers tersedia; koordinasikan dulu.
- Jangan melemahkan assert yang gagal. Perbaiki kode produksi atau laporkan ketidaksesuaian.
- Setelah menambah route, tambah assertion path di `test_openapi_spec_builds_with_all_module_paths`.

## 13. Runtime config

| Variabel | Dipakai | Default/perilaku |
| --- | --- | --- |
| `DATABASE_URL` | API, worker | fallback dev `postgres://ternakmatric:ternakmatric@localhost:5432/ternakmatric` |
| `BIND_ADDR` | API | `0.0.0.0:8080` |
| `JWT_SECRET` | API | fallback dev; **wajib** di-set di produksi |
| `JWT_TTL_MINUTES` | API | 1440 menit |
| `RUST_LOG` | API, worker | `api_server=info,tower_http=info` / `worker=info` |
| `APP_ENV`, `ENV_FILE` | `load_env_files` | memuat `.env.<APP_ENV>`, lalu `.env` |
| `DB_MAX_CONNECTIONS` | `DbConfig::from_env` | 16; bins saat ini membuat pool langsung (API 16, worker 8) |

`infra_db::create_pool`/`DbConfig` tersedia, tetapi `bins/api-server` dan `bins/worker` saat ini memakai `PgPoolOptions` langsung (`connect_lazy`). Jangan mengubah pola itu tanpa alasan.

## 14. Definition of done

- [ ] `cargo fmt --all` bersih
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` hijau
- [ ] `cargo test --workspace` hijau
- [ ] Query tenant berada di `begin_tx` dan bind `organization_id` eksplisit
- [ ] `auth.require(Permission::...)` dipanggil sebelum bekerja
- [ ] Audit/outbox ditulis di transaksi yang sama dengan perubahan domain
- [ ] Command stok/ledger/saldo punya `idempotency_key`
- [ ] Kapasitas/saldo/uniqueness memakai `FOR UPDATE`
- [ ] Perubahan DTO/route: OpenAPI + TypeScript regenerated, `packages/api-contracts` dan `apps/web` typecheck hijau
- [ ] Test kalkulasi/edge case diperbarui; assertion OpenAPI path ditambah
- [ ] Tidak ada `unwrap`/`expect`/`panic!`/`println!`/`dbg!` di handler/service
- [ ] Dokumentasi (`docs/`, termasuk `docs/data/` & `docs/notes/`) diselaraskan bila nama/semantik berubah

## 15. Anti-pattern

- Bare `PgPool` untuk tabel tenant (kecuali katalog global/bootstrap yang disengaja).
- Mempercayai `X-Organization-Id` tanpa `AuthPrincipal`.
- `f64`/`f32` untuk uang/berat/laju; string ID di domain.
- `UPDATE`/`DELETE` pada ledger append-only.
- Menambah akses tabel lintas modul atau dependency `mod-*` → `mod-*`.
- Query SQL/handler HTTP di `shared-kernel`/`mod-calculation`.
- `Utc::now()` atau I/O di `mod-calculation`; branch berdasarkan nama species.
- `unwrap`/`expect`/`panic!` di request path; menelan error kalkulasi dengan `.ok()`.
- Hand-roll mapping SQLSTATE.
- Route `:id` (Axum 0.8 memakai `{id}`).
- Lupa merge `XxxApi`/router atau lupa assertion path OpenAPI.
- Mengedit migrasi yang sudah di-apply atau generated OpenAPI/TypeScript dengan tangan.
- Komputasi berat atau HTTP di dalam transaksi DB.

## 16. Referensi cepat

| Butuh | Baca |
| --- | --- |
| Aturan repo | `AGENTS.md` |
| Handler + audit + outbox | `crates/mod-production/src/lib.rs` |
| Auth/bootstrap/tenant | `crates/mod-tenant/src/lib.rs` |
| RLS/transaksi/audit/outbox | `crates/infra-db/src/{context,audit,outbox,pool}.rs` |
| Extractor/RBAC/sync | `crates/infra-auth/src/{extractor,rbac,sync}.rs` |
| Kalkulasi | `crates/mod-calculation/src/*.rs`, `tests.rs` |
| Router/OpenAPI/export | `bins/api-server/src/main.rs` |
| Worker | `bins/worker/src/main.rs` |
| Schema/migrasi | `backend/migrations/`, `docs/data/`, `docs/notes/note-4-migration-audit.md` |
