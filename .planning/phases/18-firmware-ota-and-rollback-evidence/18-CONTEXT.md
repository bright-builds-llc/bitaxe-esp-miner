---
generated_by: gsd-discuss-phase
lifecycle_mode: yolo
phase_lifecycle_id: 18-2026-07-03T14-06-29
generated_at: 2026-07-03T14:09:29.677Z
---

# Phase 18: Firmware OTA And Rollback Evidence - Context

**Gathered:** 2026-07-03
**Status:** Ready for planning
**Mode:** Yolo

<domain>
## Phase Boundary

Phase 18 closes the firmware OTA and rollback/boot-validation evidence gap for the current Ultra 205 firmware chain. It must prove or explicitly block valid firmware OTA upload, invalid firmware rejection, post-OTA reboot identity, selected partition or boot-validation state, and safe post-OTA operation for board `205`.

This phase does not cover recovery fault-injection regressions, OTAWWW whole-`www` update behavior, active voltage/fan/mining stress, live mining/soak, non-205 boards, all-board release images, or an Angular AxeOS rewrite. Recovery regression and OTAWWW evidence belong to Phase 19; active safety telemetry belongs to Phase 20; live mining and soak belong to Phase 21.

</domain>

<decisions>
## Implementation Decisions

### Current Package And Device Identity

- **D-01:** Treat the source commit present when Phase 18 starts as the only eligible package identity. Run `just package` and manifest-backed release-gate validation before any OTA upload, then record source commit, reference commit, package manifest path, firmware OTA image path, checksum, board, selected port, and `DEVICE_URL` provenance in Phase 18 evidence.
- **D-02:** Do not reuse Phase 17 OTA route-presence evidence as valid OTA proof. Phase 17 proved route reachability only; Phase 18 must upload the firmware image or record an exact blocker.
- **D-03:** Start hardware work with `just detect-ultra205`. Continue only when exactly one likely ESP32-S3 serial port is found and `espflash board-info --chip esp32s3 --port <port> --non-interactive` succeeds for board `205`.
- **D-04:** Live HTTP/OTA work requires an explicit origin-only `DEVICE_URL` for the just-flashed device, either supplied directly or loaded from a redaction-reviewed target lock or flash evidence artifact. Do not scan the network, infer targets from ARP/mDNS/router state, or guess from serial logs.

### Valid And Invalid Firmware OTA Evidence

- **D-05:** Reuse the existing firmware OTA helper pattern where practical, but create Phase 18 evidence under `docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/` with phase-specific summaries, logs, sanitized HTTP artifacts, and redaction review.
- **D-06:** Invalid firmware upload evidence should use a fixed local invalid image fixture and record HTTP status, sanitized body marker, expected rejection text, and conclusion. It is failed-update or invalid-rejection proof only; it is not rollback proof.
- **D-07:** Valid firmware OTA evidence must upload the manifest-listed `esp-miner.bin` to `/api/system/OTA`, record response status/body marker, reboot scheduling behavior, post-reboot serial or HTTP identity, selected partition or explicit unavailable state, and safe post-OTA operation.
- **D-08:** If a valid OTA upload would upload the same current image, that is acceptable for evidence only when the manifest checksum, public response, reboot identity, and boot-validation marker still prove the OTA path exercised the next app partition behavior. If the firmware refuses same-version/self-image updates, record the exact public/internal status and keep valid OTA below verified.

### Rollback And Boot-Validation Gate

- **D-09:** Prefer non-destructive boot-validation evidence from the firmware adapter and serial logs: `ota_boot_validation=marked_valid`, `ota_boot_validation=not_pending state=...`, selected partition markers, source commit identity, and safe state after reboot.
- **D-10:** Do not run raw rollback invalidation, erase, interrupted update, or forced boot-failure commands unless the active plan documents exact commands, allow flags, stop conditions, recovery image, restore command, expected safe-state markers, and redaction requirements.
- **D-11:** A rejected invalid upload is not rollback or boot-validation proof. Rollback proof requires captured bootloader/ESP-IDF boot-validation state before and after a valid OTA or an explicitly gated rollback/fault case.
- **D-12:** If rollback cannot be safely exercised, record boot-validation evidence when available and keep destructive rollback/fault-injection behavior as a non-claim with owner, blocker, and follow-up path.

