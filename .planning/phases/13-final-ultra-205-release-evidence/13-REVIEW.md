---
phase: 13-final-ultra-205-release-evidence
reviewed: 2026-06-30T18:01:17Z
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
  critical: 1
  warning: 6
  info: 0
  total: 7
status: issues_found
---

# Phase 13: Code Review Report

**Reviewed:** 2026-06-30T18:01:17Z
**Depth:** standard
**Files Reviewed:** 33
**Status:** issues_found

## Summary

Reviewed the Phase 13 release evidence, release docs, Bazel script targets, and shell helpers against the repo-local Ultra 205 hardware gates, Bright Builds verification/code-shape rules, and the Phase 13 evidence contract. The Markdown evidence is generally conservative about current blockers, but the live helper paths can still produce trusted-looking evidence from unsafe or under-validated runs.

Material context loaded: `AGENTS.md`, `AGENTS.bright-builds.md`, `standards-overrides.md`, `standards/core/code-shape.md`, `standards/core/verification.md`, `standards/core/testing.md`, and `standards/core/architecture.md`.

## Critical Issues

### CR-01: Large Erase Can Run Without Re-Validating The Ultra 205 Detector Gate

**File:** `scripts/phase13-recovery-regression.sh:248-272`

**Issue:** Once `--allow-large-erase` is provided and the factory image exists, the helper immediately runs `espflash erase-flash --chip esp32s3 --port "$port" --non-interactive`. It does not rerun `just detect-ultra205`, verify that exactly one detector-approved port exists, compare that port to `--port`, or run `espflash board-info` immediately before the destructive operation. Repo-local guidance requires detector-gated hardware use and phase-gated destructive verification. A stale or mistyped port can erase the wrong ESP32-S3-class device.

**Fix:** Add a destructive-operation preflight and call it before erase or other fault-injection actions. Keep the allow flag, but make it insufficient by itself.

```bash
require_ultra205_destructive_gate() {
	local detector_log="${out_dir}/detect-ultra205-before-destructive.log"
	local detected_port

	just detect-ultra205 >"$detector_log" 2>&1
	detected_port="$(awk -F= '/^port=/{print $2}' "$detector_log")"

	if [[ -z "$detected_port" || "$detected_port" != "$port" ]]; then
		log_both "$large_log" "large_erase_status: blocked - detector port mismatch"
		return 1
	fi

	espflash board-info --chip esp32s3 --port "$port" --non-interactive >>"$large_log" 2>&1
}

run_large_erase() {
	# existing allow/image checks...
	require_ultra205_destructive_gate
	espflash erase-flash --chip esp32s3 --port "$port" --non-interactive >>"$large_log" 2>&1
}
```

## Warnings

### WR-01: Failed-Update Evidence Is Marked Captured Without Proving Rejection

**File:** `scripts/phase13-recovery-regression.sh:224-245`

**Issue:** `run_failed_update` records `failed_update_status: captured` regardless of `curl_status`, HTTP status, or response body. A `200` success body, a curl failure, a 404 from the wrong target, or a route crash would all be logged under the same captured status and followed by `failed update conclusion: captured`. That can turn a non-rejection into failed-update evidence.

**Fix:** Classify the request result before writing a captured conclusion, and fail/block when the invalid image was not clearly rejected.

```bash
if [[ "$curl_status" -ne 0 ]]; then
	log_main "failed_update_status: blocked - invalid-image request failed"
	return 1
fi

if [[ "$status" == "200" ]]; then
	log_main "failed_update_status: blocked - invalid image was accepted"
	log_main "failed update recovery steps: use recovery runbook and collect post-failure boot evidence"
	return 1
fi

if ! grep -Eiq 'invalid|reject|validation|activation|error' "$body"; then
	log_main "failed_update_status: blocked - rejection body did not contain expected failure marker"
	return 1
fi

log_main "failed_update_status: captured"
```

### WR-02: Interrupted OTA Evidence Is Marked Captured Even If The Upload Completes

**File:** `scripts/phase13-recovery-regression.sh:317-332`

**Issue:** `run_interrupted_ota` always logs `interrupted_update_status: captured` after the bounded curl call. It does not reject a `200` upload response, does not reject the firmware OTA success body, and does not inspect whether the post-failure HTTP/static smoke actually passed. If the upload unexpectedly completes, the log still says the interrupted-update path was captured.

**Fix:** Treat a completed OTA response as blocked evidence, and require a post-failure operability marker before emitting captured status.

```bash
if [[ "$status" == "200" ]] && grep -Fq "Firmware update complete, rebooting now!" "$body"; then
	log_both "$interrupted_log" "interrupted_update_status: blocked - upload completed instead of interrupting"
	return 1
fi

run_http_static_smoke "interrupted_update_post_failure" "${out_dir}/interrupted-ota-http-static"
if ! grep -Fq "http_static_status: passed" "${out_dir}/interrupted-ota-http-static/http-static-smoke.log"; then
	log_both "$interrupted_log" "interrupted_update_status: blocked - post-interruption operability not proven"
	return 1
fi

log_both "$interrupted_log" "interrupted_update_status: captured"
```

