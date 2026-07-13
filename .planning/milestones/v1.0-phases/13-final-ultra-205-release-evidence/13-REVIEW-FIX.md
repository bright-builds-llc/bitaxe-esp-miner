---
phase: 13
fixed_at: 2026-06-30T19:05:25Z
review_path: .planning/phases/13-final-ultra-205-release-evidence/13-REVIEW.md
iteration: 5
findings_in_scope: 14
fixed: 14
skipped: 0
status: all_fixed
---

# Phase 13: Code Review Fix Report

**Fixed at:** 2026-06-30T19:05:25Z
**Source review:** .planning/phases/13-final-ultra-205-release-evidence/13-REVIEW.md
**Iteration:** 5

**Summary:**
- Findings in scope: 14
- Fixed: 14
- Skipped: 0

## Fixed Issues

### CR-01: Large Erase Can Run Without Re-Validating The Ultra 205 Detector Gate

**Files modified:** `scripts/phase13-recovery-regression.sh`, `scripts/phase13-recovery-regression-test.sh`
**Commit:** c1b100e
**Applied fix:** Added a destructive preflight that reruns `just detect-ultra205`, requires exactly one detected `port=`, requires that port to match `--port`, and runs immediate `espflash board-info --chip esp32s3 --port "$port" --non-interactive` before large erase. Added fixture coverage that proves the gate runs before the fake erase path.

### WR-01: Failed-Update Evidence Is Marked Captured Without Proving Rejection

**Files modified:** `scripts/phase13-recovery-regression.sh`, `scripts/phase13-recovery-regression-test.sh`
**Commit:** b4b7654
**Applied fix:** Classified failed firmware update results as captured only when the helper sees an expected rejection status and a rejection marker in the response body. Blocked curl failures, `200` responses that accept invalid firmware, unexpected statuses, and missing rejection markers, with negative fixture coverage.

### WR-02: Interrupted OTA Evidence Is Marked Captured Even If The Upload Completes

**Files modified:** `scripts/phase13-recovery-regression.sh`, `scripts/phase13-recovery-regression-test.sh`
**Commit:** 263a4c7
**Applied fix:** Treated an interrupted upload that returns `200` as blocked, required post-interruption HTTP/static smoke to pass before captured evidence is logged, and added regressions for completed uploads and failed post-action smoke.

### WR-03: Large-Erase Recovery Conclusion Does Not Validate Post-Restore Markers

**Files modified:** `scripts/phase13-recovery-regression.sh`, `scripts/phase13-recovery-regression-test.sh`
**Commit:** 5a46e6b
**Applied fix:** Required `DEVICE_URL` before the destructive large-erase path can run, checked post-restore monitor markers for firmware commit, reference commit, safe-state mining disabled, and SPIFFS availability, then required post-restore HTTP/static smoke to pass. Added regressions for missing markers and failed post-restore smoke.

### WR-04: HTTP Smoke Logs Body Snippets That Are Not Actually Sanitized

**Files modified:** `scripts/phase13-http-static-smoke.sh`, `scripts/phase13-http-static-smoke-test.sh`
**Commit:** 7ac22b3
**Applied fix:** Replaced misleading sanitized body snippet logging with redacted body snippets, redacted sensitive JSON keys plus IP and MAC-like values, and added tests proving sensitive fixture values are not written to logs.

### WR-05: WebSocket Route Checks Pass On Server Errors

**Files modified:** `scripts/phase13-http-static-smoke.sh`, `scripts/phase13-http-static-smoke-test.sh`
**Commit:** 4f89741
**Applied fix:** Tightened WebSocket no-upgrade route expectations to accept only `400` or `426`, so `500` server errors block the smoke result. Added a fixture test for WebSocket server-error classification.

### WR-06: Recovery Regression Tests Do Not Cover Dangerous Success/Overclaim Cases

**Files modified:** `scripts/phase13-recovery-regression-test.sh`
**Commit:** 90ad1d2
**Applied fix:** Added explicit detector port-mismatch coverage for the destructive large-erase gate. The WR-01, WR-02, WR-03, and WR-05 fix commits also added the negative tests for accepted invalid firmware, completed interrupted OTA, failed post-action smoke, missing post-restore markers, and WebSocket server errors.

## Additional Documentation Update