### Checklist, Release Docs, Redaction, And Verification

- **D-13:** Promote checklist rows only to the exact evidence tier supported by Phase 18 artifacts. `OTA-001`, `REL-001`, `REL-002`, `REL-003`, `REL-07`, `REL-08`, and `EVD-05` notes must distinguish valid upload, invalid rejection, reboot identity, selected partition, boot validation, rollback, and non-claims.
- **D-14:** Update release docs, requirements traceability, and phase evidence only after evidence artifacts exist. Documentation should cite commands and artifacts, not implementation existence or goals as proof.
- **D-15:** Redaction review is mandatory before commit. It must cover `DEVICE_URL`, IP addresses, MAC addresses, SSIDs, Wi-Fi credentials, pool credentials, worker secrets, API tokens, NVS secret values, raw terminal secrets, request/response bodies, serial logs, and recovery logs.
- **D-16:** Final verification must include repo-native checks for changed paths, helper tests for any modified helper scripts, `just package`, manifest-backed release-gate validation, `just parity`, `just verify-reference`, lifecycle validation, and every hardware/network command actually used. No final commit/push should happen unless `18-VERIFICATION.md` has `status: passed` and lifecycle validation passes for `18-2026-07-03T14-06-29`.

### the agent's Discretion

The agent may choose exact helper names, evidence JSON field names, timeout values, whether Phase 18 wraps or adapts Phase 13 OTA helpers, and the final plan split. Those choices must preserve explicit target input, current package identity, repo-owned ESP/esp-rs tooling, functional core plus imperative shell, read-only reference files, redaction before promotion, no standalone body `---` separators in parsed Markdown, and conservative evidence claims.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase Scope And Governance

- `.planning/ROADMAP.md` - Phase 18 goal, requirements, success criteria, gap closure, verification expectations, and recovery research flags.
- `.planning/REQUIREMENTS.md` - REL-02, REL-08, REL-07, EVD-05, and Phase 17-21 gap-closure traceability.
- `.planning/PROJECT.md` - Ultra 205 first target, ESP-IDF Rust stack, read-only reference, evidence policy, architecture, licensing, and safety constraints.
- `.planning/STATE.md` - Current project position after Phase 17 and accumulated release/evidence decisions.
- `AGENTS.md` - Repo-local autonomous Ultra 205 hardware gate, detector stop conditions, evidence metadata requirements, destructive/fault-injection limits, secret handling, and frontmatter separator rule.
- `standards/core/verification.md` - Repo-native verification and pre-commit expectations.
- `standards/core/testing.md` - Unit-test expectations for changed pure logic.
- `standards/languages/rust.md` - Rust module, naming, invariant, test, and verification guidance.

### Prior Phase Decisions And Evidence

- `.planning/phases/07-ota-filesystem-and-release-packaging/07-CONTEXT.md` - Firmware OTA, OTAWWW gap, release packaging, boot-validation, and release evidence decisions.
- `.planning/phases/13-final-ultra-205-release-evidence/13-CONTEXT.md` - Historical final-release evidence strategy, valid/invalid OTA requirements, rollback gate, and redaction decisions.
- `.planning/phases/13-final-ultra-205-release-evidence/13-REVIEW-FIX.md` - Prior fixes for invalid OTA false positives, interrupted OTA evidence, and response redaction.
- `.planning/phases/16-current-commit-release-evidence-completion/16-CONTEXT.md` - Current-commit identity, explicit `DEVICE_URL` gate, OTA route boundaries, recovery gates, and redaction policy.
- `.planning/phases/17-live-http-api-and-static-evidence/17-CONTEXT.md` - Explicit target lock, route-presence-only OTA boundary, HTTP/static/WebSocket evidence, and Phase 18 non-claim boundaries.
- `.planning/phases/17-live-http-api-and-static-evidence/17-VERIFICATION.md` - Passed Phase 17 verification and residual OTA/rollback/boot-validation gaps.
- `docs/parity/evidence/phase-17-live-http-api-and-static-evidence/summary.md` - Final Phase 17 ledger showing OTA route presence only and Phase 18 residual work.
- `docs/parity/evidence/phase-17-live-http-api-and-static-evidence/package-release-gate.md` - Latest package manifest, artifact, and release-gate evidence.
- `docs/parity/evidence/phase-17-live-http-api-and-static-evidence/target-lock.json` - Redacted explicit target-lock pattern from Phase 17.
- `docs/parity/evidence/phase-17-live-http-api-and-static-evidence/redaction-review.md` - Latest redaction review pattern and residual non-claims.

