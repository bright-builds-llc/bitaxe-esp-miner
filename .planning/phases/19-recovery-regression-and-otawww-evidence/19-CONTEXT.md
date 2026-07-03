---
generated_by: gsd-discuss-phase
lifecycle_mode: yolo
phase_lifecycle_id: 19-2026-07-03T17-34-52
generated_at: 2026-07-03T17:34:52.677Z
---

# Phase 19: Recovery Regression And OTAWWW Evidence - Context

**Gathered:** 2026-07-03
**Status:** Ready for planning
**Mode:** Yolo

<domain>
## Phase Boundary

Phase 19 closes the recovery-regression and OTAWWW/static-update evidence gap
for the Ultra 205 release chain. It must produce bounded evidence, or explicit
below-verified documentation, for failed-update recovery beyond invalid image
rejection, large erase or factory restore, interrupted update behavior, and
whole-`www` OTAWWW/static asset update behavior.

This phase does not cover active voltage/fan/mining stress, live mining or soak
evidence, production pool behavior, non-205 boards, all-board release images,
Stratum v2, BAP, or an Angular AxeOS rewrite. Destructive or fault-injection
actions may run only after the active plan documents allow flags, detector and
board-info gates, exact commands, recovery image, stop conditions, restore path,
safe-state markers, and redaction requirements. If any prerequisite is missing,
the phase should write useful pending or blocked evidence and keep the affected
claims below `verified`.

</domain>

<decisions>
## Implementation Decisions

### Recovery And Fault-Injection Gates

- **D-01:** Start Phase 19 from the current package identity. Run `just package`
  and manifest-backed release-gate validation before citing package, recovery,
  OTAWWW, or restore evidence.
- **D-02:** Before any live hardware action, run `just detect-ultra205` and
  require exactly one likely ESP32-S3 serial port plus successful
  `espflash board-info --chip esp32s3 --port <port> --non-interactive` for
  board `205`.
- **D-03:** Failed-update, large-erase or factory-restore, and interrupted OTA
  actions require explicit phase-owned allow flags. Default no-allow behavior
  must create pending evidence without running destructive or fault-injection
  commands.
- **D-04:** Every allowed action must record abort conditions, exact commands,
  detector output, board-info output, package manifest, source commit,
  reference commit, selected port, restore command, post-action safe-state
  markers, observed behavior, conclusion, and redaction status.
- **D-05:** Do not run ad hoc raw erase, rollback, write, or interrupted-upload
  commands. Use repo-owned helpers or commands whose plan names the recovery
  path and expected safe-state markers.

### Recovery Regression Evidence Shape

- **D-06:** Reuse the Phase 16 recovery-regression helper pattern rather than
  inventing a new raw-command flow, but write Phase 19 artifacts under
  `docs/parity/evidence/phase-19-recovery-regression-and-otawww-evidence/`.
- **D-07:** Failed-update recovery beyond invalid rejection should prove that an
  intentionally invalid or failed upload leaves the device operable through
  HTTP/static/recovery/API checks. If `DEVICE_URL` or another prerequisite is
  absent, record the blocker and do not promote recovery behavior.
- **D-08:** Large erase evidence is destructive. It may be captured only when
  the factory image, manifest, selected port, detector gate, board-info gate,
  restore command, monitor capture, and post-restore HTTP/static or serial
  safe-state checks are all available.
- **D-09:** Interrupted OTA evidence must use a bounded interruption point,
  capture public status/body or curl failure, then prove post-interruption
  operability through HTTP/static/recovery/API or serial safe-state evidence.

### OTAWWW And Static Update Claim Boundary

- **D-10:** Keep OTAWWW separate from firmware OTA. Phase 18 valid upload and
  invalid firmware rejection do not prove whole-`www` static update behavior.
- **D-11:** Full OTAWWW parity may be claimed only if a plan implements or uses
  a documented whole-`www` partition update procedure with size checks, chunked
  erase/write behavior, recovery access, and interrupted-update
  hardware-regression evidence.
- **D-12:** If full OTAWWW parity cannot be proven safely in Phase 19, preserve
  it as an explicit REL-03 V1 parity gap with owner, blocker, operator impact,
  follow-up path, and current public route behavior. A `www.bin` package
  artifact or `Wrong API input` response alone must not verify OTAWWW.
- **D-13:** Static asset update evidence may cite Phase 17 live static serving
  and Phase 18 package identity only as supporting context. It must not convert
  static serving or package generation into whole-`www` update proof.

### Release Docs, Checklist, Redaction, And Verification

