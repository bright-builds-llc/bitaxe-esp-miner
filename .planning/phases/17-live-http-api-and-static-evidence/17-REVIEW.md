---
phase: 17-live-http-api-and-static-evidence
reviewed: 2026-07-02T03:41:55Z
depth: standard
files_reviewed: 21
files_reviewed_list:
  - docs/parity/checklist.md
  - docs/parity/evidence/phase-17-live-http-api-and-static-evidence/README.md
  - docs/parity/evidence/phase-17-live-http-api-and-static-evidence/http-static-api.md
  - docs/parity/evidence/phase-17-live-http-api-and-static-evidence/http-static-api/http-static-api.log
  - docs/parity/evidence/phase-17-live-http-api-and-static-evidence/package-release-gate.md
  - docs/parity/evidence/phase-17-live-http-api-and-static-evidence/package-release-gate/bitaxe-ultra205-package.json
  - docs/parity/evidence/phase-17-live-http-api-and-static-evidence/package-release-gate/package-command.log
  - docs/parity/evidence/phase-17-live-http-api-and-static-evidence/package-release-gate/release-gate.log
  - docs/parity/evidence/phase-17-live-http-api-and-static-evidence/redaction-review.md
  - docs/parity/evidence/phase-17-live-http-api-and-static-evidence/serial-boot.md
  - docs/parity/evidence/phase-17-live-http-api-and-static-evidence/serial-boot/detect-ultra205.log
  - docs/parity/evidence/phase-17-live-http-api-and-static-evidence/serial-boot/flash-command-evidence.json
  - docs/parity/evidence/phase-17-live-http-api-and-static-evidence/serial-boot/flash-monitor.log
  - docs/parity/evidence/phase-17-live-http-api-and-static-evidence/summary.md
  - docs/parity/evidence/phase-17-live-http-api-and-static-evidence/websocket.md
  - docs/parity/evidence/phase-17-live-http-api-and-static-evidence/websocket/websocket-capture.log
  - docs/release/ultra-205.md
  - scripts/BUILD.bazel
  - scripts/phase17-live-http-api-smoke-test.sh
  - scripts/phase17-live-http-api-smoke.sh
  - scripts/phase17-websocket-capture.mjs
findings:
  critical: 1
  warning: 2
  info: 0
  total: 3
status: issues_found
---

# Phase 17: Code Review Report

**Reviewed:** 2026-07-02T03:41:55Z
**Depth:** standard
**Files Reviewed:** 21
**Status:** issues_found

## Summary

Reviewed the listed Phase 17 evidence, release docs, Bazel target, shell helper, shell tests, and WebSocket capture helper. Repo guidance applied: `AGENTS.md`, `AGENTS.bright-builds.md`, `standards/index.md`, `standards/core/code-shape.md`, `standards/core/verification.md`, `standards/core/testing.md`, `standards/languages/typescript-javascript.md`, and the local frontmatter lesson.

The docs and checklist keep the missing-`DEVICE_URL` state conservative and do not promote unsupported live HTTP, WebSocket, OTA, rollback, mining, safety telemetry, or soak claims. The findings are in the helper code that will be used when future live captures run.

## Critical Issues

### CR-01: WebSocket Frame Redaction Leaks Raw Private Endpoints And Secrets

**File:** `scripts/phase17-websocket-capture.mjs:72`

**Issue:** `redactText` only redacts selected JSON key/value fields, `http(s)`/`ws(s)` URLs, IPs, and MACs. The Phase 17 policy requires redacting private endpoints, pool credentials, worker secrets, tokens, and raw URLs before citation, and `/api/ws` is explicitly a raw-log frame surface. A non-JSON raw log frame such as `raw pool stratum+tcp://pool.example:3333 host private-bitaxe.local worker secret-token` is written unchanged to `websocket_frame_1`, so a future successful capture can commit pool endpoints or secrets into evidence. I reproduced this with the script's fake WebSocket mode; the existing test at `scripts/phase17-live-http-api-smoke-test.sh:480` only covers JSON-shaped fields and misses raw-log text.

