---
generated_by: gsd-discuss-phase
lifecycle_mode: interactive
phase_lifecycle_id: 17-2026-07-02T01-09-48
generated_at: 2026-07-02T01:09:48.162Z
---

# Phase 17: Live HTTP API And Static Evidence - Context

**Gathered:** 2026-07-02
**Status:** Ready for planning
**Mode:** Interactive fallback with advisor recommendations

<domain>
## Phase Boundary

Phase 17 closes the live HTTP/API/static/recovery/WebSocket evidence gap for a
just-flashed Ultra 205 at an explicit reachable `DEVICE_URL`. The phase should
prove route reachability, static asset serving, missing-static behavior,
`/recovery` page reachability, API route coexistence, and bounded WebSocket
behavior for the current package/flash evidence chain.

This phase does not run or verify valid firmware OTA upload, invalid OTA
rejection, rollback, boot validation, large erase, failed-update recovery,
interrupted-update recovery, whole-`www` OTAWWW update behavior, non-205 boards,
production mining, or long soak behavior. Those remain later phases unless their
own phase gates and recovery procedures are present.

</domain>

<decisions>
## Implementation Decisions

### Target Gate And Current-Device Identity

- **D-01:** Accept a live target only from an explicit `--device-url`, explicit
  `DEVICE_URL`, or a phase target-lock manifest created from explicit input. Do
  not scan the network, infer a target from serial logs, parse mDNS, inspect ARP
  tables, or guess from router state.
- **D-02:** Operator observations from a serial console, AP UI, router UI, or
  bench note may be recorded as target provenance, but helpers must not treat
  those observations as automatic discovery.
- **D-03:** Before live probes, require the repo hardware gate: `just
  detect-ultra205` finds exactly one likely ESP32-S3 port and `espflash
  board-info --chip esp32s3 --port <port> --non-interactive` succeeds for board
  `205`.
- **D-04:** Bind live evidence to current package and flash identity. Record
  source commit, reference commit, package manifest, selected board, selected
  port, detector output, flash/serial evidence dependency, exact commands, and
  conclusion. If the package/flash identity is stale, mismatched, or unknown,
  write blocked evidence instead of probing.
- **D-05:** Treat `DEVICE_URL` as an origin-only `http://` or `https://` URL.
  Reject or block userinfo, query, fragment, and path-bearing targets unless a
  future plan deliberately documents why a non-origin URL is safe.
- **D-06:** Run bounded reachability and `/api/system/info` board/device sanity
  before promoting any live route claim. Missing, invalid, unreachable, or
  wrong-device targets produce blocked evidence with no network scan fallback.

### Live Probe Set And Pass Criteria

- **D-07:** Start from the existing Phase 16 route helper behavior, but harden
  Phase 17 pass criteria where needed instead of treating logged headers as
  proof. Keep helper tests for missing URL, invalid URL, successful fake route
  coverage, response redaction, and blocked route outcomes.
- **D-08:** The bounded live route set is:
  `/`, `/assets/app.css.gz`, a representative missing static path, `/recovery`,
  `/api/system/info`, an unknown `/api/*` route, `/api/ws`, `/api/ws/live`,
  `POST /api/system/OTA` as route-presence evidence only, and `POST
  /api/system/OTAWWW` as the expected fail-closed gap response.
- **D-09:** Passing HTTP/static evidence requires route-specific markers:
  root returns 200 with release/static entry markers; `/assets/app.css.gz`
  returns 200 with CSS/gzip/cache headers; missing static returns 302 to `/`
  with the captive-portal redirect body; `/recovery` returns 200 with recovery
  page markers; `/api/system/info` returns JSON for the current device; unknown
  API returns the expected JSON 404 body.
- **D-10:** `GET /api/ws` and `GET /api/ws/live` returning 400 or 426 without
  WebSocket upgrade is route-coexistence evidence only. It proves those paths do
  not fall through to static wildcard handling; it does not verify WebSocket
  framing.
- **D-11:** `POST /api/system/OTA` in Phase 17 proves route presence and
  validation-path reachability only. It must not be cited as valid OTA upload,
  invalid image rejection, reboot, rollback, selected partition, or
  boot-validation evidence.
