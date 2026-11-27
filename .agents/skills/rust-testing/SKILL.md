---
name: Rust Testing
description: Use when adding or fixing Rust tests in the TernakMatric backend, including calculation unit tests, deterministic Decimal and time assertions, OpenAPI contract tests, and guidance for new database-backed integration tests.
---

# Rust Testing

## Commands

```powershell
# whole workspace
cargo test --manifest-path backend/Cargo.toml --workspace

# one crate
cargo test --manifest-path backend/Cargo.toml -p mod-calculation

# one test with output
cargo test --manifest-path backend/Cargo.toml -p mod-calculation test_population -- --nocapture

# lint as CI does
cargo clippy --manifest-path backend/Cargo.toml --workspace --all-targets -- -D warnings
```

## Current test layout

- Pure calculation tests: `backend/crates/mod-calculation/src/tests.rs`, declared with `#[cfg(test)] mod tests;` in `lib.rs`. Synchronous `#[test]`; no Tokio runtime needed.
- API contract test: `#[cfg(test)] mod tests` at the bottom of `backend/bins/api-server/src/main.rs`. It builds `ApiDoc::openapi()` and asserts that every module path is present.

There is no database integration test harness yet. Do not assume one exists.

## Writing calculation tests

Follow the existing style in `mod-calculation/src/tests.rs`:

- `use crate::*;`
- a `fixed_time()` helper using `Utc.with_ymd_and_hms(...).unwrap()` instead of the real clock,
- `rust_decimal_macros::dec!` for all decimal literals,
- exact `assert_eq!` when the result has a deterministic scale,
- tolerance comparisons for irrational results such as SGR:

  ```rust
  assert!(sgr.value > dec!(4.9) && sgr.value < dec!(5.1));
  ```

- error paths with `assert!(matches!(result, Err(CalculationError::InvalidInput { .. })))`.

Test business invariants, not implementation details:

- egg and milk collection must not change live headcount,
- partial harvest and mortality reduce population correctly,
- capacity boundaries fit or overflow as specified,
- biological and economic FCR differ and zero gain is rejected,
- formula and parameter versions are recorded in `CalculationResult`.

Never weaken or delete a failing assertion to make the suite pass. Fix the production code or raise the discrepancy.

## Extending the OpenAPI test

When adding a module or a top-level route, add its path assertion to `test_openapi_spec_builds_with_all_module_paths`. This is the cheapest guard against forgetting to wire a router or an `ApiDoc`.

## Database-backed tests

If a change needs real PostgreSQL behavior (RLS, triggers, constraints), prefer testing it explicitly. Before adding infrastructure, agree on the approach; the crate currently has no `sqlx` macro feature or test fixture.

A safe pattern with the current stack:

1. Start PostgreSQL with `docker compose up -d postgres` and run the migrations.
2. Read `DATABASE_URL` from the environment; skip the test with a clear message if it is absent.
3. Create a pool with `infra_db::create_pool`, then open every tenant operation through `OrganizationContext::begin_tx`, binding the organization explicitly.
4. Keep each test isolated: unique organization/user rows, or a transaction that rolls back.
5. Verify the negative case too: a query for another organization must return no rows or fail the RLS policy.

Do not introduce `#[sqlx::test]` or Testcontainers without updating the `sqlx` features (the `macros` feature is required for `#[sqlx::test]`) and documenting the setup. Do not write tests that depend on a shared persistent database state.

## Rules of thumb

- One behavior per test, named `test_<behavior>`.
- Deterministic inputs: fixed timestamps, fixed decimals, no sleeps, no network.
- Test both the success path and the guard that prevents corrupt data.
- `unwrap()` and `expect()` are acceptable in tests to surface failures; they are not acceptable in handlers.
- Run fmt and clippy after adding tests; warnings are denied.
