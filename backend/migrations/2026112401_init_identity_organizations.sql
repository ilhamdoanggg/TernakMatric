-- 2026112401_init_identity_organizations.sql
-- Base trigger/context functions, SaaS identity, multi-tenant organizations, plans, and RBAC.

CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE OR REPLACE FUNCTION set_updated_at()
RETURNS trigger LANGUAGE plpgsql
SET search_path = pg_catalog, public
AS $$
BEGIN
  NEW.updated_at = now();
  RETURN NEW;
END;
$$;

CREATE OR REPLACE FUNCTION current_organization_id()
RETURNS uuid LANGUAGE sql STABLE
SET search_path = pg_catalog, public
AS $$
  SELECT nullif(current_setting('app.current_organization_id', true), '')::uuid;
$$;

CREATE OR REPLACE FUNCTION current_user_id()
RETURNS uuid LANGUAGE sql STABLE
SET search_path = pg_catalog, public
AS $$
  SELECT nullif(current_setting('app.current_user_id', true), '')::uuid;
$$;

CREATE OR REPLACE FUNCTION set_created_by_from_context()
RETURNS trigger LANGUAGE plpgsql
SET search_path = pg_catalog, public
AS $$
DECLARE
  v_uid uuid;
BEGIN
  v_uid := current_user_id();
  IF v_uid IS NOT NULL THEN
    NEW.created_by = v_uid;
  END IF;
  RETURN NEW;
END;
$$;

CREATE OR REPLACE FUNCTION set_updated_audit()
RETURNS trigger LANGUAGE plpgsql
SET search_path = pg_catalog, public
AS $$
BEGIN
  NEW.updated_at = now();
  NEW.updated_by = current_user_id();
  RETURN NEW;
END;
$$;

CREATE OR REPLACE FUNCTION assert_global_or_tenant_ref()
RETURNS trigger LANGUAGE plpgsql
SET search_path = pg_catalog, public
AS $$
DECLARE
  v_ref_id uuid;
  v_parent_org uuid;
  v_current_org uuid;
BEGIN
  IF TG_NARGS <> 2 THEN
    RAISE EXCEPTION 'assert_global_or_tenant_ref requires child column and parent table arguments';
  END IF;

  v_ref_id := (to_jsonb(NEW) ->> TG_ARGV[0])::uuid;
  IF v_ref_id IS NULL THEN
    RETURN NEW;
  END IF;

  v_current_org := current_organization_id();
  IF v_current_org IS NULL THEN
    RAISE EXCEPTION 'app.current_organization_id must be set for tenant references';
  END IF;

  IF v_current_org IS DISTINCT FROM NEW.organization_id THEN
    RAISE EXCEPTION 'tenant context mismatch on %.%: session org % <> row org %',
      TG_TABLE_NAME, TG_ARGV[0], v_current_org, NEW.organization_id
      USING ERRCODE = 'insufficient_privilege';
  END IF;

  EXECUTE format('SELECT organization_id FROM %I WHERE id = $1', TG_ARGV[1])
    INTO v_parent_org
    USING v_ref_id;

  IF NOT FOUND THEN
    RAISE EXCEPTION 'invalid or cross-tenant reference on %.% -> %.id (%)',
      TG_TABLE_NAME, TG_ARGV[0], TG_ARGV[1], v_ref_id
      USING ERRCODE = 'foreign_key_violation';
  END IF;

  IF v_parent_org IS NOT NULL AND v_parent_org <> NEW.organization_id THEN
    RAISE EXCEPTION 'cross-tenant reference on %.% -> %.id (% <> %)',
      TG_TABLE_NAME, TG_ARGV[0], TG_ARGV[1], v_parent_org, NEW.organization_id
      USING ERRCODE = 'foreign_key_violation';
  END IF;

  RETURN NEW;
END;
$$;

CREATE OR REPLACE FUNCTION deny_ledger_mutation()
RETURNS trigger LANGUAGE plpgsql
SET search_path = pg_catalog, public
AS $$
BEGIN
  RAISE EXCEPTION '% is append-only; use a reversal transaction', TG_TABLE_NAME;
END;
$$;

CREATE OR REPLACE FUNCTION deny_posted_inventory_tx_mutation()
RETURNS trigger LANGUAGE plpgsql
SET search_path = pg_catalog, public
AS $$
BEGIN
  IF TG_OP = 'DELETE' THEN
    RAISE EXCEPTION 'inventory_transactions is append-only; use a reversal transaction';
  END IF;

  IF OLD.status <> 'draft' THEN
    RAISE EXCEPTION 'posted or voided inventory_transactions are append-only; use a reversal transaction';
  END IF;

  IF NEW.id IS DISTINCT FROM OLD.id
     OR NEW.organization_id IS DISTINCT FROM OLD.organization_id
     OR NEW.warehouse_id IS DISTINCT FROM OLD.warehouse_id
     OR NEW.transaction_type IS DISTINCT FROM OLD.transaction_type
     OR NEW.idempotency_key IS DISTINCT FROM OLD.idempotency_key
     OR NEW.reversal_of IS DISTINCT FROM OLD.reversal_of
     OR NEW.created_by IS DISTINCT FROM OLD.created_by
     OR NEW.created_at IS DISTINCT FROM OLD.created_at THEN
    RAISE EXCEPTION 'immutable header columns on inventory_transactions cannot be modified';
  END IF;

  RETURN NEW;
