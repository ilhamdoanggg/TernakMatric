---
name: Axum Endpoint
description: Use when adding, changing, or debugging an HTTP endpoint in the TernakMatric Axum API, covering DTOs, utoipa annotations, AuthPrincipal and RBAC, tenant-scoped transactions, router wiring, and OpenAPI plus TypeScript contract regeneration.
---

# Adding an HTTP Endpoint

Follow this end-to-end path so the handler, security, router, and generated contract stay in sync.

## 1. Choose the location

- Domain handlers live in `backend/crates/mod-<domain>/src/lib.rs`, following `mod-health` and `mod-production`.
- System or cross-cutting endpoints (health, sync) live in `backend/bins/api-server/src/main.rs`.
- Never add HTTP or SQL to `shared-kernel` or `mod-calculation`.

Read the existing handler and DTOs in the target module before writing; match their naming and structure.

## 2. Define request and response types

- Request: `#[derive(Debug, Serialize, Deserialize, ToSchema)]`.
- Response DTO: add `utoipa::ToSchema`; add `sqlx::FromRow` when it is read directly from a query.
- Use `Uuid`, `Decimal`, `DateTime<Utc>`, `NaiveDate`, and `Option<T>` to match the domain. Use an enum for closed database sets and `String` for extensible lookup codes.
- Write commands that change stock, ledgers, or other critical balances include `idempotency_key: Uuid`, following the existing production and health requests.
- Build an explicit DTO; never serialize a database row that contains internal-only columns.

```rust
#[derive(Debug, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct BatchDto {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub batch_code: String,
    // ...
    pub created_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
    pub updated_at: DateTime<Utc>,
    pub updated_by: Option<Uuid>,
}
```

## 3. Write the handler

```rust
#[utoipa::path(
    post,
    path = "/api/v1/batches",
    request_body = CreateBatchRequest,
    responses((status = 201, body = BatchDto)),
    tag = "Production"
)]
pub async fn create_batch<S: AppBackendState>(
    State(state): State<S>,
    auth: AuthPrincipal,
    Json(req): Json<CreateBatchRequest>,
) -> Result<(StatusCode, Json<BatchDto>), ApiError> {
    auth.require(Permission::ProductionWrite).map_err(ApiError)?;
    // validate before opening the transaction
    let mut tx = auth.context().begin_tx(state.pool()).await.map_err(ApiError)?;
    // query, audit, outbox ...
    tx.commit().await.map_err(|e| ApiError(map_sqlx_err(e)))?;
    Ok((StatusCode::CREATED, Json(dto)))
}
```

Rules:

- Be generic over `S: AppBackendState`; do not depend on the concrete `AppState`.
- Use the `AuthPrincipal` extractor. It verifies the Bearer JWT and the `X-Organization-Id` membership; never parse or trust the organization header yourself.
- Call `auth.require(...)` before any work. Permission codes: `org.manage`, `farm.manage`, `species.manage`, `production.write`, `feeding.write`, `environment.write`, `health.write`, `finance.write`, `analytics.read`.
- Validate input (positive counts, date ranges, enum values) before starting a transaction; return `AppError::Validation`.
- Return `(StatusCode::CREATED, Json(dto))` for creates and `Json(dto)` for reads and updates.
- Use `axum` 0.8 route syntax `/api/v1/items/{id}` with braces, not `:id`.
- Declare path parameters in `params(("id" = Uuid, Path, description = "..."))`.

## 4. Data access

- Always open the tenant transaction with `auth.context().begin_tx(state.pool()).await`. This sets `app.current_organization_id` for RLS and `app.current_user_id` for audit triggers.
- Bind `auth.organization_id.0` explicitly even though RLS is enabled. RLS is the second layer, not a replacement for a tenant predicate.
- Map SQLx errors with `map_sqlx_err` and wrap with `ApiError(...)`; do not hand-roll SQLSTATE checks.
- Lock rows with `FOR UPDATE` before capacity or balance checks, then call the pure functions in `mod-calculation`.
- Record mutations with `AuditLogger::record(..., &mut *tx, ...)` and publish domain events with `OutboxWriter::append(..., &mut *tx, ...)` inside the same transaction.
- See the `sqlx-postgres` skill for query, migration, and idempotency details.

## 5. Wire the router and OpenAPI

In the module:

```rust
pub fn router<S: AppBackendState>() -> Router<S> {
    Router::new().route("/api/v1/items", get(list_items::<S>).post(create_item::<S>))
}

#[derive(OpenApi)]
#[openapi(
    paths(list_items, create_item),
    components(schemas(CreateItemRequest, ItemDto))
)]
pub struct ItemApi;
```

In `backend/bins/api-server/src/main.rs`:

```rust
.merge(mod_item::router::<AppState>())
```

and add the document to the `nest(...)` list in `ApiDoc`. If you add a top-level path, extend `test_openapi_spec_builds_with_all_module_paths` at the bottom of that file.

The router factory is generic; the binary instantiates it with `::<AppState>`.

## 6. Regenerate the contract

The Utoipa annotations are the source of truth. Never hand-edit the generated JSON or TypeScript.

```powershell
npm --prefix packages/api-contracts run gen:openapi
npm --prefix packages/api-contracts run gen:ts
npm --prefix packages/api-contracts run typecheck
```

Commit `packages/api-contracts/openapi/ternakmatric.v1.json` and the generated TypeScript schema when the contract changes.

## 7. Done checklist

- [ ] Handler is generic over `AppBackendState` and uses `AuthPrincipal` plus `require`.
- [ ] Validation happens before the transaction.
- [ ] Every tenant query binds `auth.organization_id.0` and runs inside `begin_tx`.
- [ ] Audit and outbox writes share the transaction.
- [ ] Router and `ApiDoc` are wired.
- [ ] `cargo fmt`, `clippy -D warnings`, and `cargo test --workspace` pass.
- [ ] OpenAPI JSON, TypeScript schema, and `typecheck` are up to date.
