---
generated_by: gsd-discuss-phase
lifecycle_mode: yolo
phase_lifecycle_id: 16-2026-07-01T12-36-46
generated_at: 2026-07-01T12:36:46.631Z
---

# Phase 16: Current Commit Release Evidence Completion - Context

**Gathered:** 2026-07-01
**Status:** Ready for planning
**Mode:** Yolo

<domain>
## Phase Boundary

Phase 16 closes the current release-candidate evidence gap for the exact source
commit that is present when this phase runs. It must produce same-commit package,
flash, serial boot, live HTTP/static/recovery/OTA, rollback, erase,
failed-update, interrupted-update, parity, release documentation, and redaction
evidence for Ultra 205 board `205`, or record precise blockers without promoting
unsupported claims.

This phase does not expand V1 to non-205 boards, additional ASIC families,
Stratum v2, BAP, all-board release images, an Angular AxeOS rewrite, active
voltage/fan/mining stress, or unbounded failure injection. Hardware and
destructive recovery work must stop unless the detector gate, package identity,
explicit `DEVICE_URL`, recovery image, documented recovery path, allow flags,
and redaction requirements are all satisfied.

</domain>

<decisions>
## Implementation Decisions

### Same-Commit Release Identity

- **D-01:** Treat the current source commit as the only release-candidate identity for this phase. Run `just package` after the phase starts, record the current source commit, reference commit, package manifest, artifact paths, checksums, install notes, and release-gate result, then flash and monitor artifacts from that same manifest.
- **D-02:** Do not reuse Phase 13 package, serial, HTTP, OTA, or recovery evidence as current release proof. Phase 13 source commit `190849539700b8f9a7909fd2b6ebd84142557968` is historical evidence and must remain below current-commit release parity unless the current commit matches it.
- **D-03:** Package-to-hardware identity is a hard gate. If `flash-monitor` evidence, HTTP probes, OTA runs, or recovery runs point at a different source commit, unknown manifest, stale factory image, wrong board, or untrusted wrapper output, record the mismatch and keep affected rows below `verified`.

### Live Device URL And HTTP/OTA Evidence

- **D-04:** Require an explicit reachable `DEVICE_URL` for live HTTP/static/recovery/OTA evidence. The phase must not infer the target through network scanning; it may accept `DEVICE_URL` from environment, argument, or a documented file-backed source if the artifact records how the target was identified.
- **D-05:** Live HTTP evidence must cover `/`, `/assets/app.css.gz`, a missing static path redirect/body, `/recovery`, API route coexistence, `/api/ws`, `/api/ws/live`, `/api/system/OTA`, and `/api/system/OTAWWW`. Each probe should record method, path, sanitized target, status, selected headers, sanitized response snippet or marker, expected result, actual result, and conclusion.
- **D-06:** Firmware OTA evidence must cover a valid `esp-miner.bin` upload, invalid image rejection, AP/APSTA rejection when applicable, public response/status, selected next partition or explicit unavailable state, reboot scheduling, post-reboot firmware identity, and boot-validation or rollback state.
- **D-07:** OTAWWW remains the REL-03 static-update gap unless this phase captures whole-`www` partition update behavior, recovery access, and interrupted-update hardware-regression evidence. A `www.bin` package artifact or `Wrong API input` response alone must not verify OTAWWW parity.

### Rollback, Erase, Failed-Update, And Interrupted-Update Gates

- **D-08:** Destructive and fault-injection evidence may run only through documented, phase-owned helpers or repo-owned commands that name exact commands, prerequisites, allow flags, abort conditions, recovery steps, expected safe-state markers, and evidence output paths.
- **D-09:** Before any erase, rollback, failed-update, interrupted-update, or raw recovery operation, rerun `just detect-ultra205`, require exactly one selected port, require successful `espflash board-info --chip esp32s3 --port <port> --non-interactive`, require board `205`, require the current package manifest and factory image, and record the detector output.
- **D-10:** Large erase evidence must record the erase command, tool version, factory reflash command, monitor command, post-erase boot identity, safe state, SPIFFS/static/recovery/API reachability, and final restore conclusion. If any prerequisite is missing, write pending evidence and do not run erase.
- **D-11:** Failed-update and interrupted-update evidence must record route, artifact name and checksum, failure or interruption point, public status/body, post-failure partition/static/API state, recovery steps, final safe state, and redaction review. Invalid upload rejection is not rollback proof by itself.

### Checklist, Release Docs, Redaction, And Final Verification

