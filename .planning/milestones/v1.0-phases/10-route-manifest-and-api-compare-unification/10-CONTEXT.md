---
generated_by: gsd-discuss-phase
lifecycle_mode: yolo
phase_lifecycle_id: 10-2026-06-29T15-52-48
generated_at: 2026-06-29T15:53:51.833Z
---

# Phase 10: Route Manifest And API Compare Unification - Context

**Gathered:** 2026-06-29
**Status:** Ready for planning
**Mode:** Yolo

<domain>
## Phase Boundary

Phase 10 closes the route-manifest drift gap. Firmware route reporting and API/static/OTA compare tooling must consume the same Phase 7 route manifest so missing routes, downgraded ownership, or incorrect verification classifications are caught before release evidence.

This phase proves manifest alignment, reporting, fixtures, and regression guards. It does not prove live HTTP, live static serving, recovery page reachability, firmware OTA behavior, OTAWWW behavior, rollback, erase, failed-update recovery, interrupted-update recovery, ASIC initialization, mining, voltage, fan, thermal, or power behavior.

</domain>

<decisions>
## Implementation Decisions

### Manifest Ownership

- **D-01:** Treat `crates/bitaxe-api/src/route_shell.rs` as the pure route manifest owner for Phase 10.
- **D-02:** Use the existing Phase 7 superset route list as the canonical manifest direction. `phase07_routes()` should be the source consumed by firmware route reporting and compare tooling.
- **D-03:** Preserve `phase05_routes()` as a compatibility projection or phase-specific view for Phase 5 API schema and captured-response evidence. Do not let the Phase 5 view remain the authority for Phase 7 static, recovery, firmware OTA, or OTAWWW gap route ownership.
- **D-04:** Do not add a checked-in JSON manifest plus Rust code generation in this phase. That would add build graph and generated-output drift without enough value over the existing typed route table.
- **D-05:** Do not rewrite firmware handler registration to be fully manifest-driven in this phase. Keep handlers as explicit ESP-IDF adapters and prove reporting/compare alignment through typed helpers and tests.

### Firmware Route Reporting

- **D-06:** Firmware route count and route-report logging should consume the Phase 7 route manifest, not `phase05_routes().len()`.
- **D-07:** Reporting should include or make testable the ownership of Phase 7 routes: firmware OTA, OTAWWW gap, recovery, and static wildcard.
- **D-08:** Firmware adapters may keep manual registration order for ESP-IDF correctness: explicit recovery, API routes, update routes, WebSockets, unknown API fallbacks, and static wildcard last.
- **D-09:** Add host-testable route reporting helpers in the pure API crate where practical so firmware only formats or logs manifest-derived data.
- **D-10:** If actual handler registration cannot be mechanically derived from the manifest in this phase, add regression tests that compare the declared Phase 7 manifest against the adapter/reporting contract.

### API Compare Regression Behavior

- **D-11:** `tools/parity api-compare` should consume the same Phase 7 manifest data for route presence and classification checks.
- **D-12:** Keep Phase 5 schema and captured-response checks. Phase 10 should not discard the existing OpenAPI and fixture evidence for `/api/system/*`, logs, WebSockets, commands, and settings.
- **D-13:** Move route presence and ownership assertions away from duplicate Phase 5 JSON route lists where practical. JSON fixtures may still describe schema, captured response, static usage, and evidence policy, but should not be the independent route source of truth.
- **D-14:** Compare tooling must fail when a required Phase 7 route is missing from the Rust manifest.
- **D-15:** Compare tooling must fail when a required Phase 7 route is downgraded, for example firmware OTA losing `FirmwareUpdate`, OTAWWW losing `AxeOsStaticUpdateGap`, `/recovery` losing `Recovery`, or `/*` losing `StaticFiles`.
- **D-16:** Compare tooling must fail when fixture or policy data incorrectly classifies static, recovery, firmware OTA, OTAWWW, or release-sensitive routes as verified from unit, workflow, package, or API compare evidence alone.

### Parity Evidence Classification

- **D-17:** Record Phase 10 evidence as tooling and manifest evidence only, using existing evidence labels such as `unit`, `workflow`, and `api-compare`.
- **D-18:** Prefer updating existing relevant checklist rows and adding a Phase 10 evidence record over inventing a new evidence taxonomy such as `manifest-compare`.
- **D-19:** If a dedicated checklist row is needed to make the manifest/tooling claim queryable, word it narrowly so it cannot be confused with live HTTP route verification.
- **D-20:** Keep `API-004`, `API-007`, `API-008`, `FS-001`, `OTA-001`, `OTA-002`, `REL-001`, `REL-002`, and `REL-003` below live/release verification where their current blockers still apply.
- **D-21:** Phase 10 evidence should include an explicit claim-boundary matrix stating which surfaces were proven by manifest/tooling checks and which live surfaces remain not run or Phase 13-owned.

