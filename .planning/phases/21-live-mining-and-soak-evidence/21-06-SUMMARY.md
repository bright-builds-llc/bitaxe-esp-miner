---
phase: 21-live-mining-and-soak-evidence
plan: "06"
subsystem: parity-evidence
tags:
  - live-mining
  - telemetry
  - redaction
  - ultra205
  - bm1366
requires:
  - phase: 21-live-mining-and-soak-evidence
    provides: "Plans 21-01 through 21-05 preflight, live-mining enablement, and BM1366 diagnostic prerequisite ledgers"
provides:
  - "Blocked live-mining smoke ledger for missing explicit target and pool input categories"
  - "Blocked explicit-target API/WebSocket telemetry correlation ledger"
  - "Redaction-reviewed blocked live evidence and SAFE-09 validation sampling update"
affects:
  - phase-21-live-mining
  - phase-21-telemetry
  - parity-checklist
tech-stack:
  added: []
  patterns:
    - "Missing live prerequisites are recorded as blocked evidence, not controlled no-share proof"
    - "Explicit-target API/WebSocket evidence must not infer DEVICE_URL from hardware or network state"
key-files:
  created:
    - docs/parity/evidence/phase-21-live-mining-and-soak-evidence/live-mining-smoke.md
    - docs/parity/evidence/phase-21-live-mining-and-soak-evidence/live-mining-smoke/detect-ultra205.log
    - docs/parity/evidence/phase-21-live-mining-and-soak-evidence/live-mining-smoke/pool-input-bridge.md
    - docs/parity/evidence/phase-21-live-mining-and-soak-evidence/live-api-websocket-telemetry.md
    - docs/parity/evidence/phase-21-live-mining-and-soak-evidence/live-api-websocket-telemetry/api-system-info.redacted.json
    - docs/parity/evidence/phase-21-live-mining-and-soak-evidence/live-api-websocket-telemetry/api-system-info.error.txt
    - docs/parity/evidence/phase-21-live-mining-and-soak-evidence/live-api-websocket-telemetry/websocket/api-ws-live.txt
    - .planning/phases/21-live-mining-and-soak-evidence/21-06-USER-SETUP.md
  modified:
    - docs/parity/evidence/phase-21-live-mining-and-soak-evidence/redaction-review.md
    - .planning/phases/21-live-mining-and-soak-evidence/21-VALIDATION.md
key-decisions:
  - "Treat absent DEVICE_URL or pool input categories as missing_live_prerequisites and do not downgrade them into controlled no-share evidence."
  - "Keep live API/WebSocket correlation blocked when no explicit DEVICE_URL exists; network scanning and target inference remain disabled."
  - "Do not claim successful BM1366 initialization, production mining, shares, telemetry freshness, watchdog mining-load behavior, or soak evidence from prerequisite-only ledgers."
patterns-established:
  - "Blocked live evidence can be redaction-reviewed when committed artifacts contain only labels, placeholders, and redacted detector output."
  - "SAFE-09 sampling can cite blocked boundaries when the validation row states the exact non-claims."
requirements-completed:
  - ASIC-07
  - STR-06
  - STR-07
  - SAFE-09
  - EVD-05
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 21-2026-07-04T01-35-47
generated_at: 2026-07-04T05:58:34Z
duration: 13min
completed: 2026-07-04
---

# Phase 21 Plan 06: Live Mining Smoke and Telemetry Correlation Summary

**Detector-gated Ultra 205 live smoke and API/WebSocket telemetry correlation were precisely blocked by missing explicit live prerequisites, with redaction-reviewed evidence and no mining overclaim.**

## Performance

- **Duration:** 13 min
- **Started:** 2026-07-04T05:45:14Z
- **Completed:** 2026-07-04T05:58:34Z
- **Tasks:** 3
- **Files modified:** 11

## Accomplishments

- Ran a fresh `just detect-ultra205` gate and recorded a redacted detector pass for board `205`.
- Wrote `missing_live_prerequisites` live-smoke and pool-input bridge ledgers because explicit target and pool input categories were absent.
- Wrote blocked API/WebSocket telemetry artifacts for `/api/system/info` and `/api/ws/live` without probing or inferring a target.
- Closed the plan redaction review for the blocked live artifacts and updated SAFE-09 validation sampling.

## Task Commits

1. **Task 1: Live mining smoke evidence** - `31e637d` (`docs`)
2. **Task 2: Explicit-target API/WebSocket telemetry evidence** - `f935e23` (`docs`)
3. **Task 3: Redaction and citation closure** - `1d77483` (`docs`)

## Files Created/Modified

