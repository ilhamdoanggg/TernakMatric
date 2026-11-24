-- 2026112407_init_offline_sync.sql
-- Mobile offline-first devices, idempotent sync command queue, conflicts, and transactional outbox.

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
  CHECK (
    resolution IS NULL
    OR resolution <> 'client_wins'
    OR entity_type NOT IN (
      'stock_movements','inventory_transactions','inventory_transaction_lines',
      'expenses','sales','production_outputs','feeding_records','mortality_records'
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
