-- 2026112406_init_finance_calculation.sql
-- Expense categories, expenses, sales, sale_items, calculation_versions, calculation_runs, audit_logs.

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
    'weight_gain','sgr','egg_rate','milk_yield','harvest_forecast','hpp','profitability'
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