END;
$$;

CREATE TABLE users (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  email varchar(320) NOT NULL UNIQUE,
  display_name varchar(160) NOT NULL,
  password_hash text,
  status text NOT NULL DEFAULT 'active' CHECK (status IN ('active','invited','suspended','deleted')),
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now(),
  updated_by uuid REFERENCES users(id)
);

CREATE TABLE organizations (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  name varchar(180) NOT NULL,
  slug varchar(100) NOT NULL UNIQUE,
  timezone varchar(64) NOT NULL DEFAULT 'Asia/Jakarta',
  default_currency char(3) NOT NULL DEFAULT 'IDR',
  status text NOT NULL DEFAULT 'active' CHECK (status IN ('active','suspended','closed')),
  created_at timestamptz NOT NULL DEFAULT now(),
  created_by uuid REFERENCES users(id),
  updated_at timestamptz NOT NULL DEFAULT now(),
  updated_by uuid REFERENCES users(id)
);

CREATE TABLE plans (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  code varchar(50) NOT NULL UNIQUE,
  name varchar(100) NOT NULL,
  limits jsonb NOT NULL DEFAULT '{}'::jsonb,
  features jsonb NOT NULL DEFAULT '{}'::jsonb,
  is_active boolean NOT NULL DEFAULT true,
  created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE subscriptions (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  organization_id uuid NOT NULL REFERENCES organizations(id),
  plan_id uuid NOT NULL REFERENCES plans(id),
  status text NOT NULL CHECK (status IN ('trialing','active','past_due','cancelled','expired')),
  starts_at timestamptz NOT NULL,
  ends_at timestamptz,
  external_customer_ref text,
  external_subscription_ref text,
  created_at timestamptz NOT NULL DEFAULT now(),
  created_by uuid REFERENCES users(id),
  updated_at timestamptz NOT NULL DEFAULT now(),
  updated_by uuid REFERENCES users(id),
  UNIQUE (id, organization_id),
  CHECK (ends_at IS NULL OR ends_at > starts_at)
);

CREATE INDEX idx_subscriptions_org_status
  ON subscriptions(organization_id, status, ends_at);

CREATE TABLE roles (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  organization_id uuid REFERENCES organizations(id),
  code varchar(60) NOT NULL,
  name varchar(100) NOT NULL,
  is_system boolean NOT NULL DEFAULT false,
  created_at timestamptz NOT NULL DEFAULT now(),
  created_by uuid REFERENCES users(id),
  updated_at timestamptz NOT NULL DEFAULT now(),
  updated_by uuid REFERENCES users(id),
  UNIQUE NULLS NOT DISTINCT (organization_id, code),
  UNIQUE (id, organization_id),
  CHECK ((is_system AND organization_id IS NULL) OR (NOT is_system AND organization_id IS NOT NULL))
);

CREATE TABLE permissions (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  code varchar(100) NOT NULL UNIQUE,
  description text
);

CREATE TABLE role_permissions (
  role_id uuid NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
  permission_id uuid NOT NULL REFERENCES permissions(id) ON DELETE CASCADE,
  PRIMARY KEY (role_id, permission_id)
);

CREATE TABLE organization_members (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  organization_id uuid NOT NULL REFERENCES organizations(id),
  user_id uuid NOT NULL REFERENCES users(id),
  status text NOT NULL DEFAULT 'active' CHECK (status IN ('invited','active','suspended','removed')),
  invited_by uuid REFERENCES users(id),
  joined_at timestamptz,
  created_at timestamptz NOT NULL DEFAULT now(),
  created_by uuid REFERENCES users(id),
  updated_at timestamptz NOT NULL DEFAULT now(),
  updated_by uuid REFERENCES users(id),
  UNIQUE (organization_id, user_id),
  UNIQUE (id, organization_id)
);

CREATE TABLE member_roles (
  organization_id uuid NOT NULL REFERENCES organizations(id),
  member_id uuid NOT NULL,
  role_id uuid NOT NULL REFERENCES roles(id),
  created_at timestamptz NOT NULL DEFAULT now(),
  created_by uuid REFERENCES users(id),
  PRIMARY KEY (member_id, role_id),
  FOREIGN KEY (member_id, organization_id)
    REFERENCES organization_members(id, organization_id) ON DELETE CASCADE
);

CREATE INDEX idx_member_roles_org_member
  ON member_roles(organization_id, member_id);
