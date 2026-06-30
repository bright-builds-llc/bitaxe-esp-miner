---
generated_by: gsd-discuss-phase
lifecycle_mode: yolo
phase_lifecycle_id: 13-2026-06-30T14-53-46
generated_at: 2026-06-30T14:53:46.141Z
---

# Phase 13: Final Ultra 205 Release Evidence - Context

**Gathered:** 2026-06-30
**Status:** Ready for planning
**Mode:** Yolo

<domain>
## Phase Boundary

Phase 13 closes the final Ultra 205 V1 release-evidence gap. It must tie the final source commit to package artifacts, factory flash, serial boot, live HTTP/static/recovery behavior, firmware OTA behavior, recovery and rollback procedures, checklist updates, release docs, and final release-gate validation.

This phase does not expand V1 to non-205 boards, additional ASIC families, Stratum v2, BAP, an Angular UI rewrite, production mining optimization, or unbounded hardware stress. If a reachable `DEVICE_URL`, safe recovery path, board detector gate, package manifest, or redaction review is missing, the phase should record that exact blocker and keep affected parity rows below `verified`.

</domain>

<decisions>
## Implementation Decisions

### Final Package Identity And Release Gate

- **D-01:** Treat the current source commit as the release-candidate identity only after `just package` produces the Ultra 205 manifest and the manifest records source commit, reference commit, artifact paths, offsets, checksums, ESP-IDF version, Rust target, and release/license/provenance paths.
- **D-02:** Run manifest-backed release-gate validation before hardware evidence is trusted: `bazel run //tools/parity:report -- release-gate --manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json`.
- **D-03:** The final package evidence must cite `bitaxe-ultra205.elf`, `esp-miner.bin`, `www.bin`, `bitaxe-ultra205-factory.bin`, `otadata-initial.bin`, `docs/release/license-inventory.md`, and `docs/release/provenance-manifest.md` when present in the manifest. Missing release-critical artifacts are blockers, not papered-over notes.

### Hardware And Network Evidence Gates

- **D-04:** Start every live Ultra 205 hardware attempt with `just detect-ultra205`. Continue only when exactly one likely ESP USB serial port is found and `espflash board-info --chip esp32s3 --port <port> --non-interactive` succeeds.
- **D-05:** Use repo-owned commands before raw tools: `just package`, `just flash-monitor board=205 port=<path> evidence-dir=<path>`, `just monitor port=<path>`, `just parity`, and any Phase 13 evidence helper added by the plans. Direct `espflash`, `esptool.py`, erase, rollback, interrupted-update, or raw write commands are allowed only when the active plan names the exact command, recovery path, stop conditions, and evidence artifacts.
- **D-06:** Live HTTP/static/recovery/OTA probes require a reachable `DEVICE_URL` that points at the just-flashed Ultra 205. If `DEVICE_URL` is unset, ambiguous, unreachable, or appears to target the wrong device, stop the live network portion and record `DEVICE_URL status: blocked` with the reason.
- **D-07:** Evidence must prove package-to-hardware identity by flashing the same source commit and manifest that will be cited in release documents. Do not mix package artifacts from one commit with serial, HTTP, or OTA evidence from another commit.

### HTTP, Static, Recovery, And OTA Evidence

- **D-08:** Live static smoke must cover `/`, `/assets/app.css.gz`, missing static redirect behavior, `/recovery`, and coexistence with `/api/*`, `/api/ws`, `/api/ws/live`, `/api/system/OTA`, and `/api/system/OTAWWW`.
- **D-09:** Firmware OTA evidence must cover a valid `esp-miner.bin` upload to `/api/system/OTA`, invalid image rejection, public response/status, selected next app partition or explicit unavailable state, reboot scheduling, post-reboot source identity, and boot-validation or rollback logs.
- **D-10:** A rejected invalid upload is not rollback proof. Rollback requires captured post-update bootloader or boot-validation state showing a pending app marked valid or invalid and the resulting boot path.
- **D-11:** OTAWWW remains the explicit REL-03 gap with public response `Wrong API input` unless the plan adds a fully documented whole-`www` update procedure with recovery and interrupted-update hardware-regression evidence. Package generation of `www.bin` alone must never verify OTAWWW.
- **D-12:** Recovery evidence must capture `/recovery` page load, upload attempt or documented no-op gap behavior, response body, restart or recovery action, post-action static route state, and final operability conclusion.

