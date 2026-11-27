---
name: Rust Backend Conventions
description: Use when writing, reviewing, or refactoring Rust code in the TernakMatric backend workspace (Axum, SQLx, Utoipa), including crate boundaries, shared-kernel types, errors, Decimal/time conventions, and required cargo commands.
---

# TernakMatric Rust Backend

`backend/` is one Cargo workspace (`backend/Cargo.toml`), a modular monolith using Axum 0.8, SQLx 0.8 with PostgreSQL 16, Utoipa 5, and Tokio. Edition is `2021`.

## Commands

Run from the repository root:

```powershell
cargo fmt --manifest-path backend/Cargo.toml --all
cargo clippy --manifest-path backend/Cargo.toml --workspace --all-targets -- -D warnings
cargo test --manifest-path backend/Cargo.toml --workspace
```

Focused runs:

```powershell
cargo test --manifest-path backend/Cargo.toml -p mod-calculation
cargo clippy --manifest-path backend/Cargo.toml -p mod-health --all-targets -- -D warnings
```

`backend/Cargo.lock` is tracked on purpose; never delete it or add it to `.gitignore`. Declare shared dependencies once in `[workspace.dependencies]` and inherit them with `dep.workspace = true` in member crates.

## Dependency direction

The rules come from `docs/07-SYSTEM-ARCHITECTURE.md`:

- `shared-kernel`: pure domain types. It must not depend on `sqlx`, `axum`, or network I/O.
- `mod-calculation`: pure calculation engine. No DB, HTTP, clock, or network. Every input, including timestamps, is an explicit parameter.
- `infra-db`: `PgPool`, transactions, RLS context, audit log, outbox. Depends on `shared-kernel` plus infrastructure crates.
- `infra-auth`: JWT, extractors, RBAC. Depends on `shared-kernel`, `infra-db`, `axum`, `jsonwebtoken`.
- `mod-*`: domain modules with handlers and SQL. They may use `shared-kernel`, `infra-db`, `infra-auth`, and `mod-calculation`. Do not read another module's tables directly without a clear contract.
- `bins/api-server`: router, application state, and combined OpenAPI document only.
- `bins/worker`: background jobs.

Do not add HTTP handlers or SQL to `shared-kernel` or `mod-calculation`. Do not introduce dependency cycles between `mod-*` crates.

## Domain conventions

- **IDs**: use the newtypes in `backend/crates/shared-kernel/src/ids.rs` (`OrganizationId`, `UserId`, `BatchId`, ...). Add new ones with the existing `define_id!` macro. Bind the inner UUID with `.0` at the SQL boundary and keep the newtype in domain and handler signatures.
- **Numbers**: use `rust_decimal::Decimal` for money, weights, rates, and measurements. Never use `f64` for business values. Tests use `rust_decimal_macros::dec!`.
- **Enums**: persisted enums derive `Serialize`, `Deserialize`, and `ToSchema`, use `#[serde(rename_all = "snake_case")]`, and expose `as_str()` returning the exact database code. Keep values aligned with `data/` and the migrations.
- **Time**: `chrono::DateTime<Utc>` for instants, `NaiveDate` for calendar dates. Pass time into pure functions; do not call `Utc::now()` inside `mod-calculation`.
- **Audit columns**: mutable tenant tables carry `created_at`, `updated_at`, `created_by`, `updated_by`. Triggers read `app.current_user_id`, so writes must run inside `OrganizationContext::begin_tx`.
- **Errors**: return `shared_kernel::AppError`; convert SQLx errors with `infra_db::map_sqlx_err`. Avoid `unwrap`, `expect`, and `panic!` on request paths. Add an `AppError` variant only when it needs a distinct HTTP status or stable client-facing code.
- **Logging**: use `tracing` spans and structured fields. No `println!` or `dbg!` in services.

## Adding a crate or module

1. Add the member and any `[workspace.dependencies]` entry in `backend/Cargo.toml`.
2. In the new `Cargo.toml`, inherit `version.workspace` and `edition.workspace`, and use `x.workspace = true`.
3. If the crate exposes HTTP, put `pub fn router<S: AppBackendState>() -> Router<S>` and an `ApiDoc` in `lib.rs`.
4. Wire the router and OpenAPI in `backend/bins/api-server/src/main.rs`.
5. Run fmt, clippy, and tests, then regenerate the API contract (see the `axum-endpoint` skill).

## References

- `docs/07-SYSTEM-ARCHITECTURE.md` — crate responsibilities and dependency rules.
- `docs/decisions/003-calculation-engine.md` — purity, explicit inputs, and formula versioning.
- `docs/09-REPOSITORY-STRUCTURE.md` — intended layout and CI checks.
- `data/` and `backend/migrations/` — canonical schema and data-dictionary names.
