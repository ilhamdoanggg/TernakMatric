# TernakMatric — Struktur Repository Resmi

Diselaraskan dengan rancangan modular lintas spesies dan keputusan arsitektur di `notes/note-1.md` sampai `notes/note-3.md` serta `docs/decisions/`. Status: **struktur yang disepakati**, scaffold kode masih perlu dibuat.

## Keputusan yang menjadi dasar

| Keputusan | Sumber | Dampak ke struktur |
| --- | --- | --- |
| Modular monolith, bukan microservices | note-1, note-2 | `backend/crates/*`, bukan `services/*` per domain |
| Backend Rust + Axum + Utoipa; SQLx (bukan SQLC) | note-2, note-3 | `sqlx prepare --check` di CI; OpenAPI di-export dari handler Rust |
| Kontrak API kanonik | Utoipa | `packages/api-contracts/openapi/ternakmatric.v1.yaml` |
| Mobile React Native + Expo + SQLite, offline-first | note-2, note-3 | `apps/mobile/src/{storage,sync,offline}` |
| `organization_id` sebagai tenant boundary + `app.current_organization_id` | note-3, ADR 001 | `infra-db`, RLS, composite FK |
| Multi-species & multi-unit (`fish`, `crustacean`, `mollusk`, `poultry`, `mammal`, `reptile`, `insect`, `other`) | note-2, note-3 | `mod-production-unit`, `mod-species`, `mod-environment` generik |
| Full inventory management | note-3 | `mod-inventory` terpisah dari `mod-feeding` |
| Full offline synchronization | note-3, ADR 002 | `mod-sync`, `infra-outbox`, migration sync |
| Calculation engine murni, berversi, spesifik spesies/sistem | note-1, note-2, note-3, ADR 003 | `mod-calculation`; parameter dari `species_production_profiles` |
| Manual logging dulu, IoT V2 | note-2 | `mod-environment` tetap ada; alert IoT bukan prioritas worker MVP |

## Struktur direktori

```text
TernakMatric/
├── .github/workflows/
│   ├── backend-ci.yml            # fmt, clippy, sqlx prepare --check, cargo test,
│   │                             # + migration/RLS/isolation/idempotency test
│   ├── web-ci.yml                # typecheck, lint, build
│   └── mobile-ci.yml             # typecheck, lint, build & test
│
├── docs/
│   ├── 01-PROBLEM-DISCOVERY.md
│   ├── 02-PRODUCT-REQUIREMENTS.md
│   ├── 03-USER-STORIES.md
│   ├── 04-BUSINESS-RULES.md
│   ├── 05-DOMAIN-MODEL.md
│   ├── 06-DATABASE-ERD.md
│   ├── 07-SYSTEM-ARCHITECTURE.md
│   ├── 08-API-SPECIFICATION.md
│   ├── 09-REPOSITORY-STRUCTURE.md    # file ini
│   ├── 10-DATABASE-HARDENING.md
│   └── decisions/
│       ├── 001-tenant-boundary.md
│       ├── 002-offline-sync.md
│       └── 003-calculation-engine.md
│
├── data/                             # Baseline SQL referensi + data dictionary
│   ├── 001_ternakmatric_schema.sql
│   ├── 002_ternakmatric_seed.sql     # species per kategori, unit types, production systems
│   ├── DATA_DICTIONARY.md
│   └── ERD.md
│
├── deploy/
│   ├── docker/
│   │   ├── backend.Dockerfile
│   │   └── web.Dockerfile
│   └── docker-compose.yml            # PostgreSQL, Redis, Rust API, Worker, Web
│                                     # DB role non-superuser / non-BYPASSRLS
│
├── packages/
│   └── api-contracts/
│       ├── openapi/
│       │   └── ternakmatric.v1.yaml  # di-export dari Utoipa via `make gen-openapi`
│       └── generated/typescript/     # tipe + client untuk Web & Mobile
│
├── backend/
│   ├── Cargo.toml
│   ├── Cargo.lock
│   ├── migrations/                   # SQLx migrations + RLS & hardening
│   │   ├── 2026112401_init_identity_organizations.sql
│   │   ├── 2026112402_init_farm_units_species.sql
│   │   ├── 2026112403_init_batches_ledgers.sql
│   │   ├── 2026112404_init_inventory_ledger.sql
│   │   ├── 2026112405_init_finance.sql
│   │   ├── 2026112406_init_calculation_catalog.sql
│   │   ├── 2026112407_init_offline_sync.sql
│   │   ├── 2026112408_init_environment_output.sql
│   │   └── 2026112499_rls_and_hardening.sql
│   ├── bins/
│   │   ├── api-server/               # Axum router, state, middleware, TenantContext
│   │   └── worker/                   # sync queue, rekonsiliasi ledger, snapshot
│   └── crates/
│       ├── shared-kernel/            # OrganizationId, BatchId, UnitId, AppError, Decimal
│       ├── infra-db/                 # PgPool, SET LOCAL app.current_organization_id
│       ├── infra-auth/               # JWT, RBAC extractor, idempotency guard
│       ├── infra-outbox/             # transactional outbox + change-feed cursor
│       │
│       ├── mod-calculation/          # pure domain, deterministik, tanpa DB/HTTP
│       │   └── src/
│       │       ├── lib.rs
│       │       ├── capacity.rs       # kapasitas kolam/kandang/cage/hive per spesies
│       │       ├── population.rs     # initial/birth/purchase/mortality/cull/transfer
│       │       ├── growth.rs         # ABW, ADG, SGR, egg rate, milk yield
│       │       ├── biomass.rs
│       │       ├── survival.rs
│       │       ├── fcr.rs
│       │       ├── output.rs         # estimasi daging/telur/susu/fry/offspring
│       │       ├── finance.rs
│       │       └── version.rs
│       │
│       ├── mod-tenant/               # organizations, members, membership, RBAC
│       ├── mod-production-unit/      # production_units, unit_types, production_systems
│       ├── mod-species/              # species & species_production_profiles berversi
│       ├── mod-production/           # batches, batch_unit_assignments, stock movements, production_outputs
│       ├── mod-feeding/              # feeding records & guidelines
│       ├── mod-inventory/            # warehouses, feed products, ledger, balances
│       ├── mod-environment/          # environment metrics & readings (air/udara/tanah/kandang)
│       ├── mod-health/               # health records & biosecurity
│       ├── mod-finance/              # expenses, sales, P&L per batch
│       ├── mod-analytics/            # calculation_runs/versions, forecasts (V1)
│       └── mod-sync/                 # devices, sync_operations, conflicts, idempotency
│
├── apps/
│   ├── web/                          # Vue 3 SSR + TypeScript / Vite (Dashboard & Analitik)
│   │   └── src/{views,components,features,router,lib}/
│   └── mobile/                       # React Native + Expo
│       └── src/
│           ├── screens/
│           ├── storage/              # SQLite lokal
│           ├── sync/                 # command queue, retry, cursor
│           └── offline/              # pending operations & conflict UI
│
├── Makefile                          # up, migrate, gen-openapi, test, test-db
└── README.md
```

