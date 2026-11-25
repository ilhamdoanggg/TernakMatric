# TernakMatric Database Docker Compose

## Menjalankan PostgreSQL + Migration DB

1. Salin environment default (opsional, ada default dev di `docker-compose.yml`):
   ```bash
   cp .env.example .env
   ```
2. Jalankan PostgreSQL dan eksekusi semua migrasi (`backend/migrations/*.sql`):
   ```bash
   docker compose up -d postgres && docker compose up db-migrate
   ```
   *(Atau cukup `docker compose up` untuk menjalankan keduanya.)*
3. Untuk mengulang runner migrasi setelah menambah file SQL baru di `backend/migrations/`:
   ```bash
   docker compose run --rm db-migrate
   ```

## Role Keamanan RLS
- `POSTGRES_USER` (`ternakmatric_migrator`): pemilik tabel, dipakai **hanya** saat migrasi.
- `APP_DB_USER` (`ternakmatric_app`): dibuat otomatis oleh `deploy/db/init-app-role.sql` sebagai `NOSUPERUSER NOBYPASSRLS`, dipakai oleh `api-server` dan `worker` (`DATABASE_URL` di `.env.example`) agar kebijakan PostgreSQL RLS benar-benar aktif.
