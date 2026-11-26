-- 2026112409_rls_triggers_hardening.sql
-- Cross-tenant reference guards, append-only ledger guards, audit actor/timestamp triggers,
-- and PostgreSQL Row-Level Security policies across all tenant and hybrid catalog tables.

-- 1. Cross-tenant guards for global-or-tenant parent references
CREATE TRIGGER trg_member_roles_tenant
  BEFORE INSERT OR UPDATE OF role_id, organization_id ON member_roles
  FOR EACH ROW EXECUTE FUNCTION assert_global_or_tenant_ref('role_id', 'roles');

CREATE TRIGGER trg_species_profile_tenant
  BEFORE INSERT OR UPDATE OF species_id, organization_id ON species_production_profiles
  FOR EACH ROW EXECUTE FUNCTION assert_global_or_tenant_ref('species_id', 'species');

CREATE TRIGGER trg_batch_species_tenant
  BEFORE INSERT OR UPDATE OF species_id, organization_id ON production_batches
  FOR EACH ROW EXECUTE FUNCTION assert_global_or_tenant_ref('species_id', 'species');

CREATE TRIGGER trg_expense_category_tenant
  BEFORE INSERT OR UPDATE OF category_id, organization_id ON expenses
  FOR EACH ROW EXECUTE FUNCTION assert_global_or_tenant_ref('category_id', 'expense_categories');

-- 2. Append-only ledger immutability triggers
CREATE TRIGGER trg_stock_movements_immutable
  BEFORE UPDATE OR DELETE ON stock_movements
  FOR EACH ROW EXECUTE FUNCTION deny_ledger_mutation();

CREATE TRIGGER trg_inventory_lines_immutable
  BEFORE UPDATE OR DELETE ON inventory_transaction_lines
  FOR EACH ROW EXECUTE FUNCTION deny_ledger_mutation();

CREATE TRIGGER trg_production_outputs_immutable
  BEFORE UPDATE OR DELETE ON production_outputs
  FOR EACH ROW EXECUTE FUNCTION deny_ledger_mutation();

CREATE TRIGGER trg_audit_logs_immutable
  BEFORE UPDATE OR DELETE ON audit_logs
  FOR EACH ROW EXECUTE FUNCTION deny_ledger_mutation();

CREATE TRIGGER trg_inventory_transactions_immutable
  BEFORE UPDATE OR DELETE ON inventory_transactions
  FOR EACH ROW EXECUTE FUNCTION deny_posted_inventory_tx_mutation();

-- 3. Automatic created_by triggers (from SET LOCAL app.current_user_id)
DO $$
DECLARE t text;
BEGIN
  FOREACH t IN ARRAY ARRAY[
    'organizations', 'organization_members', 'roles', 'member_roles', 'subscriptions',
    'farms', 'production_units', 'species', 'species_production_profiles',
    'growth_stages', 'feeding_guidelines', 'production_batches',
    'batch_unit_assignments', 'stock_movements', 'individual_tags',
    'growth_samples', 'production_outputs', 'warehouses', 'feed_products',
    'feed_unit_conversions', 'inventory_transactions', 'inventory_transaction_lines',
    'feeding_records', 'environment_readings', 'health_records',
    'vaccination_schedules', 'mortality_records', 'biosecurity_logs',
    'expense_categories', 'expenses', 'sales', 'sale_items',
    'calculation_runs', 'devices'
  ]
  LOOP
    EXECUTE format(
      'CREATE TRIGGER %I_set_created_by BEFORE INSERT ON %I FOR EACH ROW EXECUTE FUNCTION set_created_by_from_context()',
      t, t
    );
  END LOOP;
END $$;

-- 4. Automatic updated_at + updated_by triggers (clears updated_by when worker runs without user context)
DO $$
DECLARE t text;
BEGIN
  FOREACH t IN ARRAY ARRAY[
    'users', 'organizations', 'organization_members', 'roles', 'subscriptions',
    'farms', 'production_units', 'species', 'species_production_profiles',
    'growth_stages', 'feeding_guidelines', 'production_batches',
    'batch_unit_assignments', 'population_balances', 'individual_tags',
    'growth_samples', 'production_outputs', 'warehouses', 'feed_products',
    'feed_unit_conversions', 'inventory_transactions', 'inventory_balances',
    'feeding_records', 'environment_readings', 'health_records',
    'vaccination_schedules', 'mortality_records', 'biosecurity_logs',
    'expense_categories', 'expenses', 'sales', 'sale_items',
    'devices', 'sync_operations', 'sync_conflicts'
  ]
  LOOP
    EXECUTE format(
      'CREATE TRIGGER %I_set_updated_audit BEFORE UPDATE ON %I FOR EACH ROW EXECUTE FUNCTION set_updated_audit()',
      t, t
    );
  END LOOP;
END $$;

