---
phase: 17
phase_name: live-http-api-and-static-evidence
verified: 2026-07-03T13:56:00Z
status: passed
score: "4/4 must-haves verified"
generated_by: gsd-verifier
lifecycle_mode: interactive
phase_lifecycle_id: "17-2026-07-02T01-09-48"
generated_at: 2026-07-03T13:56:00Z
lifecycle_validated: true
overrides_applied: 0
---

# Phase 17: Live HTTP API And Static Evidence Verification Report

**Phase Goal:** The just-flashed Ultra 205 exposes live administration surfaces at an explicit `DEVICE_URL` with static asset, recovery page, API route, and WebSocket evidence.
**Verified:** 2026-07-03T13:56:00Z
**Status:** passed
**Re-verification:** Yes - after gap-closure plans `17-06` and `17-07`

## Evidence Reviewed

Primary phase artifacts reviewed:

| Artifact | Result |
| --- | --- |
| `.planning/ROADMAP.md` Phase 17 | Requires explicit reachable target evidence for live HTTP/static/recovery/API/WebSocket surfaces. |
| `.planning/REQUIREMENTS.md` | Records Phase 17 live HTTP API, static, and WebSocket gap closure for `API-09`, `REL-01`, `REL-07`, and `EVD-05`. |
| `17-01` through `17-07` plans and summaries | All plans have valid summaries; `17-06` and `17-07` close the earlier live-evidence gaps. |
| `package-release-gate.md` and `serial-boot.md` | Package, detector, board `205`, selected port, source/reference commits, trusted flash-monitor identity, Wi-Fi join, SPIFFS mount, and route registration are recorded. |
| `target-lock.json` | Present; records sanitized target provenance, `created_from_explicit_input: true`, `device_url_source: usb_flash_monitor_log`, and `network_scan: disabled`. |
| `http-static-api.md` and `http-static-api/*` | Present; records passed live HTTP/static/API route probes and per-route selected headers, redacted body snippets, and curl-error artifacts. |
| `websocket.md`, `websocket/api-ws-live.txt`, and `websocket/api-ws.txt` | Present; records bounded `/api/ws/live` and `/api/ws` frame captures with redacted snippets. |
| `redaction-review.md` | `redaction_status: passed`; cited artifacts were reviewed for raw targets, network identifiers, credentials, tokens, and secrets. |
| `docs/release/ultra-205.md`, `docs/parity/checklist.md`, and `.planning/REQUIREMENTS.md` | Updated to cite exact artifacts and preserve OTA, rollback, boot-validation, whole-`www` OTAWWW, mining, safety telemetry, and soak as non-claims. |

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
| --- | --- | --- | --- |
| 1 | Ultra 205 detector output, board `205`, selected port, source commit, reference commit, package manifest, and explicit reachable `DEVICE_URL` are recorded without network scanning or secrets. | VERIFIED | `serial-boot.md`, `package-release-gate.md`, and `target-lock.json` record the board, port, commits, trusted flash evidence, sanitized target provenance, and `network_scan: disabled`; no raw target URL is committed. |
| 2 | Live evidence captures `/`, `/assets/app.css.gz`, representative missing static behavior, `/recovery`, API route coexistence, `/api/ws`, and `/api/ws/live` from the just-flashed device. | VERIFIED | `http-static-api.md` records passed live probes for the D-08 route set, including static, recovery, system info, unknown API, WebSocket no-upgrade route coexistence, firmware OTA route-presence, and OTAWWW fail-closed response. |
| 3 | Evidence records exact commands, HTTP status and response summaries, relevant device logs, observed behavior, conclusion, and redaction review. | VERIFIED | `summary.md`, `http-static-api/http-static-api.log`, per-route artifacts, `websocket/websocket-capture.log`, `serial-boot/flash-monitor.log`, and `redaction-review.md` record commands, observed statuses, conclusions, and redaction review. |
| 4 | Release docs, parity checklist, and requirements traceability are updated without marking rows `verified` unless evidence criteria are met. | VERIFIED | `docs/release/ultra-205.md`, `docs/parity/checklist.md`, and `.planning/REQUIREMENTS.md` cite exact Phase 17 artifacts; `OTA-001`, `OTA-002`, and `REL-003` remain below verified or deferred where broader OTA behavior was not observed. |

