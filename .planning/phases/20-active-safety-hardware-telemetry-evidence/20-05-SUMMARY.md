---
phase: 20-active-safety-hardware-telemetry-evidence
plan: "05"
subsystem: parity-evidence
generated_by: gsd-execute-plan
tags:
  - live-telemetry
  - websocket
  - safety-evidence
  - redaction
  - target-lock
requires:
  - 20-02 safe baseline and blocked target lock
  - 20-03 active safety hardware observation pack
  - 20-04 failure-path evidence pack
provides:
  - Explicit blocked live API safety telemetry evidence
  - Live API/WebSocket allow manifest with network scanning disabled
  - Bounded /api/ws/live capture artifact with max_frames=5
  - Phase 20 safe-state correlation ledger
affects:
  - docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence
tech_stack:
  added: []
  patterns:
    - Conservative evidence boundary recording
    - Explicit target-only live telemetry capture
    - Bounded WebSocket frame capture
key_files:
  created:
    - docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/live-api-websocket-telemetry.md
    - docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/live-api-websocket-telemetry/allow-live-telemetry.json
    - docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/live-api-websocket-telemetry/live-telemetry.log
    - docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/live-api-websocket-telemetry/websocket/api-ws-live.txt
  modified: []
decisions:
  - Live API/WebSocket telemetry stayed blocked because neither explicit DEVICE_URL nor trusted origin-only target lock existed.
  - The WebSocket bounded-capture contract is satisfied by a target-scoped missing-target artifact with duration_ms=10000 and max_frames=5.
  - Phase 20 safe-state evidence is cited only as correlation context, not as live telemetry freshness or cadence proof.
requirements_completed:
  - SAFE-01
  - SAFE-02
  - SAFE-04
  - SAFE-07
  - SAFE-08
  - SAFE-09
  - EVD-05
metrics:
  tasks_completed: 2
  files_changed: 4
  started_at: 2026-07-03T23:04:07Z
  completed_at: 2026-07-03T23:11:51Z
  duration: 7m44s
generated_at: 2026-07-03T23:11:51Z
lifecycle_mode: yolo
phase_lifecycle_id: 20-2026-07-03T20-48-00
---

# Phase 20 Plan 05: Live API/WebSocket Safety Telemetry Evidence Summary

Explicit-target live API/WebSocket safety telemetry evidence is blocked with a bounded `/api/ws/live` capture contract and Phase 20 safe-state correlation.

## Performance Summary

| Metric | Value |
| --- | --- |
| Tasks completed | 2/2 |
| Files changed | 4 |
| Duration | 7m44s |
| Final status | Complete |

## Accomplishments

- Created a live telemetry allow manifest for the API/WebSocket safety surface with `network_scan: disabled`, the `api-websocket-projection` claim tier, and checklist rows `API-002`, `API-006`, `STAT-002`, `PWR-006`, `THR-001`, `THR-002`, and `SAFE-07`.
- Recorded blocked HTTP telemetry evidence because no explicit `DEVICE_URL` and no trusted origin-only target lock were available. The helper wrote a clear missing-target status instead of discovering a target.
- Recorded bounded WebSocket evidence for `/api/ws/live` with `duration_ms=10000` and `max_frames=5`. With no explicit target, the artifact records a missing-target block rather than opening a socket.
- Wrote a correlation ledger tying the live telemetry boundary to Phase 20 safe-state, active power/voltage, thermal/fan, self-test/watchdog/load, display/input, and failure-path evidence without upgrading route presence or stale data into live telemetry proof.

## Task Commits

| Task | Name | Commit | Files |
| --- | --- | --- | --- |
| 1 | Capture explicit-target HTTP safety telemetry or blocked evidence | `2ffdd1b` | `live-api-websocket-telemetry.md`, `allow-live-telemetry.json`, `live-telemetry.log` |
| 2 | Capture bounded live WebSocket frames and write correlation ledger | `71345ca` | `live-api-websocket-telemetry.md`, `websocket/api-ws-live.txt` |

