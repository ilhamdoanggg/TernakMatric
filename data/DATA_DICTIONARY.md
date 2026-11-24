# TernakMatric Data Dictionary (table inventory)

The SQL migration is the authoritative source for column types and constraints. This document explains table purpose and ownership.

## Identity and SaaS
| Table | Scope | Purpose |
|---|---|---|
| `users` | Global | Login identity and profile |
| `organizations` | Global | Tenant boundary |
| `organization_members` | Tenant | Membership and invitation status |
| `roles` | Global/tenant | System or custom role definitions |
| `permissions` | Global | Permission catalog |
| `role_permissions` | Global/tenant | Role-to-permission mapping |
| `member_roles` | Tenant | Roles assigned to organization member |
| `plans` | Global | SaaS plan definitions and limits |
| `subscriptions` | Tenant | Organization subscription lifecycle |

## Farm, production unit, and species
| Table | Scope | Purpose |
|---|---|---|
| `farms` | Tenant | Physical farm/site |
| `unit_types` | Global | Generic unit catalog: pond, coop, pen, cage, pasture, hive, etc. |
| `production_systems` | Global | Generic production system catalog |
| `production_units` | Tenant | Physical growing/housing unit with capacity and optional dimensions |
| `species` | Global/tenant | Built-in and custom species with category |
| `species_production_profiles` | Tenant | Versioned species/system parameters in `parameters jsonb` |
| `growth_stages` | Tenant | Weight/age bands per production profile |
| `feeding_guidelines` | Tenant | Feeding recommendations per growth stage |

## Production
| Table | Scope | Purpose |
|---|---|---|
| `production_batches` | Tenant | One production cycle for any species |
| `batch_unit_assignments` | Tenant | Time-bounded batch occupancy in production units |
| `stock_movements` | Tenant | Append-oriented animal population ledger |
| `population_balances` | Tenant | Transaction-maintained population projection |
| `mortality_records` | Tenant | Detailed mortality/cull event notes |
| `growth_samples` | Tenant | Sample count, weight, and optional length |
| `environment_metrics` | Global | Metric catalog: water, air, soil, housing, feed |
| `environment_readings` | Tenant | Manual or IoT measurements with explicit unit |
| `health_records` | Tenant | Observation, suspected disease, and treatment |
| `feeding_records` | Tenant | Actual feed given to a batch |

## Inventory
| Table | Scope | Purpose |
|---|---|---|
| `warehouses` | Tenant | Feed/storage locations |
| `feed_products` | Tenant | Feed SKU and product attributes |
| `feed_unit_conversions` | Tenant | Conversion factors to canonical units |
| `inventory_transactions` | Tenant | Receipt, issue, transfer, adjustment header |
| `inventory_transaction_lines` | Tenant | Product quantities, units, costs, lots, batch reference |
| `inventory_balances` | Tenant | Transaction-maintained inventory projection |

## Output and finance
| Table | Scope | Purpose |
|---|---|---|
| `production_outputs` | Tenant | Meat, egg, milk, fry, cull, offspring, and other outputs |
| `expense_categories` | Global/tenant | Expense category catalog |
| `expenses` | Tenant | Cost entries |
| `sales` | Tenant | Sale header |
| `sale_items` | Tenant | Sold output quantities, units, and prices |
| `calculation_versions` | Global | Formula version and definition |
| `calculation_runs` | Tenant | Input/result snapshots for reproducibility |
| `audit_logs` | Global/tenant | Actor, action, entity, and before/after snapshots |

## Offline synchronization
| Table | Scope | Purpose |
|---|---|---|
| `devices` | Tenant | Registered web/mobile device |
| `sync_operations` | Tenant | Idempotent client mutation queue and processing result |
| `sync_conflicts` | Tenant | Stale-write conflicts and resolution history |
| `outbox_events` | Tenant | Change feed for reliable sync responses and notifications |

## Important field semantics
- Quantities carry explicit units: `quantity`, `unit`, `capacity_value`, `capacity_unit`, `metric_value`, `unit`.
- Species-specific husbandry values belong in `species_production_profiles.parameters`, not in generic columns.
- Store event times as `timestamptz`; store business dates such as `start_date` as `date`.
- `organization_id` is a security boundary, not merely a reporting column.
- `base_version` in sync operations is the entity version last observed by the client; the server must compare it with the current server version.
- `client_operation_id` must be generated on the device and remain stable across retries.
- `input_snapshot` and `result_snapshot` preserve the exact inputs and outputs of a calculation.
- Inventory, population, and output ledgers are corrected by reversal/adjustment events, never destructive edits.
- `client_wins` conflict resolution is forbidden for stock, inventory, output, and finance entities.