## Endpoint inti

Semua endpoint memakai prefix `/api/v1` dan parameter path `{id}` (bukan `:id`).

| Method | Endpoint | Fungsi |
| --- | --- | --- |
| POST | `/production-units` | Membuat unit produksi (kolam, kandang, cage, hive) |
| GET | `/production-units/{id}` | Detail unit produksi |
| GET | `/environment-readings` | Daftar bacaan lingkungan |
| POST | `/environment-readings` | Mencatat bacaan air/udara/tanah/kandang |
| POST | `/batches` | Membuat batch produksi ternak |
| GET | `/batches/{id}` | Detail batch |
| POST | `/batches/{id}/stock-movements` | Pergerakan stok ternak |
| POST | `/feeding` | Pencatatan pakan/ransum |
| POST | `/inventory/transactions` | Transaksi gudang/inventory |
| POST | `/production-outputs` | Pencatatan output (meat/egg/milk/fry/offspring/other) |
| POST | `/finance/expenses` | Pencatatan biaya |

Kontrak final berada di `packages/api-contracts/openapi/ternakmatric.v1.yaml` yang dihasilkan Utoipa dari handler Axum.

## Perubahan terhadap struktur awal

| Sebelum | Sesudah | Alasan |
| --- | --- | --- |
| `init_tenants_users.sql` | `init_identity_organizations.sql` | `organization_id`, bukan `tenant_id` (ADR 001) |
| `SET LOCAL app.tenant_id` | `SET LOCAL app.current_organization_id` | Satu nama kanonik untuk RLS |
| `2026112402_init_farm_pond_species.sql` | `2026112402_init_farm_units_species.sql` | `unit_types`/`production_units`/`production_systems` |
| `mod-pond` | `mod-production-unit` | Unit produksi tidak hanya kolam |
| `mod-waterquality` | `mod-environment` | Air, udara, tanah, dan kandang |
| `mod-calculation/src/harvest.rs` | `mod-calculation/src/output.rs` | Estimasi output lintas spesies |
| `mod-feeding` menyimpan inventory | `mod-inventory` terpisah | Full inventory (ADR 002) |
| Tidak ada modul sync | `mod-sync` + `infra-outbox` | Full offline sync |
| 3 migration | 9 migration | Cakupan tabel note-2/3 lebih luas |
| Worker: snapshot + alert | + sync queue & rekonsiliasi ledger | Alert IoT masuk V2 |

## Catatan

- ADR 001–003 menutup keputusan yang sebelumnya masih terbuka: nama GUC, strategi offline sync, dan versi parameter calculation engine.
- `data/` memakai nama kanonik: `unit_types`, `production_systems`, `production_units`, `species` dengan kategori `fish | crustacean | mollusk | poultry | mammal | reptile | insect | other`, `species_production_profiles`, `environment_metrics`, `environment_readings`, `production_outputs`, dll. Migration backend wajib mengikuti nama yang sama.
- Ledger stok ternak, inventory, output produksi, dan finance tidak boleh memakai `client_wins`.

## Urutan implementasi

Ikuti urutan note-3, bukan UI lebih dulu:

1. Database hardening + migration/RLS test
2. Backend foundation (config, PgPool, transaksi, error handling)
3. Auth & multi-tenant (JWT, membership, RBAC, OrganizationContext)
4. Core animal production (farm, production units, species, batch, stock movement)
5. Calculation engine (+ versioning parameter spesies)
6. Inventory & finance
7. Offline sync engine
8. Web & mobile
