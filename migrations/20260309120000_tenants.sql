-- Phase 6B: Tenant scoping for hosted B2B (Tier 2)
--
-- Each company using the hosted platform is a tenant.
-- Existing workspaces are assigned to the 'platform' tenant (Tier 1 / your own use).
-- Standalone deployments (Tier 3) skip tenant resolution entirely (deployment_mode: standalone).

CREATE TABLE tenants (
    id          TEXT PRIMARY KEY,                   -- 'platform', 'acme', 'pharma-co', ...
    name        TEXT NOT NULL,                      -- Display name
    branding    TEXT,                               -- JSON: { name, logo, favicon, primary_color, support_email }
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Insert the platform tenant representing Tier 1 (your own hosted use)
INSERT INTO tenants (id, name) VALUES ('platform', 'Platform');

-- Add tenant_id to workspaces
ALTER TABLE workspaces ADD COLUMN tenant_id TEXT REFERENCES tenants(id);

-- Backfill: all existing workspaces belong to the platform tenant
UPDATE workspaces SET tenant_id = 'platform';

-- Index for per-tenant workspace queries
CREATE INDEX idx_workspaces_tenant_id ON workspaces(tenant_id);

-- Add tenant_id to users (drives session resolution)
ALTER TABLE users ADD COLUMN tenant_id TEXT REFERENCES tenants(id) DEFAULT 'platform';

-- Backfill: all existing users belong to the platform tenant
UPDATE users SET tenant_id = 'platform';

CREATE INDEX idx_users_tenant_id ON users(tenant_id);
