# TernakMatric Database & Environment Setup

## 1. Menjalankan PostgreSQL + Migration DB

1. Salin template environment sesuai target (`dev`, `staging`, `production`):
   ```bash
   # Development lokal
   cp .env.example .env
   ```
   Untuk staging/production tersedia [`.env.staging.example`](../.env.staging.example) dan [`.env.production.example`](../.env.production.example).
2. Jalankan PostgreSQL dan eksekusi seluruh migrasi (`backend/migrations/*.sql`):
   ```bash
   docker compose up -d postgres && docker compose up db-migrate
   ```
   *(Untuk staging/prod atau file terpisah: `docker compose --env-file .env.staging ...`.)*
3. Untuk mengulang runner migrasi setelah menambah file SQL baru di `backend/migrations/`:
   ```bash
   docker compose run --rm db-migrate
   ```

## 2. Role Keamanan RLS
- `POSTGRES_USER` (`ternakmatric_migrator`): pemilik tabel, dipakai **hanya** saat migrasi.
- `APP_DB_USER` (`ternakmatric_app`): dibuat otomatis oleh `deploy/db/init-app-role.sql` sebagai `NOSUPERUSER NOBYPASSRLS`, dipakai oleh `api-server` dan `worker` (`DATABASE_URL`) agar kebijakan PostgreSQL RLS benar-benar aktif.

## 3. Environment Backend, Web, dan Mobile

- **Backend (`api-server` & `worker`)**: mendukung pemuatan file `.env` opsional (prioritas: env OS/container > `ENV_FILE` > `.env.<APP_ENV>` > `.env`), jadi aman dijalankan lokal maupun di container tanpa `.env`.
  ```bash
  # Dev (membaca .env atau .env.development sesuai APP_ENV)
  cargo run --manifest-path backend/Cargo.toml -p api-server
  cargo run --manifest-path backend/Cargo.toml -p worker
  ```
- **Web SSR (`apps/web/`)**: salin template di [`apps/web/`](../apps/web/):
  - Dev: `cp apps/web/.env.example apps/web/.env.development && npm --prefix apps/web run dev`
  - Staging: siapkan `apps/web/.env.staging`, build dengan `npm --prefix apps/web run build:staging`, lalu jalankan `NODE_ENV=production PORT=3000 API_INTERNAL_URL=http://api-server:8080 node apps/web/server.mjs`
  - Prod: siapkan `apps/web/.env.production`, build dengan `npm --prefix apps/web run build`, lalu jalankan server SSR dengan env runtime `PORT`, `NODE_ENV=production`, dan `API_INTERNAL_URL`.
- **Mobile (`apps/mobile/`)**: tersedia template `.env.example`, `.env.staging.example`, `.env.production.example`, dan [`eas.json`](../apps/mobile/eas.json) untuk profil build `development` / `staging` / `production`.