### Destructive And Fault-Recovery Evidence

- **D-13:** Rollback, large erase, failed-update recovery, and interrupted-update recovery may run only after the active plan documents the exact recovery procedure using the current package manifest and factory image.
- **D-14:** Large erase is destructive. It must record board, port, source commit, reference commit, package manifest, factory image path, exact erase command, tool version, factory reflash command, monitor command, post-erase boot status, filesystem status, recovery route status, and API reachability.
- **D-15:** Failed-update and interrupted-update evidence must record the request route, artifact name and checksum, interruption or failure point, observed public/internal status, post-failure partition or static state, recovery steps, and final conclusion.
- **D-16:** If any destructive prerequisite is missing, write a useful pending evidence artifact and keep the corresponding checklist rows below `verified`; do not widen scope or run ad hoc recovery experiments.

### Checklist, Release Docs, Redaction, And Final Verification

- **D-17:** Update `docs/parity/checklist.md` only for claims supported by the exact evidence class. `FS-001`, `OTA-001`, `REL-001`, `REL-002`, and `REL-003` may move only as far as the captured live evidence supports; `OTA-002` remains deferred unless whole-`www` interrupted-update evidence exists.
- **D-18:** Produce a Phase 13 evidence ledger that names package manifest, source commit, reference commit, board, port, `DEVICE_URL` status, exact commands, HTTP responses, OTA/recovery observations, generated logs, checklist conclusions, and residual risks.
- **D-19:** Add or update generated machine-readable evidence only when it reduces manual transcription risk. Prose evidence must cite command outputs and artifacts, not replace them.
- **D-20:** Every generated log, JSON, Markdown evidence file, and release-summary update must receive a redaction review before commit. Do not commit pool credentials, Wi-Fi credentials, API tokens, private endpoints, NVS secret values, worker secrets, or raw terminal secrets.
- **D-21:** Final verification must include repo-native checks for changed paths plus `just package`, manifest-backed `release-gate`, `just parity`, reference cleanliness, and hardware/network evidence commands that actually ran. No commit/push should happen if verification does not pass.

### the agent's Discretion

The agent may choose the exact plan split, evidence helper script or Rust tool shape, generated JSON schema, evidence directory names, release-summary layout, checklist wording, and whether HTTP probes live in shell, Rust, or a small repo-owned script. Those choices must keep `reference/esp-miner` read-only, keep `just` and Bazel as the user/build surfaces, preserve functional core plus imperative shell, prefer typed parsing for manifests and responses, keep destructive actions phase-gated, and avoid overclaiming verified release parity.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase Scope And Requirements

- `.planning/ROADMAP.md` - Phase 13 goal, gap closure, requirements, success criteria, verification expectations, and `DEVICE_URL` research flag.
- `.planning/REQUIREMENTS.md` - FND-06, API-09, REL-01, REL-02, REL-03, REL-04, REL-08, EVD-05, and traceability for final release evidence.
- `.planning/PROJECT.md` - Ultra 205 first target, ESP-IDF Rust stack, reference, parity, hardware, architecture, licensing, and safety constraints.
- `.planning/STATE.md` - Current project position after Phase 12 and accumulated evidence decisions.
- `AGENTS.md` - Repo-local autonomous Ultra 205 hardware-verification permission, detector gate, stop conditions, evidence requirements, and frontmatter separator rule.
- `standards/core/verification.md` - Repo-native verification and pre-commit expectations.
- `standards/core/testing.md` - Unit-test expectations for changed pure logic.
- `standards/languages/rust.md` - Rust module, naming, invariant, and verification guidance.

### Prior Phase Decisions And Evidence

