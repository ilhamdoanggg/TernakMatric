-- 2026112404_init_feeding_inventory.sql
-- Warehouses, feed products, unit conversions, append-only inventory ledger, and feeding records.

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
  UNIQUE (id, organization_id),
  FOREIGN KEY (transaction_id, organization_id) REFERENCES inventory_transactions(id, organization_id),
  FOREIGN KEY (feed_product_id, organization_id) REFERENCES feed_products(id, organization_id),
  FOREIGN KEY (destination_warehouse_id, organization_id) REFERENCES warehouses(id, organization_id),
  FOREIGN KEY (batch_id, organization_id) REFERENCES production_batches(id, organization_id)
);

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
