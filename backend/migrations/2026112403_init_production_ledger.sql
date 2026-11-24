-- 2026112403_init_production_ledger.sql
-- Production batches (cohorts), unit assignments, append-only population ledger,
-- individual tags, growth samples, and production outputs.

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

CREATE TABLE population_balances (
  organization_id uuid NOT NULL REFERENCES organizations(id),
  batch_id uuid NOT NULL,
  quantity integer NOT NULL DEFAULT 0 CHECK (quantity >= 0),
  version integer NOT NULL DEFAULT 1 CHECK (version > 0),
  updated_at timestamptz NOT NULL DEFAULT now(),
  PRIMARY KEY (organization_id, batch_id),
  FOREIGN KEY (batch_id, organization_id) REFERENCES production_batches(id, organization_id)
);

CREATE TABLE individual_tags (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  organization_id uuid NOT NULL REFERENCES organizations(id),
  batch_id uuid NOT NULL,
  tag_code varchar(80) NOT NULL,
  tag_type text NOT NULL DEFAULT 'ear_tag' CHECK (tag_type IN ('ear_tag','rfid','leg_band','microchip','tattoo','other')),
  sex text NOT NULL DEFAULT 'unknown' CHECK (sex IN ('male','female','unknown')),
  birth_date date,
  sire_tag_id uuid,
  dam_tag_id uuid,
  initial_weight_g numeric(12,3) CHECK (initial_weight_g IS NULL OR initial_weight_g >= 0),
  status text NOT NULL DEFAULT 'active' CHECK (status IN ('active','culled','deceased','sold','transferred')),
  notes text,
  version integer NOT NULL DEFAULT 1 CHECK (version > 0),
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE (organization_id, tag_code),
  UNIQUE (id, organization_id),
  FOREIGN KEY (batch_id, organization_id) REFERENCES production_batches(id, organization_id),
  FOREIGN KEY (sire_tag_id, organization_id) REFERENCES individual_tags(id, organization_id),
  FOREIGN KEY (dam_tag_id, organization_id) REFERENCES individual_tags(id, organization_id),
  CHECK (sire_tag_id IS NULL OR sire_tag_id <> id),
  CHECK (dam_tag_id IS NULL OR dam_tag_id <> id)
);

CREATE INDEX idx_individual_tags_batch ON individual_tags(organization_id, batch_id, status);

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