- **D-14:** Update release docs, parity checklist, requirements traceability,
  and evidence ledgers only after Phase 19 artifacts exist. Documentation should
  cite commands and artifact paths, not goals or implementation existence as
  proof.
- **D-15:** Promote checklist rows only to the exact evidence tier supported by
  Phase 19 artifacts. Failed-update recovery, large erase, interrupted update,
  OTAWWW, REL-03, REL-08, REL-07, API-09, and EVD-05 notes must distinguish
  verified behavior from blocked, pending, below-verified, and non-claim
  behavior.
- **D-16:** Redaction review is mandatory before commit. It must cover
  `DEVICE_URL`, IP addresses, MAC addresses, SSIDs, Wi-Fi credentials, pool
  credentials, worker secrets, API tokens, NVS secret values, raw terminal
  secrets, request and response bodies, serial logs, detector logs, board-info
  logs, and recovery logs.
- **D-17:** Final verification must include repo-native checks for changed
  paths, helper tests for any modified helper scripts, `just package`,
  manifest-backed release-gate validation, `just parity`,
  `just verify-reference`, lifecycle validation, and every hardware/network
  command actually used.

### the agent's Discretion

The agent may choose exact helper names, evidence JSON fields, timeout values,
plan split, redaction-review wording, and whether Phase 19 wraps the Phase 16
recovery helper or creates a phase-specific script. Those choices must preserve
explicit target input, current package identity, repo-owned ESP/esp-rs tooling,
functional core plus imperative shell, read-only reference files, no standalone
body `---` separators in parsed Markdown, and conservative evidence claims.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase Scope And Governance

- `.planning/ROADMAP.md` - Phase 19 goal, requirements, gap closure, success
  criteria, verification expectations, and destructive-test research flags.
- `.planning/REQUIREMENTS.md` - REL-03, REL-08, REL-07, API-09, EVD-05, and
  Phase 17-21 evidence traceability.
- `.planning/PROJECT.md` - Ultra 205 first target, ESP-IDF Rust stack,
  read-only reference, evidence policy, architecture, licensing, and safety
  constraints.
- `.planning/STATE.md` - Current project position after Phase 18 and
  accumulated release, OTA, recovery, and evidence decisions.
- `AGENTS.md` - Repo-local Ultra 205 hardware gate, detector stop conditions,
  evidence metadata requirements, destructive/fault-injection limits, secret
  handling, and frontmatter separator rule.
- `standards/core/verification.md` - Repo-native verification and pre-commit
  expectations.
- `standards/core/testing.md` - Unit-test expectations for changed pure logic.
- `standards/languages/rust.md` - Rust module, naming, invariant, test, and
  verification guidance.

### Prior Phase Decisions And Evidence

- `.planning/phases/07-ota-filesystem-and-release-packaging/07-CONTEXT.md` -
  Firmware OTA, OTAWWW gap, static/recovery behavior, release packaging, and
  release evidence decisions.
- `.planning/phases/10-route-manifest-and-api-compare-unification/10-CONTEXT.md`
  - Phase 7 route manifest ownership, OTAWWW gap route classification, and
  live HTTP/OTA evidence boundaries.
- `.planning/phases/13-final-ultra-205-release-evidence/13-CONTEXT.md` -
  Historical final-release evidence strategy, `DEVICE_URL` policy, recovery
  gates, failed-update, large-erase, interrupted-update, OTAWWW, checklist, and
  redaction decisions.
- `.planning/phases/16-current-commit-release-evidence-completion/16-CONTEXT.md`
  - Current-commit identity, explicit `DEVICE_URL` gate, recovery helper
  pattern, OTAWWW boundary, and redaction policy.
- `.planning/phases/17-live-http-api-and-static-evidence/17-CONTEXT.md` -
  Explicit target lock, live HTTP/static/recovery evidence, route-presence-only
  OTA boundary, and OTAWWW fail-closed response boundary.
- `.planning/phases/18-firmware-ota-and-rollback-evidence/18-CONTEXT.md` -
  Firmware OTA, invalid rejection, boot-validation, rollback, and Phase 19
  non-claim boundaries.
- `.planning/phases/18-firmware-ota-and-rollback-evidence/18-VERIFICATION.md`
  - Latest passed verification and residual Phase 19 recovery/OTAWWW gaps.
- `docs/parity/evidence/phase-17-live-http-api-and-static-evidence/summary.md`
  - Final live HTTP/static/recovery/API/WebSocket ledger and OTAWWW non-claim.
