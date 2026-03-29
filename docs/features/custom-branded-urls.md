# Custom Branded URLs for Customers

**Status:** Future Plan

## Architecture: Dual-Domain Model

Two separate domains, one server:

| Domain | Purpose | Routing |
|--------|---------|---------|
| **mustcato.com** | Platform — marketing site, dashboard, API, auth | Standard routes |
| **\*.mustcato.app** | Customer published sites via subdomain | Host-based lookup → workspace + site |

Example:
```
mustcato.com              → platform UI, API, auth
mustcato.com/dashboard    → workspace management
acme.mustcato.app         → customer "acme" published site
hello.mustcato.app        → customer "hello" published site
app.acme.com (CNAME)      → custom domain for customer "acme"
```

Both domains resolve to the same server. The Axum middleware inspects the `Host` header to decide which routing path to use.

## URL Strategies (progressive)

### 1. Path-based (simplest, implement first)
```
mustcato.com/sites/acme/app
mustcato.com/sites/globex/app
```
- Route: `/sites/{customer_slug}/{*path}`
- Handler looks up branding config by slug
- No DNS changes needed

### 2. Subdomain-based (dual-domain)
```
acme.mustcato.app/app
globex.mustcato.app/app
```
- Separate domain with wildcard DNS: `*.mustcato.app A <server-ip>`
- Resolve tenant from `Host` header subdomain
- Platform domain stays clean — no customer routes on `mustcato.com`

### 3. Custom domains (most professional)
```
app.acme.com → server → lookup branding by domain
```
- Customer adds CNAME: `app.acme.com CNAME mustcato.app`
- Server resolves tenant from `Host` header via `custom_domains` table
- On-demand TLS auto-provisions certs

## Database Schema

```sql
CREATE TABLE site_domains (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    workspace_id TEXT NOT NULL,
    site_slug TEXT NOT NULL,
    subdomain TEXT UNIQUE,           -- e.g. "acme" → acme.mustcato.app
    custom_domain TEXT UNIQUE,       -- e.g. "app.acme.com"
    active BOOLEAN NOT NULL DEFAULT true,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(workspace_id, site_slug)
);
```

## Axum Middleware: Host-Based Routing

```rust
// Pseudocode for the routing middleware
async fn resolve_site(host: &str) -> Option<(String, String)> {
    let platform_domain = "mustcato.com";
    let app_domain = "mustcato.app";

    if host == platform_domain || host.ends_with(&format!(".{}", platform_domain)) {
        return None; // platform routes, no site resolution
    }

    // Check subdomain: acme.mustcato.app → subdomain = "acme"
    if let Some(sub) = host.strip_suffix(&format!(".{}", app_domain)) {
        return db::lookup_by_subdomain(sub).await;
    }

    // Check custom domain: app.acme.com
    db::lookup_by_custom_domain(host).await
}
```

The middleware runs early, injects `SiteContext { workspace_id, site_slug, branding }` into request extensions. Downstream handlers serve the site's generated static files from the existing workspace storage path.

## Caddy 2 Setup

Caddy handles wildcard certs and on-demand TLS natively.

### Wildcard certificate (subdomains)

Requires DNS challenge — needs a DNS provider plugin (e.g., Cloudflare, Route53).

```
*.mustcato.app {
    tls {
        dns cloudflare {env.CF_API_TOKEN}
    }
    reverse_proxy localhost:3000
}
```

### Platform domain

```
mustcato.com {
    reverse_proxy localhost:3000
}
```

### On-demand TLS (custom domains)

When a request arrives for an unknown custom domain, Caddy calls the validation endpoint. If approved, it auto-provisions a Let's Encrypt cert.

```
{
    on_demand_tls {
        ask http://localhost:3000/api/check-domain
    }
}

:443 {
    tls {
        on_demand
    }
    reverse_proxy localhost:3000
}
```

The `/api/check-domain?domain=app.acme.com` endpoint returns 200 if the domain exists and is active in `site_domains`, 404 otherwise.

### Combined Caddyfile

```
mustcato.com {
    reverse_proxy localhost:3000
}

*.mustcato.app {
    tls {
        dns cloudflare {env.CF_API_TOKEN}
    }
    reverse_proxy localhost:3000
}

:443 {
    tls {
        on_demand
    }
    reverse_proxy localhost:3000
}
```

### DNS

- Platform: `mustcato.com A <server-ip>`
- Wildcard: `*.mustcato.app A <server-ip>`
- Custom domains: customer adds `CNAME app.acme.com mustcato.app`

## Static File Serving

Customer sites are already generated as static files under:
```
storage/workspaces/{workspace_id}/websites/{site_slug}/dist/
```

The site resolution middleware maps `Host` → `(workspace_id, site_slug)` → serves files from the corresponding `dist/` directory. No separate web server per customer — just filesystem path routing.

## Implementation Steps

1. Add `site_domains` table (migration)
2. Create `Host` header middleware to resolve site context
3. Add static file serving handler for resolved sites (reads from `dist/`)
4. Add `/api/check-domain` endpoint for Caddy on-demand TLS validation
5. Add domain management UI in workspace dashboard (claim subdomain, configure custom domain)
6. Inject `Branding` struct into request extensions for per-tenant styling
7. Configure Caddy with dual-domain + on-demand TLS
8. Start with subdomain routing on `*.mustcato.app`, add custom domains later
