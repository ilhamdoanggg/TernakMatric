# TernakMatric

Smart Animal Production Management Platform — SaaS multi-tenant untuk ikan, unggas, babi, kambing, sapi, dan ternak lainnya.

## Struktur Workspace

- [`docs/`](docs/): Seluruh dokumentasi produk & arsitektur terpusat — [`indeks dokumentasi`](docs/README.md), `01..12`, [`ADR`](docs/decisions/), [`catatan desain & audit`](docs/notes/), serta [`kamus data & ERD`](docs/data/)
- [`backend/`](backend/): Rust Cargo Workspace (Axum + SQLx + Utoipa modular monolith, seluruh SQL schema & seed di [`backend/migrations/`](backend/migrations/), API server, background worker)
- [`packages/api-contracts/`](packages/api-contracts/): Spesifikasi OpenAPI dari Utoipa dan TypeScript API client
- [`apps/web/`](apps/web/): Dashboard analitik berbasis Vue 3 SSR + TypeScript (Vite)
- [`apps/mobile/`](apps/mobile/): Aplikasi mobile field input offline-first (React Native + Expo + SQLite)

Status: fondasi produk, schema tergabung (`2026112401`–`2026112409`), dan backend sudah terbentuk. Status *production-readiness* migrasi tercatat di [`docs/notes/note-4-migration-audit.md`](docs/notes/note-4-migration-audit.md).

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
