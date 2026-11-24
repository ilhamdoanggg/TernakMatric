-- 2026112402_init_farm_enclosure_species.sql
-- Multi-farm, enclosures (production_units), dynamic multi-species & versioned profiles.

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
  species_id uuid NOT NULL REFERENCES species(id),
  code varchar(80) NOT NULL,
  name varchar(140) NOT NULL,
  production_system_id uuid REFERENCES production_systems(id),
  parameters jsonb NOT NULL DEFAULT '{}'::jsonb,
  version integer NOT NULL DEFAULT 1 CHECK (version > 0),
  is_active boolean NOT NULL DEFAULT true,
  valid_from timestamptz NOT NULL DEFAULT now(),
  valid_until timestamptz,
  created_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE (id, organization_id),
  UNIQUE (organization_id, code, version),
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
