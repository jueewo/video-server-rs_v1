# Rate Limit Customization for Workspaces

## Current Behavior

Workspace routes currently use the `api_mutate` rate limit class:
- **Default:** 30 requests/min, burst 10
- **Refill rate:** 1 token every 2 seconds
- **Behavior:** After burst, allows 1 request per 2 seconds

## Quick Fix: Adjust via Environment Variables

Add to `.env` or deployment config:

```bash
# Conservative limits (recommended for production)
RATE_LIMIT_API_MUTATE_RPM=10
RATE_LIMIT_API_MUTATE_BURST=3

# Very strict limits (high-security environments)
RATE_LIMIT_API_MUTATE_RPM=5
RATE_LIMIT_API_MUTATE_BURST=2

# Disable rate limiting (development only)
RATE_LIMIT_ENABLED=false
```

## Testing Rate Limits

```bash
# Test script
API_KEY="your_api_key_here"
WORKSPACE_ID="your_workspace_id"

# Send rapid requests to trigger rate limit
for i in {1..15}; do
  echo "Request $i:"
  curl -w "\nHTTP %{http_code}\n" \
       -H "Authorization: Bearer $API_KEY" \
       "http://localhost:3000/workspaces/$WORKSPACE_ID/browse"
  echo "---"
done

# Expected output:
# Requests 1-10:  HTTP 200 (burst allowance)
# Request 11:     HTTP 429 Too Many Requests
# Wait 2 seconds...
# Request 12:     HTTP 200 (1 token replenished)
```

## Token Bucket Math

| RPM Setting | Period (ms) | Human-Friendly |
|-------------|-------------|----------------|
| 60 | 1,000 | 1 request per second |
| 30 | 2,000 | 1 request per 2 seconds |
| 20 | 3,000 | 1 request per 3 seconds |
| 10 | 6,000 | 1 request per 6 seconds |
| 5 | 12,000 | 1 request per 12 seconds |
| 3 | 20,000 | 1 request per 20 seconds |
| 1 | 60,000 | 1 request per minute |

Formula: `period_ms = 60,000 / RPM`

## Advanced: Separate Workspace Rate Limits

If you need workspace operations to have different limits than other API mutations, we can add a new endpoint class. Let me know if you need this!

### Implementation Would Add:

1. **New endpoint class** in `rate-limiter/src/lib.rs`:
   ```rust
   pub enum EndpointClass {
       Auth,
       Upload,
       Validation,
       ApiMutate,
       WorkspaceOps,  // ← New
       General,
   }
   ```

2. **New environment variables**:
   ```bash
   RATE_LIMIT_WORKSPACE_OPS_RPM=5
   RATE_LIMIT_WORKSPACE_OPS_BURST=2
   ```

3. **Update workspace routes** in `main.rs`:
   ```rust
   .merge({
       let r = workspace_routes(workspace_state).route_layer(...);
       if let Some(layer) = rate_limit.workspace_ops_layer() {  // ← New method
           r.layer(layer)
       } else {
           r
       }
   })
   ```

---

## Recommendations

**For Production:**
```bash
RATE_LIMIT_API_MUTATE_RPM=10
RATE_LIMIT_API_MUTATE_BURST=3
```
- Allows 3 quick operations, then 1 per 6 seconds
- Prevents abuse while allowing legitimate workflows

**For Development:**
```bash
RATE_LIMIT_API_MUTATE_RPM=60
RATE_LIMIT_API_MUTATE_BURST=20
```
- More lenient for testing
- 1 request per second sustained

**For High-Security:**
```bash
RATE_LIMIT_API_MUTATE_RPM=5
RATE_LIMIT_API_MUTATE_BURST=2
```
- Very strict: 2 quick ops, then 1 per 12 seconds
- Minimal attack surface