### WR-03: Large-Erase Recovery Conclusion Does Not Validate Post-Restore Markers

**File:** `scripts/phase13-recovery-regression.sh:281-285`

**Issue:** After factory reflash, the helper logs `post-restore monitor result: captured` and `large_erase_conclusion: captured - factory image recovery path completed` as soon as the monitor helper exits. The monitor helper can return success for a bounded timeout and the recovery helper does not require boot identity, safe-state, SPIFFS, firmware/reference commit, or HTTP/static recovery markers. This can overstate recovery from erase.

**Fix:** Gate the conclusion on required markers in `large-erase-post-restore-monitor.log`, and keep the result pending/blocked when `DEVICE_URL` is absent or HTTP/static smoke does not pass.

```bash
for marker in \
	"firmware_commit=" \
	"reference_commit=" \
	"safe_state: mining=disabled" \
	"spiffs_mount=available"; do
	if ! grep -Fq "$marker" "$monitor_log"; then
		log_both "$large_log" "large_erase_conclusion: blocked - missing post-restore marker ${marker}"
		return 1
	fi
done

run_http_static_smoke "large_erase_post_restore" "${out_dir}/large-erase-http-static"
if [[ -n "$device_url" ]] && ! grep -Fq "http_static_status: passed" "${out_dir}/large-erase-http-static/http-static-smoke.log"; then
	log_both "$large_log" "large_erase_conclusion: blocked - post-restore HTTP/static smoke failed"
	return 1
fi
```

### WR-04: HTTP Smoke Logs Body Snippets That Are Not Actually Sanitized

**File:** `scripts/phase13-http-static-smoke.sh:116-120`

**Issue:** `body_snippet` removes nulls/CRs and truncates, but it does not redact. The caller logs the result as `sanitized_body_snippet` for every route at lines 215-217, including `/api/system/info`. A future live run can commit SSIDs, private network names, diagnostic values, or newly added sensitive API fields while labeling them sanitized.

**Fix:** Either avoid logging API response bodies entirely, or run body snippets through an explicit redactor before writing evidence.

```bash
redacted_body_snippet() {
	local body_file="$1"

	LC_ALL=C tr -d '\000\r' <"$body_file" \
		| head -c 240 \
		| sed -E 's/"(ssid|wifiPass|stratumUser|stratumPassword|stratumCert)"[[:space:]]*:[[:space:]]*"[^"]*"/"\1":"[redacted]"/g' \
		| tr '\n\t' '  '
}

# For sensitive API probes, prefer marker-only logging:
log "body_marker firmware_commit: $(grep -Fq 'firmware_commit' "$body_file" && printf present || printf missing)"
```

### WR-05: WebSocket Route Checks Pass On Server Errors

**File:** `scripts/phase13-http-static-smoke.sh:129-135`

**Issue:** The `any-non-static` status class accepts anything except `000`, `302`, and `404`. The `/api/ws` and `/api/ws/live` probes use it at lines 263-264, so an HTTP `500` route panic would pass as "not static wildcard". That weakens the HTTP/static coexistence evidence.

**Fix:** Accept only the explicit non-upgrade statuses the firmware is expected to return for a plain HTTP WebSocket probe, and add a regression fixture for `500`.

```bash
status_matches() {
	local expected="$1"
	local actual="$2"

	if [[ "$expected" == "websocket-no-upgrade" ]]; then
		[[ "$actual" == "400" || "$actual" == "426" ]]
		return
	fi

	[[ "$expected" == "$actual" ]]
}

probe_route "api-ws" "GET" "/api/ws" "websocket-no-upgrade" "WebSocket" \
	"bounded WebSocket route coexistence response, not static wildcard"
```

### WR-06: Recovery Regression Tests Do Not Cover Dangerous Success/Overclaim Cases

**File:** `scripts/phase13-recovery-regression-test.sh:170-220`

**Issue:** The tests assert the happy evidence fields for failed update and large erase command rendering, but they do not cover the cases that would prevent evidence overclaims: invalid firmware returning `200`, interrupted OTA completing successfully, detector-port mismatch before erase, missing post-restore boot markers, or HTTP/static smoke failing after a destructive action.

**Fix:** Add negative tests for each blocked classification so the helper cannot regress to captured/passed status without proof.

```bash
test_failed_update_blocks_if_invalid_image_is_accepted() {
	# Arrange: fake curl returns 200 and the OTA success body.
	# Act: run with --allow-failed-update.
	# Assert: command fails or logs failed_update_status: blocked.
}

test_large_erase_requires_detector_port_match() {
	# Arrange: detector fixture prints a different port.
	# Act: run with --allow-large-erase.
	# Assert: erase command is not invoked and evidence is blocked.
}
```

_Reviewed: 2026-06-30T18:01:17Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: standard_
