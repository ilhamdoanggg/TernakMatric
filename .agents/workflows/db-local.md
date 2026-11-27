---
description: Menjalankan PostgreSQL lokal dan menerapkan migrasi via Docker Compose
---

Gunakan workflow ini untuk menyalakan PostgreSQL 16 lokal, menjalankan `deploy/db/run-migrations.sh`, atau memverifikasi migrasi dari database kosong.

1. **Nyalakan container PostgreSQL**:
```powershell
docker compose up -d postgres
```

2. **Jalankan runner migrasi & provisioning role aplikasi (`ternakmatric_app`)**:
```powershell
docker compose run --rm db-migrate
```

3. *(Opsional — uji dari database benar-benar kosong)* Jika sedang memverifikasi migrasi baru dari awal (clean bootstrap):
   > Peringatan: perintah berikut menghapus seluruh data pada volume lokal `ternakmatric_pgdata`.
   ```powershell
   docker compose down -v
   docker compose up -d postgres
   docker compose run --rm db-migrate
   ```

4. **Verifikasi tabel, migrasi, dan role non-superuser**:
   Pastikan migrasi tercatat di `public.schema_migrations` dan role `ternakmatric_app` aktif dengan `NOSUPERUSER NOBYPASSRLS`:
   ```powershell
   docker compose exec postgres psql -U ternakmatric_migrator -d ternakmatric -c "SELECT version, applied_at FROM public.schema_migrations ORDER BY version;"
   docker compose exec postgres psql -U ternakmatric_migrator -d ternakmatric -c "SELECT rolname, rolsuper, rolbypassrls FROM pg_roles WHERE rolname IN ('ternakmatric_migrator', 'ternakmatric_app');"
   ```
