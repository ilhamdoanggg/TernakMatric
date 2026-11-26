# 09 — Struktur Repository

**Dokumen**: `TM-DOC-09` · **Status**: Diselaraskan dengan kondisi aktual · **Tanggal**: 2026-11-26

`apps/web` adalah klien utama dan ikut diimplementasikan saat perubahan menyentuh kontrak API atau tampilan. `apps/mobile` belum menjadi fokus sampai ada keputusan terpisah.

Dokumen ini mendeskripsikan struktur repository **sebagaimana adanya**, bukan rancangan awal. Untuk arsitektur dan batas crate, lihat [`07-SYSTEM-ARCHITECTURE.md`](07-SYSTEM-ARCHITECTURE.md). Untuk status skema, lihat [`06-DATABASE-ERD.md`](06-DATABASE-ERD.md) dan [`notes/note-4-migration-audit.md`](notes/note-4-migration-audit.md).

---

## 1. Keputusan yang menjadi dasar

| Keputusan | Sumber | Dampak ke struktur |
| --- | --- | --- |
| Modular monolith, bukan microservices | `docs/notes/note-1.md`, `docs/notes/note-2.md` | Satu workspace di `backend/`, bukan `services/*` per domain |
| Backend Rust + Axum + Utoipa; SQLx | `docs/notes/note-2.md`, `docs/notes/note-3.md` | OpenAPI diekspor dari anotasi handler ke `ternakmatric.v1.json` |
| Kontrak API kanonik | Utoipa → JSON → TypeScript | `packages/api-contracts/openapi/ternakmatric.v1.json` |
| Mobile React Native + Expo + SQLite, offline-first | `docs/notes/note-2.md`, `docs/notes/note-3.md` | `apps/mobile/` dengan lapisan penyimpanan dan sinkronisasi lokal |
| `organization_id` sebagai tenant boundary | `docs/notes/note-3.md`, [ADR 001](decisions/001-tenant-boundary.md) | `infra-db`, RLS, composite FK |
| Multi-species & multi-unit | `docs/notes/note-2.md`, `docs/notes/note-3.md` | `mod-enclosure`, `mod-species`, `mod-production` generik |
| Inventory dicatat dengan transaksi audit | `docs/notes/note-3.md`, [ADR 002](decisions/002-offline-sync.md) | `warehouses`, `inventory_transactions`, `inventory_balances` di migrasi; saat ini dilayani `mod-feeding` |
| Offline synchronization penuh | `docs/notes/note-3.md`, [ADR 002](decisions/002-offline-sync.md) | Tabel sync di migrasi `2026112407`; handler di `infra-auth/src/sync.rs` dan `bins/api-server` |
| Calculation engine murni, berversi, spesifik spesies/sistem | `docs/notes/note-1..3.md`, [ADR 003](decisions/003-calculation-engine.md) | `mod-calculation`; parameter dari `species_production_profiles` |
| Manual logging dulu, IoT V2 | `docs/notes/note-2.md` | `mod-environment` tetap ada; gateway IoT bukan prioritas worker MVP |

---

## 2. Struktur direktori aktual

