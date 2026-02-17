# TECHNICAL DEBT BACKLOG — `video-server-rs_v1`

## Purpose

This backlog captures technical debt items discovered during the audit and translates them into executable work packages.  
It is prioritized to reduce production risk first, then improve maintainability and delivery speed.

## Priority Scale

- **P0** — Critical: security, data integrity, production stability
- **P1** — High: significant reliability/maintainability impact
- **P2** — Medium: quality and velocity improvements
- **P3** — Low: polish and optimizations

## Effort Scale

- **S**: 0.5–2 days
- **M**: 3–7 days
- **L**: 1–3 weeks
- **XL**: 3+ weeks

---

## Backlog Overview (Prioritized)

| ID | Title | Priority | Effort | Category | Impact |
|---|---|---:|---:|---|---|
| TD-001 | Enforce production secret management | P0 | M | Security | Prevent insecure deployments |
| TD-002 | Session/cookie security hardening baseline | P0 | S | Security | Reduce auth/session risk |
| TD-003 | Centralize authorization checks for all media routes | P0 | L | Security/Architecture | Eliminate authz bypass risk |
| TD-004 | Repository hygiene and artifact policy cleanup | P0 | M | DevEx/Ops | Reduce leakage/noise/CI cost |
| TD-005 | Define and enforce migration governance workflow | P1 | M | Data/Release | Prevent schema drift |
| TD-006 | Decompose entrypoint composition layer | P1 | L | Architecture | Improve maintainability |
| TD-007 | Standardize API error envelope and mapping | P1 | M | Backend | Better reliability + client behavior |
| TD-008 | Add integration tests for authz-critical paths | P1 | L | Testing | Increase release confidence |
| TD-009 | Implement upload validation policy (MIME/magic/limits) | P1 | M | Media/Security | Block malicious/bad uploads |
| TD-010 | Add rate limiting for auth/upload/token endpoints | P1 | M | Security/Ops | Protect from abuse |
| TD-011 | Structured logging standard + request correlation | P1 | M | Observability | Faster incident triage |
| TD-012 | Canonical docs map + archive governance | P2 | S | Docs/Governance | Reduce onboarding errors |
| TD-013 | UI component governance and design tokens | P2 | M | UI/UX | Improve consistency |
| TD-014 | Accessibility baseline enforcement | P2 | M | UI/UX | Better usability/compliance |
| TD-015 | Background jobs for long-running media tasks | P2 | XL | Architecture/Media | Improve responsiveness/retry |
| TD-016 | Storage lifecycle management and orphan cleanup policy | P2 | M | Media/Ops | Control storage growth |
| TD-017 | Backup/restore validation automation | P2 | M | Ops | Improve disaster readiness |
| TD-018 | SLO dashboard + alert policy | P2 | M | Observability | Proactive operations |
| TD-019 | Define crate ownership and API contracts | P3 | S | Architecture/Governance | Reduce cross-crate coupling |
| TD-020 | Reduce `common` crate scope creep | P3 | M | Architecture | Preserve boundaries |

---

## Detailed Work Items

## TD-001 — Enforce production secret management
- **Priority:** P0
- **Effort:** M
- **Problem:** Risk of insecure defaults and secret handling drift.
- **Scope:**
  - Define all required secrets/env vars in one canonical config contract.
  - Fail startup in production if required secrets are missing/weak.
  - Replace placeholder/default secret behavior in production mode.
- **Acceptance Criteria:**
  - Production startup fails fast without strong required secrets.
  - Secret values are never logged.
  - Security checklist references exact env contract.
- **Dependencies:** None

## TD-002 — Session/cookie security hardening baseline
- **Priority:** P0
- **Effort:** S
- **Problem:** Session policy may not be uniformly hardened for production.
- **Scope:**
  - Enforce secure cookie settings (`Secure`, `HttpOnly`, strict `SameSite` policy by environment).
  - Define session TTL/rotation policy.
- **Acceptance Criteria:**
  - Production profile applies hardened cookie settings by default.
  - Session expiration and renewal behavior is documented and tested.
- **Dependencies:** TD-001

## TD-003 — Centralize authorization checks for all media routes
- **Priority:** P0
- **Effort:** L
- **Problem:** Authz logic can drift across endpoints (preview/thumb/HLS/raw files).
- **Scope:**
  - Single reusable authorization decision path for media resource access.
  - Deny-by-default for private resources.
  - Uniform enforcement across all media-serving routes.
- **Acceptance Criteria:**
  - Test matrix covers public/private/group/access-code/API-key/session paths.
  - No media route bypasses centralized authz policy.
- **Dependencies:** TD-008

