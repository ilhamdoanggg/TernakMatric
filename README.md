# TernakMatric

Smart Animal Production Management Platform — SaaS multi-tenant untuk ikan, unggas, babi, kambing, sapi, dan ternak lainnya.

## Struktur Workspace

- [`docs/`](docs/): Seluruh dokumentasi produk & arsitektur terpusat — [`indeks dokumentasi`](docs/README.md), `01..12`, [`ADR`](docs/decisions/), [`catatan desain & audit`](docs/notes/), serta [`kamus data & ERD`](docs/data/)
- [`backend/`](backend/): Rust Cargo Workspace (Axum + SQLx + Utoipa modular monolith, seluruh SQL schema & seed di [`backend/migrations/`](backend/migrations/), API server, background worker)
- [`deploy/`](deploy/): Runner migrasi database dan provisioning role aplikasi non-superuser ([`deploy/README.md`](deploy/README.md))
- [`packages/api-contracts/`](packages/api-contracts/): Spesifikasi OpenAPI dari Utoipa dan TypeScript API client ([`packages/api-contracts/README.md`](packages/api-contracts/README.md))
- [`apps/web/`](apps/web/): Dashboard analitik berbasis Vue 3 SSR + TypeScript (Vite + Tailwind) dengan arsitektur terpisah: `types/`, `services/`, `stores/`, `components/`, `features/`, dan `views/`
- [`apps/mobile/`](apps/mobile/): Aplikasi mobile field input offline-first (Flutter + SQLite) ([`apps/mobile/README.md`](apps/mobile/README.md))
- [`.agents/`](.agents/): Skill dan workflow runbook untuk coding agent (referensi aturan utama di [`AGENTS.md`](AGENTS.md))

Status: fondasi produk, schema tergabung (`2026112401`–`2026112410`), dan backend sudah terbentuk. Status *production-readiness* migrasi tercatat di [`docs/notes/note-4-migration-audit.md`](docs/notes/note-4-migration-audit.md).

## Perintah Cepat

Perintah berikut dijalankan dari root repository. Perintah `docker compose`, `cargo`, dan `npm --prefix` di bawah ini sama untuk **Linux/macOS (`bash`/`zsh`)** dan **Windows (`PowerShell`)**, kecuali yang diberi catatan khusus OS.

### Setup Environment

Linux / macOS (`bash`/`zsh`):

```bash
cp .env.example .env
cp apps/web/.env.example apps/web/.env
```

Windows (`PowerShell`):

```powershell
Copy-Item .env.example .env
Copy-Item apps/web/.env.example apps/web/.env
```

### Database Lokal (PostgreSQL 16 + Migrasi)

```bash
docker compose up -d postgres
docker compose run --rm db-migrate
```

### Backend Rust

```bash
cargo fmt --manifest-path backend/Cargo.toml --all
cargo clippy --manifest-path backend/Cargo.toml --workspace --all-targets -- -D warnings
cargo test --manifest-path backend/Cargo.toml --workspace
```

### Kontrak API (`packages/api-contracts`)

```bash
npm --prefix packages/api-contracts run gen:openapi
npm --prefix packages/api-contracts run gen:ts
npm --prefix packages/api-contracts run typecheck
```

### Web Dashboard (`apps/web`)

```bash
npm --prefix apps/web install
npm --prefix apps/web run typecheck
npm --prefix apps/web run build
npm --prefix apps/web run dev
```

Menjalankan hasil build produksi (`preview`):

```bash
# Linux / macOS (bash/zsh)
npm --prefix apps/web run preview
```

```powershell
# Windows (PowerShell)
$env:NODE_ENV = "production"
node apps/web/server.mjs
```

### Mobile App (`apps/mobile` — Flutter)

Linux / macOS (`bash`/`zsh`):

```bash
(
  cd apps/mobile
  flutter pub get
  flutter analyze
  flutter test
  flutter run --dart-define-from-file=.env.example
)
```

Windows (`PowerShell`):

```powershell
Push-Location apps/mobile
flutter pub get
flutter analyze
flutter test
flutter run --dart-define-from-file=.env.example
Pop-Location
```

---

## Logging, Monitoring & Database Archive

Backend TernakMatric dilengkapi sistem logging harian dan dashboard realtime:

1. **Daily Rolling Log File**:
   - Berkas log berotasi otomatis setiap hari dengan format nama `tanggal-bulan-tahun.log` (contoh: `27-11-2026.log`).
   - Lokasi folder log dapat dikustomisasi via environment variable `APP_LOG_DIR` atau `LOG_DIR` (default: `logs/`).
2. **Database Archive Tracking (`system_log_archives`)**:
   - Metadata berkas log (tanggal, service, path, ukuran byte, jumlah baris, checksum, dan status) dicatat di PostgreSQL melalui migrasi `2026112410_system_log_archives.sql`.
   - Disinkronkan secara otomatis oleh background `worker` dan dapat disinkronkan manual via endpoint `POST /api/v1/system/logs/sync`.
3. **Realtime Log Stream (SSE)**:
   - Endpoint `GET /api/v1/system/logs/stream` menyediakan streaming log realtime Server-Sent Events untuk operator/admin (`ORG_OWNER` / `ORG_ADMIN`).
4. **Web Dashboard (`/logs`)**:
   - Tampilan live console log realtime (`LiveLogTerminal`), daftar arsip berkas database (`LogArchiveList`), dan modal viewer pembaca isi berkas log (`LogContentModal`).
   - Kode tersusun rapi dengan pemisahan tanggung jawab: `src/types/`, `src/services/`, dan `src/stores/`.