### Testing And Verification Shape

- **D-22:** Unit tests should prove `phase07_routes()` contains the Phase 7-owned routes with the expected method and kind.
- **D-23:** Unit tests should prove route reporting derives from the Phase 7 manifest count and ownership metadata.
- **D-24:** API compare tests should cover missing route failure, route kind downgrade failure, and verified-overclaim failure for static/recovery/update routes.
- **D-25:** Run `just parity` after implementation and record the command result in Phase 10 evidence.

### the agent's Discretion

The agent may choose exact helper names, fixture names, report wording, checklist row wording, and whether route policy lives entirely in `tools/parity/src/api_compare.rs` or partly in `crates/bitaxe-api/src/route_shell.rs`. Those choices must preserve the functional-core/imperative-shell boundary, keep firmware handlers thin, keep the upstream reference read-only, use typed data instead of ad hoc string scanning where practical, and avoid claiming live HTTP or OTA behavior.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase Scope And Requirements

- `.planning/ROADMAP.md` - Phase 10 goal, gap closure, success criteria, verification expectations, and dependency on Phase 9.
- `.planning/REQUIREMENTS.md` - API-09, API-10, REL-01, REL-02, REL-03, EVD-01, and evidence semantics relevant to route manifest unification.
- `.planning/PROJECT.md` - Ultra 205 target, ESP-IDF Rust stack, parity evidence policy, read-only reference constraint, and current state.
- `.planning/STATE.md` - Prior route, OTA, static, release, and evidence decisions leading into Phase 10.
- `.planning/phases/05-axeos-api-logs-and-telemetry/05-CONTEXT.md` - Phase 5 API compare, static asset compatibility, and route shell decisions.
- `.planning/phases/07-ota-filesystem-and-release-packaging/07-CONTEXT.md` - Phase 7 static, recovery, firmware OTA, OTAWWW gap, and manifest decisions.
- `.planning/phases/08-parity-evidence-and-ultra-205-release-gate/08-CONTEXT.md` - Release evidence boundaries and live HTTP/OTA blocker policy.
- `.planning/phases/09-flash-monitor-evidence-wrapper-hardening/09-CONTEXT.md` - Recent evidence hardening and no-overclaim pattern.

### Existing Implementation Surfaces

- `crates/bitaxe-api/src/route_shell.rs` - Current `phase05_routes()`, `phase07_routes()`, `AxeosRoute`, `RouteMethod`, and `RouteKind` definitions.
- `crates/bitaxe-api/src/lib.rs` - Public exports that firmware and parity tooling consume.
- `firmware/bitaxe/src/http_api.rs` - Firmware HTTP/WebSocket route shell and current route-count log using `phase05_routes().len()`.
- `firmware/bitaxe/src/static_files.rs` - Recovery and static wildcard registration plus static route handling.
- `firmware/bitaxe/src/filesystem.rs` - SPIFFS `www` mount status used by static/recovery behavior.
- `tools/parity/src/api_compare.rs` - API compare implementation that currently imports `phase05_routes()` and parses the route fixture.
- `tools/parity/src/main.rs` - API compare command wiring and route manifest file loading.
- `tools/parity/fixtures/api/phase05-required-routes.json` - Existing Phase 5 route/property/captured-response fixture that currently duplicates route paths.
- `tools/parity/fixtures/api/axeos-route-usage.json` - Static route usage fixture covering Phase 5 API routes plus Phase 7-owned update, recovery, and static boundaries.
- `docs/parity/checklist.md` - API, filesystem, OTA, release, and evidence rows to update without overclaiming.
- `docs/parity/evidence/phase-07-ota-filesystem-release.md` - Phase 7 package/static/recovery/OTA evidence and live evidence deferrals.
- `docs/parity/evidence/phase-08-ultra-205-release-gate.md` - Device URL blocker and live HTTP/static/recovery/OTA evidence deferrals.
- `docs/parity/evidence/phase-09-flash-monitor-evidence-wrapper-hardening.md` - Recent pattern for wrapper-owned evidence with explicit non-claims.

### Upstream Reference Behavior

