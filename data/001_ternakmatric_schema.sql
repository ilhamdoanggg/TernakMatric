-- TernakMatric PostgreSQL baseline schema
-- Version: 2.0-generic-livestock-draft
-- Target: PostgreSQL 16+
--
-- Covers fish, poultry, pigs, goats/sheep, cattle/buffalo, rabbits, insects,
-- and other farmable animals. Species-specific husbandry parameters are data,
-- not hardcoded columns.
--
-- Apply with a migration role. Global seed rows (organization_id IS NULL) must be
-- inserted by a role with BYPASSRLS or with RLS temporarily disabled.
-- Runtime application role must NOT be superuser, table owner, or BYPASSRLS.
-- Every tenant-scoped transaction must call:
--   SET LOCAL app.current_organization_id = '<uuid>';

BEGIN;

CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE OR REPLACE FUNCTION set_updated_at()
RETURNS trigger LANGUAGE plpgsql
SET search_path = pg_catalog, public
AS $$
BEGIN
  NEW.updated_at = now();
  RETURN NEW;
END;
$$;

CREATE OR REPLACE FUNCTION current_organization_id()
RETURNS uuid LANGUAGE sql STABLE
SET search_path = pg_catalog, public
AS $$
  SELECT nullif(current_setting('app.current_organization_id', true), '')::uuid;
$$;

CREATE OR REPLACE FUNCTION assert_global_or_tenant_ref()
RETURNS trigger LANGUAGE plpgsql
SET search_path = pg_catalog, public
AS $$
DECLARE
  v_ref_id uuid;
  v_parent_org uuid;
BEGIN
  IF TG_NARGS <> 2 THEN
    RAISE EXCEPTION 'assert_global_or_tenant_ref requires child and parent table arguments';
  END IF;

  v_ref_id := (to_jsonb(NEW) ->> TG_ARGV[0])::uuid;
  IF v_ref_id IS NULL THEN
    RETURN NEW;
  END IF;

  IF current_organization_id() IS NULL THEN
    RAISE EXCEPTION 'app.current_organization_id must be set for tenant references';
  END IF;

  EXECUTE format('SELECT organization_id FROM %I WHERE id = $1', TG_ARGV[1])
    INTO v_parent_org
    USING v_ref_id;

  IF v_parent_org IS NOT NULL AND v_parent_org <> NEW.organization_id THEN
    RAISE EXCEPTION 'cross-tenant reference on %.% -> %.id (% <> %)',
      TG_TABLE_NAME, TG_ARGV[0], TG_ARGV[1], v_parent_org, NEW.organization_id;
  END IF;

  RETURN NEW;
END;
$$;

CREATE OR REPLACE FUNCTION deny_ledger_mutation()
RETURNS trigger LANGUAGE plpgsql
SET search_path = pg_catalog, public
AS $$
BEGIN
  RAISE EXCEPTION '% is append-only; use a reversal transaction', TG_TABLE_NAME;
END;
$$;

-- =========================
-- IDENTITY / SAAS / TENANCY
-- =========================

