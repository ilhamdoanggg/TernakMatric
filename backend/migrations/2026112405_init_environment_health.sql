-- 2026112405_init_environment_health.sql
-- Environment monitoring (water/air/soil/housing/feed), health records,
-- vaccination schedules, mortality records, and biosecurity logs.

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
  source text NOT NULL DEFAULT 'manual' CHECK (source IN ('manual','iot_sensor','import')),
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
  withdrawal_days integer CHECK (withdrawal_days IS NULL OR withdrawal_days >= 0),
  version integer NOT NULL DEFAULT 1 CHECK (version > 0),
  created_by uuid REFERENCES users(id),
  created_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE (id, organization_id),
  FOREIGN KEY (batch_id, organization_id) REFERENCES production_batches(id, organization_id)
);

CREATE TABLE vaccination_schedules (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  organization_id uuid NOT NULL REFERENCES organizations(id),
  batch_id uuid NOT NULL,
  vaccine_name varchar(160) NOT NULL,
  target_disease varchar(160),
  route text NOT NULL CHECK (route IN (
    'oral','drinking_water','spray','eye_drop','subcutaneous','intramuscular','immersion','other'
  )),
  scheduled_date date NOT NULL,
  dosage text,
  status text NOT NULL DEFAULT 'scheduled' CHECK (status IN ('scheduled','completed','overdue','skipped','cancelled')),
  completed_at timestamptz,
  health_record_id uuid,
  notes text,
  version integer NOT NULL DEFAULT 1 CHECK (version > 0),
  created_by uuid REFERENCES users(id),
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE (id, organization_id),
  FOREIGN KEY (batch_id, organization_id) REFERENCES production_batches(id, organization_id),
  FOREIGN KEY (health_record_id, organization_id) REFERENCES health_records(id, organization_id)
);

CREATE INDEX idx_vaccination_due
  ON vaccination_schedules(organization_id, status, scheduled_date);

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

CREATE TABLE biosecurity_logs (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  organization_id uuid NOT NULL REFERENCES organizations(id),
  farm_id uuid NOT NULL,
  unit_id uuid,
  activity_type text NOT NULL CHECK (activity_type IN (
    'disinfection','footbath','quarantine','visitor_entry','vehicle_wash','pest_control','equipment_sanitation','other'
  )),
  performed_at timestamptz NOT NULL,
  agent_used text,
  concentration text,
  actor_name varchar(160),
  passed boolean NOT NULL DEFAULT true,
  notes text,
  created_by uuid REFERENCES users(id),
  created_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE (id, organization_id),
  FOREIGN KEY (farm_id, organization_id) REFERENCES farms(id, organization_id),
  FOREIGN KEY (unit_id, organization_id) REFERENCES production_units(id, organization_id)
);

CREATE INDEX idx_biosecurity_farm_time
  ON biosecurity_logs(organization_id, farm_id, performed_at DESC);
