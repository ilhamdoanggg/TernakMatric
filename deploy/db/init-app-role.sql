-- Provisions the least-privilege runtime role used by api-server and worker.
-- Must NEVER be superuser, table owner, or have BYPASSRLS so tenant RLS applies.

SELECT set_config('ternakmatric.app_user', :'app_user', true);
SELECT set_config('ternakmatric.app_password', :'app_password', true);

DO $$
DECLARE
  v_user text := current_setting('ternakmatric.app_user');
  v_pass text := current_setting('ternakmatric.app_password');
  v_db   text := current_database();
BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = v_user) THEN
    EXECUTE format(
      'CREATE ROLE %I WITH LOGIN PASSWORD %L NOSUPERUSER NOCREATEDB NOCREATEROLE NOINHERIT NOREPLICATION NOBYPASSRLS',
      v_user, v_pass
    );
  ELSE
    EXECUTE format(
      'ALTER ROLE %I WITH LOGIN PASSWORD %L NOSUPERUSER NOCREATEDB NOCREATEROLE NOINHERIT NOREPLICATION NOBYPASSRLS',
      v_user, v_pass
    );
  END IF;

  EXECUTE format('GRANT CONNECT ON DATABASE %I TO %I', v_db, v_user);
  EXECUTE format('GRANT USAGE ON SCHEMA public TO %I', v_user);
  EXECUTE format('GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA public TO %I', v_user);
  EXECUTE format('GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA public TO %I', v_user);
  EXECUTE format('GRANT EXECUTE ON ALL FUNCTIONS IN SCHEMA public TO %I', v_user);

  -- Global-only reference catalogs are read-only for the runtime role
  EXECUTE format(
    'REVOKE INSERT, UPDATE, DELETE ON TABLE plans, permissions, unit_types, production_systems, environment_metrics, calculation_versions, schema_migrations FROM %I',
    v_user
  );

  -- Append-only ledgers cannot be mutated even at privilege level
  EXECUTE format(
    'REVOKE UPDATE, DELETE ON TABLE stock_movements, inventory_transaction_lines, audit_logs FROM %I',
    v_user
  );
END $$;