### Current Tooling And Implementation Surfaces

- `Justfile` - Human command surface for `package`, `flash-monitor`, `monitor`, `detect-ultra205`, `verify-reference`, `parity`, `build`, and `test`.
- `scripts/detect-ultra205.sh` - Required detector and board-info preflight.
- `scripts/phase13-firmware-ota-smoke.sh` - Existing firmware OTA and invalid-image live evidence helper pattern.
- `scripts/phase13-firmware-ota-smoke-test.sh` - Helper tests for missing prerequisites, invalid rejection, post-OTA markers, and redaction-sensitive cases.
- `scripts/phase17-live-http-api-smoke.sh` - Explicit target-lock and route-probe pattern, including OTA route-presence non-claims.
- `scripts/BUILD.bazel` - Bazel shell targets and tests for phase helper integration.
- `scripts/package-firmware.sh` - Canonical package wrapper behind `just package`.
- `tools/flash/src/main.rs` - Flash, monitor, flash-monitor, package manifest resolution, evidence JSON/log capture, trusted-output behavior, and wrapper command contracts.
- `tools/xtask/src/package_manifest.rs` - Package manifest v2 and artifact metadata.
- `tools/parity/src/main.rs` - Checklist validation, release/static/OTA evidence-token guards, blocker-language checks, and report command.
- `tools/parity/src/release_gate.rs` - Manifest-backed release-gate validation.
- `crates/bitaxe-api/src/update_plan.rs` - Pure firmware OTA and OTAWWW route decisions.
- `firmware/bitaxe/src/http_api.rs` - ESP-IDF HTTP route shell, firmware OTA handler, OTAWWW gap handler, access gate, and reboot scheduling.
- `firmware/bitaxe/src/ota_update.rs` - ESP-IDF firmware OTA adapter.
- `firmware/bitaxe/src/boot_validation.rs` - ESP-IDF rollback boot-validation adapter and retained log markers.
- `docs/release/ultra-205.md` - Operator release guide, OTA, rollback, recovery, current non-claims, and restore instructions.
- `docs/release/provenance-manifest.md` - Release provenance manifest and OTA artifact publication limits.
- `docs/parity/checklist.md` - Final parity audit ledger to update conservatively.

### Upstream Reference And Policy

- `reference/esp-miner/main/http_server/http_server.c` - Reference firmware OTA route, OTAWWW route, update responses, and HTTP behavior.
- `reference/esp-miner/main/http_server/openapi.yaml` - Reference API/update route contract.
- `reference/esp-miner/readme.md` - Upstream owner OTA, recovery, and update workflow expectations.
- `reference/esp-miner/flashing.md` - Upstream flash workflow expectations.
- `reference/esp-miner/partitions.csv` - Reference partition layout.
- `reference/esp-miner/merge_bin_update.sh` - Reference update image behavior.
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