## Files Created

- `docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/live-api-websocket-telemetry.md` - conservative ledger for API telemetry status, WebSocket status, correlation, and non-claims.
- `docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/live-api-websocket-telemetry/allow-live-telemetry.json` - allow manifest for the target-scoped live telemetry helper.
- `docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/live-api-websocket-telemetry/live-telemetry.log` - HTTP helper output showing the explicit target prerequisite and disabled target discovery.
- `docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/live-api-websocket-telemetry/websocket/api-ws-live.txt` - bounded WebSocket helper artifact showing `/api/ws/live`, `duration_ms=10000`, `max_frames=5`, and missing target status.

## Decisions Made

- Live telemetry remains blocked unless an explicit `DEVICE_URL` or trusted origin-only target lock is provided.
- The WebSocket contract is satisfied for this plan by a bounded, target-scoped blocked artifact because the helper records the exact capture path, duration, max frames, and missing target status.
- Phase 20 safe-state evidence can be used as correlation context, but not as proof of live telemetry freshness, WebSocket cadence, production mining, soak behavior, or active control behavior.

## Deviations from Plan

### Auto-fixed Issues

None - plan behavior executed as written.

### Process Adjustments

**1. Generated Task 1 evidence normalized before commit**
- **Found during:** Task 1 commit review
- **Issue:** `git diff --cached --check` flagged trailing whitespace in generated helper output and one extra blank line at EOF in the ledger.
- **Fix:** Applied mechanical whitespace normalization only.
- **Files modified:** `live-telemetry.log`, `live-api-websocket-telemetry.md`
- **Verification:** `git diff --cached --check` and the full Rust pre-commit sequence passed before commit.
- **Commit:** `2ffdd1b`

## Issues Encountered

No blocker stopped execution. Live API/WebSocket telemetry remains blocked by missing explicit target input, which is the intended conservative outcome for this plan.

## Verification

- `node /Users/peterryszkiewicz/.codex/get-shit-done/bin/gsd-tools.cjs verify lifecycle 20 --require-plans --raw` - `valid` before plan execution and again after task commits.
- `node /Users/peterryszkiewicz/.codex/get-shit-done/bin/gsd-tools.cjs verify key-links .planning/phases/20-active-safety-hardware-telemetry-evidence/20-05-PLAN.md --raw` - `valid`.
- `bazel test //scripts:phase14_live_telemetry_test` - passed.
- `node scripts/phase17-websocket-capture.mjs --help` - passed.
- Targeted evidence scans for `api_telemetry_status`, `safety_telemetry_fields`, `websocket_frame_status`, `telemetry_correlation_status`, `max_frames: 5`, and `max_frames=5` - passed.
- Redaction scan over `live-api-websocket-telemetry/` - passed.
- `git diff -- reference/esp-miner --exit-code` - passed.
- `just test` - passed, 30/30 Bazel tests.
- `just parity` - passed with `validation_errors: none`.
- `just verify-reference` - passed with reference commit `c1915b0a63bfabebdb95a515cedfee05146c1d50`.
- Rust pre-commit sequence before each task commit: `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features` - passed.
- `git diff --check` - passed.

## Known Stubs

None. Stub-pattern scanning found only intentional blocked-evidence wording for missing explicit `DEVICE_URL`; that wording documents the evidence boundary and does not prevent the plan goal.

## Threat Flags

None. This plan added evidence artifacts only; it introduced no new endpoints, auth paths, schema changes, or trust-boundary code.

## Auth Gates

None.

## User Setup Required

None for this plan. Capturing live HTTP/WebSocket telemetry later still requires an explicit, trusted device target such as `DEVICE_URL` or an origin-only target lock.

## Next Phase Readiness

Plan 20-06 can consume the conservative 20-05 evidence boundary. The live telemetry freshness claim remains unavailable until a trusted explicit target produces redacted HTTP body or WebSocket frame evidence.

## Self-Check: PASSED

- Found summary file and all four plan evidence files.
- Found task commits `2ffdd1b` and `71345ca`.