## TD-004 — Repository hygiene and artifact policy cleanup
- **Priority:** P0
- **Effort:** M
- **Problem:** Runtime/build artifacts in repo increase risk and noise.
- **Scope:**
  - Define tracked vs generated artifact policy.
  - Update ignore strategy and remove tracked generated artifacts.
  - Add pre-commit/CI guardrails for forbidden artifact commits.
- **Acceptance Criteria:**
  - No runtime DB/log/backup/build/dependency artifacts tracked by default.
  - CI fails on forbidden artifact patterns.
- **Dependencies:** None

## TD-005 — Define and enforce migration governance workflow
- **Priority:** P1
- **Effort:** M
- **Problem:** Schema change process lacks explicit governance.
- **Scope:**
  - One migration source-of-truth process (generate, review, apply, rollback).
  - Add migration checklist to PR template.
  - Ensure local/dev/prod parity process is documented.
- **Acceptance Criteria:**
  - Every schema change includes migration + rollback note.
  - Migration process is reproducible in CI/staging.
- **Dependencies:** None

## TD-006 — Decompose entrypoint composition layer
- **Priority:** P1
- **Effort:** L
- **Problem:** Entrypoint orchestration is too dense and high-risk.
- **Scope:**
  - Split bootstrap concerns into clear modules (`config`, `infra`, `app wiring`, `routes`).
  - Keep entrypoint declarative and minimal.
- **Acceptance Criteria:**
  - Entrypoint reduced to high-level composition.
  - Domain wiring is isolated and testable.
- **Dependencies:** TD-019, TD-020

## TD-007 — Standardize API error envelope and mapping
- **Priority:** P1
- **Effort:** M
- **Problem:** Inconsistent error handling harms clients and operations.
- **Scope:**
  - Define canonical error schema.
  - Map domain/infrastructure errors to stable HTTP status + codes.
  - Ensure user-safe messages + operator-level details via logs.
- **Acceptance Criteria:**
  - API endpoints return consistent error shape.
  - Error code catalog exists and is documented.
- **Dependencies:** None

## TD-008 — Add integration tests for authz-critical paths
- **Priority:** P1
- **Effort:** L
- **Problem:** Complex access matrix is hard to validate manually.
- **Scope:**
  - Build scenario-based integration tests for media access.
  - Include anonymous/session/access-code/group/api-key flows.
- **Acceptance Criteria:**
  - Authz regression suite runs in CI.
  - Critical routes covered by deny/allow tests.
- **Dependencies:** TD-003

## TD-009 — Implement upload validation policy
- **Priority:** P1
- **Effort:** M
- **Problem:** Upload vectors require strict controls.
- **Scope:**
  - Validate extension + MIME + file signatures.
  - Per-type size limits.
  - Filename normalization and path traversal prevention.
- **Acceptance Criteria:**
  - Rejected uploads provide consistent validation errors.
  - Security tests cover malicious payload patterns.
- **Dependencies:** TD-007

## TD-010 — Add rate limiting for sensitive endpoints
- **Priority:** P1
- **Effort:** M
- **Problem:** Auth/upload/token endpoints are abuse-prone.
- **Scope:**
  - Add endpoint class-based throttling policy.
  - Introduce safe defaults and alerting hooks.
- **Acceptance Criteria:**
  - Excess traffic yields predictable 429 handling.
  - Limits are configurable by environment.
- **Dependencies:** TD-018

## TD-011 — Structured logging + request correlation
- **Priority:** P1
- **Effort:** M
- **Problem:** Incident troubleshooting is harder without uniform log schema.
- **Scope:**
  - Define common structured fields (`request_id`, `user_id`, `route`, `result`, etc.).
  - Ensure correlation across services/components.
- **Acceptance Criteria:**
  - Logs are queryable by request ID end-to-end.
  - Security-sensitive events are consistently logged.
- **Dependencies:** None

## TD-012 — Canonical docs map + archive governance
- **Priority:** P2
- **Effort:** S
- **Problem:** Documentation sprawl causes drift.
- **Scope:**
  - Add a single “start here” documentation index.
  - Separate active docs vs archive with validation metadata.
- **Acceptance Criteria:**
  - New contributors can find active architecture/deploy/security docs quickly.
  - Archive items are clearly marked non-canonical.
- **Dependencies:** None

## TD-013 — UI component governance and design tokens
- **Priority:** P2
- **Effort:** M
- **Problem:** Template-based UIs drift without strict component patterns.
- **Scope:**
  - Define reusable component patterns and visual tokens.
  - Add usage guidelines and examples.
- **Acceptance Criteria:**
  - New screens use standard primitives.
  - Reduced duplicated markup/styles.
