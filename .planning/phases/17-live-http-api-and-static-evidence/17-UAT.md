---
status: diagnosed
phase: 17-live-http-api-and-static-evidence
source:
  - 17-01-SUMMARY.md
  - 17-02-SUMMARY.md
  - 17-03-SUMMARY.md
  - 17-04-SUMMARY.md
  - 17-05-SUMMARY.md
started: 2026-07-02T21:11:41Z
updated: 2026-07-03T02:15:15Z
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
  root_cause: "Phase 17 ran the intended no-scan blocked HTTP/static/API path because no explicit origin-only DEVICE_URL was provided. The helper exited before curl probes, target-lock.json creation, and per-route artifact writes; Plan 17-03 accepted blocked evidence, but UAT requires live artifacts from a reachable just-flashed device."
  artifacts:
    - path: "scripts/phase17-live-http-api-smoke.sh"
      issue: "Correctly blocks before live probes when DEVICE_URL is missing."
    - path: ".planning/phases/17-live-http-api-and-static-evidence/17-03-PLAN.md"
      issue: "Completion gate accepted blocked evidence for a live-evidence objective."
    - path: "docs/parity/evidence/phase-17-live-http-api-and-static-evidence/http-static-api.md"
      issue: "Records blocked status instead of live route artifacts."
  missing:
    - "Provide an explicit reachable origin-only DEVICE_URL for the just-flashed Ultra 205."
    - "Generate sanitized target-lock.json from explicit target input."
    - "Capture per-route HTTP headers, bodies, curl errors, statuses, and response summaries for the required Phase 17 routes."
  debug_session: ".planning/debug/phase17-live-http-static-api-evidence.md"
- truth: "summary.md, websocket.md, and the websocket/ artifacts show bounded /api/ws/live and /api/ws captures from the explicit target with redacted frame, open, or timeout evidence and no raw endpoint, credential, or secret leakage."
  status: failed
  reason: "Agent-performed artifact check found only websocket-capture.log, no websocket/api-ws-live.txt or websocket/api-ws.txt frame artifacts, and ledgers explicitly recording websocket_status: blocked because DEVICE_URL was missing."
  severity: major
  test: 4
  root_cause: "Phase 17 ran only the conservative no-target WebSocket path because no explicit origin-only DEVICE_URL or explicit-input target-lock.json was available. The WebSocket helper was never run against /api/ws/live or /api/ws, so blocked ledgers are correct but do not satisfy UAT's bounded live frame/open/timeout evidence requirement."
  artifacts:
    - path: "docs/parity/evidence/phase-17-live-http-api-and-static-evidence/websocket.md"
      issue: "Records blocked WebSocket evidence instead of live capture evidence."
    - path: "docs/parity/evidence/phase-17-live-http-api-and-static-evidence/websocket/"
      issue: "Missing api-ws-live.txt and api-ws.txt frame artifacts."
    - path: "scripts/phase17-websocket-capture.mjs"
      issue: "Correctly requires explicit origin-only --device-url before live capture can run."
  missing:
    - "Provide an explicit reachable origin-only DEVICE_URL or target-lock generated from explicit input."
    - "Run bounded captures for /api/ws/live and /api/ws."
    - "Update WebSocket ledger and redaction review from api-ws-live.txt and api-ws.txt artifacts."
  debug_session: ".planning/debug/phase17-live-websocket-evidence.md"