**Files modified:** `docs/parity/evidence/phase-13-final-ultra-205-release-evidence.md`, `docs/parity/evidence/phase-13-final-ultra-205-release-evidence/http-static-recovery.md`, `docs/parity/evidence/phase-13-final-ultra-205-release-evidence/recovery-regression.md`, `docs/parity/evidence/phase-13-final-ultra-205-release-evidence/recovery-runbook.md`, `docs/release/ultra-205.md`
**Commit:** ba3e42f
**Applied fix:** Updated Phase 13 evidence, runbook, and release language to reflect the stricter helper classifications, destructive detector gate, post-restore marker requirements, redacted HTTP logging, and WebSocket status expectations. The documentation remains conservative and does not promote claims to verified while `DEVICE_URL` evidence is missing.

## Follow-Up Review Fixes

### FU-01: Firmware OTA Smoke Accepts Any Non-200 As Invalid-Image Rejection

**Files modified:** `scripts/phase13-firmware-ota-smoke.sh`, `scripts/phase13-firmware-ota-smoke-test.sh`
**Commit:** a160b1c
**Applied fix:** Required the invalid-image OTA response body to contain an OTA validation marker before captured evidence is logged, and added a regression where an unrelated non-200 response blocks the evidence result.

### FU-02: Failed-Update Recovery Is Marked Captured Before Post-Failure Smoke Passes

**Files modified:** `scripts/phase13-recovery-regression.sh`, `scripts/phase13-recovery-regression-test.sh`
**Commit:** a160b1c
**Applied fix:** Required post-failure HTTP/static smoke to pass before failed-update evidence can be marked captured, and added a regression where blocked post-failure smoke keeps the evidence blocked.

### FU-03: Wrong-Route Body Can Be Captured As Failed-Update Evidence

**Files modified:** `scripts/phase13-recovery-regression.sh`, `scripts/phase13-recovery-regression-test.sh`
**Commit:** 4102879
**Applied fix:** Rejected `Wrong API input` as failed-update proof, removed the generic `firmware` match from failed-update body markers, and added a regression that a wrong-route body blocks evidence.

### FU-04: Interrupted OTA Evidence Does Not Require A Real Interruption

**Files modified:** `scripts/phase13-recovery-regression.sh`, `scripts/phase13-recovery-regression-test.sh`
**Commit:** 4102879
**Applied fix:** Required interrupted OTA evidence to come from curl timeout status `28`, so fast non-timeout responses block instead of being captured. Added a fast-400 regression that proves the blocked classification.

### FU-05: OTA And Recovery Helpers Log Live Response Bodies Without Redaction

**Files modified:** `scripts/phase13-firmware-ota-smoke.sh`, `scripts/phase13-firmware-ota-smoke-test.sh`, `scripts/phase13-recovery-regression.sh`, `scripts/phase13-recovery-regression-test.sh`
**Commit:** 4102879
**Applied fix:** Redacted sensitive JSON keys, IPv4 addresses, and MAC-like values from OTA and recovery body snippets, with fixture coverage proving representative sensitive values are not written to logs.

### FU-06: Redaction Truncates Long Secret Values Before Matching Them

**Files modified:** `scripts/phase13-http-static-smoke.sh`, `scripts/phase13-http-static-smoke-test.sh`, `scripts/phase13-firmware-ota-smoke.sh`, `scripts/phase13-firmware-ota-smoke-test.sh`, `scripts/phase13-recovery-regression.sh`, `scripts/phase13-recovery-regression-test.sh`
**Commit:** e4017d2
**Applied fix:** Moved redaction before snippet truncation for all Phase 13 live-response body snippets, and added long secret fixtures proving prefixes of long sensitive values are not persisted in HTTP/static, firmware OTA, failed-update, or interrupted-OTA logs.

### FU-07: Curl Error Snippets Can Expose Hostname-Style Private Endpoints

**Files modified:** `scripts/phase13-http-static-smoke.sh`, `scripts/phase13-http-static-smoke-test.sh`, `scripts/phase13-firmware-ota-smoke.sh`, `scripts/phase13-firmware-ota-smoke-test.sh`, `scripts/phase13-recovery-regression.sh`, `scripts/phase13-recovery-regression-test.sh`
**Commit:** 5509739
**Applied fix:** Extended the shared snippet redaction to cover URL-like strings plus common curl `Could not resolve host` and `Failed to connect to` hostname messages before truncation. Added stderr fixtures for HTTP/static, firmware OTA, failed-update, and interrupted-OTA helper paths.

_Fixed: 2026-06-30T19:05:25Z_
_Fixer: the agent (gsd-code-fixer)_
_Iteration: 5_