CREATE TRIGGER outbox_events_set_updated_at
  BEFORE UPDATE ON outbox_events
  FOR EACH ROW EXECUTE FUNCTION set_updated_at();

-- 5. Strict Tenant RLS (ENABLE + FORCE) across all tenant-owned tables
DO $$
DECLARE t text;
BEGIN
  FOREACH t IN ARRAY ARRAY[
    'subscriptions', 'member_roles', 'farms', 'production_units',
    'species_production_profiles', 'growth_stages', 'feeding_guidelines',
    'production_batches', 'batch_unit_assignments', 'stock_movements',
    'population_balances', 'individual_tags', 'mortality_records', 'growth_samples',
    'environment_readings', 'health_records', 'vaccination_schedules', 'biosecurity_logs',
    'warehouses', 'feed_products', 'feed_unit_conversions',
    'inventory_transactions', 'inventory_transaction_lines', 'inventory_balances',
    'feeding_records', 'production_outputs', 'expenses', 'sales', 'sale_items',
    'calculation_runs', 'audit_logs', 'devices', 'sync_operations',
    'sync_conflicts', 'outbox_events'
  ]
  LOOP
    EXECUTE format('ALTER TABLE %I ENABLE ROW LEVEL SECURITY', t);
    EXECUTE format('ALTER TABLE %I FORCE ROW LEVEL SECURITY', t);
    EXECUTE format(
      'CREATE POLICY tenant_isolation ON %I USING (organization_id = current_organization_id()) WITH CHECK (organization_id = current_organization_id())',
      t
    );
  END LOOP;
END $$;

-- organization_members permits self-scoped SELECT when no organization context is active
-- (e.g., GET /api/v1/organizations), while mutations remain strictly tenant-scoped.
ALTER TABLE organization_members ENABLE ROW LEVEL SECURITY;
ALTER TABLE organization_members FORCE ROW LEVEL SECURITY;
CREATE POLICY organization_members_select ON organization_members
  FOR SELECT USING (
    organization_id = current_organization_id()
    OR (current_organization_id() IS NULL AND user_id = current_user_id())
  );
CREATE POLICY organization_members_insert ON organization_members
  FOR INSERT WITH CHECK (organization_id = current_organization_id());
CREATE POLICY organization_members_update ON organization_members
  FOR UPDATE
  USING (organization_id = current_organization_id())
  WITH CHECK (organization_id = current_organization_id());
CREATE POLICY organization_members_delete ON organization_members
  FOR DELETE USING (organization_id = current_organization_id());

-- 6. Global/tenant hybrid catalogs: global rows readable by all, writable only by owning tenant
ALTER TABLE species ENABLE ROW LEVEL SECURITY;
ALTER TABLE species FORCE ROW LEVEL SECURITY;
CREATE POLICY species_read ON species FOR SELECT USING (
  organization_id IS NULL OR organization_id = current_organization_id()
);
CREATE POLICY species_write ON species FOR ALL USING (
  organization_id = current_organization_id()
) WITH CHECK (
  organization_id = current_organization_id()
);

ALTER TABLE roles ENABLE ROW LEVEL SECURITY;
ALTER TABLE roles FORCE ROW LEVEL SECURITY;
CREATE POLICY roles_read ON roles FOR SELECT USING (
  organization_id IS NULL OR organization_id = current_organization_id()
);
CREATE POLICY roles_write ON roles FOR ALL USING (
  organization_id = current_organization_id()
) WITH CHECK (
  organization_id = current_organization_id()
);

ALTER TABLE expense_categories ENABLE ROW LEVEL SECURITY;
ALTER TABLE expense_categories FORCE ROW LEVEL SECURITY;
CREATE POLICY expense_categories_read ON expense_categories FOR SELECT USING (
  organization_id IS NULL OR organization_id = current_organization_id()
);
CREATE POLICY expense_categories_write ON expense_categories FOR ALL USING (
  organization_id = current_organization_id()
) WITH CHECK (
  organization_id = current_organization_id()
);

ALTER TABLE role_permissions ENABLE ROW LEVEL SECURITY;
ALTER TABLE role_permissions FORCE ROW LEVEL SECURITY;
CREATE POLICY role_permissions_read ON role_permissions FOR SELECT USING (
  EXISTS (
    SELECT 1 FROM roles r
    WHERE r.id = role_permissions.role_id
      AND (r.organization_id IS NULL OR r.organization_id = current_organization_id())
  )
);
CREATE POLICY role_permissions_write ON role_permissions FOR ALL USING (
  EXISTS (
    SELECT 1 FROM roles r
    WHERE r.id = role_permissions.role_id
      AND r.organization_id = current_organization_id()
  )
) WITH CHECK (
  EXISTS (
    SELECT 1 FROM roles r
    WHERE r.id = role_permissions.role_id
      AND r.organization_id = current_organization_id()
  )
);
