-- Add tenant_id to federation tables for multi-tenant isolation.
-- All existing rows default to 'platform' (the root/default tenant).

ALTER TABLE federation_peers ADD COLUMN tenant_id TEXT NOT NULL DEFAULT 'platform';
ALTER TABLE remote_media_cache ADD COLUMN tenant_id TEXT NOT NULL DEFAULT 'platform';

-- Indexes for tenant-scoped queries
CREATE INDEX idx_federation_peers_tenant ON federation_peers(tenant_id);
CREATE INDEX idx_remote_media_cache_tenant ON remote_media_cache(tenant_id, origin_server);
