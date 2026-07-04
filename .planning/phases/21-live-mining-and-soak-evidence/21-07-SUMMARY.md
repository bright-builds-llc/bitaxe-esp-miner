---
phase: 21-live-mining-and-soak-evidence
plan: "07"
title: "Bounded Soak and Watchdog Blocked Evidence"
status: complete
created: 2026-07-04T06:16:06Z
completed: 2026-07-04T06:18:39Z
duration_seconds: 735
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 21-2026-07-04T01-35-47
generated_at: 2026-07-04T06:18:39Z
subsystem: parity-evidence
tags:
  - bounded-soak
  - watchdog
  - live-mining
  - redaction
  - blocked-evidence
requirements:
  - ASIC-07
  - STR-06
  - STR-07
  - SAFE-09
  - EVD-05
dependency_graph:
  requires:
    - 21-06
  provides:
    - blocked bounded soak evidence pack
    - blocked watchdog responsiveness ledger
    - redaction-reviewed bounded soak artifacts
  affects:
    - docs/parity/evidence/phase-21-live-mining-and-soak-evidence
    - .planning/phases/21-live-mining-and-soak-evidence/21-VALIDATION.md
tech_stack:
  added:
    - Phase 21 bounded soak evidence artifacts
  patterns:
    - exact-claim blocked evidence
    - redaction-reviewed evidence ledgers
    - prerequisite-gated hardware execution
key_files:
  created:
    - docs/parity/evidence/phase-21-live-mining-and-soak-evidence/bounded-soak.md
    - docs/parity/evidence/phase-21-live-mining-and-soak-evidence/bounded-soak/allow-bounded-soak.json
    - docs/parity/evidence/phase-21-live-mining-and-soak-evidence/bounded-soak/detect-ultra205.log
    - docs/parity/evidence/phase-21-live-mining-and-soak-evidence/bounded-soak/bounded-soak.log
    - docs/parity/evidence/phase-21-live-mining-and-soak-evidence/bounded-soak/api-system-info-snapshots.redacted.jsonl
    - docs/parity/evidence/phase-21-live-mining-and-soak-evidence/bounded-soak/websocket/api-ws-live.txt
    - docs/parity/evidence/phase-21-live-mining-and-soak-evidence/bounded-soak/watchdog-observations.md
  modified:
    - docs/parity/evidence/phase-21-live-mining-and-soak-evidence/redaction-review.md
    - .planning/phases/21-live-mining-and-soak-evidence/21-VALIDATION.md
decisions:
  - Bounded soak stays blocked when live smoke has missing_live_prerequisites, share_outcome not-run, and no controlled package boot or pool-input bridge proof.
  - SAFE-09 watchdog responsiveness remains unproven without bounded mining or soak observations; startup watchdog breadcrumbs are not bounded soak proof.
  - Blocked bounded-soak placeholders can be redaction-reviewed when they contain only status/category labels and no raw endpoint, credential, address, token, or secret values.
metrics:
  task_count: 3
  file_count: 9
  task_commits:
    - e361393
    - de46c78
    - 707e609
---

# Phase 21 Plan 07: Bounded Soak and Watchdog Blocked Evidence Summary

Bounded mining soak and watchdog responsiveness evidence were precisely blocked because Plan 21-06 left live mining smoke at `blocker: missing_live_prerequisites` with `share_outcome: not-run` and no controlled package boot, pool-input bridge, live pool command, API/WebSocket connection, soak, or share claim.

## Result

Plan 21-07 did not run soak hardware. The bounded soak ledger records `bounded_soak_status: blocked`, `live_smoke_prerequisite: failed`, `duration_seconds: 300`, and `hardware_command_status: not-run`.

The watchdog ledger records `watchdog_responsiveness_status: blocked - bounded soak not run` and explicitly preserves that startup watchdog breadcrumbs are not bounded soak proof. SAFE-09 remains below verified for live bounded mining responsiveness.

## Task Results

| Task | Status | Commit | Files |
| --- | --- | --- | --- |
| Validate and run or block bounded soak | complete | e361393 | `bounded-soak.md`, `allow-bounded-soak.json`, copied detector log, blocked run/API/WebSocket placeholders |
| Record watchdog observations | complete | de46c78 | `bounded-soak/watchdog-observations.md` |
| Close redaction and validation sampling | complete | 707e609 | `redaction-review.md`, `21-VALIDATION.md` |

## Hardware Outcome

No 21-07 hardware soak command was run. The prior detector log was copied into the bounded-soak pack as allowed by the plan, but the failed live-smoke prerequisite blocked fresh detector, mining-allow, wrapper, API, WebSocket, and soak execution for this plan.

No share, stability, watchdog responsiveness, thermal/power, telemetry correlation, safe-stop, or bounded-soak claim is promoted by this plan.

## Verification

Passed:

- `cargo fmt --all`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo build --all-targets --all-features`
- `cargo test --all-features`
- `bash -n scripts/phase21-live-mining-evidence.sh`
- `bazel test //scripts:phase21_live_mining_evidence_test`
- `cargo test -p bitaxe-stratum --all-features mining_loop`
- `cargo test -p bitaxe-api --all-features telemetry`
- `cargo test -p bitaxe-parity --all-features mining_allow`
- `just parity`
- `just verify-reference`
- `git diff -- reference/esp-miner --exit-code`
- `just test`
- Targeted bounded-soak field scan for blocked status, duration, hardware-not-run, and watchdog blocked status
- Targeted redaction scan over bounded-soak artifacts and `redaction-review.md`

Verification note: the plan's out-of-range duration grep pattern can match the leading digits of the required `duration_seconds: 300` value. The exact field checks in the committed markdown and JSON artifacts confirm the bounded-soak duration is 300 seconds, not an out-of-range value.

## Deviations from Plan

None - the plan executed the blocked branch as written. Missing live prerequisites were treated as a hard gate and did not become controlled-no-share or approved soak evidence.

## Auth Gates

None.

## Blockers

Bounded soak and watchdog observations remain blocked by the missing live prerequisites carried from Plan 21-06:

- no controlled package boot proof
- no applied pool-input bridge proof
- no live pool command
- no API/WebSocket target capture
- no runtime snapshot update
- no share submission outcome
- no safe-stop observation from a bounded soak

## Known Stubs

The blocked API snapshot and WebSocket files are intentional evidence placeholders:

- `bounded-soak/api-system-info-snapshots.redacted.jsonl` records zero snapshots because no soak ran.
- `bounded-soak/websocket/api-ws-live.txt` records a missing explicit target block and no WebSocket capture.
- `redaction-review.md` uses the word placeholder only to describe blocked-target placeholders reviewed by the scan.

These placeholders do not prevent the plan goal because Plan 21-07 explicitly allowed precise blocked ledgers when live-smoke prerequisites failed.

## Threat Surface Review

No new code, endpoint, credential path, network behavior, file access pattern, or schema boundary was introduced. This plan changed evidence and planning documents only.

## Self-Check: PASSED

Verified created files exist and task commits `e361393`, `de46c78`, and `707e609` are present in git history.