CREATE TABLE users (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  email varchar(320) NOT NULL UNIQUE,
  display_name varchar(160) NOT NULL,
  password_hash text,
  status text NOT NULL DEFAULT 'active' CHECK (status IN ('active','invited','suspended','deleted')),
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE organizations (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  name varchar(180) NOT NULL,
  slug varchar(100) NOT NULL UNIQUE,
  timezone varchar(64) NOT NULL DEFAULT 'Asia/Jakarta',
  default_currency char(3) NOT NULL DEFAULT 'IDR',
  status text NOT NULL DEFAULT 'active' CHECK (status IN ('active','suspended','closed')),
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE plans (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  code varchar(50) NOT NULL UNIQUE,
  name varchar(100) NOT NULL,
  limits jsonb NOT NULL DEFAULT '{}'::jsonb,
  features jsonb NOT NULL DEFAULT '{}'::jsonb,
  is_active boolean NOT NULL DEFAULT true,
  created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE subscriptions (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  organization_id uuid NOT NULL REFERENCES organizations(id),
  plan_id uuid NOT NULL REFERENCES plans(id),
  status text NOT NULL CHECK (status IN ('trialing','active','past_due','cancelled','expired')),
  starts_at timestamptz NOT NULL,
  ends_at timestamptz,
  external_customer_ref text,
  external_subscription_ref text,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE (id, organization_id),
  CHECK (ends_at IS NULL OR ends_at > starts_at)
);

CREATE TABLE roles (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  organization_id uuid REFERENCES organizations(id),
  code varchar(60) NOT NULL,
  name varchar(100) NOT NULL,
  is_system boolean NOT NULL DEFAULT false,
  UNIQUE NULLS NOT DISTINCT (organization_id, code),
  UNIQUE (id, organization_id),
  CHECK ((is_system AND organization_id IS NULL) OR (NOT is_system AND organization_id IS NOT NULL))
);

CREATE TABLE permissions (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  code varchar(100) NOT NULL UNIQUE,
  description text
);

CREATE TABLE role_permissions (
  role_id uuid NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
  permission_id uuid NOT NULL REFERENCES permissions(id) ON DELETE CASCADE,
  PRIMARY KEY (role_id, permission_id)
);

CREATE TABLE organization_members (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  organization_id uuid NOT NULL REFERENCES organizations(id),
  user_id uuid NOT NULL REFERENCES users(id),
  status text NOT NULL DEFAULT 'active' CHECK (status IN ('invited','active','suspended','removed')),
  invited_by uuid REFERENCES users(id),
  joined_at timestamptz,
  created_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE (organization_id, user_id),
  UNIQUE (id, organization_id)
);

CREATE TABLE member_roles (
  organization_id uuid NOT NULL,
  member_id uuid NOT NULL,
  role_id uuid NOT NULL REFERENCES roles(id),
  PRIMARY KEY (member_id, role_id),
  FOREIGN KEY (member_id, organization_id)
    REFERENCES organization_members(id, organization_id) ON DELETE CASCADE
);

-- =========================
-- FARM / PRODUCTION UNIT
-- Generic: kolam, kandang, cage, padang, sarang, dll.
-- =========================

CREATE TABLE unit_types (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  code varchar(50) NOT NULL UNIQUE,
  name varchar(120) NOT NULL,
  habitat text NOT NULL CHECK (habitat IN ('water','land','cage','pasture','hive','other')),
  description text
);

CREATE TABLE production_systems (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  code varchar(50) NOT NULL UNIQUE,
  name varchar(120) NOT NULL,
  description text
);

CREATE TABLE farms (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  organization_id uuid NOT NULL REFERENCES organizations(id),
  code varchar(60) NOT NULL,
  name varchar(180) NOT NULL,
  address text,
  latitude numeric(9,6),
  longitude numeric(9,6),
  status text NOT NULL DEFAULT 'active' CHECK (status IN ('active','inactive')),
  version integer NOT NULL DEFAULT 1 CHECK (version > 0),
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE (organization_id, code),
  UNIQUE (id, organization_id),
  CHECK (latitude IS NULL OR latitude BETWEEN -90 AND 90),
  CHECK (longitude IS NULL OR longitude BETWEEN -180 AND 180)
);

CREATE TABLE production_units (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  organization_id uuid NOT NULL REFERENCES organizations(id),
  farm_id uuid NOT NULL,
  code varchar(60) NOT NULL,
  name varchar(160) NOT NULL,
  unit_type_id uuid NOT NULL REFERENCES unit_types(id),
  production_system_id uuid REFERENCES production_systems(id),
  -- Fish-specific dimensions live here as optional keys; livestock units may
  -- leave this empty and use capacity_value/capacity_unit instead.
  dimensions jsonb NOT NULL DEFAULT '{}'::jsonb,
  capacity_value numeric(14,3),
  capacity_unit text CHECK (capacity_unit IN ('head','kg','m3','m2','tray','crate','colony','hive')),
  area_m2 numeric(14,3),
  volume_m3 numeric(14,3),
  status text NOT NULL DEFAULT 'active' CHECK (status IN ('active','inactive','maintenance')),
  version integer NOT NULL DEFAULT 1 CHECK (version > 0),
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE (organization_id, code),
  UNIQUE (id, organization_id),
  FOREIGN KEY (farm_id, organization_id) REFERENCES farms(id, organization_id),
  CHECK ((capacity_value IS NULL AND capacity_unit IS NULL)
      OR (capacity_value IS NOT NULL AND capacity_unit IS NOT NULL)),
  CHECK (capacity_value IS NULL OR capacity_value > 0),
  CHECK (area_m2 IS NULL OR area_m2 >= 0),
  CHECK (volume_m3 IS NULL OR volume_m3 >= 0)
);

CREATE INDEX idx_production_units_farm ON production_units(organization_id, farm_id);

-- =========================
-- SPECIES / PRODUCTION PROFILES
-- =========================

CREATE TABLE species (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  organization_id uuid REFERENCES organizations(id),
  category text NOT NULL CHECK (category IN (
    'fish','crustacean','mollusk','poultry','mammal','reptile','insect','other'
  )),
  common_name varchar(120) NOT NULL,
  scientific_name varchar(180),
  code varchar(80) NOT NULL,
  description text,
  is_system boolean NOT NULL DEFAULT false,
  is_active boolean NOT NULL DEFAULT true,
  created_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE NULLS NOT DISTINCT (organization_id, code),
  UNIQUE (id, organization_id),
  CHECK ((is_system AND organization_id IS NULL) OR (NOT is_system AND organization_id IS NOT NULL))
);

CREATE TABLE species_production_profiles (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  organization_id uuid NOT NULL REFERENCES organizations(id),
  species_id uuid NOT NULL,
  code varchar(80) NOT NULL,
  name varchar(140) NOT NULL,
  production_system_id uuid REFERENCES production_systems(id),
  -- Density, temperature, humidity, pH, DO, NH3, roughage ratio, etc. are
  -- stored per species/strain/system/version instead of hardcoded columns.
  parameters jsonb NOT NULL DEFAULT '{}'::jsonb,
  version integer NOT NULL DEFAULT 1 CHECK (version > 0),
  is_active boolean NOT NULL DEFAULT true,
  valid_from timestamptz NOT NULL DEFAULT now(),
  valid_until timestamptz,
  created_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE (id, organization_id),
  UNIQUE (organization_id, code, version),
  FOREIGN KEY (species_id) REFERENCES species(id),
  CHECK (valid_until IS NULL OR valid_until > valid_from)
);

CREATE TABLE growth_stages (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  organization_id uuid NOT NULL REFERENCES organizations(id),
  production_profile_id uuid NOT NULL,
  name varchar(100) NOT NULL,
  min_weight_g numeric(12,3),
  max_weight_g numeric(12,3),
  min_age_days integer,
  max_age_days integer,
  sort_order integer NOT NULL,
  FOREIGN KEY (production_profile_id, organization_id)
    REFERENCES species_production_profiles(id, organization_id),
  UNIQUE (id, organization_id),
  UNIQUE (production_profile_id, sort_order),
  CHECK (min_weight_g IS NULL OR min_weight_g >= 0),
  CHECK (max_weight_g IS NULL OR max_weight_g >= min_weight_g),
  CHECK (min_age_days IS NULL OR min_age_days >= 0),
  CHECK (max_age_days IS NULL OR max_age_days >= min_age_days)
);

CREATE TABLE feeding_guidelines (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  organization_id uuid NOT NULL REFERENCES organizations(id),
  growth_stage_id uuid NOT NULL,
  feeding_rate_value numeric(12,3) NOT NULL CHECK (feeding_rate_value >= 0),
  feeding_rate_basis text NOT NULL CHECK (feeding_rate_basis IN (
    'percent_biomass','percent_body_weight','gram_per_head_day','kg_per_head_day'
  )),
  meals_per_day integer CHECK (meals_per_day > 0),
  notes text,
  source text,
  version integer NOT NULL DEFAULT 1 CHECK (version > 0),
  is_active boolean NOT NULL DEFAULT true,
  valid_from timestamptz NOT NULL DEFAULT now(),
  valid_until timestamptz,
  created_at timestamptz NOT NULL DEFAULT now(),
  FOREIGN KEY (growth_stage_id, organization_id)
    REFERENCES growth_stages(id, organization_id),
  CHECK (valid_until IS NULL OR valid_until > valid_from)
);

-- =========================
-- PRODUCTION BATCHES
-- =========================

CREATE TABLE production_batches (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  organization_id uuid NOT NULL REFERENCES organizations(id),
  batch_code varchar(80) NOT NULL,
  species_id uuid NOT NULL REFERENCES species(id),
  production_profile_id uuid,
  start_date date NOT NULL,
  target_end_date date,
  initial_count integer NOT NULL CHECK (initial_count > 0),
  initial_avg_weight_g numeric(12,3) CHECK (initial_avg_weight_g IS NULL OR initial_avg_weight_g >= 0),
  status text NOT NULL DEFAULT 'planned' CHECK (status IN ('planned','active','harvesting','completed','cancelled')),
  notes text,
  version integer NOT NULL DEFAULT 1 CHECK (version > 0),
  created_by uuid REFERENCES users(id),
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE (organization_id, batch_code),
  UNIQUE (id, organization_id),
  FOREIGN KEY (production_profile_id, organization_id)
    REFERENCES species_production_profiles(id, organization_id),
  CHECK (target_end_date IS NULL OR target_end_date >= start_date)
);

CREATE TABLE batch_unit_assignments (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  organization_id uuid NOT NULL REFERENCES organizations(id),
  batch_id uuid NOT NULL,
  unit_id uuid NOT NULL,
  assigned_at timestamptz NOT NULL,
  released_at timestamptz,
  allocated_count integer CHECK (allocated_count IS NULL OR allocated_count >= 0),
  notes text,
  created_at timestamptz NOT NULL DEFAULT now(),
  FOREIGN KEY (batch_id, organization_id) REFERENCES production_batches(id, organization_id),
  FOREIGN KEY (unit_id, organization_id) REFERENCES production_units(id, organization_id),
  CHECK (released_at IS NULL OR released_at > assigned_at)
);

-- Satu unit boleh beberapa batch aktif, tetapi satu batch hanya boleh punya
-- satu assignment aktif.
CREATE INDEX idx_batch_unit_active ON batch_unit_assignments(organization_id, unit_id) WHERE released_at IS NULL;
CREATE UNIQUE INDEX uq_one_active_assignment_per_batch
  ON batch_unit_assignments(organization_id, batch_id) WHERE released_at IS NULL;
CREATE INDEX idx_batches_status ON production_batches(organization_id, status);

CREATE TABLE stock_movements (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  organization_id uuid NOT NULL REFERENCES organizations(id),
  batch_id uuid NOT NULL,
  movement_type text NOT NULL CHECK (movement_type IN (
    'initial_stock','additional_stock','birth','purchase','mortality','cull',
    'harvest','egg_collection','milk_collection','transfer_in','transfer_out',
    'sale','adjustment','reversal'
  )),
  quantity_delta integer NOT NULL CHECK (quantity_delta <> 0),
  occurred_at timestamptz NOT NULL,
  reference_id uuid,
  reversal_of uuid,
  reason text,
  idempotency_key uuid NOT NULL,
  created_by uuid REFERENCES users(id),
  created_at timestamptz NOT NULL DEFAULT now(),
  FOREIGN KEY (batch_id, organization_id) REFERENCES production_batches(id, organization_id),
  FOREIGN KEY (reversal_of) REFERENCES stock_movements(id),
  UNIQUE (organization_id, idempotency_key),
  UNIQUE (id, organization_id),
  CHECK (reversal_of IS NULL OR reversal_of <> id)
);

CREATE INDEX idx_stock_movements_batch_time ON stock_movements(organization_id, batch_id, occurred_at);

-- Projection saldo populasi. Harus diperbarui dalam transaksi yang sama
-- dengan penulisan stock_movements, dengan row lock pada batch.
CREATE TABLE population_balances (
  organization_id uuid NOT NULL REFERENCES organizations(id),
  batch_id uuid NOT NULL,
  quantity integer NOT NULL DEFAULT 0 CHECK (quantity >= 0),
  version integer NOT NULL DEFAULT 1 CHECK (version > 0),
  updated_at timestamptz NOT NULL DEFAULT now(),
  PRIMARY KEY (organization_id, batch_id),
  FOREIGN KEY (batch_id, organization_id) REFERENCES production_batches(id, organization_id)
);

CREATE TABLE mortality_records (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  organization_id uuid NOT NULL REFERENCES organizations(id),
  batch_id uuid NOT NULL,
  stock_movement_id uuid,
  quantity integer NOT NULL CHECK (quantity > 0),
  occurred_at timestamptz NOT NULL,
  suspected_cause text,
  notes text,
  version integer NOT NULL DEFAULT 1 CHECK (version > 0),
  created_by uuid REFERENCES users(id),
  created_at timestamptz NOT NULL DEFAULT now(),
  FOREIGN KEY (batch_id, organization_id) REFERENCES production_batches(id, organization_id),
  FOREIGN KEY (stock_movement_id, organization_id)
    REFERENCES stock_movements(id, organization_id),
  UNIQUE (organization_id, stock_movement_id)
);

CREATE TABLE growth_samples (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  organization_id uuid NOT NULL REFERENCES organizations(id),
  batch_id uuid NOT NULL,
  sampled_at timestamptz NOT NULL,
  sample_count integer NOT NULL CHECK (sample_count > 0),
  total_sample_weight_g numeric(14,3) NOT NULL CHECK (total_sample_weight_g > 0),
  average_weight_g numeric(12,3),
  average_length_cm numeric(10,3) CHECK (average_length_cm IS NULL OR average_length_cm > 0),
  method text,
  notes text,
  version integer NOT NULL DEFAULT 1 CHECK (version > 0),
  created_by uuid REFERENCES users(id),
  created_at timestamptz NOT NULL DEFAULT now(),
  FOREIGN KEY (batch_id, organization_id) REFERENCES production_batches(id, organization_id)
);

CREATE INDEX idx_growth_samples_batch_time
  ON growth_samples(organization_id, batch_id, sampled_at DESC);

-- =========================
-- ENVIRONMENT MONITORING
-- Generic: air, air kandang, tanah, pakan, air kolam, dll.
-- =========================

CREATE TABLE environment_metrics (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  code varchar(60) NOT NULL UNIQUE,
  name varchar(140) NOT NULL,
  default_unit varchar(40) NOT NULL,
  category text NOT NULL CHECK (category IN ('water','air','soil','housing','feed','other')),
  description text
);

CREATE TABLE environment_readings (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  organization_id uuid NOT NULL REFERENCES organizations(id),
  unit_id uuid NOT NULL,
  batch_id uuid,
  metric_id uuid NOT NULL REFERENCES environment_metrics(id),
  metric_value numeric(14,4) NOT NULL,
  unit varchar(40) NOT NULL CHECK (unit <> ''),
  measured_at timestamptz NOT NULL,
  notes text,
  version integer NOT NULL DEFAULT 1 CHECK (version > 0),
  created_by uuid REFERENCES users(id),
  created_at timestamptz NOT NULL DEFAULT now(),
  FOREIGN KEY (unit_id, organization_id) REFERENCES production_units(id, organization_id),
  FOREIGN KEY (batch_id, organization_id) REFERENCES production_batches(id, organization_id)
);

CREATE INDEX idx_environment_readings_unit_time
  ON environment_readings(organization_id, unit_id, measured_at DESC);

CREATE TABLE health_records (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  organization_id uuid NOT NULL REFERENCES organizations(id),
  batch_id uuid NOT NULL,
  observed_at timestamptz NOT NULL,
  category text NOT NULL CHECK (category IN ('observation','disease_suspected','treatment','vaccination','other')),
  description text NOT NULL,
  treatment_product text,
  dosage text,
  version integer NOT NULL DEFAULT 1 CHECK (version > 0),
  created_by uuid REFERENCES users(id),
  created_at timestamptz NOT NULL DEFAULT now(),
  FOREIGN KEY (batch_id, organization_id) REFERENCES production_batches(id, organization_id)
);

-- =========================
-- INVENTORY / FEED
-- =========================

CREATE TABLE warehouses (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  organization_id uuid NOT NULL REFERENCES organizations(id),
  farm_id uuid,
  code varchar(60) NOT NULL,
  name varchar(140) NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE (organization_id, code),
  UNIQUE (id, organization_id),
  FOREIGN KEY (farm_id, organization_id) REFERENCES farms(id, organization_id)
);

CREATE TABLE feed_products (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  organization_id uuid NOT NULL REFERENCES organizations(id),
  sku varchar(80) NOT NULL,
  name varchar(180) NOT NULL,
  brand varchar(120),
  pellet_size_mm numeric(7,3),
  protein_percent numeric(5,2),
  default_unit text NOT NULL DEFAULT 'kg' CHECK (default_unit IN ('kg','g','bag','unit','liter')),
  is_active boolean NOT NULL DEFAULT true,
  created_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE (organization_id, sku),
  UNIQUE (id, organization_id),
  CHECK (protein_percent IS NULL OR protein_percent BETWEEN 0 AND 100)
);

-- Faktor konversi ke unit kanonik (disarankan kg untuk pakan).
CREATE TABLE feed_unit_conversions (
  organization_id uuid NOT NULL REFERENCES organizations(id),
  feed_product_id uuid NOT NULL,
  from_unit text NOT NULL,
  to_unit text NOT NULL,
  factor numeric(18,6) NOT NULL CHECK (factor > 0),
  PRIMARY KEY (feed_product_id, from_unit, to_unit),
  FOREIGN KEY (feed_product_id, organization_id) REFERENCES feed_products(id, organization_id)
);

CREATE TABLE inventory_transactions (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  organization_id uuid NOT NULL REFERENCES organizations(id),
  warehouse_id uuid NOT NULL,
  transaction_type text NOT NULL CHECK (transaction_type IN ('receipt','issue','transfer','adjustment','return')),
  status text NOT NULL DEFAULT 'posted' CHECK (status IN ('draft','posted','voided')),
  occurred_at timestamptz NOT NULL,
  reference_code varchar(100),
  notes text,
  reversal_of uuid,
  idempotency_key uuid NOT NULL,
  created_by uuid REFERENCES users(id),
  created_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE (organization_id, idempotency_key),
  UNIQUE (id, organization_id),
  FOREIGN KEY (warehouse_id, organization_id) REFERENCES warehouses(id, organization_id),
  FOREIGN KEY (reversal_of) REFERENCES inventory_transactions(id),
  CHECK (reversal_of IS NULL OR reversal_of <> id)
);

-- Baris ledger bersifat append-only. Header hanya bertransisi status
-- draft -> posted/voided melalui domain layer.
CREATE TABLE inventory_transaction_lines (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  organization_id uuid NOT NULL REFERENCES organizations(id),
  transaction_id uuid NOT NULL,
  feed_product_id uuid NOT NULL,
  quantity numeric(14,3) NOT NULL CHECK (quantity > 0),
  unit text NOT NULL DEFAULT 'kg' CHECK (unit IN ('kg','g','bag','unit','liter')),
  unit_cost numeric(16,2) CHECK (unit_cost IS NULL OR unit_cost >= 0),
  lot_code varchar(100),
  expiry_date date,
  destination_warehouse_id uuid,
  batch_id uuid,
  created_at timestamptz NOT NULL DEFAULT now(),
  FOREIGN KEY (transaction_id, organization_id) REFERENCES inventory_transactions(id, organization_id),
  FOREIGN KEY (feed_product_id, organization_id) REFERENCES feed_products(id, organization_id),
  FOREIGN KEY (destination_warehouse_id, organization_id) REFERENCES warehouses(id, organization_id),
  FOREIGN KEY (batch_id, organization_id) REFERENCES production_batches(id, organization_id)
);

-- Projection saldo inventory. Dijaga dalam transaksi yang sama dengan ledger.
-- lot_code memakai sentinel 'DEFAULT' untuk baris tanpa lot.
CREATE TABLE inventory_balances (
  organization_id uuid NOT NULL REFERENCES organizations(id),
  warehouse_id uuid NOT NULL,
  feed_product_id uuid NOT NULL,
  lot_code varchar(100) NOT NULL DEFAULT 'DEFAULT',
  quantity numeric(14,3) NOT NULL DEFAULT 0 CHECK (quantity >= 0),
  version integer NOT NULL DEFAULT 1 CHECK (version > 0),
  updated_at timestamptz NOT NULL DEFAULT now(),
  PRIMARY KEY (organization_id, warehouse_id, feed_product_id, lot_code),
  FOREIGN KEY (warehouse_id, organization_id) REFERENCES warehouses(id, organization_id),
  FOREIGN KEY (feed_product_id, organization_id) REFERENCES feed_products(id, organization_id)
);

CREATE INDEX idx_inventory_lines_product ON inventory_transaction_lines(organization_id, feed_product_id);
CREATE INDEX idx_inventory_tx_time ON inventory_transactions(organization_id, occurred_at);

CREATE TABLE feeding_records (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  organization_id uuid NOT NULL REFERENCES organizations(id),
  batch_id uuid NOT NULL,
  unit_id uuid NOT NULL,
  warehouse_id uuid,
  feed_product_id uuid NOT NULL,
  inventory_line_id uuid,
  fed_at timestamptz NOT NULL,
  quantity numeric(14,3) NOT NULL CHECK (quantity > 0),
  unit text NOT NULL DEFAULT 'kg' CHECK (unit IN ('kg','g','bag','unit','liter')),
  unit_cost numeric(16,2) CHECK (unit_cost IS NULL OR unit_cost >= 0),
  feeding_method text,
  notes text,
  version integer NOT NULL DEFAULT 1 CHECK (version > 0),
  idempotency_key uuid NOT NULL,
  created_by uuid REFERENCES users(id),
  created_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE (organization_id, idempotency_key),
  UNIQUE (organization_id, inventory_line_id),
  FOREIGN KEY (batch_id, organization_id) REFERENCES production_batches(id, organization_id),
  FOREIGN KEY (unit_id, organization_id) REFERENCES production_units(id, organization_id),
  FOREIGN KEY (warehouse_id, organization_id) REFERENCES warehouses(id, organization_id),
  FOREIGN KEY (feed_product_id, organization_id) REFERENCES feed_products(id, organization_id),
  FOREIGN KEY (inventory_line_id, organization_id) REFERENCES inventory_transaction_lines(id, organization_id)
);

CREATE INDEX idx_feeding_batch_time ON feeding_records(organization_id, batch_id, fed_at DESC);

-- =========================
-- PRODUCTION OUTPUT / FINANCE
-- Generic: daging, telur, susu, fry, cull, offspring, dll.
-- =========================

CREATE TABLE production_outputs (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  organization_id uuid NOT NULL REFERENCES organizations(id),
  batch_id uuid NOT NULL,
  stock_movement_id uuid,
  occurred_at timestamptz NOT NULL,
  output_type text NOT NULL CHECK (output_type IN ('meat','egg','milk','fry','cull','offspring','other')),
  quantity numeric(14,3) NOT NULL CHECK (quantity > 0),
  unit text NOT NULL CHECK (unit IN ('head','kg','g','egg','liter','tray','crate','other')),
  average_weight_g numeric(12,3),
  grade text,
  notes text,
  version integer NOT NULL DEFAULT 1 CHECK (version > 0),
  created_by uuid REFERENCES users(id),
  created_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE (id, organization_id),
  UNIQUE (organization_id, stock_movement_id),
  FOREIGN KEY (batch_id, organization_id) REFERENCES production_batches(id, organization_id),
  FOREIGN KEY (stock_movement_id, organization_id)
    REFERENCES stock_movements(id, organization_id)
);

CREATE TABLE expense_categories (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  organization_id uuid REFERENCES organizations(id),
  code varchar(60) NOT NULL,
  name varchar(120) NOT NULL,
  is_system boolean NOT NULL DEFAULT false,
  UNIQUE NULLS NOT DISTINCT (organization_id, code),
  UNIQUE (id, organization_id),
  CHECK ((is_system AND organization_id IS NULL) OR (NOT is_system AND organization_id IS NOT NULL))
);

CREATE TABLE expenses (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  organization_id uuid NOT NULL REFERENCES organizations(id),
  batch_id uuid,
  farm_id uuid,
  category_id uuid REFERENCES expense_categories(id),
  expense_date date NOT NULL,
  description text NOT NULL,
  quantity numeric(14,3) NOT NULL DEFAULT 1 CHECK (quantity > 0),
  unit_cost numeric(16,2) NOT NULL CHECK (unit_cost >= 0),
  total_amount numeric(16,2) NOT NULL CHECK (total_amount >= 0),
  currency char(3) NOT NULL DEFAULT 'IDR',
  source_type text,
  source_id uuid,
  version integer NOT NULL DEFAULT 1 CHECK (version > 0),
  created_by uuid REFERENCES users(id),
  created_at timestamptz NOT NULL DEFAULT now(),
  FOREIGN KEY (batch_id, organization_id) REFERENCES production_batches(id, organization_id),
  FOREIGN KEY (farm_id, organization_id) REFERENCES farms(id, organization_id)
);

CREATE TABLE sales (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  organization_id uuid NOT NULL REFERENCES organizations(id),
  batch_id uuid NOT NULL,
  sale_date date NOT NULL,
  buyer_name varchar(180),
  currency char(3) NOT NULL DEFAULT 'IDR',
  notes text,
  version integer NOT NULL DEFAULT 1 CHECK (version > 0),
  created_at timestamptz NOT NULL DEFAULT now(),
  FOREIGN KEY (batch_id, organization_id) REFERENCES production_batches(id, organization_id),
  UNIQUE (id, organization_id)
);

CREATE TABLE sale_items (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  organization_id uuid NOT NULL REFERENCES organizations(id),
  sale_id uuid NOT NULL,
  production_output_id uuid,
  description text NOT NULL,
  quantity numeric(14,3) NOT NULL CHECK (quantity > 0),
  unit text NOT NULL DEFAULT 'kg' CHECK (unit IN ('head','kg','g','egg','liter','tray','crate','other')),
  unit_price numeric(16,2) NOT NULL CHECK (unit_price >= 0),
  line_total numeric(16,2) NOT NULL CHECK (line_total >= 0),
  FOREIGN KEY (sale_id, organization_id) REFERENCES sales(id, organization_id),
  FOREIGN KEY (production_output_id, organization_id)
    REFERENCES production_outputs(id, organization_id)
);

-- =========================
-- CALCULATION / FORECAST / AUDIT
-- =========================

CREATE TABLE calculation_versions (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  code varchar(80) NOT NULL,
  version integer NOT NULL CHECK (version > 0),
  formula_name varchar(120) NOT NULL,
  formula_definition jsonb NOT NULL,
  effective_from timestamptz NOT NULL,
  effective_until timestamptz,
  created_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE (code, version),
  CHECK (effective_until IS NULL OR effective_until > effective_from)
);

CREATE TABLE calculation_runs (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  organization_id uuid NOT NULL REFERENCES organizations(id),
  batch_id uuid,
  unit_id uuid,
  calculation_version_id uuid NOT NULL REFERENCES calculation_versions(id),
  calculation_type text NOT NULL CHECK (calculation_type IN (
    'capacity','biomass','population','survival_rate','mortality_rate','fcr',
    'weight_gain','egg_rate','milk_yield','harvest_forecast','profitability'
  )),
  formula_version integer NOT NULL CHECK (formula_version > 0),
  parameter_version integer CHECK (parameter_version > 0),
  input_snapshot jsonb NOT NULL,
  result_snapshot jsonb NOT NULL,
  assumptions jsonb NOT NULL DEFAULT '{}'::jsonb,
  sample_at timestamptz,
  calculated_at timestamptz NOT NULL DEFAULT now(),
  FOREIGN KEY (batch_id, organization_id) REFERENCES production_batches(id, organization_id),
  FOREIGN KEY (unit_id, organization_id) REFERENCES production_units(id, organization_id),
  CHECK ((batch_id IS NULL) <> (unit_id IS NULL))
);

CREATE TABLE audit_logs (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  organization_id uuid REFERENCES organizations(id),
  actor_user_id uuid REFERENCES users(id),
  action varchar(100) NOT NULL,
  entity_type varchar(100) NOT NULL,
  entity_id uuid,
  before_data jsonb,
  after_data jsonb,
  occurred_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX idx_audit_org_time ON audit_logs(organization_id, occurred_at DESC);
CREATE INDEX idx_calc_batch_time ON calculation_runs(organization_id, batch_id, calculated_at DESC);
CREATE INDEX idx_calc_unit_time ON calculation_runs(organization_id, unit_id, calculated_at DESC);

-- =========================
-- OFFLINE SYNC / OUTBOX
-- =========================

CREATE TABLE devices (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  organization_id uuid NOT NULL REFERENCES organizations(id),
  user_id uuid NOT NULL REFERENCES users(id),
  device_label varchar(160),
  platform text NOT NULL CHECK (platform IN ('web','ios','android')),
  device_public_id varchar(160) NOT NULL,
  last_seen_at timestamptz,
  revoked_at timestamptz,
  created_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE (organization_id, device_public_id),
  UNIQUE (id, organization_id)
);

CREATE TABLE sync_operations (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  organization_id uuid NOT NULL REFERENCES organizations(id),
  device_id uuid NOT NULL,
  client_operation_id uuid NOT NULL,
  entity_type varchar(100) NOT NULL,
  entity_id uuid NOT NULL,
  operation_type text NOT NULL CHECK (operation_type IN ('create','update','delete')),
  base_version integer CHECK (base_version IS NULL OR base_version > 0),
  device_occurred_at timestamptz NOT NULL,
  payload jsonb NOT NULL,
  status text NOT NULL DEFAULT 'pending' CHECK (status IN ('pending','applied','rejected','conflict')),
  received_at timestamptz NOT NULL DEFAULT now(),
  processed_at timestamptz,
  response_payload jsonb,
  FOREIGN KEY (device_id, organization_id) REFERENCES devices(id, organization_id),
  UNIQUE (organization_id, device_id, client_operation_id),
  UNIQUE (id, organization_id)
);

CREATE TABLE sync_conflicts (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  organization_id uuid NOT NULL REFERENCES organizations(id),
  sync_operation_id uuid NOT NULL,
  entity_type varchar(100) NOT NULL,
  entity_id uuid NOT NULL,
  conflict_type text NOT NULL,
  server_snapshot jsonb,
  client_snapshot jsonb,
  resolution text CHECK (resolution IN ('server_wins','client_wins','merged','manual')),
  resolved_by uuid REFERENCES users(id),
  resolved_at timestamptz,
  created_at timestamptz NOT NULL DEFAULT now(),
  FOREIGN KEY (sync_operation_id, organization_id)
    REFERENCES sync_operations(id, organization_id),
  -- Ledger dan keuangan tidak boleh diselesaikan dengan client_wins.
  CHECK (
    resolution IS NULL
    OR resolution <> 'client_wins'
    OR entity_type NOT IN (
      'stock_movements','inventory_transactions','inventory_transaction_lines',
      'expenses','sales','production_outputs','feeding_records'
    )
  )
);

CREATE INDEX idx_sync_pending ON sync_operations(organization_id, status, received_at);
CREATE INDEX idx_sync_entity ON sync_operations(organization_id, entity_type, entity_id);

CREATE TABLE outbox_events (
  id bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
  organization_id uuid NOT NULL REFERENCES organizations(id),
  aggregate_type varchar(100) NOT NULL,
  aggregate_id uuid NOT NULL,
  event_type varchar(100) NOT NULL,
  payload jsonb NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  published_at timestamptz
);

CREATE INDEX idx_outbox_unpublished
  ON outbox_events(organization_id, id) WHERE published_at IS NULL;

-- =========================
-- TRIGGERS
-- =========================

CREATE TRIGGER trg_member_roles_tenant
  BEFORE INSERT OR UPDATE OF role_id, organization_id ON member_roles
  FOR EACH ROW EXECUTE FUNCTION assert_global_or_tenant_ref('role_id', 'roles');

CREATE TRIGGER trg_species_profile_tenant
  BEFORE INSERT OR UPDATE OF species_id, organization_id ON species_production_profiles
  FOR EACH ROW EXECUTE FUNCTION assert_global_or_tenant_ref('species_id', 'species');

CREATE TRIGGER trg_batch_species_tenant
  BEFORE INSERT OR UPDATE OF species_id, organization_id ON production_batches
  FOR EACH ROW EXECUTE FUNCTION assert_global_or_tenant_ref('species_id', 'species');

CREATE TRIGGER trg_expense_category_tenant
  BEFORE INSERT OR UPDATE OF category_id, organization_id ON expenses
  FOR EACH ROW EXECUTE FUNCTION assert_global_or_tenant_ref('category_id', 'expense_categories');

CREATE TRIGGER trg_stock_reversal_tenant
  BEFORE INSERT OR UPDATE OF reversal_of, organization_id ON stock_movements
  FOR EACH ROW EXECUTE FUNCTION assert_global_or_tenant_ref('reversal_of', 'stock_movements');

CREATE TRIGGER trg_inventory_reversal_tenant
  BEFORE INSERT OR UPDATE OF reversal_of, organization_id ON inventory_transactions
  FOR EACH ROW EXECUTE FUNCTION assert_global_or_tenant_ref('reversal_of', 'inventory_transactions');

CREATE TRIGGER trg_stock_movements_immutable
  BEFORE UPDATE OR DELETE ON stock_movements
  FOR EACH ROW EXECUTE FUNCTION deny_ledger_mutation();

CREATE TRIGGER trg_inventory_lines_immutable
  BEFORE UPDATE OR DELETE ON inventory_transaction_lines
  FOR EACH ROW EXECUTE FUNCTION deny_ledger_mutation();

CREATE TRIGGER trg_audit_logs_immutable
  BEFORE UPDATE OR DELETE ON audit_logs
  FOR EACH ROW EXECUTE FUNCTION deny_ledger_mutation();

DO $$
DECLARE t text;
BEGIN
  FOREACH t IN ARRAY ARRAY[
    'users','organizations','subscriptions','farms','production_units',
    'production_batches','population_balances','inventory_balances'
  ]
  LOOP
    EXECUTE format(
      'CREATE TRIGGER %I_set_updated_at BEFORE UPDATE ON %I FOR EACH ROW EXECUTE FUNCTION set_updated_at()',
      t, t
    );
  END LOOP;
END $$;

-- =========================
-- TENANT RLS
-- Application must set app.current_organization_id with SET LOCAL in every
-- tenant-scoped transaction. Never use a session-level setting with pooling.
-- =========================

DO $$
DECLARE t text;
BEGIN
  FOREACH t IN ARRAY ARRAY[
    'subscriptions','organization_members','member_roles','farms','production_units',
    'species_production_profiles','growth_stages','feeding_guidelines',
    'production_batches','batch_unit_assignments','stock_movements',
    'population_balances','mortality_records','growth_samples','environment_readings',
    'health_records','warehouses','feed_products','feed_unit_conversions',
    'inventory_transactions','inventory_transaction_lines','inventory_balances',
    'feeding_records','production_outputs','expenses','sales','sale_items',
    'calculation_runs','audit_logs','devices','sync_operations',
    'sync_conflicts','outbox_events'
  ]
  LOOP
    EXECUTE format('ALTER TABLE %I ENABLE ROW LEVEL SECURITY', t);
    EXECUTE format('ALTER TABLE %I FORCE ROW LEVEL SECURITY', t);
    EXECUTE format(
      'CREATE POLICY tenant_isolation ON %I USING (organization_id = current_organization_id()) WITH CHECK (organization_id = current_organization_id())',
      t
    );
  END LOOP;
END $$;

-- Global/tenant hybrid catalogs: readable by all tenants, writable only by
-- the owning tenant. NULL organization_id rows are global system rows.
ALTER TABLE species ENABLE ROW LEVEL SECURITY;
ALTER TABLE species FORCE ROW LEVEL SECURITY;
CREATE POLICY species_read ON species FOR SELECT USING (
  organization_id IS NULL OR organization_id = current_organization_id()
);
CREATE POLICY species_write ON species FOR ALL USING (
  organization_id = current_organization_id()
) WITH CHECK (
  organization_id = current_organization_id()
);

ALTER TABLE roles ENABLE ROW LEVEL SECURITY;
ALTER TABLE roles FORCE ROW LEVEL SECURITY;
CREATE POLICY roles_read ON roles FOR SELECT USING (
  organization_id IS NULL OR organization_id = current_organization_id()
);
CREATE POLICY roles_write ON roles FOR ALL USING (
  organization_id = current_organization_id()
) WITH CHECK (
  organization_id = current_organization_id()
);

ALTER TABLE expense_categories ENABLE ROW LEVEL SECURITY;
ALTER TABLE expense_categories FORCE ROW LEVEL SECURITY;
CREATE POLICY expense_categories_read ON expense_categories FOR SELECT USING (
  organization_id IS NULL OR organization_id = current_organization_id()
);
CREATE POLICY expense_categories_write ON expense_categories FOR ALL USING (
  organization_id = current_organization_id()
) WITH CHECK (
  organization_id = current_organization_id()
);

-- role_permissions has no organization_id column; visibility follows the role.
ALTER TABLE role_permissions ENABLE ROW LEVEL SECURITY;
ALTER TABLE role_permissions FORCE ROW LEVEL SECURITY;
CREATE POLICY role_permissions_read ON role_permissions FOR SELECT USING (
  EXISTS (
    SELECT 1 FROM roles r
    WHERE r.id = role_permissions.role_id
      AND (r.organization_id IS NULL OR r.organization_id = current_organization_id())
  )
);
CREATE POLICY role_permissions_write ON role_permissions FOR ALL USING (
  EXISTS (
    SELECT 1 FROM roles r
    WHERE r.id = role_permissions.role_id
      AND r.organization_id = current_organization_id()
  )
) WITH CHECK (
  EXISTS (
    SELECT 1 FROM roles r
    WHERE r.id = role_permissions.role_id
      AND r.organization_id = current_organization_id()
  )
);

-- users, organizations, plans, permissions, unit_types, production_systems,
-- environment_metrics, and calculation_versions intentionally have no RLS here:
-- identity lookup and global catalogs require membership-based policies and
-- explicit GRANTs that depend on an app.current_user_id setting.
-- Restrict writes via GRANT/REVOKE.

COMMIT;
