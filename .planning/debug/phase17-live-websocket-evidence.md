---
status: diagnosed
trigger: "Diagnose Phase 17 UAT gap root cause only. Do not apply fixes. Do not edit source or existing evidence files. UAT gap: missing bounded /api/ws/live and /api/ws captures because DEVICE_URL/evidence artifacts were absent."
created: 2026-07-02T00:00:00-05:00
updated: 2026-07-02T21:12:20-05:00
---

## Current Focus

hypothesis: CONFIRMED - Missing explicit origin-only DEVICE_URL is the root cause of the Phase 17 WebSocket UAT gap; the helper and evidence intentionally blocked capture rather than inferring a target.
test: Compare UAT truth, ledgers, artifact directory, capture log, and no-write helper probes for both /api/ws/live and /api/ws.
expecting: All sources show that no frame/open/timeout evidence could be generated because the only allowed live target input was absent.
next_action: Return root-cause-only diagnosis without applying fixes.

## Symptoms

expected: summary.md, websocket.md, and websocket/ artifacts show bounded /api/ws/live and /api/ws captures from the explicit target with redacted frame, open, or timeout evidence and no raw endpoint, credential, or secret leakage.
actual: Agent-performed artifact check found only websocket-capture.log, no websocket/api-ws-live.txt or websocket/api-ws.txt frame artifacts, and ledgers explicitly recording websocket_status: blocked because DEVICE_URL was missing.
errors: None reported beyond missing DEVICE_URL/evidence artifacts.
reproduction: Test 4 in Phase 17 UAT.
started: Discovered during UAT.

## Eliminated

## Evidence

- timestamp: 2026-07-02T00:00:00-05:00
  checked: git merge-base HEAD d2f5cdf45dc933cb5f6cc4ec729108fc26d75254
  found: Merge base matched d2f5cdf45dc933cb5f6cc4ec729108fc26d75254.
  implication: Repository history is based on the expected commit, so diagnosis can proceed without reset or history changes.
- timestamp: 2026-07-02T00:00:00-05:00
  checked: Common bug pattern quick map.
  found: Symptoms match Environment/Config, specifically missing env var / missing runtime configuration.
  implication: Test the hypothesis that missing DEVICE_URL blocked live WebSocket artifact generation before considering implementation bugs.
- timestamp: 2026-07-02T21:12:20-05:00
  checked: .planning/phases/17-live-http-api-and-static-evidence/17-UAT.md
  found: Test 4 expected bounded /api/ws/live and /api/ws captures, but recorded an issue because only websocket-capture.log existed, no websocket/api-ws-live.txt or websocket/api-ws.txt frame artifacts existed, and the ledger recorded websocket_status blocked due to missing DEVICE_URL.
  implication: The failing UAT condition is absent frame evidence, not leakage or malformed redaction.
- timestamp: 2026-07-02T21:12:20-05:00
  checked: .planning/phases/17-live-http-api-and-static-evidence/17-04-SUMMARY.md
  found: Plan 17-04 explicitly chose the blocked path when no explicit DEVICE_URL and no explicit-input target-lock.json existed; it did not run live WebSocket commands and intentionally left both frame artifacts absent - not cited.
  implication: The missing artifacts were a planned no-target outcome, but that outcome does not satisfy the UAT truth requiring live capture artifacts.
- timestamp: 2026-07-02T21:12:20-05:00
  checked: .planning/phases/17-live-http-api-and-static-evidence/17-VERIFICATION.md
  found: Verification marked the WebSocket proof link NOT WIRED because capture commands were documented but not run; it also classified websocket.md as hollow because it contained no live data.
  implication: Independent verification already supports the mechanism that blocked evidence is conservative but insufficient.
- timestamp: 2026-07-02T21:12:20-05:00
  checked: docs/parity/evidence/phase-17-live-http-api-and-static-evidence/summary.md and websocket.md
  found: Both ledgers record websocket_status blocked, device_url_status blocked, target-lock absent, network_scan disabled, and api-ws-live/api-ws frame artifacts absent - not cited.
  implication: The evidence source of truth confirms the live WebSocket surfaces were not captured from an explicit target.
- timestamp: 2026-07-02T21:12:20-05:00
  checked: scripts/phase17-websocket-capture.mjs
  found: The script requires --device-url to parse as origin-only HTTP(S), converts it to ws/wss, and writes blocked output with websocket_frame_status=not-run when DEVICE_URL is missing or invalid.
  implication: A missing explicit DEVICE_URL is sufficient to prevent any frame capture output from proving /api/ws/live or /api/ws.
- timestamp: 2026-07-02T21:12:20-05:00
  checked: WebSocket artifact directory and websocket/websocket-capture.log
  found: The directory contains only websocket-capture.log. The log records device_url_status blocked, target_lock_status absent, network_scan disabled, capture_decision not-run, both planned frame artifacts absent - not cited, and no frame artifacts generated.
  implication: The artifact state exactly matches the missing-target blocked path and cannot satisfy Test 4's required live frame/open/timeout evidence.
- timestamp: 2026-07-02T21:12:20-05:00
  checked: No-write helper probe for /api/ws/live without --device-url
  found: The helper printed websocket_target_status=blocked - missing DEVICE_URL, websocket_open_status=blocked, and websocket_frame_status=not-run.
  implication: The script itself directly confirms missing DEVICE_URL blocks /api/ws/live before any open or frame evidence can exist.
- timestamp: 2026-07-02T21:12:20-05:00
  checked: No-write helper probe for /api/ws without --device-url
  found: The helper printed websocket_target_status=blocked - missing DEVICE_URL, websocket_open_status=blocked, and websocket_frame_status=not-run.
  implication: The same missing target input blocks raw-log WebSocket evidence before any open timeout or raw-log frame status can exist.

## Resolution

root_cause: Phase 17 Test 4 failed because Plan 17-04 only executed the conservative no-target path. No explicit origin-only DEVICE_URL and no explicit-input target-lock.json were available, so scripts/phase17-websocket-capture.mjs was never run against a live target for /api/ws/live or /api/ws. The evidence ledgers correctly recorded blocked/absent-not-cited states, but those blocked artifacts do not meet the UAT truth requiring bounded live WebSocket frame, open, or timeout artifacts.
fix: Not applied by request. Gap closure should rerun the Phase 17 target/evidence flow with an explicit origin-only DEVICE_URL, create a sanitized target-lock.json from that explicit input, run bounded captures for /api/ws/live and /api/ws, and update ledgers/redaction from the resulting artifacts.
verification: Diagnosis verified by UAT report, Phase 17 plan summary, verification report, summary ledger, websocket ledger, artifact directory inspection, websocket-capture.log, and no-write helper probes for both WebSocket paths without DEVICE_URL.
files_changed: []
