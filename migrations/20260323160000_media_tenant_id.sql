-- Add tenant_id to media_items for multi-tenant isolation.
-- All existing rows default to 'platform' (the root/default tenant).

ALTER TABLE media_items ADD COLUMN tenant_id TEXT NOT NULL DEFAULT 'platform';

-- Indexes for tenant-scoped queries
CREATE INDEX idx_media_items_tenant_status ON media_items(tenant_id, status);
CREATE INDEX idx_media_items_tenant_user ON media_items(tenant_id, user_id);
