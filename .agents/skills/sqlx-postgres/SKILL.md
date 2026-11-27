---
name: SQLx and PostgreSQL
description: Use when writing or reviewing SQLx queries, transactions, tenant isolation, audit logs, outbox events, or database migrations in the TernakMatric Rust backend and PostgreSQL 16 schema.
---

# SQLx and PostgreSQL

The database is PostgreSQL 16 with RLS multi-tenancy. Tenant data is reachable only through a verified `OrganizationContext`.

## Tenant isolation is non-negotiable

- Start every tenant read or write with `let mut tx = auth.context().begin_tx(state.pool()).await.map_err(ApiError)?;`.
- `begin_tx` runs `SET LOCAL app.current_organization_id` and `SET LOCAL app.current_user_id`. These settings are transaction-scoped, so a query on the bare pool has no tenant context.
- Always include an explicit `organization_id = $n` predicate bound to `auth.organization_id.0`, even with RLS enabled. RLS is defense in depth, not a substitute.
- Never trust `X-Organization-Id` directly. `AuthPrincipal` verifies membership and RBAC before the handler runs.
- The application database role must not be a superuser and must not have `BYPASSRLS`, otherwise `FORCE ROW LEVEL SECURITY` is bypassed.
- Do not read another module's tables directly without a clear contract; prefer that module's API or a documented join.

```rust
let mut tx = auth.context().begin_tx(state.pool()).await.map_err(ApiError)?;
let rows: Vec<VaccinationScheduleDto> = sqlx::query_as(
    r#"
    SELECT id, batch_id, vaccine_name, target_disease, route, scheduled_date, dosage,
           status, completed_at, created_at, created_by, updated_at, updated_by
    FROM vaccination_schedules
    WHERE organization_id = $1 AND batch_id = $2
    ORDER BY scheduled_date ASC
    "#,
)
.bind(auth.organization_id.0)
.bind(batch_id)
.fetch_all(&mut *tx)
.await
.map_err(|e| ApiError(map_sqlx_err(e)))?;
tx.commit().await.map_err(|e| ApiError(map_sqlx_err(e)))?;
```

## Query patterns

- Use `sqlx::query_as` with `#[derive(sqlx::FromRow)]` DTOs, `query_scalar` for single values, and `fetch_optional` for lookups that can be absent.
- Map a missing row to `AppError::NotFound`; do not return `Option` all the way to the response unless absence is a valid result.
- Bind typed values. Never format user input into SQL.
- Dynamic filters follow the `audit.rs` pattern:

  ```sql
  WHERE organization_id = $1
    AND ($2::text IS NULL OR entity_type = $2)
    AND ($3::uuid IS NULL OR entity_id = $3)
  ```

- Clamp pagination (`limit.clamp(1, 200)`) before binding.
- Use `FOR UPDATE` on parent rows when checking capacity, balances, or uniqueness before writing.
- Commit explicitly. An early `return` drops the transaction and rolls it back, which is the intended behavior on errors.
- Keep transactions short: no HTTP calls, file I/O, or expensive CPU work inside them.

## Error mapping and idempotency

`infra_db::map_sqlx_err` is the single mapping point:

| SQLSTATE | `AppError` |
| --- | --- |
| `23505` unique violation | `Conflict` (HTTP 409) |
| `23503` foreign key violation | `Validation` (HTTP 400) |
| `23514` check violation | `Validation` (HTTP 400) |
| `RowNotFound` | `NotFound` |
| other | `Database` (HTTP 500) |

- Add new mappings in `map_sqlx_err`, not ad hoc in handlers.
- Critical write commands carry a client-generated `idempotency_key`; enforce uniqueness with an `(organization_id, idempotency_key)` constraint and handle `23505` as a retry, not an internal error.
- Convert at the handler boundary: `AppError` becomes `ApiError`, which renders the standard `ErrorEnvelope`.

## Migrations

- Location: `backend/migrations/<YYYYMMDDNN>_snake_name.sql`. The current latest is `2026112410_audit_actor_columns.sql`, so the next file is `2026112411_...`.
- Follow the canonical table and column names in `data/`, `docs/06-DATABASE-ERD.md`, and the existing migrations. Do not invent synonyms.
- Migrations already applied on a shared branch are immutable. Add a new migration instead of editing history.
- `run_migrations` uses `sqlx::migrate!("../../migrations")` compiled into `infra-db`. The path is relative to the crate; do not move the directory.
- For a new tenant table include:
  - `organization_id uuid NOT NULL`,
  - `UNIQUE (id, organization_id)` so children can use composite foreign keys,
  - composite child foreign keys `(parent_id, organization_id) REFERENCES parent(id, organization_id)`,
  - `ENABLE ROW LEVEL SECURITY` and `FORCE ROW LEVEL SECURITY` with the `tenant_isolation` policy using `current_organization_id()`,
  - audit columns `created_at`, `updated_at`, `created_by`, `updated_by` and the audit triggers,
  - indexes leading with `organization_id` for common query paths.
- Global-or-tenant references use the `assert_global_or_tenant_ref` trigger pattern from `2026112408_rls_triggers_hardening.sql`.
- Ledger tables (`stock_movements`, `inventory_transaction_lines`, `audit_logs`) are append-only via `deny_ledger_mutation`; correct errors with reversal or adjustment rows, never updates or deletes.
- New tables must be added to RLS setup in their own migration; do not edit the old hardening migration.

## Audit and outbox

Write both inside the same transaction as the domain change:

```rust
AuditLogger::record(
    &mut *tx,
    Some(auth.organization_id),
    Some(auth.user_id),
    "create",
    "batches",
    Some(batch_id),
    None,
    Some(serde_json::json!({ "batch_code": code })),
)
.await
.map_err(ApiError)?;

OutboxWriter::append(
    &mut *tx,
    auth.organization_id,
    "batch",
    batch_id,
    "batch.created",
    serde_json::json!({ "batch_id": batch_id }),
)
.await
.map_err(ApiError)?;
```

Audit and outbox rows are evidence for sync and background jobs; never write them outside the business transaction.

## Offline query cache

Current handlers use runtime `sqlx::query_as`, so no `.sqlx` cache is committed. If you introduce compile-time `query!` or `query_as!` macros:

```powershell
cargo sqlx prepare --workspace --manifest-path backend/Cargo.toml
```

Commit the generated `.sqlx/` directory; CI is expected to run `sqlx prepare --check`.

## Verification

```powershell
docker compose up -d postgres
cargo test --manifest-path backend/Cargo.toml --workspace
cargo clippy --manifest-path backend/Cargo.toml --workspace --all-targets -- -D warnings
```

For a schema change, apply migrations to a clean database and re-run the API OpenAPI test. See the `rust-testing` skill before adding database-backed tests.