- **D-12:** `POST /api/system/OTAWWW` may record the live fail-closed public
  response such as `Wrong API input`, but OTAWWW whole-`www` update parity
  remains deferred until Phase 19-style hardware-regression evidence exists.

### WebSocket Proof Depth

- **D-13:** Use both no-upgrade route checks and bounded real WebSocket capture
  before making any verified WebSocket claim. The no-upgrade checks prove route
  precedence; WebSocket frame capture proves actual upgrade/message behavior.
- **D-14:** `/api/ws/live` may be promoted only when a bounded capture records a
  redacted connect or cadence frame from the just-flashed device.
- **D-15:** `/api/ws` may be promoted only when a bounded capture records a
  redacted log frame or other accepted raw-log stream evidence. If the WebSocket
  opens but no log frame appears before timeout, record the open/timeout result
  and keep `/api/ws` frame evidence pending.
- **D-16:** WebSocket capture must stay bounded and evidence-focused. It should
  not introduce pool credentials, mining smoke, production soak, or long-running
  telemetry collection into Phase 17.

### Evidence Artifacts And Redaction

- **D-17:** Write Phase 17 live evidence under
  `docs/parity/evidence/phase-17-live-http-api-and-static-evidence/`.
  Expected artifacts include a summary ledger, helper logs, per-route sanitized
  header/body/error artifacts, WebSocket frame metadata or redacted snippets,
  detector and package/flash identity references, and a redaction review.
- **D-18:** Prefer committed, redacted micro-artifacts plus a summary ledger for
  replayable proof. Do not commit raw live response dumps or full WebSocket
  payloads unless a plan introduces a quarantine/retention contract and the
  public ledger cites only redacted or allowlisted values.
- **D-19:** Store only typed allowlisted fields and sanitized snippets for API
  responses and WebSocket frames where practical. Redact `DEVICE_URL`, private
  endpoints, IP addresses, MAC addresses, Wi-Fi credentials, pool credentials,
  worker secrets, API tokens, NVS secret values, and local terminal secrets.
- **D-20:** Mark absent artifacts explicitly as `absent - not cited` in the
  redaction review. Do not cite body/header/frame artifacts that were not
  generated or reviewed.
- **D-21:** Use standalone `---` only for YAML frontmatter delimiters at the top
  of parsed Markdown artifacts. Use headings or `***` for body separation.

### Checklist, Release Docs, And Traceability

- **D-22:** Promote only rows and notes supported by exact Phase 17 artifacts.
  Guard-aligned promotion is allowed for live HTTP/API/static/recovery/WebSocket
  surfaces when evidence contains the required live terms and no blocker
  language.
- **D-23:** `FS-001` can become `verified` only with hardware evidence that
  names live static behavior, `/assets/app.css.gz`, missing static redirect,
  and `/recovery`, with no blocked/pending language.
- **D-24:** API rows such as `API-004`, `API-005`, `API-006`, `API-007`, and
  `API-008` can be updated or promoted only for the precise live route, frame,
  recovery page, and static asset behaviors captured. Do not let route
  registration or no-upgrade responses stand in for frame-level WebSocket proof.
- **D-25:** Keep `OTA-001`, `OTA-002`, and `REL-003` below verified in Phase 17.
  Keep valid OTA, invalid image rejection, rollback, boot validation, large
  erase, failed-update, interrupted-update, and whole-`www` OTAWWW behavior as
  explicit non-claims.
- **D-26:** Release docs, parity checklist, and requirements traceability should
  cite commands and artifacts, not goals or implementation existence. If
  `DEVICE_URL` or WebSocket capture is unavailable, update docs with blocked or
  pending evidence instead of promoting rows.

### the agent's Discretion

The agent may choose exact helper names, whether Phase 17 wraps or copies the
Phase 16 HTTP/static helper, the target-lock manifest schema, JSON field names,
timeout values, test fixture layout, and whether WebSocket capture uses Node,
Rust host tooling, or another repo-owned bounded client. Those choices must
preserve explicit target input, no network scanning, current package/flash
identity, redaction, test coverage, and exact-claim checklist boundaries.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase Scope And Governance