- `docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/summary.md`
  - Firmware OTA evidence ledger, invalid rejection, valid upload-response
  blocker, and residual recovery/OTAWWW non-claims.
- `docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/redaction-review.md`
  - Latest redaction-review pattern for target, HTTP, OTA, serial, and absent
  artifacts.

### Current Tooling And Implementation Surfaces

- `Justfile` - Human command surface for `package`, `flash`, `flash-monitor`,
  `monitor`, `detect-ultra205`, `verify-reference`, `parity`, `build`, and
  `test`.
- `scripts/detect-ultra205.sh` - Required detector and board-info preflight.
- `scripts/phase16-recovery-regression.sh` - Existing allow-flag-gated recovery
  regression helper for failed update, large erase, interrupted OTA, restore,
  and safe-state checks.
- `scripts/phase16-recovery-regression-test.sh` - Existing helper regression
  tests for pending behavior, allow gates, command ordering, and redaction.
- `scripts/phase18-firmware-ota-evidence.sh` - Latest explicit target-lock and
  firmware OTA evidence wrapper pattern.
- `scripts/phase18-firmware-ota-evidence-test.sh` - Phase 18 helper tests and
  target-lock expectations.
- `scripts/BUILD.bazel` - Bazel shell targets and tests for phase helpers.
- `scripts/package-firmware.sh` - Canonical package wrapper behind
  `just package`.
- `tools/flash/src/main.rs` - Flash, monitor, flash-monitor, package manifest
  resolution, evidence JSON/log capture, trusted-output behavior, and wrapper
  command contracts.
- `tools/xtask/src/package_manifest.rs` - Package manifest v2 and artifact
  metadata.
- `tools/parity/src/main.rs` - Checklist validation, release/static/OTA
  evidence-token guards, blocker-language checks, and report command.
- `tools/parity/src/release_gate.rs` - Manifest-backed release-gate validation.
- `tools/parity/src/api_compare.rs` - Route policy, static/recovery/update
  guard behavior, and API compare checks.
- `crates/bitaxe-api/src/route_shell.rs` - `phase07_routes()` manifest for
  firmware OTA, OTAWWW gap, `/recovery`, and static wildcard.
- `crates/bitaxe-api/src/update_plan.rs` - Pure firmware OTA and OTAWWW route
  decisions.
- `firmware/bitaxe/src/http_api.rs` - ESP-IDF HTTP route shell, firmware OTA
  handler, OTAWWW gap handler, access gate, and reboot scheduling.
- `firmware/bitaxe/src/ota_update.rs` - ESP-IDF firmware OTA adapter.
- `firmware/bitaxe/src/boot_validation.rs` - ESP-IDF rollback boot-validation
  adapter and retained log markers.
- `firmware/bitaxe/src/filesystem.rs` - SPIFFS mount/status adapter.
- `firmware/bitaxe/src/static_files.rs` - Static and recovery route serving
  adapter.
- `firmware/bitaxe/static/www/` - Rust-owned AxeOS-compatible static asset tree
  including `/assets/app.css.gz`.
- `firmware/bitaxe/static/recovery_page.html` - Rust-owned recovery page.
- `docs/release/ultra-205.md` - Operator release guide, OTA, rollback,
  recovery, current non-claims, and restore instructions.
- `docs/parity/checklist.md` - Final parity audit ledger to update
  conservatively.

### Upstream Reference And Policy

- `reference/esp-miner/main/http_server/http_server.c` - Reference firmware
  OTA route, OTAWWW route, static serving, recovery, update responses, and HTTP
  behavior.
- `reference/esp-miner/main/http_server/openapi.yaml` - Reference API/update
  route contract.
- `reference/esp-miner/readme.md` - Upstream owner OTA, OTAWWW, recovery, and
  update workflow expectations.
- `reference/esp-miner/flashing.md` - Upstream flash workflow expectations.
- `reference/esp-miner/partitions.csv` - Reference partition layout.
- `reference/esp-miner/merge_bin_update.sh` - Reference update image behavior.
- `docs/adr/0001-device-user-parity.md` - Observable device-user parity
  definition.
- `docs/adr/0005-read-only-reference-implementation.md` - Read-only upstream
  reference policy.
- `docs/adr/0006-parity-checklist-as-audit-evidence.md` - Checklist evidence
  policy.
- `docs/adr/0010-axeos-api-and-asset-compatibility.md` - API/static asset
  compatibility boundary before UI rewrite.
- `docs/adr/0012-parity-verification-evidence.md` - Evidence class and
  verified status semantics.
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

