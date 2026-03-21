-- Add folder-scoped access codes
-- When vault_id is set, the code grants access to all media_items in that vault.
-- When NULL, the code uses access_code_permissions rows (per-item, existing behaviour).
ALTER TABLE access_codes ADD COLUMN vault_id TEXT;

CREATE INDEX IF NOT EXISTS idx_access_codes_vault_id ON access_codes(vault_id);
