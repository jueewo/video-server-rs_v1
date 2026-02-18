# SECURITY DEPLOYMENT CHECKLIST — `video-server-rs_v1`

## Purpose

Use this checklist before every production deployment to reduce security risk for media uploads, streaming access, and administrative operations.

**How to use**
- Mark each item: `[ ]` not done, `[x]` done, `[N/A]` not applicable.
- Deployment is **blocked** if any **P0** item is unchecked.
- Attach this completed checklist to the release record.

---

## Release Metadata

- Date:
- Environment: (staging / production)
- Release version / commit:
- Prepared by:
- Reviewed by:
- Approved by:

---

## P0 — Must Pass Before Production

### 1) Secrets and Credentials

- [ ] **No secrets hardcoded** in source code (tokens, API keys, passwords, signing secrets).
- [ ] All secrets are loaded from environment/secret manager.
- [ ] Secret values are not present in logs, error traces, or metrics labels.
- [ ] Example/default secrets are disabled in production.
- [ ] Stream publishing token(s) rotated from development defaults.
- [ ] Session signing/encryption keys are set and strong.

**Evidence**
- Secret source:
- Rotation date:
- Reviewer:

---

### 2) Session and Cookie Security

- [ ] Cookies use `HttpOnly`.
- [ ] Cookies use `Secure` in production (HTTPS only).
- [ ] `SameSite` is explicitly set and justified.
- [ ] Session TTL is configured (idle + absolute timeout).
- [ ] Session invalidation on logout is verified.
- [ ] Session fixation protections are in place (renew session after login/privilege change).

**Evidence**
- Session config location:
- Test proof:

---

### 3) Authentication and Authorization

- [ ] Private media is denied by default.
- [ ] Access checks are enforced on all media delivery paths (video/image/document/preview/thumbnail/HLS).
- [ ] Access code validation enforces expiry and scope.
- [ ] API key scope/permissions are limited to intended routes.
- [ ] Unauthorized requests return safe error responses (no internal details).
- [ ] Admin routes are protected and tested.

**Evidence**
- Authz test report:
- Penetration checks run:

---

### 4) Upload Security Controls

- [ ] Upload file size limits configured and enforced server-side.
- [ ] Allowed file types are restricted by policy.
- [ ] MIME type and extension are both validated.
- [ ] Magic-byte/signature validation performed where applicable.
- [ ] Filenames sanitized; no path traversal possible.
- [ ] Upload destination is outside executable/static code paths.
- [ ] Malicious filename cases tested (`../`, null bytes, unicode tricks).

**Evidence**
- Upload policy doc:
- Negative test cases:

---

### 5) Transport and Network Security

- [ ] TLS enforced end-to-end for user-facing endpoints.
- [ ] Internal service traffic policy documented (trusted network/VPN/service mesh).
- [ ] HTTP security headers configured (at app or proxy): `HSTS`, `X-Content-Type-Options`, `Referrer-Policy`, `X-Frame-Options`/CSP policy.
- [ ] CORS policy is explicit and minimal.
- [ ] Unused ports are closed.
- [ ] Admin/metrics interfaces restricted by network policy.

**Evidence**
- Reverse proxy config:
- Port scan output:

---

### 6) Logging, Audit, and Monitoring Security

- [ ] Security events are logged: login success/failure, permission denials, key/code use, destructive actions.
- [ ] Logs exclude secrets and sensitive payloads.
- [ ] Request correlation ID is present for traceability.
- [ ] Alerting configured for suspicious patterns (bruteforce, repeated denied access, high error bursts).
- [ ] Time synchronization verified for all hosts.

**Evidence**
- Alert rules:
- Sample sanitized logs:

---

### 7) Data Protection and Recovery

- [ ] SQLite/media storage backups are configured and encrypted (at rest/in transit where applicable).
- [ ] Backup retention policy is defined and enforced.
- [ ] Restore test completed recently and documented.
- [ ] Least-privilege filesystem permissions are set for DB/storage/log directories.
- [ ] Sensitive data lifecycle/deletion policy exists.

**Evidence**
- Last restore test date:
- Backup location/policy:

---

## P1 — Strongly Recommended for Every Release

### 8) Dependency and Supply Chain

- [ ] Dependency vulnerability scan executed (Rust + Node).
- [ ] No critical/high known vulnerabilities unresolved (or explicit risk acceptance documented).
- [ ] Lockfiles committed and reviewed.
- [ ] Build process is reproducible in CI.

**Evidence**
- Scan report link:
- Exceptions approved by:

---

### 9) Runtime Hardening

- [ ] Application runs as non-root user.
- [ ] File permissions are minimal (`umask`, ownership, read/write boundaries).
- [ ] Container image (if used) is minimal and pinned.
- [ ] Debug endpoints/tools disabled in production.
- [ ] Panic/error pages do not expose internals.

**Evidence**
- Runtime user:
- Container/image digest:

---

### 10) Abuse Prevention

- [ ] Rate limits for login, upload, and token/code validation endpoints.
- [ ] Anti-automation controls for abuse-prone endpoints (challenge, lockout, progressive delay, etc.).
- [ ] Upload quotas and storage thresholds configured.
- [ ] Alerting on unusual upload volume or failed auth spikes.

**Evidence**
- Rate-limit policy:
- Threshold values:

---

### 11) Streaming-Specific Controls (MediaMTX + HLS/WebRTC)

- [ ] Publish endpoint requires valid token/auth.
- [ ] Playback endpoints enforce session/access checks as designed.
- [ ] Recording retention policy configured and verified.
- [ ] Public vs private stream paths documented and tested.
- [ ] Stream key/token leak response procedure documented.

**Evidence**
- Streaming auth test:
- Retention settings:

---

## P2 — Maturity and Governance

### 12) Security Process

- [ ] Threat model for upload + stream access reviewed in last quarter.
- [ ] Security contact/escalation path documented.
- [ ] Incident response runbook available and tested.
- [ ] Security backlog items tracked with owner and target date.
- [ ] Annual/quarterly access review process defined.

---

## Verification Tests (Minimum Set)

Run and attach evidence:

- [ ] Unauthorized user cannot access private video.
- [ ] Unauthorized user cannot access private image/document.
- [ ] Expired access code is rejected.
- [ ] Invalid stream publish token is rejected.
- [ ] Oversized upload is rejected.
- [ ] Disallowed file type is rejected.
- [ ] Path traversal filename upload is rejected.
- [ ] Session invalid after logout.
- [ ] Security headers present on key pages.
- [ ] Backup restore smoke test succeeds.

---

## Go/No-Go Gate

### Blockers (must be empty to release)

- [ ] P0 checklist has **no open items**
- [ ] No unresolved critical vulnerabilities
- [ ] No unresolved auth/authz regressions
- [ ] Rollback plan verified

**Release decision**
- [ ] GO
- [ ] NO-GO

Approver:
Date:

---

## Exceptions / Risk Acceptance

Document any unchecked item approved for temporary release:

| Item | Risk | Mitigation | Expiry Date | Approver |
|---|---|---|---|---|
|  |  |  |  |  |

---

## Post-Deployment Validation (within 24h)

- [ ] Error rates normal
- [ ] Auth failure anomaly check complete
- [ ] Upload success/failure ratio normal
- [ ] Stream auth events normal
- [ ] No sensitive log leakage detected
- [ ] Alerts and dashboards healthy

Owner:
Date:

---

## Notes

- This checklist is a living document. Update it when architecture, auth model, or deployment topology changes.
- If an item is not applicable, provide a short justification under evidence.