**Score:** 4/4 truths verified

## Required Artifacts

| Artifact | Expected | Status | Details |
| --- | --- | --- | --- |
| `package-release-gate/bitaxe-ultra205-package.json` | Current package manifest for the flashed evidence set | VERIFIED | Present; source/reference commits match the trusted flash evidence used for Phase 17. |
| `serial-boot/flash-command-evidence.json` | Wrapper-owned flash-monitor identity | VERIFIED | Present with board `205`, trusted output, commit-ready redaction metadata, and matching commits. |
| `target-lock.json` | Sanitized target lock from trusted explicit target source | VERIFIED | Present with `created_from_explicit_input: true`, `device_url_source: usb_flash_monitor_log`, `network_scan: disabled`, and redacted URL. |
| `http-static-api.md` | Live HTTP/static/API route ledger | VERIFIED | Present and records `http_static_api_status: passed`, `target_lock_status: present`, and route-level claim boundaries. |
| `http-static-api/*.headers.txt`, `*.body.txt`, `*.curl-error.txt` | Per-route HTTP artifacts for the D-08 route set | VERIFIED | Present for `root`, `app-css-gz`, `missing-static`, `recovery`, `system-info`, `unknown-api`, `api-ws`, `api-ws-live`, `firmware-ota`, and `otawww`. |
| `websocket/api-ws-live.txt` | Bounded `/api/ws/live` frame capture | VERIFIED | Present with `websocket_open_status=opened` and `websocket_frame_status=passed frames=3`. |
| `websocket/api-ws.txt` | Bounded `/api/ws` raw-log frame capture | VERIFIED | Present with `websocket_open_status=opened` and `websocket_frame_status=passed frames=1`. |
| `redaction-review.md` | Final redaction review | VERIFIED | Present with `redaction_status: passed`. |
| `docs/release/ultra-205.md`, `docs/parity/checklist.md`, `.planning/REQUIREMENTS.md` | Conservative traceability updates | VERIFIED | Present and cite exact Phase 17 artifacts without promoting unobserved OTA, rollback, mining, safety telemetry, or soak behavior. |

## Key Link Verification

| From | To | Via | Status | Details |
| --- | --- | --- | --- | --- |
| Package manifest | Flash evidence JSON | Matching source/reference commits | WIRED | `summary.md` and `serial-boot.md` record matching package and flash identity for source commit `9a2bf5850ea042731f6a7947cc7eb04dc4589e90`. |
| Flash evidence | `target-lock.json` | Trusted USB flash-monitor `device_url` source | WIRED | Target lock records `device_url_source: usb_flash_monitor_log`, `created_from_explicit_input: true`, and `network_scan: disabled`. |
| Target lock | HTTP artifacts | `phase17-live-http-api-smoke.sh` | WIRED | `http-static-api.md` and `http-static-api.log` record `identity_status: passed`, `target_status: passed`, and `http_static_api_status: passed`. |
| Target lock | WebSocket frame artifacts | `phase17-websocket-capture.mjs` | WIRED | `websocket.md`, `api-ws-live.txt`, and `api-ws.txt` record target status passed and bounded frame evidence. |
| Summary/redaction | Release docs/checklist/requirements | Exact artifact citations only | WIRED | Traceability files cite Phase 17 artifacts and preserve explicit non-claims for unsupported release-sensitive behavior. |

## Data-Flow Trace

| Artifact | Data Variable | Source | Produces Real Data | Status |
| --- | --- | --- | --- | --- |
| `serial-boot/flash-command-evidence.json` | source/reference identity | Package manifest and wrapper serial output | Yes | FLOWING |
| `target-lock.json` | sanitized target provenance | Trusted USB flash-monitor device URL source | Yes, redacted | FLOWING |
| `http-static-api.md` | live route statuses | `scripts/phase17-live-http-api-smoke.sh` | Yes | FLOWING |
| `websocket.md` | WebSocket open/frame statuses | `scripts/phase17-websocket-capture.mjs` | Yes | FLOWING |
| `docs/parity/checklist.md` | parity row status | Phase 17 summary and route/frame artifacts | Yes | VERIFIED |

## Behavioral Spot-Checks