```text
TernakMatric/
├── AGENTS.md                         # Aturan kerja coding agent (sumber aturan repo)
├── README.md
├── docker-compose.yml                # PostgreSQL 16 + runner migrasi
├── .env.example
│
├── docs/                             # Seluruh dokumentasi produk, arsitektur, data, & catatan
│   ├── README.md                     # Indeks dokumentasi
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
│   ├── 11-MORTALITY-FAILURE-PREVENTION.md
│   ├── 12-SUPPLY-AND-MARKET-LINKAGE.md
│   ├── data/                         # Kamus data & ERD domain
│   │   ├── README.md
│   │   ├── DATA_DICTIONARY.md
│   │   └── ERD.md
│   ├── decisions/
│   │   ├── 001-tenant-boundary.md
│   │   ├── 002-offline-sync.md
│   │   └── 003-calculation-engine.md
│   └── notes/                        # Catatan perjalanan desain + audit
│       ├── note-1.md
│       ├── note-2.md
│       ├── note-3.md
│       └── note-4-migration-audit.md
│
├── backend/
│   ├── Cargo.toml                    # Workspace; dependency di [workspace.dependencies]
│   ├── Cargo.lock                    # Tracked, jangan di-gitignore
│   ├── migrations/                   # Satu-satunya sumber SQL schema & seed (tergabung)
│   │   ├── 2026112401_init_identity_organizations.sql
│   │   ├── 2026112402_init_farm_enclosure_species.sql
│   │   ├── 2026112403_init_production_ledger.sql
│   │   ├── 2026112404_init_feeding_inventory.sql
│   │   ├── 2026112405_init_environment_health.sql
│   │   ├── 2026112406_init_finance_calculation.sql
│   │   ├── 2026112407_init_offline_sync.sql
│   │   ├── 2026112408_seed_catalogs.sql
│   │   └── 2026112409_rls_triggers_hardening.sql
│   ├── bins/
│   │   ├── api-server/               # Router Axum, AppState, ApiDoc, export OpenAPI
│   │   └── worker/                   # Snapshot harian, reminder vaksin, alert
│   └── crates/
│       ├── shared-kernel/            # ID newtype, AppError, tipe domain murni
│       ├── infra-db/                 # PgPool, OrganizationContext, AuditLogger, OutboxWriter
│       ├── infra-auth/               # JWT, AuthPrincipal, RBAC, handler sync
│       ├── mod-calculation/          # Engine murni; tanpa DB/HTTP/clock/network
│       ├── mod-tenant/               # Auth, organizations, farms, audit log
│       ├── mod-enclosure/            # Production units & assignment
│       ├── mod-species/              # Species & profil produksi berversi
│       ├── mod-production/           # Batch, stock movement, output
│       ├── mod-feeding/              # Feeding records & inventory transaction
│       ├── mod-environment/          # Environment metrics & readings
│       ├── mod-health/               # Health, mortality, vaccination, biosecurity
│       └── mod-finance/              # Expense, sale, sale item
│
├── deploy/
│   ├── README.md
│   └── db/
│       ├── run-migrations.sh         # Tracker: public.schema_migrations
│       └── init-app-role.sql         # Role runtime NOSUPERUSER NOBYPASSRLS
│
├── packages/api-contracts/
│   ├── package.json                  # gen:openapi, gen:ts, typecheck
│   ├── tsconfig.json
│   ├── README.md
│   ├── src/index.ts                  # Client TypeScript yang dipakai web
│   └── openapi/
│       └── ternakmatric.v1.json      # Generated; jangan diedit manual
│                                     # gen:ts menghasilkan generated/typescript/schema.d.ts
│
└── apps/
    ├── web/                          # Vue 3 SSR + TypeScript (Vite)
    └── mobile/                       # React Native + Expo + SQLite
```

### Modul yang direncanakan tetapi belum berupa crate

| Rencana di catatan awal | Kondisi aktual |
| --- | --- |
| `mod-production-unit` | Diterapkan sebagai `mod-enclosure` |
| `mod-inventory` | Tabel inventory ada di migrasi; handler saat ini di `mod-feeding` |
| `mod-analytics` | Kalkulasi di `mod-calculation`; persistensi dan worker di `bins/worker` |
| `mod-sync` | Handler di `infra-auth/src/sync.rs`; tabel di migrasi `2026112407` |
| `infra-outbox` | `OutboxWriter` berada di `infra-db/src/outbox.rs` |
| `.github/workflows/` | Belum ada; perintah CI masih dijalankan manual sesuai `AGENTS.md` |
| `Makefile`, `deploy/docker/` | Belum ada; pakai `cargo`, `npm --prefix`, dan `docker compose` langsung |

---

## 3. Endpoint

Rancangan lengkap ada di [`08-API-SPECIFICATION.md`](08-API-SPECIFICATION.md). Kontrak yang dihasilkan berada di `packages/api-contracts/openapi/ternakmatric.v1.json` dan kini mencakup seluruh modul terpasang (`api-server`, `mod-tenant`, `mod-enclosure`, `mod-species`, `mod-production`, `mod-feeding`, `mod-environment`, `mod-health`, `mod-finance`) dengan path bersih `/api/v1/...`; generasi tipe ke `generated/typescript/schema.d.ts` dilakukan via `npm --prefix packages/api-contracts run gen:ts` setelah dependensi paket kontrak terpasang. Router yang terpasang mencakup grup berikut:

| Grup | Contoh path | Crate |
| --- | --- | --- |
| Auth & tenant | `/api/v1/auth/register`, `/api/v1/auth/login`, `/api/v1/organizations`, `/api/v1/farms` | `mod-tenant` |
| Enclosure | `/api/v1/enclosures`, `/api/v1/enclosures/{id}/calculate-capacity` | `mod-enclosure` |
| Produksi | `/api/v1/batches`, `/api/v1/batches/{id}`, `/api/v1/batches/{id}/outputs` | `mod-production` |
| Pakan & inventory | `/api/v1/feeding`, `/api/v1/batches/{id}/fcr`, `/api/v1/inventory/...` | `mod-feeding` |
| Lingkungan | `/api/v1/environment/metrics`, `/api/v1/environment/readings` | `mod-environment` |
| Kesehatan | `/api/v1/batches/{id}/vaccinations`, `/api/v1/batches/{id}/health-records` | `mod-health` |
| Keuangan | `/api/v1/finance/expenses`, `/api/v1/finance/sales` | `mod-finance` |
| Sinkronisasi | `/api/v1/sync/devices`, `/api/v1/sync/push`, `/api/v1/sync/pull`, `/api/v1/sync/conflicts` | `infra-auth` + `bins/api-server` |
| Sistem | `/health`, `/api-docs/openapi.json` | `bins/api-server` |

Semua endpoint memakai prefix `/api/v1` dan sintaks path Axum 0.8 `{id}` (bukan `:id`).

---

## 4. Perubahan terhadap struktur awal

| Sebelum | Sesudah | Alasan |
| --- | --- | --- |
| `init_tenants_users.sql` | `2026112401_init_identity_organizations.sql` | `organization_id`, bukan `tenant_id` (ADR 001) |
| `SET LOCAL app.tenant_id` | `SET LOCAL app.current_organization_id` | Satu nama kanonik untuk RLS |
| `2026112402_init_farm_units_species.sql` | `2026112402_init_farm_enclosure_species.sql` | `production_units` + `batch_unit_assignments` |
| `2026112403_init_batches_ledgers.sql` | `2026112403_init_production_ledger.sql` | Ledger produksi + `individual_tags` |
| `2026112404_init_inventory_ledger.sql` | `2026112404_init_feeding_inventory.sql` | Feeding dan inventory dalam satu migrasi |
| `2026112405_init_finance.sql` + `2026112406_init_calculation_catalog.sql` | `2026112405_init_environment_health.sql` + `2026112406_init_finance_calculation.sql` | Urutan domain disesuaikan |
| `2026112499_rls_and_hardening.sql` | `2026112408_seed_catalogs.sql`, `2026112409_rls_triggers_hardening.sql` | Seed global dijalankan sebelum RLS agar bebas `BYPASSRLS` |
| `mod-pond` | `mod-enclosure` | Unit produksi tidak hanya kolam |
| `mod-waterquality` | `mod-environment` | Air, udara, tanah, dan kandang |
| `openapi/ternakmatric.v1.yaml` | `openapi/ternakmatric.v1.json` | Format hasil export Utoipa aktual |
| `data/` & `notes/` di root | `docs/data/`, `docs/notes/`, & `backend/migrations/` | Dokumentasi terpusat di `docs/`; seluruh SQL disatukan ke migrasi |

---

## 5. Catatan

- Seluruh SQL schema & seed disatukan ke [`../backend/migrations/`](../backend/migrations/) (`2026112401`–`2026112409`). Riwayat audit tercatat di [`notes/note-4-migration-audit.md`](notes/note-4-migration-audit.md).
- Migration yang sudah di-apply bersifat immutable. Penambahan berikutnya dibuat sebagai file baru `2026112410_...`, bukan mengedit file lama.
- Terdapat dua runner migrasi yang masih coexistence: `deploy/db/run-migrations.sh` (`public.schema_migrations`) dan `infra-db::run_migrations()` (`_sqlx_migrations`). Jangan mengaktifkan keduanya tanpa keputusan.
- Ledger stok, inventory, output, dan finance tidak boleh memakai `client_wins` (ADR 002).

---

## 6. Urutan implementasi

Urutan berikut mengikuti `docs/notes/note-3.md`, bukan UI terlebih dahulu:

1. Database hardening + migrasi/RLS test — **terimplementasi bersih di SQL (`2026112401`–`2026112409`) & `deploy/db/init-app-role.sql`**, menunggu eksekusi/test di PostgreSQL & CI.
2. Backend foundation (config, PgPool, transaksi, error handling) — **ada**.
3. Auth & multi-tenant (JWT, membership, RBAC, OrganizationContext) — **ada**.
4. Core animal production (farm, unit, species, batch, stock movement) — **ada**.
5. Calculation engine + versioning parameter spesies — **ada**, perlu seed profil berversi sesuai ADR 003.
6. Inventory & finance — **ada**.
7. Offline sync engine — **terimplementasi sebagian**: policy konflik & audit sinkronisasi sudah terpasang di migrasi `2026112407`/`2026112409`, menunggu pemrosesan queue sync di worker dan uji integrasi DB.
8. Web & mobile — **scaffold**, belum fokus utama.