- **D-12:** Promote checklist rows only to the exact evidence tier supported by the current Phase 16 artifacts. Static, recovery, firmware OTA, rollback, erase, failed-update, interrupted-update, and release rows remain below `verified` when live evidence is blocked, partial, stale, or redaction-uncleared.
- **D-13:** Update release documentation, parity checklist, requirements traceability, milestone audit, and release-summary material only after evidence artifacts exist. Documentation should cite commands and artifacts, not restate goals as proof.
- **D-14:** Every generated log, JSON, Markdown evidence file, API response, WebSocket capture, and pasted terminal snippet must receive redaction review before commit. The review must cover pool credentials, worker secrets, Wi-Fi credentials, private endpoints, explicit `DEVICE_URL`, API tokens, NVS secret values, local terminal secrets, and recovery logs.
- **D-15:** Final verification must run the relevant repo-native checks for changed paths, `just package`, manifest-backed release gate, `just parity`, `just verify-reference`, redaction review, lifecycle validation, and any hardware/network/recovery commands the phase actually used. No final wrapper commit or push should happen unless `16-VERIFICATION.md` has `status: passed` and lifecycle validation passes for `16-2026-07-01T12-36-46`.

### the agent's Discretion

The agent may choose the exact plan split, evidence helper names, evidence
directory layout, JSON field names, sanitized HTTP output shape, checklist
wording, and whether current-commit release evidence helpers live in shell,
Rust host tools, or existing parity/flash tooling. Those choices must preserve
repo-owned ESP/esp-rs tooling, keep `reference/esp-miner` read-only, prefer
typed parsing for manifests and probe results where practical, avoid ad hoc
destructive commands, preserve functional core plus imperative shell, and avoid
standalone body `---` separators in parsed Markdown artifacts.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase Scope And Governance

- `.planning/ROADMAP.md` - Phase 16 goal, requirements, success criteria, gap closure, verification expectations, and recovery research flags.
- `.planning/REQUIREMENTS.md` - FND-06, API-09, REL-01, REL-02, REL-03, REL-04, REL-07, REL-08, and EVD-05 traceability for current-commit release evidence.
- `.planning/PROJECT.md` - Ultra 205 first target, ESP-IDF Rust stack, read-only reference, evidence policy, architecture, licensing, and safety constraints.
- `.planning/STATE.md` - Current milestone state after Phase 15 and accumulated release, safety, ASIC, mining, and evidence decisions.
- `AGENTS.md` - Repo-local autonomous Ultra 205 hardware gate, detector stop conditions, evidence metadata requirements, destructive/fault-injection limits, and frontmatter separator rule.
- `standards/core/verification.md` - Repo-native verification and pre-commit expectations.
- `standards/core/testing.md` - Unit-test expectations for changed pure logic.
- `standards/languages/rust.md` - Rust module, naming, invariant, test, and verification guidance.

### Prior Phase Decisions And Evidence

- `.planning/phases/08-parity-evidence-and-ultra-205-release-gate/08-CONTEXT.md` - Evidence governance, release gate, `DEVICE_URL` blocker policy, and destructive-test limits.
- `.planning/phases/09-flash-monitor-evidence-wrapper-hardening/09-CONTEXT.md` - Wrapper-owned noninteractive serial evidence contract and no-overclaim boundary.
- `.planning/phases/10-route-manifest-and-api-compare-unification/10-CONTEXT.md` - Phase 7 route manifest ownership, live HTTP/OTA evidence boundaries, and route overclaim guards.
- `.planning/phases/13-final-ultra-205-release-evidence/13-CONTEXT.md` - Package identity, HTTP/static/recovery/OTA, rollback, erase, failed-update, interrupted-update, checklist, and redaction decisions.
- `.planning/phases/13-final-ultra-205-release-evidence/13-VERIFICATION.md` - Historical Phase 13 verification and residual live release blockers.
- `.planning/phases/14-safety-hardware-evidence-completion/14-CONTEXT.md` - Allow-manifest pattern, destructive/fault recovery gating, live telemetry blocker policy, and final verification gate.
- `.planning/phases/14-safety-hardware-evidence-completion/14-VERIFICATION.md` - Current safety evidence boundary and residual live telemetry blockers.
- `.planning/phases/15-bm1366-mining-evidence-completion/15-CONTEXT.md` - Latest exact-claim hardware evidence policy, `DEVICE_URL` blocker, redaction expectations, and final verification gate.
- `.planning/phases/15-bm1366-mining-evidence-completion/15-VERIFICATION.md` - Latest passed Phase 15 verification and optional blockers for explicit `DEVICE_URL` and live pool prerequisites.
- `docs/parity/evidence/phase-13-final-ultra-205-release-evidence.md` - Historical package/serial evidence, missing `DEVICE_URL`, blocked HTTP/OTA/recovery, and pending destructive recovery evidence.
- `docs/parity/evidence/phase-14-safety-hardware-evidence-completion.md` - Safety evidence ledger and allow-manifest/redaction pattern.
- `docs/parity/evidence/phase-15-bm1366-mining-evidence-completion.md` - Latest BM1366/mining evidence ledger and remaining release non-claims.

