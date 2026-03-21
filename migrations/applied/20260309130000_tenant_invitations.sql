-- Tenant invitations for B2B onboarding
-- When a new user logs in via OIDC for the first time and their email matches
-- an invitation, they are automatically assigned to that tenant.

CREATE TABLE tenant_invitations (
    email      TEXT NOT NULL,
    tenant_id  TEXT NOT NULL REFERENCES tenants(id),
    invited_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (email, tenant_id)
);

CREATE INDEX idx_tenant_invitations_email ON tenant_invitations(email);
