export const SQLITE_MIGRATIONS = [
  `CREATE TABLE IF NOT EXISTS local_enclosures (
    id TEXT PRIMARY KEY NOT NULL,
    organization_id TEXT NOT NULL,
    name TEXT NOT NULL,
    capacity_value REAL,
    capacity_unit TEXT,
    version INTEGER NOT NULL DEFAULT 1
  );`,
  `CREATE TABLE IF NOT EXISTS local_batches (
    id TEXT PRIMARY KEY NOT NULL,
    organization_id TEXT NOT NULL,
    batch_code TEXT NOT NULL,
    species_id TEXT NOT NULL,
    enclosure_id TEXT NOT NULL,
    current_population INTEGER NOT NULL,
    version INTEGER NOT NULL DEFAULT 1
  );`,
  `CREATE TABLE IF NOT EXISTS pending_sync_operations (
    client_operation_id TEXT PRIMARY KEY NOT NULL,
    entity_type TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    operation_type TEXT NOT NULL,
    base_version INTEGER,
    payload_json TEXT NOT NULL,
    device_occurred_at TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending'
  );`,
  `CREATE TABLE IF NOT EXISTS local_sync_conflicts (
    id TEXT PRIMARY KEY NOT NULL,
    entity_type TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    conflict_type TEXT NOT NULL,
    server_snapshot_json TEXT,
    client_snapshot_json TEXT,
    created_at TEXT NOT NULL
  );`,
  `CREATE TABLE IF NOT EXISTS sync_state (
    key TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL
  );`,
];
