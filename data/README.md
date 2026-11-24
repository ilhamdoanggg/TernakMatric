# TernakMatric Database Design — v2.0 generic livestock draft

## Files
- `001_ternakmatric_schema.sql`: PostgreSQL baseline schema, constraints, indexes, triggers, and starter RLS policies.
- `002_ternakmatric_seed.sql`: Global reference data, species catalog, and formula metadata.
- `DATA_DICTIONARY.md`: Table inventory and domain notes.
- `ERD.md`: Relationship overview and design decisions.

## Apply
1. Provision PostgreSQL 16+.
2. Review the schema with your database/security engineer.
3. Apply `001_ternakmatric_schema.sql` using a migration role.
4. Apply `002_ternakmatric_seed.sql`.
5. Create a non-owner application role. Do not run the application as a superuser or table owner.
6. Grant only required table/sequence/function permissions.
7. Set `app.current_organization_id` using `SET LOCAL` inside every tenant-scoped transaction.

Example transaction:
```sql
BEGIN;
SET LOCAL app.current_organization_id = '00000000-0000-0000-0000-000000000001';
-- tenant-scoped queries here
COMMIT;
```

## Important implementation notes
- This is a reviewable baseline, not a production-certified migration.
- The model covers fish, crustaceans, mollusks, poultry, pigs, goats/sheep, cattle/buffalo, rabbits, reptiles, insects, and tenant-defined species. Do not add species-specific columns to generic tables.
- RLS is defense in depth. Backend authorization and tenant membership checks remain mandatory.
- PostgreSQL owners, superusers, and roles with BYPASSRLS can bypass policies. Use a dedicated least-privilege runtime role.
- `species`, `unit_types`, `production_systems`, `environment_metrics`, `roles`, `expense_categories`, and `calculation_versions` have global rows (`organization_id IS NULL`). Application write permissions must be restricted accordingly.
- Multiple active batches per production unit are supported. Capacity validation requires an application/domain transaction that locks the unit row and calculates combined occupancy/biomass using the unit's `capacity_value` and `capacity_unit`.
- `species_production_profiles.parameters` is versioned species/strain/system-specific data. Universal husbandry values must not be hardcoded or seeded as defaults.
- Stock and inventory ledgers should be append-only after posting. Corrections should be compensating transactions, not destructive edits.
- Offline sync requires client-generated UUIDs, idempotency keys, base versions, conflict resolution, and server-side validation. Do not blindly apply stale updates.
- Inventory quantity must be normalized to a canonical unit per product (recommended kg for feed). Transfers require paired issue/receipt ledger effects in one transaction.
- A production batch has one current unit assignment at a time. Transfers close the old assignment and open a new one in a single transaction.
- `production_outputs` is the generic output ledger: meat, eggs, milk, fry, cull, offspring, or other outputs. It must be written in the same transaction as its `stock_movements` event.
- `environment_readings.metric_id` points to a global metric catalog; `metric_value` and `unit` are explicit so the same table can store water, air, soil, housing, and feed measurements.
- Review composite tenant foreign keys, RLS policies, uniqueness rules, and global reference-table grants before production.
- Add automated migration tests, tenant-isolation tests, ledger reconciliation tests, cross-species calculation tests, and offline replay tests.

## Known items for the next migration iteration
1. Add explicit roles, grants, revokes, and membership-based policies for `users` and `organizations`.
2. Add posting workflow and immutable-ledger triggers for posted inventory headers and production outputs.
3. Add transaction-safe population/inventory balance projection maintenance routines with row locking.
4. Add unit transfer transaction and capacity reservation model.
5. Add attachment storage metadata and access policy.
6. Add plan/subscription lifecycle and usage metering.
7. Add richer accounting model if formal accounting is required.
8. Add tenant-facing UI for managing custom species and custom production profiles.