### Current Tooling And Implementation Surfaces

- `Justfile` - Human command surface for `package`, `flash-monitor`, `monitor`, `detect-ultra205`, `verify-reference`, `parity`, `build`, and `test`.
- `scripts/detect-ultra205.sh` - Required detector and board-info preflight.
- `scripts/phase13-http-static-smoke.sh` - Existing explicit-`DEVICE_URL` HTTP/static/recovery/OTAWWW probe pattern.
- `scripts/phase13-firmware-ota-smoke.sh` - Existing firmware OTA and invalid-image live evidence helper pattern.
- `scripts/phase13-recovery-regression.sh` - Existing recovery, erase, failed-update, and interrupted-update helper pattern with allow flags.
- `scripts/package-firmware.sh` - Canonical package wrapper behind `just package`.
- `tools/flash/src/main.rs` - Flash, monitor, flash-monitor, package manifest resolution, evidence JSON/log capture, trusted-output behavior, and wrapper command contracts.
- `tools/xtask/src/package_manifest.rs` - Package manifest v2 and artifact metadata.
- `tools/parity/src/main.rs` - Checklist validation, evidence-token guards, release/OTA verified-row guards, and report command.
- `tools/parity/src/release_gate.rs` - Manifest-backed release-gate validation.
- `tools/parity/src/api_compare.rs` - Route policy, static/recovery/update guard behavior, and API compare checks.
- `crates/bitaxe-api/src/route_shell.rs` - `phase07_routes()` manifest for firmware OTA, OTAWWW gap, `/recovery`, and static wildcard.
- `crates/bitaxe-api/src/static_plan.rs` - Pure static and recovery request decisions.
- `crates/bitaxe-api/src/update_plan.rs` - Pure firmware OTA and OTAWWW gap decisions.
- `firmware/bitaxe/src/http_api.rs` - ESP-IDF HTTP route shell, firmware OTA handler, OTAWWW gap handler, access gate, WebSocket/API coexistence, and route registration logs.
- `firmware/bitaxe/src/ota_update.rs` - ESP-IDF firmware OTA adapter.
- `firmware/bitaxe/src/boot_validation.rs` - ESP-IDF rollback boot-validation adapter.
- `firmware/bitaxe/src/filesystem.rs` - SPIFFS mount/status adapter.
- `firmware/bitaxe/src/static_files.rs` - Static and recovery route serving adapter.
- `firmware/bitaxe/static/www/` - Rust-owned AxeOS-compatible static asset tree including `/assets/app.css.gz`.
- `firmware/bitaxe/static/recovery_page.html` - Rust-owned recovery page.
- `docs/release/ultra-205.md` - Operator release guide and current Phase 13 blocker status.
- `docs/release/license-inventory.md` - Release license inventory.
- `docs/release/provenance-manifest.md` - Release provenance manifest.
- `docs/parity/checklist.md` - Final parity audit ledger to update conservatively.

### Upstream Reference And Policy