- `.planning/ROADMAP.md` - Phase 17 goal, requirements, success criteria, gap
  closure, verification expectations, and future Phase 18-21 boundaries.
- `.planning/REQUIREMENTS.md` - API-09, REL-01, REL-07, EVD-05, and the
  Phase 17-21 gap-closure traceability note.
- `.planning/PROJECT.md` - Ultra 205 first target, ESP-IDF Rust stack,
  read-only reference, evidence policy, architecture, licensing, and safety
  constraints.
- `.planning/STATE.md` - Current milestone state after Phase 16 and accumulated
  release, safety, ASIC, mining, and evidence decisions.
- `AGENTS.md` - Repo-local autonomous Ultra 205 hardware gate, detector stop
  conditions, evidence metadata requirements, destructive/fault-injection
  limits, secret handling, and frontmatter separator rule.
- `standards/core/verification.md` - Repo-native verification and pre-commit
  expectations.
- `standards/core/testing.md` - Unit-test expectations for changed pure logic.
- `standards/languages/rust.md` - Rust module, naming, invariant, test, and
  verification guidance.

### Prior Phase Decisions And Evidence

- `.planning/phases/16-current-commit-release-evidence-completion/16-CONTEXT.md`
  - Current-commit release identity, explicit `DEVICE_URL` gate, Phase 16
  helper scope, redaction, checklist, and non-claim decisions.
- `.planning/phases/16-current-commit-release-evidence-completion/16-VERIFICATION.md`
  - Phase 16 passed verification and remaining blocked/pending live HTTP,
  WebSocket, OTA, and recovery evidence.
- `docs/parity/evidence/phase-16-current-commit-release-evidence-completion.md`
  - Final Phase 16 ledger, package/serial identity, blocked live route evidence,
  and residual risks.
- `docs/parity/evidence/phase-16-current-commit-release-evidence-completion/http-static-recovery.md`
  - Blocked Phase 16 HTTP/static/recovery evidence and missing `DEVICE_URL`
  behavior.
- `.planning/phases/13-final-ultra-205-release-evidence/13-CONTEXT.md` -
  Historical HTTP/static/recovery/OTA evidence strategy, `DEVICE_URL` policy,
  and destructive recovery gate.
- `.planning/phases/10-route-manifest-and-api-compare-unification/10-CONTEXT.md`
  - Phase 7 route manifest ownership and API/static/OTA route compare boundary.
- `.planning/phases/07-ota-filesystem-and-release-packaging/07-CONTEXT.md` -
  Static, recovery, SPIFFS, OTA, OTAWWW gap, and release packaging ownership.
- `.planning/phases/14-safety-hardware-evidence-completion/14-CONTEXT.md` -
  Live telemetry blocker policy, allow-manifest pattern, and redaction approach.

### Current Tooling And Implementation Surfaces

- `Justfile` - Human command surface for `package`, `flash-monitor`, `monitor`,
  `detect-ultra205`, `verify-reference`, `parity`, `build`, and `test`.
- `scripts/detect-ultra205.sh` - Required detector and board-info preflight.
- `scripts/phase16-http-static-smoke.sh` - Existing explicit-`DEVICE_URL`
  HTTP/static/recovery/API/WebSocket route helper to harden or wrap for Phase 17.
- `scripts/phase16-http-static-smoke-test.sh` - Existing helper tests for
  missing/invalid URL, route coverage, and redaction.
- `scripts/phase14-live-telemetry.sh` - Existing live telemetry/WebSocket helper
  pattern and evidence boundary.
- `scripts/BUILD.bazel` - Bazel shell targets and tests for phase helper
  integration.
- `tools/flash/src/main.rs` - Flash, monitor, flash-monitor, package manifest
  resolution, evidence JSON/log capture, trusted-output behavior, and wrapper
  command contracts.
- `tools/xtask/src/package_manifest.rs` - Package manifest v2 and artifact
  metadata.
