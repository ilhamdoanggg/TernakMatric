-- 2026112410_audit_actor_columns.sql
-- Adds uniform created_by, updated_at, and updated_by audit columns across mutable tables
-- plus automatic triggers reading SET LOCAL app.current_user_id.

CREATE OR REPLACE FUNCTION current_user_id()
RETURNS uuid LANGUAGE sql STABLE
SET search_path = pg_catalog, public
AS $$
  SELECT nullif(current_setting('app.current_user_id', true), '')::uuid;
$$;

CREATE OR REPLACE FUNCTION set_created_by_from_context()
RETURNS trigger LANGUAGE plpgsql
SET search_path = pg_catalog, public
AS $$
DECLARE
  v_uid uuid;
BEGIN
  v_uid := current_user_id();
  IF v_uid IS NOT NULL THEN
    NEW.created_by = v_uid;
  END IF;
  RETURN NEW;
END;
$$;

CREATE OR REPLACE FUNCTION set_updated_audit()
RETURNS trigger LANGUAGE plpgsql
SET search_path = pg_catalog, public
AS $$
DECLARE
  v_uid uuid;
BEGIN
  NEW.updated_at = now();
  v_uid := current_user_id();
  IF v_uid IS NOT NULL THEN
    NEW.updated_by = v_uid;
  END IF;
  RETURN NEW;
END;
$$;

-- 1. Add created_at where missing
ALTER TABLE roles ADD COLUMN IF NOT EXISTS created_at timestamptz NOT NULL DEFAULT now();
ALTER TABLE growth_stages ADD COLUMN IF NOT EXISTS created_at timestamptz NOT NULL DEFAULT now();
ALTER TABLE expense_categories ADD COLUMN IF NOT EXISTS created_at timestamptz NOT NULL DEFAULT now();

-- 2. Add created_by to tables that did not yet track creator
DO $$
DECLARE t text;
BEGIN
  FOREACH t IN ARRAY ARRAY[
    'organizations', 'organization_members', 'roles', 'subscriptions',
    'farms', 'production_units', 'species', 'species_production_profiles',
    'growth_stages', 'feeding_guidelines', 'batch_unit_assignments',
    'individual_tags', 'warehouses', 'feed_products', 'inventory_transaction_lines',
    'expense_categories', 'sales', 'devices'
  ]
  LOOP
    EXECUTE format('ALTER TABLE %I ADD COLUMN IF NOT EXISTS created_by uuid REFERENCES users(id)', t);
  END LOOP;
END $$;

-- 3. Add updated_at and updated_by to all mutable tables
DO $$
DECLARE t text;
BEGIN
  FOREACH t IN ARRAY ARRAY[
    'users', 'organizations', 'organization_members', 'roles', 'subscriptions',
    'farms', 'production_units', 'species', 'species_production_profiles',
    'growth_stages', 'feeding_guidelines', 'production_batches',
    'batch_unit_assignments', 'population_balances', 'individual_tags',
    'growth_samples', 'production_outputs', 'warehouses', 'feed_products',
    'inventory_transactions', 'inventory_balances', 'feeding_records',
    'environment_readings', 'health_records', 'vaccination_schedules',
    'mortality_records', 'biosecurity_logs', 'expense_categories',
    'expenses', 'sales', 'devices'
  ]
  LOOP
    EXECUTE format('ALTER TABLE %I ADD COLUMN IF NOT EXISTS updated_at timestamptz NOT NULL DEFAULT now()', t);
    EXECUTE format('ALTER TABLE %I ADD COLUMN IF NOT EXISTS updated_by uuid REFERENCES users(id)', t);
  END LOOP;
END $$;

-- 4. Attach BEFORE INSERT trigger for created_by across all creator-tracked tables
DO $$
DECLARE t text;
BEGIN
  FOREACH t IN ARRAY ARRAY[
    'organizations', 'organization_members', 'roles', 'subscriptions',
    'farms', 'production_units', 'species', 'species_production_profiles',
    'growth_stages', 'feeding_guidelines', 'production_batches',
    'batch_unit_assignments', 'stock_movements', 'individual_tags',
    'growth_samples', 'production_outputs', 'warehouses', 'feed_products',
    'inventory_transactions', 'inventory_transaction_lines', 'feeding_records',
    'environment_readings', 'health_records', 'vaccination_schedules',
    'mortality_records', 'biosecurity_logs', 'expense_categories',
    'expenses', 'sales', 'devices'
  ]
  LOOP
    EXECUTE format('DROP TRIGGER IF EXISTS %I_set_created_by ON %I', t, t);
    EXECUTE format(
      'CREATE TRIGGER %I_set_created_by BEFORE INSERT ON %I FOR EACH ROW EXECUTE FUNCTION set_created_by_from_context()',
      t, t
    );
  END LOOP;
END $$;

-- 5. Replace set_updated_at trigger with unified set_updated_audit trigger (updated_at + updated_by)
DO $$
DECLARE t text;
BEGIN
  FOREACH t IN ARRAY ARRAY[
    'users', 'organizations', 'organization_members', 'roles', 'subscriptions',
    'farms', 'production_units', 'species', 'species_production_profiles',
    'growth_stages', 'feeding_guidelines', 'production_batches',
    'batch_unit_assignments', 'population_balances', 'individual_tags',
    'growth_samples', 'production_outputs', 'warehouses', 'feed_products',
    'inventory_transactions', 'inventory_balances', 'feeding_records',
    'environment_readings', 'health_records', 'vaccination_schedules',
    'mortality_records', 'biosecurity_logs', 'expense_categories',
    'expenses', 'sales', 'devices'
  ]
  LOOP
    EXECUTE format('DROP TRIGGER IF EXISTS %I_set_updated_at ON %I', t, t);
    EXECUTE format('DROP TRIGGER IF EXISTS %I_set_updated_audit ON %I', t, t);
    EXECUTE format(
      'CREATE TRIGGER %I_set_updated_audit BEFORE UPDATE ON %I FOR EACH ROW EXECUTE FUNCTION set_updated_audit()',
      t, t
    );
  END LOOP;
END $$;