- `reference/esp-miner/flashing.md` - Upstream flash workflow expectations.
- `reference/esp-miner/readme.md` - Upstream OTA, OTAWWW, and recovery owner workflow expectations.
- `reference/esp-miner/partitions.csv` - Reference partition layout.
- `reference/esp-miner/merge_bin.sh` - Reference merged image behavior.
- `reference/esp-miner/merge_bin_update.sh` - Reference update image behavior.
- `reference/esp-miner/main/filesystem.c` - Reference SPIFFS mount/status behavior.
- `reference/esp-miner/main/http_server/http_server.c` - Reference static, recovery, firmware OTA, OTAWWW, WebSocket, and route registration behavior.
- `reference/esp-miner/main/http_server/openapi.yaml` - Reference API/update route contract.
- `reference/esp-miner/main/http_server/recovery_page.html` - Reference recovery page behavior.
- `docs/adr/0001-device-user-parity.md` - Observable device-user parity definition.
- `docs/adr/0005-read-only-reference-implementation.md` - Read-only upstream reference policy.
- `docs/adr/0006-parity-checklist-as-audit-evidence.md` - Checklist evidence policy.
- `docs/adr/0010-axeos-api-and-asset-compatibility.md` - API/static compatibility boundary before UI rewrite.
- `docs/adr/0012-parity-verification-evidence.md` - Evidence class and verified status semantics.
- `docs/adr/0013-mit-first-with-gpl-guardrails.md` - GPL guardrails for upstream-derived behavior and fixtures.
- `docs/adr/0014-pivot-to-ultra-205-bm1366-first-parity.md` - Ultra 205/BM1366 first parity target and deferred non-205 scope.
- `PROVENANCE.md` - Reference, GPL, fixture/source-attribution, dependency-license, and firmware release review policy.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- `just detect-ultra205` already implements the required Ultra 205 detector and board-info gate.
- `just flash-monitor board=205 port=<path> evidence-dir=<path>` already records wrapper-owned JSON and serial logs with board, port, source commit, reference commit, manifest, trusted-output status, and conclusion.
- `just package` already produces the Ultra 205 package manifest and release artifacts used by `tools/flash`, `tools/parity`, and operator docs.
- Phase 13 helper scripts already model explicit `DEVICE_URL` handling, sanitized HTTP/OTA output, pending evidence when prerequisites are missing, and allow-flag-gated recovery regression.
- `tools/parity release-gate --manifest` already validates release docs and manifest-backed artifact evidence.
- `tools/parity` already rejects unsupported verified claims for release-sensitive rows, OTA/static/recovery overclaims, safety-critical rows, and active-control rows.

### Established Patterns

- Package, route, update, static, release-gate, and checklist decisions stay host-testable where possible; ESP-IDF HTTP, OTA, SPIFFS, reboot, rollback, serial, USB, and destructive effects stay in firmware/tool adapters.
- Evidence ledgers use conservative conclusions and explicit non-claims. Package, workflow, serial boot, live HTTP, firmware OTA, recovery regression, hardware-regression, release-gate, redaction review, and lifecycle validation are distinct evidence classes.
- Live hardware records must name board `205`, selected port, source commit, reference commit, package manifest or firmware identity, exact commands, board-info output, logs, observed behavior, conclusion, and redaction review.
- GSD artifacts and evidence Markdown must avoid standalone body `---` separators after YAML frontmatter.

### Integration Points

- Add Phase 16 evidence under `docs/parity/evidence/phase-16-current-commit-release-evidence-completion.md` and component-scoped subdirectories where generated artifacts reduce manual transcription risk.
- Reuse or refresh Phase 13 helpers so current-commit package identity is recorded before flash, HTTP, OTA, and recovery runs.
- Update `docs/parity/checklist.md`, `docs/release/ultra-205.md`, requirements traceability, release summary, and milestone audit only after Phase 16 evidence exists.
- Extend `tools/parity` tests only if this phase needs stricter machine-checkable semantics for current-commit release evidence, redaction, or release-sensitive checklist promotion.

</code_context>

<specifics>
## Specific Ideas

- Preferred evidence directory: `docs/parity/evidence/phase-16-current-commit-release-evidence-completion/`.
- Preferred command order: `just package`, manifest-backed release gate, `just detect-ultra205`, `just flash-monitor board=205 port=<path> evidence-dir=<phase16>/serial-boot`, explicit `DEVICE_URL` validation, HTTP/static/recovery probes, valid/invalid OTA probes, gated recovery regression, redaction review, checklist/docs updates, `just parity`, `just verify-reference`, lifecycle validation.
- Live probes should preserve sanitized target details but avoid committing private endpoint values. Record `DEVICE_URL status: blocked` with a concrete reason when a reachable target is unavailable.
- Current-commit evidence must clearly distinguish Phase 16 artifacts from historical Phase 13 evidence.
- Redaction review should explicitly say whether API responses, WebSocket frames, recovery logs, and destructive-run logs exist; absent artifacts must not be cited.

</specifics>

<deferred>
## Deferred Ideas

- Non-205 boards, BM1370/BM1368/BM1397, all-board factory images, Stratum v2, BAP, Angular AxeOS replacement, production mining optimization, active voltage/fan stress, broad runtime display/input parity, and long stress/soak runs remain out of Phase 16.
- Full OTAWWW whole-`www` update parity remains deferred unless Phase 16 captures exact whole-partition write, recovery, and interrupted-update hardware-regression evidence.
- Live pool evidence and accepted/rejected share behavior remain outside Phase 16 unless needed only as context and already covered by safe, redaction-cleared prerequisites.

</deferred>

***

*Phase: 16-current-commit-release-evidence-completion*
*Context gathered: 2026-07-01*