- `tools/parity/src/main.rs` - Checklist validation, release/static/WebSocket
  evidence-token guards, blocker-language checks, and report command.
- `tools/parity/src/api_compare.rs` - Route policy, static/recovery/update guard
  behavior, and API compare checks.
- `crates/bitaxe-api/src/route_shell.rs` - `phase07_routes()` manifest,
  WebSocket route kinds, access decisions, unknown API response, and route
  report.
- `crates/bitaxe-api/src/static_plan.rs` - Pure static and recovery request
  decisions.
- `crates/bitaxe-api/src/update_plan.rs` - Pure firmware OTA and OTAWWW gap
  decisions.
- `firmware/bitaxe/src/http_api.rs` - ESP-IDF HTTP route shell, firmware OTA
  handler, OTAWWW gap handler, access gate, WebSocket registration, API
  coexistence, and route registration logs.
- `firmware/bitaxe/src/websocket_api.rs` - WebSocket state bridge, live
  telemetry frame planning, retained-log stream state, and client session
  tracking.
- `firmware/bitaxe/src/static_files.rs` - Static and recovery route serving
  adapter.
- `firmware/bitaxe/static/www/` - Rust-owned AxeOS-compatible static asset tree
  including `/assets/app.css.gz`.
- `firmware/bitaxe/static/recovery_page.html` - Rust-owned recovery page.
- `docs/release/ultra-205.md` - Operator release guide and current Phase 16
  blocker status.
- `docs/parity/checklist.md` - Parity audit ledger to update conservatively.

### Upstream Reference And Policy

- `reference/esp-miner/main/http_server/http_server.c` - Reference static,
  recovery, firmware OTA, OTAWWW, WebSocket, and route registration behavior.
- `reference/esp-miner/main/http_server/websocket.c` - Reference WebSocket
  behavior.
- `reference/esp-miner/main/http_server/websocket_api.c` - Reference live
  WebSocket telemetry behavior.
- `reference/esp-miner/main/http_server/websocket_log.c` - Reference raw log
  WebSocket behavior.
- `reference/esp-miner/main/http_server/openapi.yaml` - Reference API/update
  route contract.
- `reference/esp-miner/main/http_server/recovery_page.html` - Reference recovery
  page behavior.
- `reference/esp-miner/main/filesystem.c` - Reference SPIFFS mount/status
  behavior.
- `docs/adr/0001-device-user-parity.md` - Observable device-user parity
  definition.
- `docs/adr/0005-read-only-reference-implementation.md` - Read-only upstream
  reference policy.
- `docs/adr/0006-parity-checklist-as-audit-evidence.md` - Checklist evidence
  policy.
- `docs/adr/0010-axeos-api-and-asset-compatibility.md` - API/static
  compatibility boundary before UI rewrite.
- `docs/adr/0012-parity-verification-evidence.md` - Evidence class and verified
  status semantics.
- `docs/adr/0013-mit-first-with-gpl-guardrails.md` - GPL guardrails for
  upstream-derived behavior and fixtures.
- `docs/adr/0014-pivot-to-ultra-205-bm1366-first-parity.md` - Ultra 205/BM1366
  first parity target and deferred non-205 scope.
- `PROVENANCE.md` - Reference, GPL, fixture/source-attribution,
  dependency-license, and firmware release review policy.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- `scripts/phase16-http-static-smoke.sh`: Existing live route helper already
  handles explicit `DEVICE_URL`, missing/invalid URL blocking, sanitized target
  logging, per-route header/body/error artifacts, route status matching, and
  response snippet redaction.
- `scripts/phase16-http-static-smoke-test.sh`: Existing shell tests cover the
  helper's no-scan behavior, fake successful route set, sensitive-field
  redaction, and route failure handling.
- `scripts/phase14-live-telemetry.sh`: Useful pattern for bounded live telemetry
  and WebSocket capture decisions, especially the distinction between route
  reachability and frame-level proof.
- `crates/bitaxe-api/src/route_shell.rs`: Compile-visible route manifest and
  WebSocket route kinds provide the route source of truth.