- **Dependencies:** TD-014

## TD-014 — Accessibility baseline enforcement
- **Priority:** P2
- **Effort:** M
- **Problem:** Accessibility can regress silently.
- **Scope:**
  - Baseline rules for keyboard/focus/labels/contrast/errors.
  - Add simple a11y checks to review workflow.
- **Acceptance Criteria:**
  - Core flows pass accessibility baseline checks.
  - Regressions are caught in review/CI.
- **Dependencies:** None

## TD-015 — Background jobs for long-running media tasks
- **Priority:** P2
- **Effort:** XL
- **Problem:** Long-running media processing should not block request path.
- **Scope:**
  - Introduce async job model for thumbnail/transcode/repair tasks.
  - Retry policy + dead-letter strategy.
- **Acceptance Criteria:**
  - Upload API remains responsive during processing spikes.
  - Jobs are observable and recoverable.
- **Dependencies:** TD-011, TD-018

## TD-016 — Storage lifecycle and orphan cleanup policy
- **Priority:** P2
- **Effort:** M
- **Problem:** Storage growth and stale metadata can diverge.
- **Scope:**
  - Scheduled orphan detection/removal.
  - Retention policy for generated assets/recordings.
- **Acceptance Criteria:**
  - Periodic storage integrity report available.
  - Orphan rate trend decreases.
- **Dependencies:** TD-015

## TD-017 — Backup/restore validation automation
- **Priority:** P2
- **Effort:** M
- **Problem:** Backups are only useful if restore is tested.
- **Scope:**
  - Define restore drills for DB + media.
  - Add routine validation cadence.
- **Acceptance Criteria:**
  - Recovery procedure tested and timed.
  - Recovery point and time objectives are documented.
- **Dependencies:** None

## TD-018 — SLO dashboard + alert policy
- **Priority:** P2
- **Effort:** M
- **Problem:** Operational blind spots increase incident duration.
- **Scope:**
  - Define SLOs for upload success, auth failures, streaming health, DB latency.
  - Add dashboards and threshold-based alerts.
- **Acceptance Criteria:**
  - On-call can detect and triage key failure modes quickly.
  - Alert noise is tuned to actionable signals.
- **Dependencies:** TD-011

## TD-019 — Define crate ownership and API contracts
- **Priority:** P3
- **Effort:** S
- **Problem:** Cross-crate boundaries can erode over time.
- **Scope:**
  - Assign ownership and review rules per crate.
  - Document public interfaces and dependency constraints.
- **Acceptance Criteria:**
  - Ownership map exists and is adopted.
  - Boundary violations are visible in code review.
- **Dependencies:** None

## TD-020 — Reduce `common` crate scope creep
- **Priority:** P3
- **Effort:** M
- **Problem:** Shared crate risks becoming a catch-all.
- **Scope:**
  - Audit `common` responsibilities.
  - Move domain-specific code to owning crates.
- **Acceptance Criteria:**
  - `common` contains only true cross-cutting primitives.
  - Domain ownership becomes clearer.
- **Dependencies:** TD-019

---

## Suggested Delivery Waves

## Wave 1 (Weeks 1–3) — Production Risk Reduction
- TD-001, TD-002, TD-004, TD-009, TD-010

## Wave 2 (Weeks 4–7) — Stability and Correctness
- TD-003, TD-007, TD-008, TD-011, TD-005

## Wave 3 (Weeks 8–12) — Scale and Operational Maturity
- TD-018, TD-017, TD-015, TD-016, TD-006

## Wave 4 (Ongoing) — Governance and UX Quality
- TD-012, TD-013, TD-014, TD-019, TD-020

---

## Tracking Template (for Issues)

Use this template for each created issue:

- **ID:** TD-XXX
- **Title:**
- **Owner:**
- **Priority:** P0/P1/P2/P3
- **Effort:** S/M/L/XL
- **Status:** Todo / In Progress / Blocked / Done
- **Risk if delayed:**
- **Scope:**
- **Acceptance Criteria:**
- **Dependencies:**
- **Target milestone:**

---

## Definition of Done (for Technical Debt Items)

A debt item is considered done when:
1. Scope is fully implemented and reviewed.
2. Acceptance criteria are verified.
3. Documentation is updated (if behavior/process changed).
4. Operational implications are covered (alerts/logs/runbook updates as needed).
5. Relevant tests are added/updated and passing.

---

## Notes

- Backlog prioritization should be revisited every sprint.
- Any new feature work touching P0/P1 debt areas should include debt paydown in the same PR when feasible.
- Security and authz debt items should not be deferred behind cosmetic feature work.