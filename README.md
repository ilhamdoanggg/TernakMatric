# TernakMatric

Smart Animal Production Management Platform — SaaS multi-tenant untuk ikan, unggas, babi, kambing, sapi, dan ternak lainnya.

## Struktur Workspace

- [`docs/`](docs/): Fondasi produk & arsitektur (`01-PROBLEM-DISCOVERY.md` s.d. `10-DATABASE-HARDENING.md` & [`docs/decisions/`](docs/decisions/))
- [`notes/`](notes/): Catatan perjalanan desain produk (`note-1.md` s.d. `note-3.md`)
- [`data/`](data/): Referensi baseline schema PostgreSQL, seed data, ERD, dan data dictionary
- [`backend/`](backend/): Rust Cargo Workspace (Axum + SQLx + Utoipa modular monolith, migrations, API server, background worker)
- [`packages/api-contracts/`](packages/api-contracts/): Spesifikasi OpenAPI dari Utoipa dan TypeScript API client
- [`apps/web/`](apps/web/): Dashboard analitik berbasis Vue 3 SSR + TypeScript (Vite)
- [`apps/mobile/`](apps/mobile/): Aplikasi mobile field input offline-first (React Native + Expo + SQLite)

Status: tahap fondasi — dokumentasi, schema kanonik, dan scaffold kode sedang diselaraskan untuk produksi multi-spesies.

## Perintah Cepat

```bash
# Backend Rust
cargo test --manifest-path backend/Cargo.toml --workspace
cargo run --manifest-path backend/Cargo.toml -p api-server -- --export-openapi ../packages/api-contracts/openapi/ternakmatric.v1.json

# Web & Mobile (setelah instalasi dependensi)
npm --prefix packages/api-contracts run typecheck
npm --prefix apps/web run dev
```