- `.planning/phases/07-ota-filesystem-and-release-packaging/07-CONTEXT.md` - Manifest v2, static/recovery, OTA, OTAWWW gap, release docs, and release evidence decisions.
- `.planning/phases/08-parity-evidence-and-ultra-205-release-gate/08-CONTEXT.md` - Evidence governance, `DEVICE_URL` blocker policy, destructive-test gate, and release-gate decisions.
- `.planning/phases/09-flash-monitor-evidence-wrapper-hardening/09-CONTEXT.md` - Wrapper-owned noninteractive flash-monitor evidence contract and no-overclaim boundary.
- `.planning/phases/10-route-manifest-and-api-compare-unification/10-CONTEXT.md` - Phase 7 route manifest ownership and live HTTP/OTA evidence boundaries.
- `.planning/phases/11-safety-controller-hardware-regression-evidence/11-CONTEXT.md` - Hardware gate, recovery protocol, redaction, and safety evidence boundaries.
- `.planning/phases/12-asic-and-mining-hardware-evidence/12-CONTEXT.md` - Hardware gate, safe boot, evidence ladder, redaction, and residual mining/ASIC limits.
- `.planning/phases/12-asic-and-mining-hardware-evidence/12-VERIFICATION.md` - Latest passed verification and exact residual release evidence that Phase 13 still owns.
- `docs/parity/evidence/phase-07-ota-filesystem-release.md` - Package/static/recovery/OTA rollup and required future release evidence.
- `docs/parity/evidence/phase-08-ultra-205-release-gate.md` - Final Phase 8 blocker record, including no reachable `DEVICE_URL`.
- `docs/parity/evidence/phase-08-ultra-205-release-summary.md` - Release summary and residual live evidence gaps.
- `docs/parity/evidence/phase-09-flash-monitor-evidence-wrapper-hardening.md` - Trusted wrapper serial evidence pattern.
- `docs/parity/evidence/phase-10-route-manifest-and-api-compare-unification.md` - Manifest/tooling proof and live non-claims.
- `docs/parity/evidence/phase-11-safety-controller-hardware-regression-evidence.md` - Safety evidence ledger and residual hardware limits.
- `docs/parity/evidence/phase-12-asic-and-mining-hardware-evidence.md` - Safe boot, chip-detect boundary, mining preflight, restore evidence, and residual release non-claims.

### Current Implementation And Tooling Surfaces

- `Justfile` - Human command surface for package, flash-monitor, monitor, detect-ultra205, verify-reference, parity, build, and tests.
- `scripts/detect-ultra205.sh` - Required detector and board-info preflight.
- `scripts/package-firmware.sh` - Canonical package wrapper.
- `tools/flash/src/main.rs` - Flash, monitor, flash-monitor, manifest default/factory image resolution, evidence JSON/log capture, and trusted-output behavior.
- `tools/xtask/src/package_manifest.rs` - Package manifest v2 and artifact metadata.
- `tools/parity/src/main.rs` - Checklist validation, release/OTA verified-row guards, and report command.
- `tools/parity/src/release_gate.rs` - Manifest-backed release-gate validation.
- `tools/parity/src/api_compare.rs` - Route policy and static/recovery/update overclaim guards.
- `crates/bitaxe-api/src/route_shell.rs` - `phase07_routes()` route ownership for firmware OTA, OTAWWW gap, `/recovery`, and static wildcard.
- `crates/bitaxe-api/src/static_plan.rs` - Pure static and recovery route decisions.
- `crates/bitaxe-api/src/update_plan.rs` - Pure firmware OTA and OTAWWW gap decisions.
- `firmware/bitaxe/src/http_api.rs` - ESP-IDF HTTP route shell, firmware OTA handler, OTAWWW gap handler, access gate, and WebSocket/API coexistence.
- `firmware/bitaxe/src/ota_update.rs` - ESP-IDF firmware OTA adapter.
- `firmware/bitaxe/src/boot_validation.rs` - ESP-IDF rollback boot-validation adapter.
- `firmware/bitaxe/src/filesystem.rs` - SPIFFS mount/status adapter.
- `firmware/bitaxe/src/static_files.rs` - Static and recovery route serving adapter.
- `firmware/bitaxe/static/www/` - Rust-owned AxeOS-compatible static asset tree including `/assets/app.css.gz`.
- `firmware/bitaxe/static/recovery_page.html` - Rust-owned recovery page.
- `docs/release/ultra-205.md` - Operator release guide and destructive/recovery procedure requirements.
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
- `docs/adr/0001-device-user-parity.md` - Observable behavior parity definition.
- `docs/adr/0005-read-only-reference-implementation.md` - Read-only upstream reference policy.
- `docs/adr/0006-parity-checklist-as-audit-evidence.md` - Checklist as evidence policy.
- `docs/adr/0010-axeos-api-and-asset-compatibility.md` - API/static compatibility boundary before UI rewrite.
- `docs/adr/0012-parity-verification-evidence.md` - Evidence class and verified status semantics.
- `docs/adr/0013-mit-first-with-gpl-guardrails.md` - GPL guardrails for upstream-derived behavior and fixtures.
- `docs/adr/0014-pivot-to-ultra-205-bm1366-first-parity.md` - Ultra 205/BM1366 first parity target and deferred non-205 scope.
- `PROVENANCE.md` - Provenance, SPDX, GPL, fixture/source attribution, dependency license inventory, and firmware release review policy.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- `just detect-ultra205` already performs the required board-info gate and prints `port=<path>` only after exactly one likely Ultra 205 serial port succeeds.
- `just flash-monitor board=205 port=<path> evidence-dir=<path>` already produces wrapper-owned `flash-command-evidence.json` and `flash-monitor.log` with board, port, source commit, reference commit, manifest, flash image, commands, capture status, trusted-output flag, and conclusion.
- `just package` already produces `bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json` plus app, factory, static, and OTA data artifacts.
- `tools/parity release-gate --manifest` already validates release docs and manifest-backed artifact evidence.
- `tools/parity` already rejects verified release/OTA/static rows that contain blocker language or lack required live evidence terms.
- `phase07_routes()` already declares `/api/system/OTA`, `/api/system/OTAWWW`, `/recovery`, and `/*` with the Phase 7 route kinds Phase 13 must observe live.
- Firmware logs already show route registration for `/recovery`, `/api/system/OTA`, and `/api/system/OTAWWW` during safe-boot captures.

