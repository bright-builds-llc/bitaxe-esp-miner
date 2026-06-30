---
phase: 13-final-ultra-205-release-evidence
reviewed: 2026-06-30T18:32:00Z
depth: standard
files_reviewed: 33
files_reviewed_list:
  - docs/parity/checklist.md
  - docs/parity/evidence/phase-13-final-ultra-205-release-evidence.md
  - docs/parity/evidence/phase-13-final-ultra-205-release-evidence/README.md
  - docs/parity/evidence/phase-13-final-ultra-205-release-evidence/detect-ultra205.log
  - docs/parity/evidence/phase-13-final-ultra-205-release-evidence/firmware-ota.md
  - docs/parity/evidence/phase-13-final-ultra-205-release-evidence/firmware-ota/firmware-ota-smoke.log
  - docs/parity/evidence/phase-13-final-ultra-205-release-evidence/firmware-ota/post-ota-monitor.log
  - docs/parity/evidence/phase-13-final-ultra-205-release-evidence/hardware-detection.md
  - docs/parity/evidence/phase-13-final-ultra-205-release-evidence/http-static-recovery.md
  - docs/parity/evidence/phase-13-final-ultra-205-release-evidence/http-static-recovery/http-static-smoke.log
  - docs/parity/evidence/phase-13-final-ultra-205-release-evidence/package-release-gate.md
  - docs/parity/evidence/phase-13-final-ultra-205-release-evidence/recovery-regression.md
  - docs/parity/evidence/phase-13-final-ultra-205-release-evidence/recovery-regression/interrupted-ota.log
  - docs/parity/evidence/phase-13-final-ultra-205-release-evidence/recovery-regression/large-erase-post-restore-monitor.log
  - docs/parity/evidence/phase-13-final-ultra-205-release-evidence/recovery-regression/large-erase.log
  - docs/parity/evidence/phase-13-final-ultra-205-release-evidence/recovery-regression/recovery-regression.log
  - docs/parity/evidence/phase-13-final-ultra-205-release-evidence/recovery-runbook.md
  - docs/parity/evidence/phase-13-final-ultra-205-release-evidence/redaction-review.md
  - docs/parity/evidence/phase-13-final-ultra-205-release-evidence/serial-boot.md
  - docs/parity/evidence/phase-13-final-ultra-205-release-evidence/serial-boot/flash-command-evidence.json
  - docs/parity/evidence/phase-13-final-ultra-205-release-evidence/serial-boot/flash-monitor.log
  - docs/release/license-inventory.md
  - docs/release/provenance-manifest.md
  - docs/release/ultra-205.md
  - scripts/BUILD.bazel
  - scripts/phase13-firmware-ota-smoke-test.sh
  - scripts/phase13-firmware-ota-smoke.sh
  - scripts/phase13-http-static-smoke-test.sh
  - scripts/phase13-http-static-smoke.sh
  - scripts/phase13-monitor-capture-test.sh
  - scripts/phase13-monitor-capture.sh
  - scripts/phase13-recovery-regression-test.sh
  - scripts/phase13-recovery-regression.sh
findings:
  critical: 0
  warning: 2
  info: 0
  total: 2
status: issues_found
---

# Phase 13: Code Review Report

**Reviewed:** 2026-06-30T18:32:00Z
**Depth:** standard
**Files Reviewed:** 33
**Status:** issues_found

## Summary

Reviewed the listed Phase 13 parity evidence, release docs, Bazel script targets, and shell helpers at standard depth after the code-review-fix commits. The current Markdown evidence remains conservative about missing `DEVICE_URL` and pending destructive/fault-injection evidence. The remaining issues are both warning-level helper bugs that can overstate future live-run evidence.

Material guidance loaded: `AGENTS.md` repo-local Ultra 205 hardware rules, `AGENTS.bright-builds.md`, `standards-overrides.md`, `standards/core/code-shape.md`, `standards/core/verification.md`, `standards/core/testing.md`, and `standards/languages/rust.md`.

## Warnings

### WR-01: Firmware OTA Smoke Accepts Any Non-200 As Invalid-Image Rejection

**File:** `scripts/phase13-firmware-ota-smoke.sh:371`

**Issue:** After uploading `invalid-firmware.bin`, the helper only blocks when the response is `200`. A `302` captive-portal redirect, `404` wrong route, proxy error, or unrelated server error is logged as `invalid image rejection conclusion: captured`, and the helper can later emit `firmware_ota_status: passed` without proving that the firmware OTA validator actually rejected the image. The current test only checks a 500 response with `Validation / Activation Error`; it does not guard the overclaim case.

**Fix:** Require an expected rejection status/body marker before logging captured invalid-image evidence, and add a regression test for an unrelated non-200 response.

```bash
invalid_image_body_has_rejection_marker() {
	local body_file="$1"

	grep -Eiq 'invalid|reject|validation|activation' "$body_file"
}

if [[ "$last_http_status" == "200" ]]; then
	block_with_reason "invalid image was not rejected"
	exit 1
fi
if ! invalid_image_body_has_rejection_marker "$body_file"; then
	block_with_reason "invalid image rejection body did not contain an OTA validation marker"
	exit 1
fi
```

### WR-02: Failed-Update Recovery Is Marked Captured Before Post-Failure Smoke Passes

**File:** `scripts/phase13-recovery-regression.sh:345`

**Issue:** `run_failed_update` writes `failed_update_status: captured` before running the post-failure HTTP/static smoke helper, and it never checks `http_static_smoke_passed` afterward. If the invalid image is rejected but the device is no longer reachable, or if the smoke helper records `http_static_status: blocked`, the evidence still says the failed-update path was captured. That can turn an incomplete recovery check into trusted-looking release evidence.

**Fix:** Run and require post-failure HTTP/static smoke before emitting captured status. Add a negative test with `PHASE13_FAKE_HTTP_SMOKE_STATUS=blocked`.

```bash
local http_static_out="${out_dir}/failed-update-http-static"
run_http_static_smoke "failed_update_post_failure" "$http_static_out"
if ! http_static_smoke_passed "$http_static_out"; then
	log_main "failed_update_status: blocked - post-failure operability not proven"
	log_main "failed update recovery steps: use recovery runbook and collect post-failure boot evidence"
	return 1
fi

log_main "failed_update_status: captured"
log_main "failed update conclusion: captured - invalid image rejection evidence is not rollback proof"
```

_Reviewed: 2026-06-30T18:32:00Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: standard_