- `scripts/phase16-recovery-regression.sh` already models pending default
  behavior, allow-flag-gated failed-update, large-erase, and interrupted-OTA
  flows, detector and board-info gates, manifest/current-commit checks,
  factory restore, post-restore monitor capture, HTTP/static smoke, and
  response redaction.
- `scripts/phase16-recovery-regression-test.sh` already has fake `curl`,
  `just`, `espflash`, monitor, and HTTP/static helpers that can be reused or
  adapted for Phase 19 tests.
- `scripts/phase18-firmware-ota-evidence.sh` already validates origin-only
  `DEVICE_URL` input, target-lock output paths, phase evidence path allowlists,
  and redacted target provenance.
- `just package` and `tools/parity release-gate --manifest` already produce
  and validate current package identity before hardware evidence is cited.
- `tools/parity` already rejects unsupported verified release, OTA, static, and
  recovery claims when evidence terms are missing or blocker language is
  present.

### Established Patterns

- Package, route, update, release-gate, and checklist decisions stay
  host-testable where possible; ESP-IDF HTTP, OTA, reboot, boot-validation,
  serial, USB, erase, restore, and destructive effects stay in firmware/tool
  adapters.
- Evidence ledgers use conservative conclusions and explicit non-claims.
  Package, live HTTP/static, invalid rejection, valid OTA response, recovery
  regression, large erase, interrupted update, OTAWWW, redaction review, and
  release-gate are distinct evidence classes.
- Live hardware records must name board `205`, selected port, source commit,
  reference commit, package manifest or firmware identity, exact commands,
  board-info output, logs, observed behavior, conclusion, and redaction review.
- Phase 17 and Phase 18 introduced redacted explicit target-lock patterns and
  phase-specific evidence directories. Phase 19 should follow those patterns
  instead of committing private endpoints or raw response dumps.

### Integration Points

- Add Phase 19 artifacts under
  `docs/parity/evidence/phase-19-recovery-regression-and-otawww-evidence/`.
- Add or wrap a phase-specific recovery/OTAWWW evidence helper only when needed
  to avoid historical Phase 16 paths and wording.
- Update `scripts/BUILD.bazel` and helper tests if helper scripts change or new
  phase-specific scripts are added.
- Update `docs/parity/checklist.md`, `docs/release/ultra-205.md`,
  requirements traceability, and redaction review only after Phase 19 evidence
  artifacts exist.
- Extend `tools/parity` tests only if Phase 19 needs stricter
  machine-checkable semantics beyond existing release, static, OTA, and blocker
  guards.

</code_context>

<specifics>
## Specific Ideas

- Preferred evidence directory:
  `docs/parity/evidence/phase-19-recovery-regression-and-otawww-evidence/`.
- Preferred command order: `just package`, manifest-backed release gate,
  `just detect-ultra205`, optional `just flash-monitor board=205
  port=<path> evidence-dir=<phase19>/serial-boot`, explicit `DEVICE_URL` or
  target-lock validation, no-allow recovery helper pass, allowed failed-update
  or interrupted-update only when prerequisites are documented, optional large
  erase only with factory restore gate, OTAWWW gap or update evidence, redaction
  review, checklist/docs updates, `just parity`, `just verify-reference`, and
  lifecycle validation.
- If only no-allow evidence is safe, it should still record the exact commands
  that would be required, the omitted allow flags, the restore path, and the
  claim boundary.
- OTAWWW should remain a REL-03 gap unless Phase 19 can safely prove
  whole-`www` update plus interrupted-update recovery. The gap record should
  name owner, blocker, operator impact, follow-up path, and current public
  response behavior.
- Redaction review should explicitly mark absent artifacts as
  `absent - not cited` so future phases do not treat missing body/header/log
  files as reviewed evidence.

</specifics>

<deferred>
## Deferred Ideas

- Full OTAWWW whole-`www` update parity remains deferred if Phase 19 cannot
  safely prove whole-partition write and interrupted-update recovery.
- Active voltage, fan, thermal, and power-control telemetry evidence belongs to
  Phase 20.
- Live mining, pool behavior, share handling, watchdog responsiveness under
  mining load, and soak behavior belong to Phase 21.
- Non-205 boards, BM1370/BM1368/BM1397, all-board factory images, Stratum v2,
  BAP, and an Angular AxeOS replacement remain outside Phase 19.

</deferred>

***

*Phase: 19-recovery-regression-and-otawww-evidence*
*Context gathered: 2026-07-03*