- `firmware/bitaxe/src/http_api.rs`: Registers `/recovery`, API routes, OTA,
  OTAWWW, `/api/ws`, `/api/ws/live`, unknown `/api/*`, and static wildcard in
  the order Phase 17 must prove live.
- `firmware/bitaxe/src/static_files.rs` and `crates/bitaxe-api/src/static_plan.rs`:
  Existing static/recovery behavior covers gzip static assets, missing-static
  redirect, path traversal rejection, and recovery fallback.
- `tools/parity/src/main.rs`: Existing guards require live static/recovery terms
  and reject blocker language for verified release/static/OTA-sensitive rows.

### Established Patterns

- Live hardware evidence must be detector-gated, board `205` only, tied to a
  package manifest/source commit/reference commit, and redaction-reviewed.
- `DEVICE_URL` is explicit input only. Missing or invalid targets produce
  blocked evidence rather than scans or guesses.
- Checklist rows are exact claims. Package, route registration, and helper
  implementation do not become verified parity without live artifacts.
- Hardware-sensitive work uses narrow helpers, clear allow/stop gates, exact
  commands, redaction reviews, and conservative non-claim language.
- Pure route/static/update decisions live in `crates/bitaxe-api`; ESP-IDF,
  socket, filesystem, OTA, and WebSocket effects stay in firmware adapters and
  scripts.

### Integration Points

- Add or adapt Phase 17 helper targets in `scripts/BUILD.bazel` and keep tests
  local/bounded.
- Write evidence and redaction artifacts under
  `docs/parity/evidence/phase-17-live-http-api-and-static-evidence/`.
- Update `docs/parity/checklist.md`, `docs/release/ultra-205.md`, and
  `.planning/REQUIREMENTS.md` only to the exact evidence tier Phase 17 proves.
- Run `just parity` and `just verify-reference` before finalizing; run targeted
  helper tests and broader repo checks as required by touched paths.

</code_context>

<specifics>
## Specific Ideas

- Preferred command order: `just package`, release gate, `just detect-ultra205`,
  wrapper `flash-monitor` or explicit current serial dependency, Phase 17 target
  lock or explicit `DEVICE_URL` validation, hardened HTTP/static/API probes,
  bounded WebSocket frame capture, redaction review, checklist/docs updates,
  `just parity`, and `just verify-reference`.
- A Phase 17 target-lock manifest is useful if HTTP and WebSocket captures run
  as separate commands. It should store sanitized URL provenance, board/port,
  package manifest, source/reference commits, flash evidence dependency, helper
  command, and stale-manifest rules.
- If `DEVICE_URL` is missing or unreachable, write blocked Phase 17 evidence and
  keep affected rows below verified. Do not scan for the device.
- For `/api/ws/live`, one redacted connect or cadence frame is enough to prove
  live frame behavior for this phase. For `/api/ws`, a successful open with no
  log frame before timeout should be recorded but not promoted as raw-log frame
  proof.
- The expected live OTAWWW response is useful evidence of the current
  fail-closed gap, not proof of whole-`www` update parity.
- Route status/body/header matching should be strict enough for evidence but
  tolerant of harmless header casing/order differences.

</specifics>

<deferred>
## Deferred Ideas

- Valid firmware OTA upload, invalid image rejection, reboot identity, rollback,
  and boot-validation evidence belong to Phase 18.
- Recovery fault-injection regressions, failed-update recovery, large erase
  recovery, interrupted-update recovery, and OTAWWW whole-`www` update behavior
  belong to Phase 19.
- Active safety hardware telemetry evidence belongs to Phase 20.
- Live production mining, accepted/rejected share behavior, watchdog
  responsiveness under mining, and bounded soak evidence belong to Phase 21.
- Network discovery, mDNS-based target selection, router scraping, AP scan
  automation, non-205 boards, all-board release images, Stratum v2, BAP,
  Angular AxeOS replacement, and production mining tuning remain out of Phase 17.

</deferred>

*Phase: 17-live-http-api-and-static-evidence*
*Context gathered: 2026-07-02*
