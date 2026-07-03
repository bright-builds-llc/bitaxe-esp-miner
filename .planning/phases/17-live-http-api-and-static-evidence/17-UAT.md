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
updated: 2026-07-03T02:07:48Z
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
expected: `summary.md`, `http-static-api.md`, and the `http-static-api/` artifacts show an explicit reachable `DEVICE_URL`, a sanitized `target-lock.json`, and live HTTP status/response summaries for `/`, `/assets/app.css.gz`, representative missing static behavior, `/recovery`, `/api/system/info`, `/api/ws`, `/api/ws/live`, and OTA route coexistence from the just-flashed device.
result: issue
reported: "Agent-performed artifact check found no target-lock.json, no per-route HTTP header/body/curl-error artifacts, and ledgers explicitly recording http_static_api_status: blocked because DEVICE_URL was missing."
severity: major

### 4. Live WebSocket Evidence
expected: `summary.md`, `websocket.md`, and the `websocket/` artifacts show bounded `/api/ws/live` and `/api/ws` captures from the explicit target with redacted frame, open, or timeout evidence and no raw endpoint, credential, or secret leakage.
result: issue
reported: "Agent-performed artifact check found only websocket-capture.log, no websocket/api-ws-live.txt or websocket/api-ws.txt frame artifacts, and ledgers explicitly recording websocket_status: blocked because DEVICE_URL was missing."
severity: major

### 5. Redaction And Traceability
expected: `redaction-review.md`, `docs/release/ultra-205.md`, `docs/parity/checklist.md`, and `.planning/REQUIREMENTS.md` cite exact Phase 17 artifacts, mark redaction passed only for reviewed or absent-not-cited artifacts, and keep live HTTP/WebSocket/OTA rows below verified when evidence is absent.
result: pass

## Summary

total: 5
passed: 3
issues: 2
pending: 0
skipped: 0
blocked: 0

## Gaps

- truth: "summary.md, http-static-api.md, and the http-static-api/ artifacts show an explicit reachable DEVICE_URL, a sanitized target-lock.json, and live HTTP status/response summaries for /, /assets/app.css.gz, representative missing static behavior, /recovery, /api/system/info, /api/ws, /api/ws/live, and OTA route coexistence from the just-flashed device."
  status: failed
  reason: "Agent-performed artifact check found no target-lock.json, no per-route HTTP header/body/curl-error artifacts, and ledgers explicitly recording http_static_api_status: blocked because DEVICE_URL was missing."
  severity: major
  test: 3
  artifacts: []
  missing: []
- truth: "summary.md, websocket.md, and the websocket/ artifacts show bounded /api/ws/live and /api/ws captures from the explicit target with redacted frame, open, or timeout evidence and no raw endpoint, credential, or secret leakage."
  status: failed
  reason: "Agent-performed artifact check found only websocket-capture.log, no websocket/api-ws-live.txt or websocket/api-ws.txt frame artifacts, and ledgers explicitly recording websocket_status: blocked because DEVICE_URL was missing."
  severity: major
  test: 4
  artifacts: []
  missing: []
