-- 2026112408_rls_triggers_hardening.sql
-- Cross-tenant guards, append-only ledger triggers, updated_at triggers, and tenant RLS.

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

CREATE TRIGGER trg_stock_reversal_tenant
  BEFORE INSERT OR UPDATE OF reversal_of, organization_id ON stock_movements
  FOR EACH ROW EXECUTE FUNCTION assert_global_or_tenant_ref('reversal_of', 'stock_movements');

CREATE TRIGGER trg_inventory_reversal_tenant
  BEFORE INSERT OR UPDATE OF reversal_of, organization_id ON inventory_transactions
  FOR EACH ROW EXECUTE FUNCTION assert_global_or_tenant_ref('reversal_of', 'inventory_transactions');

CREATE TRIGGER trg_stock_movements_immutable
  BEFORE UPDATE OR DELETE ON stock_movements
  FOR EACH ROW EXECUTE FUNCTION deny_ledger_mutation();

CREATE TRIGGER trg_inventory_lines_immutable
  BEFORE UPDATE OR DELETE ON inventory_transaction_lines
  FOR EACH ROW EXECUTE FUNCTION deny_ledger_mutation();

CREATE TRIGGER trg_audit_logs_immutable
  BEFORE UPDATE OR DELETE ON audit_logs
  FOR EACH ROW EXECUTE FUNCTION deny_ledger_mutation();

DO $$
DECLARE t text;
BEGIN
  FOREACH t IN ARRAY ARRAY[
    'users','organizations','subscriptions','farms','production_units',
    'production_batches','population_balances','individual_tags',
    'vaccination_schedules','inventory_balances'
  ]
  LOOP
    EXECUTE format(
      'CREATE TRIGGER %I_set_updated_at BEFORE UPDATE ON %I FOR EACH ROW EXECUTE FUNCTION set_updated_at()',
      t, t
    );
  END LOOP;
END $$;

DO $$
DECLARE t text;
BEGIN
  FOREACH t IN ARRAY ARRAY[
    'subscriptions','organization_members','member_roles','farms','production_units',
    'species_production_profiles','growth_stages','feeding_guidelines',
    'production_batches','batch_unit_assignments','stock_movements',
    'population_balances','individual_tags','mortality_records','growth_samples',
    'environment_readings','health_records','vaccination_schedules','biosecurity_logs',
    'warehouses','feed_products','feed_unit_conversions',
    'inventory_transactions','inventory_transaction_lines','inventory_balances',
    'feeding_records','production_outputs','expenses','sales','sale_items',
    'calculation_runs','audit_logs','devices','sync_operations',
    'sync_conflicts','outbox_events'
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
