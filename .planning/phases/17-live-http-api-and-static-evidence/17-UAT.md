---
status: complete
phase: 17-live-http-api-and-static-evidence
source:
  - 17-01-SUMMARY.md
  - 17-02-SUMMARY.md
  - 17-03-SUMMARY.md
  - 17-04-SUMMARY.md
  - 17-05-SUMMARY.md
started: 2026-07-02T21:11:41Z
updated: 2026-07-03T05:50:35Z
---

## Current Test
[testing complete]

## Tests

### 1. Evidence Helper Surface
expected: The Phase 17 helper surface is present and usable: `scripts/phase17-live-http-api-smoke.sh`, `scripts/phase17-websocket-capture.mjs`, and `scripts/BUILD.bazel` expose the HTTP/static/API smoke path, bounded WebSocket capture path, and Bazel test wiring. The helpers require an explicit origin-only `DEVICE_URL`, avoid network scanning, and write redacted artifacts.
result: pass

### 2. Package And Flash Identity Evidence
expected: `package-release-gate.md`, `serial-boot.md`, and `summary.md` show package/release-gate success, exactly one detected Ultra 205 board `205` port, trusted wrapper-owned flash-monitor output, and matching source/reference commits between package and flash evidence.
result: pass

### 3. Live HTTP Static API Evidence
expected: The final Phase 17 live run shows an explicit reachable target from trusted USB flash-monitor evidence, a sanitized `target-lock.json`, and live HTTP status/response summaries for `/`, `/assets/app.css.gz`, representative missing static behavior, `/recovery`, `/api/system/info`, `/api/ws`, `/api/ws/live`, and OTA route coexistence from the just-flashed device.
result: pass

### 4. Live WebSocket Evidence
expected: The final Phase 17 live run shows bounded `/api/ws/live` and `/api/ws` captures from the explicit target with redacted frame evidence and no raw endpoint, credential, or secret leakage.
result: pass

### 5. Redaction And Traceability
expected: `redaction-review.md`, `docs/release/ultra-205.md`, `docs/parity/checklist.md`, and `.planning/REQUIREMENTS.md` cite exact Phase 17 artifacts, mark redaction passed only for reviewed or absent-not-cited artifacts, and keep live HTTP/WebSocket/OTA rows below verified when evidence is absent.
result: pass

## Summary

total: 5
passed: 5
issues: 0
pending: 0
skipped: 0
blocked: 0

## Gaps

[none]

## Resolution Evidence

- Test 3 resolved by final live HTTP/static/API run: `target/phase17-dev-raw-usb-http-static-ws-final/http-static-api/http-static-api.log` records `identity_status: passed`, `target_status: passed`, `Content-Encoding: gzip` for `/assets/app.css.gz`, and `http_static_api_status: passed`.
- Test 3 target lock: `target/phase17-dev-raw-usb-http-static-ws-final/target-lock.json` was created from the trusted USB flash-monitor device URL source and stores only sanitized target provenance.
- Test 4 resolved by bounded WebSocket captures: `target/phase17-dev-raw-usb-http-static-ws-final/websocket-live.txt` records `/api/ws/live` frame evidence and `websocket_frame_status=passed`; `target/phase17-dev-raw-usb-http-static-ws-final/websocket-api-ws.txt` records `/api/ws` raw-log frame evidence and `websocket_frame_status=passed`.
- Trusted boot source: `target/phase17-dev-raw-usb-http-static-ws-final/serial-boot/flash-command-evidence.json` records `command_kind=flash-monitor`, `board=205`, and `trusted_output=true`.
