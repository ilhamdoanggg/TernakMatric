-- 2026112410_system_log_archives.sql
-- Backend daily rolling log archives, custom file paths, and database metadata tracking.

CREATE TABLE system_log_archives (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id uuid REFERENCES organizations(id) ON DELETE CASCADE,
    log_date date NOT NULL,
    service_name text NOT NULL DEFAULT 'api-server',
    file_name text NOT NULL,
    file_path text NOT NULL,
    file_size_bytes bigint NOT NULL DEFAULT 0,
    line_count bigint NOT NULL DEFAULT 0,
    status text NOT NULL DEFAULT 'active',
    checksum_sha256 text,
    archived_at timestamptz NOT NULL DEFAULT now(),
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (id, organization_id)
);

CREATE UNIQUE INDEX idx_system_log_archives_date_service_file
    ON system_log_archives (log_date, service_name, file_name);

CREATE INDEX idx_system_log_archives_org_date
    ON system_log_archives (organization_id, log_date DESC);

ALTER TABLE system_log_archives ENABLE ROW LEVEL SECURITY;
ALTER TABLE system_log_archives FORCE ROW LEVEL SECURITY;

CREATE POLICY system_log_archives_tenant_read ON system_log_archives
    FOR SELECT USING (
        organization_id IS NULL OR organization_id = current_organization_id()
    );

CREATE POLICY system_log_archives_tenant_write ON system_log_archives
    FOR ALL USING (
        organization_id IS NULL OR organization_id = current_organization_id()
    )
    WITH CHECK (
        organization_id IS NULL OR organization_id = current_organization_id()
    );

CREATE TRIGGER system_log_archives_set_updated_at
    BEFORE UPDATE ON system_log_archives
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();