| Behavior | Command | Result | Status |
| --- | --- | --- | --- |
| Helper syntax | `bash -n scripts/phase17-live-http-api-smoke.sh scripts/phase17-live-http-api-smoke-test.sh` and `node --check scripts/phase17-websocket-capture.mjs` | Passed | PASS |
| Helper tests | `bazel test //tools/flash:tests //scripts:phase17_live_http_api_smoke_test` | Passed | PASS |
| Firmware package surface | `bazel build //firmware/bitaxe:firmware_image` | Passed | PASS |
| Parity checklist validity | `just parity` | `validation_errors: none` | PASS |
| Reference cleanliness | `just verify-reference` | `reference clean: c1915b0a63bfabebdb95a515cedfee05146c1d50` | PASS |
| Redaction scan | Phase 17 evidence, release docs, checklist, requirements, UAT, and verification artifacts | Passed | PASS |
| GSD lifecycle provenance | `gsd-tools verify lifecycle 17 --require-plans --require-verification --raw` | `valid` | PASS |

## Requirements Coverage

| Requirement | Source Plans | Description | Status | Evidence |
| --- | --- | --- | --- | --- |
| API-09 | 17-01, 17-03, 17-04, 17-05, 17-06, 17-07 | Static AxeOS assets, recovery page, API route, and WebSocket behavior remain compatible enough for administration. | SATISFIED | Live static/API/recovery route evidence and bounded WebSocket frame evidence exist under `docs/parity/evidence/phase-17-live-http-api-and-static-evidence/`. |
| REL-01 | 17-02, 17-03, 17-05, 17-06, 17-07 | Partition, filesystem, SPIFFS/static, and recovery assets support user-facing flows. | SATISFIED | Package evidence, serial SPIFFS mount evidence, `/`, `/assets/app.css.gz`, missing-static redirect, and `/recovery` route artifacts are present. |
| REL-07 | 17-02, 17-05, 17-06, 17-07 | Build, flash, monitor, OTA route, and recovery docs are sufficient and safe. | SATISFIED | Release docs cite the current Phase 17 package, flash, target, HTTP, WebSocket, and redaction evidence while preserving valid OTA and rollback non-claims. |
| EVD-05 | 17-01 through 17-07 | Verification layers include tests, API compare, hardware smoke, and hardware evidence where appropriate. | SATISFIED | Helper tests, package/build, serial evidence, live HTTP/WebSocket artifacts, redaction scan, `just parity`, `just verify-reference`, and UAT all pass. |

No orphaned Phase 17 requirements were found beyond `API-09`, `REL-01`, `REL-07`, and `EVD-05`.

## Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| --- | --- | --- | --- | --- |
| `docs/release/ultra-205.md` | 70, 77, 83-84 | Local command examples reference `wifi-credentials=wifi-credentials.json` and local developer-raw evidence paths | Info | Intentional operator/development workflow documentation; no credential contents or raw targets are committed. |
| `websocket/api-ws.txt` | 10 | `websocket_error=connection error` after frame capture | Info | Non-blocking because the same artifact records `websocket_open_status=opened` and `websocket_frame_status=passed frames=1`. |

No blocker anti-patterns were found in the reviewed helper, evidence, release, checklist, requirements, UAT, or verification files.

## Human Verification Required

None. Phase 17's user-observable acceptance criteria are evidence-driven and were verified from committed hardware, HTTP, WebSocket, redaction, parity, and reference artifacts.

## Gaps Summary

No gaps found. Phase 17's live HTTP/static/API and WebSocket evidence gaps are closed by plans `17-06` and `17-07`.

Explicitly still outside Phase 17: valid OTA upload, invalid OTA rejection, reboot identity, selected partition, rollback, boot validation, failed-update recovery, whole-`www` OTAWWW update behavior, production mining, pool behavior, active safety telemetry, and soak evidence.

## Verification Metadata

**Verification approach:** Goal-backward, derived from `.planning/ROADMAP.md` Phase 17 success criteria.
**Must-haves source:** `.planning/ROADMAP.md` plus `17-06-SUMMARY.md` and `17-07-SUMMARY.md`.
**Lifecycle provenance:** validated.
**Automated checks:** 8 passed, 0 failed.
**Human checks required:** 0.

_Verified: 2026-07-03T13:56:00Z_
_Verifier: the agent (gsd-verifier)_