- `scripts/phase13-firmware-ota-smoke.sh` already validates manifest/image prerequisites, posts invalid and valid images to `/api/system/OTA`, captures sanitized response artifacts, starts post-OTA monitor capture, and requires `firmware_commit=`, `reference_commit=`, and `ota_boot_validation=` markers before claiming a valid OTA pass.
- `scripts/phase13-firmware-ota-smoke-test.sh` already covers missing prerequisite paths, invalid-image rejection non-rollback wording, marker-present pass behavior, and marker-missing block behavior.
- `firmware/bitaxe/src/boot_validation.rs` already logs `ota_boot_validation=marked_valid`, `ota_boot_validation=marked_invalid_reboot`, and `ota_boot_validation=not_pending state=...` through retained logs.
- `firmware/bitaxe/src/http_api.rs` already registers and handles `/api/system/OTA`, schedules the post-response restart task, and routes OTA effects through the firmware adapter.
- `crates/bitaxe-api/src/update_plan.rs` already owns pure access/AP-mode/update-route decisions and upstream-visible response copy.
- `tools/parity` already rejects unsupported verified OTA/release claims that lack valid OTA, invalid image rejection, rollback/boot-validation terms, or contain blocker language.

### Established Patterns

- Package, route, update, release-gate, and checklist decisions stay host-testable where possible; ESP-IDF HTTP, OTA, reboot, boot-validation, serial, USB, and destructive effects stay in firmware/tool adapters.
- Evidence ledgers use conservative conclusions and explicit non-claims. Package, route presence, valid OTA, invalid rejection, boot validation, rollback, redaction review, and release-gate are distinct evidence classes.
- Live hardware records must name board `205`, selected port, source commit, reference commit, package manifest or firmware identity, exact commands, board-info output, logs, observed behavior, conclusion, and redaction review.
- Phase 17 introduced a redacted explicit target-lock pattern and route-specific artifact layout that Phase 18 should reuse for target provenance and private endpoint handling.

### Integration Points

- Add Phase 18 artifacts under `docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/`.
- Add or wrap a phase-specific firmware OTA helper only when needed to avoid historical Phase 13 paths and wording.
- Update `scripts/BUILD.bazel` and helper tests if helper scripts change or new phase-specific scripts are added.
- Update `docs/parity/checklist.md`, `docs/release/ultra-205.md`, requirements traceability, and redaction review only after evidence artifacts exist.
- Extend `tools/parity` tests only if Phase 18 needs stricter machine-checkable evidence semantics beyond existing OTA/release verified-row guards.

</code_context>

<specifics>
## Specific Ideas

- Preferred evidence directory: `docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/`.
- Preferred command order: `just package`, manifest-backed release gate, `just detect-ultra205`, `just flash-monitor board=205 port=<path> evidence-dir=<phase18>/serial-boot`, explicit `DEVICE_URL` or target lock validation, invalid OTA rejection, valid OTA upload, post-OTA monitor capture, post-OTA HTTP sanity, redaction review, checklist/docs updates, `just parity`, `just verify-reference`, lifecycle validation.
- Evidence should record `network_scan: disabled`, target provenance, sanitized origin, manifest path, OTA image checksum, invalid fixture checksum, upload route, status code, response marker, reboot marker, post-reboot source commit, selected partition or boot-validation marker, and safe-state conclusion.
- If live prerequisites are missing, write blocked evidence with exact missing prerequisite and keep checklist rows below `verified`; do not broaden scope to network discovery or ad hoc recovery experiments.
- If only boot-validation evidence is safe and rollback is not exercised, say that directly: boot-validation captured, destructive rollback not claimed.

</specifics>

<deferred>
## Deferred Ideas

- Recovery fault-injection, failed-update recovery beyond invalid rejection, large erase, interrupted update, and OTAWWW whole-`www` update evidence belong to Phase 19 unless a Phase 18 plan adds explicit recovery gates and the action is necessary for firmware OTA boot-validation.
- Active voltage, fan, thermal, and power-control hardware regression belongs to Phase 20.
- Live mining, share handling, watchdog responsiveness under mining load, and soak behavior belong to Phase 21.
- Non-205 boards, BM1370/BM1368/BM1397, all-board factory images, Stratum v2, BAP, and an Angular AxeOS replacement remain outside Phase 18.

</deferred>

***

*Phase: 18-firmware-ota-and-rollback-evidence*
*Context gathered: 2026-07-03*
