# TernakMatric

Smart Animal Production Management Platform — SaaS multi-tenant untuk ikan, unggas, babi, kambing, sapi, dan ternak lainnya.

## Struktur Workspace

- [`docs/`](docs/): Fondasi produk & arsitektur — mulai dari [`indeks dokumentasi`](docs/README.md), `01..12`, dan [`ADR`](docs/decisions/)
- [`notes/`](notes/): Catatan perjalanan desain `note-1..3` dan [`audit migrasi note-4`](notes/note-4-migration-audit.md)
- [`data/`](data/): Referensi baseline schema PostgreSQL, seed data, ERD, dan data dictionary (bukan schema yang dipasang)
- [`backend/`](backend/): Rust Cargo Workspace (Axum + SQLx + Utoipa modular monolith, migrations, API server, background worker)
- [`packages/api-contracts/`](packages/api-contracts/): Spesifikasi OpenAPI dari Utoipa dan TypeScript API client
- [`apps/web/`](apps/web/): Dashboard analitik berbasis Vue 3 SSR + TypeScript (Vite)
- [`apps/mobile/`](apps/mobile/): Aplikasi mobile field input offline-first (React Native + Expo + SQLite)

Status: fondasi produk, schema, dan backend sudah terbentuk. Utang teknis migrasi dan status production-readiness tercatat di [`notes/note-4-migration-audit.md`](notes/note-4-migration-audit.md).

Aturan kerja untuk coding agent ada di [`AGENTS.md`](AGENTS.md).

## Perintah Cepat

```bash
# Backend Rust
cargo test --manifest-path backend/Cargo.toml --workspace
cargo run --manifest-path backend/Cargo.toml -p api-server -- --export-openapi ../packages/api-contracts/openapi/ternakmatric.v1.json

# Kontrak API & web (setelah instalasi dependensi)
npm --prefix packages/api-contracts run typecheck
npm --prefix apps/web run typecheck
npm --prefix apps/web run build
npm --prefix apps/web run dev
```