- `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/live-mining-smoke.md` - blocked live smoke ledger with detector pass and non-claims.
- `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/live-mining-smoke/detect-ultra205.log` - redacted fresh detector output.
- `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/live-mining-smoke/pool-input-bridge.md` - blocked pool-input bridge ledger.
- `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/live-api-websocket-telemetry.md` - blocked explicit-target telemetry correlation ledger.
- `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/live-api-websocket-telemetry/api-system-info.redacted.json` - not-run API placeholder.
- `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/live-api-websocket-telemetry/api-system-info.error.txt` - missing-target API note.
- `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/live-api-websocket-telemetry/websocket/api-ws-live.txt` - bounded WebSocket capture placeholder with `/api/ws/live`, `duration_ms=10000`, and `max_frames=5`.
- `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/redaction-review.md` - plan 21-06 redaction review rows.
- `.planning/phases/21-live-mining-and-soak-evidence/21-VALIDATION.md` - SAFE-09 blocked-boundary sampling update.
- `.planning/phases/21-live-mining-and-soak-evidence/21-06-USER-SETUP.md` - prerequisite category setup note.

## Decisions Made

- Missing explicit target and pool input categories block the plan as `missing_live_prerequisites`.
- No live target was inferred from detector output, serial state, network state, mDNS, ARP, router state, or prior evidence.
- No live pool smoke command, pool PATCH, controlled package boot, API request, WebSocket connection, soak, or unsafe hardware action was run.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Evidence claim wording] Removed exact prohibited token matches from blocked non-claims**

- **Found during:** Task 1 live smoke acceptance scan.
- **Issue:** Initial non-claim text contained exact prohibited terms that the plan's blocked-branch scan treats as unsafe evidence wording.
- **Fix:** Reworded the blocked non-claims while preserving the same conservative semantics.
- **Files modified:** `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/live-mining-smoke.md`
- **Verification:** Prohibited-token scan returned no matches.
- **Committed in:** `31e637d`

**Total deviations:** 1 auto-fixed Rule 1 wording issue.
**Impact on plan:** No scope expansion; the fix tightened the evidence boundary.

## Issues Encountered

- The first environment presence check used a zsh-incompatible parameter expansion and failed before printing any values. It was rerun under bash and reported only presence categories, with all live prerequisite categories missing.

## Known Stubs

- `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/live-api-websocket-telemetry/api-system-info.redacted.json` is an intentional blocked placeholder because no explicit `DEVICE_URL` was present.
- `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/live-api-websocket-telemetry/websocket/api-ws-live.txt` is an intentional blocked placeholder documenting the expected route and capture bounds without connecting.

## Auth Gates

None.

## User Setup Required

External live prerequisites remain incomplete. See `.planning/phases/21-live-mining-and-soak-evidence/21-06-USER-SETUP.md` for the required category names and handling rules. Do not commit, print, or summarize raw values.

## Verification

- `just detect-ultra205` passed for exactly one Ultra 205 candidate; redacted output committed.
- `rg -n "live_mining_smoke_status: blocked|blocker: prerequisite tier not passed|hardware_command_status: not-run|network_scan: disabled" docs/parity/evidence/phase-21-live-mining-and-soak-evidence/live-mining-smoke.md`
- `node --check scripts/phase17-websocket-capture.mjs`
- `rg -n "telemetry_correlation_status: blocked|network_scan: disabled|path=/api/ws/live|duration_ms=10000|max_frames=5|websocket_frame_status=blocked_missing_explicit_device_url" docs/parity/evidence/phase-21-live-mining-and-soak-evidence/live-api-websocket-telemetry docs/parity/evidence/phase-21-live-mining-and-soak-evidence/live-api-websocket-telemetry.md`
- Scoped redaction scan over live-smoke, telemetry, and redaction review; matches were labels/placeholders or the review contract only.
- `just parity` passed with `validation_errors: none`.
- `bash -n scripts/phase21-live-mining-evidence.sh`
- `bazel test //scripts:phase21_live_mining_evidence_test`
- `cargo test -p bitaxe-stratum --all-features mining_loop`
- `cargo test -p bitaxe-stratum --all-features fake_pool`
- `cargo test -p bitaxe-stratum --all-features queue`
- `cargo test -p bitaxe-api --all-features mining`
- `just verify-reference` passed with `reference clean: c1915b0a63bfabebdb95a515cedfee05146c1d50`
- `git diff -- reference/esp-miner --exit-code`
- Rust pre-commit sequence before each task commit: `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, `cargo test --all-features`

## Hardware Outcome

Fresh detector gating found board `205` and board-info passed. No flash, live pool command, pool PATCH, live smoke wrapper, API request, WebSocket connection, mining load, safe-stop action, or soak was run because explicit live prerequisites were missing.

## Next Phase Readiness

The blocked artifacts are ready for downstream planning and verification. Future live-pool or telemetry work must supply explicit target and disposable/non-secret pool input categories, rerun `just detect-ultra205`, create the required allow manifest before any live smoke command, and preserve the current non-claims until new evidence proves otherwise.

## Self-Check: PASSED

- Required summary, setup, live smoke, telemetry, and redaction review files exist.
- Task commits `31e637d`, `f935e23`, and `1d77483` are reachable in git history.
- GSD Markdown frontmatter delimiters are limited to the opening and closing blocks.