### Established Patterns

- Package, route, update, and static decisions are testable in host Rust crates; ESP-IDF HTTP, OTA, SPIFFS, reboot, rollback, serial, and hardware effects stay in firmware or tools.
- Evidence docs use conservative conclusions and explicit non-claims. Package, compile, unit, workflow, API compare, serial boot, HTTP smoke, OTA smoke, hardware-regression, release-gate, and redaction review remain distinct evidence classes.
- Release-sensitive rows stay below `verified` until the checklist row, evidence artifact, and parity guard all support the exact claim.
- GSD and evidence Markdown must not use standalone body `---` separators after frontmatter.

### Integration Points

- Add Phase 13 evidence under `docs/parity/evidence/phase-13-final-ultra-205-release-evidence.md` plus component-scoped logs or JSON artifacts when useful.
- Add a small repo-owned HTTP/OTA evidence helper if it reduces ad hoc `curl` usage and captures sanitized responses, status codes, artifact checksums, and conclusions consistently.
- Update `docs/parity/checklist.md`, `docs/release/ultra-205.md`, and any final release summary only after the evidence artifact exists.
- Extend `tools/parity` tests only when Phase 13 needs new machine-checkable evidence semantics beyond existing release/OTA verified-row guards.

</code_context>

<specifics>
## Specific Ideas

- Preferred evidence directory: `docs/parity/evidence/phase-13-final-ultra-205-release-evidence/`.
- Preferred command order: sync final package identity, run package and release gate, detect Ultra 205, flash-monitor with evidence dir, establish `DEVICE_URL`, run static/recovery HTTP probes, run valid/invalid OTA probes, run only documented destructive/recovery procedures, update evidence/checklist/docs, then run final verification.
- HTTP evidence should record method, URL path, expected status or body marker, actual status, selected headers, sanitized response snippet, and conclusion for `/`, `/assets/app.css.gz`, a missing static path, `/recovery`, `/api/system/OTA`, and `/api/system/OTAWWW`.
- OTA evidence should record artifact checksum, upload route, status body, reboot observation, post-reboot firmware commit, partition/boot-validation log lines, and recovery action if needed.
- `DEVICE_URL` may be an environment variable, CLI argument, or file-backed config. It must not be inferred from private network scans without recording how the target was identified.
- Redaction review should be explicit even when no secrets are found.

</specifics>

<deferred>
## Deferred Ideas

- Non-205 boards, BM1370/BM1368/BM1397, all-board factory image matrices, Stratum v2, BAP, Angular AxeOS replacement, and production mining performance tuning remain out of Phase 13.
- Full OTAWWW whole-`www` update parity remains deferred unless a Phase 13 plan adds safe whole-partition write, recovery, and interrupted-update hardware-regression evidence.
- Broad runtime display/input, active voltage/fan stress, long mining soak, and unbounded failure injection remain outside this release-evidence phase unless a later roadmap phase defines recovery, safety, and evidence requirements.

</deferred>

***

*Phase: 13-final-ultra-205-release-evidence*
*Context gathered: 2026-06-30*