- `reference/esp-miner/main/http_server/http_server.c` - Upstream route registration, static serving, recovery, firmware OTA, OTAWWW, and WebSocket route mapping.
- `reference/esp-miner/main/http_server/openapi.yaml` - Upstream route/schema contract for API and update routes.
- `reference/esp-miner/main/http_server/recovery_page.html` - Upstream recovery page behavior reference.
- `reference/esp-miner/main/http_server/axe-os/` - AxeOS static assets and client route usage expectations.
- `reference/esp-miner/main/filesystem.c` - Upstream SPIFFS/static file behavior.

### Architecture, Evidence, And Policy

- `docs/adr/0001-device-user-parity.md` - Observable behavior parity definition.
- `docs/adr/0003-esp-idf-rust-production-stack.md` - ESP-IDF Rust stack and adapter boundary.
- `docs/adr/0005-read-only-reference-implementation.md` - Reference implementation policy.
- `docs/adr/0006-parity-checklist-as-audit-evidence.md` - Checklist as audit evidence.
- `docs/adr/0008-reference-breadcrumb-comments.md` - Breadcrumb placement policy.
- `docs/adr/0009-monorepo-package-layout.md` - Crate, firmware, and tool ownership.
- `docs/adr/0010-axeos-api-and-asset-compatibility.md` - API/static asset compatibility before UI rewrite.
- `docs/adr/0012-parity-verification-evidence.md` - Verification status semantics and evidence-class rules.
- `docs/adr/0013-mit-first-with-gpl-guardrails.md` - GPL guardrails for upstream-derived behavior and fixtures.
- `docs/adr/0014-pivot-to-ultra-205-bm1366-first-parity.md` - Ultra 205 first target and deferred non-205 evidence boundary.
- `PROVENANCE.md` - Provenance, SPDX, upstream reference, fixture/source attribution, and release review policy.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- `phase07_routes()` already lists firmware OTA, OTAWWW gap, `/recovery`, and `/*` with typed ownership. This is the natural Phase 10 source of truth.
- `RouteKind` already distinguishes `FirmwareUpdate`, `AxeOsStaticUpdateGap`, `Recovery`, and `StaticFiles`.
- `tools/parity/src/api_compare.rs` already has a route-set comparison framework and regression tests for missing routes and Phase 5 update overclaiming.
- `tools/parity/fixtures/api/axeos-route-usage.json` already records static/update/recovery Phase 7 ownership and Phase 5 non-success classification.

### Established Patterns

- Pure route and response decisions belong in `crates/bitaxe-api`; firmware registration and ESP-IDF calls stay in `firmware/bitaxe`.
- Existing API compare uses structured JSON fixtures and pure validation functions, with filesystem reads isolated to `WorkspaceFixtureLoader`.
- Checklist updates distinguish implementation evidence from live hardware or release evidence.

### Integration Points

- Replace firmware route reporting that currently uses `phase05_routes().len()` with Phase 7 manifest-derived reporting.
- Update API compare route checks to consume Phase 7 manifest data for route presence and kind policy while preserving Phase 5 schema/captured-response checks.
- Add or update fixtures/tests so route ownership changes fail visibly before parity evidence is trusted.
- Update parity checklist/evidence docs to cite the unified manifest evidence and state live HTTP/OTA evidence remains pending or Phase 13-owned.

</code_context>

<specifics>
## Specific Ideas

- The main known drift is explicit: `firmware/bitaxe/src/http_api.rs` logs `phase05_routes().len()` even though Phase 7 routes are registered and `phase07_routes()` exists.
- Prefer a small typed policy map over a new manifest generator.
- Preserve the existing route registration order. Static wildcard must remain last.
- A good regression case is changing `/api/system/OTAWWW` from `AxeOsStaticUpdateGap` to a success-like or safe-unsupported kind and proving compare tooling fails.
- A good evidence record names the unified manifest checks, route count/report output, API compare command, fixture regression tests, and non-claims for live HTTP/static/recovery/OTA/rollback.

</specifics>

<deferred>
## Deferred Ideas

- Fully manifest-driven firmware route registration is deferred unless a later phase accepts the larger ESP-IDF adapter refactor.
- A generated JSON-to-Rust route manifest pipeline is deferred unless release governance later needs a non-Rust canonical route artifact.
- Live HTTP, static asset serving, `/recovery`, valid firmware OTA, invalid OTA rejection, OTAWWW behavior, rollback, erase, failed-update recovery, and interrupted-update recovery evidence remain Phase 13 scope.

</deferred>

*Phase: 10-route-manifest-and-api-compare-unification*
*Context gathered: 2026-06-29*
