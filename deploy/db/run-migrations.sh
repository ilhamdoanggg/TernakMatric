#!/bin/sh
set -eu

echo "[db-migrate] Connecting to PostgreSQL at ${PGHOST}:${PGPORT}/${PGDATABASE} as ${PGUSER}..."

psql -v ON_ERROR_STOP=1 <<'SQL'
CREATE TABLE IF NOT EXISTS public.schema_migrations (
  filename text PRIMARY KEY,
  applied_at timestamptz NOT NULL DEFAULT now()
);
SQL

for file in /migrations/*.sql; do
  [ -e "$file" ] || continue
  name="$(basename "$file")"

  already_applied="$(psql -tA -v fname="$name" <<'SQL'
SELECT 1 FROM public.schema_migrations WHERE filename = :'fname';
SQL
)"
  if [ "$already_applied" = "1" ]; then
    echo "[db-migrate] Skipping already applied: $name"
    continue
  fi

  echo "[db-migrate] Applying migration: $name"
  psql --single-transaction -v ON_ERROR_STOP=1 -f "$file"
  psql -v ON_ERROR_STOP=1 -v fname="$name" <<'SQL'
INSERT INTO public.schema_migrations (filename) VALUES (:'fname');
SQL
done

echo "[db-migrate] Provisioning least-privilege runtime role (${APP_DB_USER})..."
psql --single-transaction \
  -v ON_ERROR_STOP=1 \
  -v app_user="${APP_DB_USER}" \
  -v app_password="${APP_DB_PASSWORD}" \
  -f /deploy-db/init-app-role.sql

echo "[db-migrate] All migrations applied and runtime role configured successfully."