**Fix:**
```javascript
function redactText(value) {
  return String(value)
    .replace(
      /"(ssid|wifiPass|wifiPassword|stratumUser|stratumPassword|stratumCert|poolUrl|fallbackPoolUrl|hostname|ip|ipAddress|gateway|netmask|dns|token|apiToken|apiKey|password|nvsSecret|secret)"\s*:\s*"[^"]*"/gi,
      '"$1":"[redacted]"',
    )
    .replace(/"(stratumPort|fallbackStratumPort)"\s*:\s*[0-9]+/gi, '"$1":[redacted]')
    .replace(/\bstratum\+(?:tcp|ssl):\/\/[^\s"<>]+/gi, "[redacted-stratum-url]")
    .replace(/\b(?:pool|fallbackPool|worker|stratumUser|token|password|secret)\s*[:=]\s*[^\s"<>]+/gi, "$1=[redacted]")
    .replace(/\b(?:[a-z0-9-]+\.)+(?:local|lan|home|internal)\b/gi, "[redacted-host]")
    .replace(/https?:\/\/[^\s"<>]+/gi, "[redacted-url]")
    .replace(/wss?:\/\/[^\s"<>]+/gi, "[redacted-url]")
    .replace(/\b(?:\d{1,3}\.){3}\d{1,3}\b/g, "[redacted-ip]")
    .replace(/\b(?:[a-f0-9]{2}:){5}[a-f0-9]{2}\b/gi, "[redacted-mac]");
}
```

Add a regression test with a non-JSON raw `/api/ws` frame containing a stratum URL, a bare private hostname, and token/password-shaped text, then assert none of those raw values appear in the output. Mirror the same redaction coverage in `scripts/phase17-live-http-api-smoke.sh:194` if HTTP artifacts can contain raw log or diagnostic text.

## Warnings

### WR-01: Invalid Origins Can Be Marked As Passed Targets

**File:** `scripts/phase17-live-http-api-smoke.sh:166`

**Issue:** `validate_origin_device_url` is string-based and accepts values that are not a clean origin, including `http://device.local//` and values with whitespace. The helper then writes `target_status: passed` and creates `target-lock.json` before curl runs. With `http://device.local//`, route probes use malformed targets such as `http://[redacted]//api/system/info`, but the target lock still says the explicit target passed. That weakens the Phase 17 no-scan/explicit-target evidence boundary.

**Fix:**
```bash
normalize_origin_device_url() {
	python3 - "$1" <<'PY'
import sys
from urllib.parse import urlsplit

value = sys.argv[1]
if any(ch.isspace() for ch in value):
    raise SystemExit(1)

parts = urlsplit(value)
if parts.scheme not in ("http", "https"):
    raise SystemExit(1)
if not parts.netloc or parts.username or parts.password:
    raise SystemExit(1)
if parts.path not in ("", "/") or parts.query or parts.fragment:
    raise SystemExit(1)

print(f"{parts.scheme}://{parts.netloc}")
PY
}

if ! base_url="$(normalize_origin_device_url "$device_url")"; then
	log "DEVICE_URL status: blocked - invalid origin-only DEVICE_URL"
	log "DEVICE_URL sanitized: $(redacted_origin "$device_url")"
	log "target_status: blocked"
	log "http_static_api_status: blocked"
	log "conclusion: blocked - DEVICE_URL must be an origin-only http:// or https:// URL without userinfo, path, query, fragment, or whitespace"
	exit 0
fi
readonly base_url
```

Add tests for `http://device.local//`, whitespace, and other parser edge cases so invalid targets do not create a passed target lock.

### WR-02: Target Lock Drops The Real Flash Evidence Port

**File:** `scripts/phase17-live-http-api-smoke.sh:571`

**Issue:** The helper writes `selected_port` to `target-lock.json`, but it reads only the `selected_port` field from flash evidence. The real Phase 17 flash evidence uses `"port": "/dev/cu.usbmodem1101"` at `docs/parity/evidence/phase-17-live-http-api-and-static-evidence/serial-boot/flash-command-evidence.json:5`. As a result, a future successful HTTP run using the real flash JSON will omit the selected serial port from `target-lock.json`, even though the target lock is later treated as explicit target provenance for WebSocket evidence.

**Fix:**
```bash
selected_port="$(json_field "$flash_evidence_json" selected_port)"
if [[ -z "$selected_port" ]]; then
	selected_port="$(json_field "$flash_evidence_json" port)"
fi
if [[ -z "$selected_port" ]]; then
	log "identity_status: blocked"
	log "identity_block_reason: flash evidence missing selected port"
	log "target_status: blocked"
	log "http_static_api_status: blocked"
	log "conclusion: blocked - flash evidence must include the selected Ultra 205 serial port"
	exit 0
fi
readonly selected_port
write_target_lock "passed" "$selected_port"
```

Update the shell test fixture to use the real `port` field, or add a separate test proving both `port` and legacy `selected_port` are handled intentionally.

## Verification

- `bazel test //scripts:phase17_live_http_api_smoke_test` passed from cache.
- `node --check scripts/phase17-websocket-capture.mjs` passed.
- Focused fake-frame redaction probe showed the CR-01 leak remains present.

_Reviewed: 2026-07-02T03:41:55Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: standard_
