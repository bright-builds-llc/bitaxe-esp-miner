---
phase: 05-axeos-api-logs-and-telemetry
plan: 06
subsystem: api
tags: [rust, serde-json, axeos, command-routes, mining-state, safety]

requires:
  - phase: 05-axeos-api-logs-and-telemetry
    provides: "Plan 05-01 API crate foundation, Serde dependencies, and fixture wiring"
  - phase: 04-stratum-v1-and-first-mining-loop
    provides: "MiningRuntimeState, MiningActivityStatus, WorkSubmissionGate, and fail-closed mining-loop gate decisions"
provides:
  - "Pure non-OTA AxeOS command response and side-effect planners"
  - "Fixture-backed public JSON responses for pause, resume, restart, identify, and block-found dismiss"
  - "Typed command effects that keep response sending separate from firmware-side restart, display, and state mutations"
  - "State-transition regressions proving pause/resume cannot unlock work submission and block-found dismiss is idempotent"
affects:
  - 05-05-firmware-route-websocket-settings-log-adapters
  - 05-07-api-compare-and-static-evidence
  - phase-06-safety
  - axeos-command-route-compatibility

tech-stack:
  added: []
  patterns:
    - "Command route planners return public response JSON separately from inert typed firmware effects"
    - "Mining command effects mutate MiningActivityStatus only and preserve existing WorkSubmissionGate"
    - "Restart is modeled as RestartAfterResponse so firmware can respond before scheduling reset"

key-files:
  created:
    - crates/bitaxe-api/src/commands.rs
    - crates/bitaxe-api/fixtures/api/command-responses.json
  modified:
    - crates/bitaxe-api/BUILD.bazel
    - crates/bitaxe-api/src/lib.rs

key-decisions:
  - "Command planners return response JSON separately from typed effects so firmware route code can send the public response before executing restart, display, or state mutations."
  - "Pause and resume only plan MiningActivityStatus updates; resume derives Active versus SafeBlocked from the existing WorkSubmissionGate and never sets work submission readiness."
  - "Identify is represented as a typed on/off display effect with the upstream 30000 ms duration, while restart is represented only as an after-response effect."
  - "Block-found dismiss preserves blockFound, clears showNewBlock, and remains deterministic across repeated dismiss requests."

patterns-established:
  - "Use CommandPlan { response, effect } for non-OTA command routes."
  - "Use apply_mining_activity_effect for visible mining activity updates without changing mining-loop readiness."
  - "Use BlockFoundNotificationState and BlockFoundDismissEffect for block-found notification transitions."

requirements-completed: [API-08]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 5-2026-06-27T18-02-49
generated_at: 2026-06-27T20:57:00Z

duration: 8 min
completed: 2026-06-27
---

# Phase 05 Plan 06: Command Route Planners Summary

**Pure AxeOS non-OTA command planners with fixture-backed responses and typed post-response firmware effects**

## Performance

- **Duration:** 8 min
- **Started:** 2026-06-27T20:48:38Z
- **Completed:** 2026-06-27T20:57:00Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments

- Added `bitaxe_api::commands` with pure planners for pause, resume, restart, identify, and block-found dismiss.
- Added `command-responses.json` fixtures and tests proving exact upstream-compatible public JSON response bodies.
- Modeled restart as `RestartAfterResponse` and identify as a typed on/off display effect with the exact 30000 ms duration.
- Added safety regression tests for active, paused, and safe-blocked mining states, proving pause/resume preserve the existing work-submission gate.
- Added deterministic block-found dismiss state application and idempotence coverage.

## Task Commits

1. **Task 1: Add command response and effect planners** - `fb8de6d` (feat)
2. **Task 2: Add command state transition and safety regression tests** - `d13ec80` (test)

## Files Created/Modified

- `crates/bitaxe-api/src/commands.rs` - Pure command planners, typed effects, state-transition helpers, and command behavior tests.
- `crates/bitaxe-api/fixtures/api/command-responses.json` - Exact public response fixtures for pause, resume, restart, identify on/off, and block-found dismiss.
- `crates/bitaxe-api/src/lib.rs` - Public command module and planner/effect exports.
- `crates/bitaxe-api/BUILD.bazel` - Adds `src/commands.rs` to the API crate source set.

## Decisions Made

- Command route response construction is kept pure and separate from effect execution. Firmware adapters choose when to run effects after the response has been sent.
- Pause and resume do not decide ASIC readiness, safety readiness, or hardware evidence readiness. They only plan visible mining activity changes over the existing `MiningRuntimeState`.
- Block-found dismiss is represented through typed notification state instead of loose response booleans alone, preserving `blockFound` while clearing `showNewBlock`.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Fixed clippy field-reassign test fixtures**
- **Found during:** Task 1 (Add command response and effect planners)
- **Issue:** The first Task 1 Rust gate failed because two command tests reassigned fields after `MiningRuntimeState::default()`, triggering `clippy::field-reassign-with-default` under `-D warnings`.
- **Fix:** Rewrote those fixtures to initialize `MiningRuntimeState` with struct literals and `..Default::default()`.
- **Files modified:** `crates/bitaxe-api/src/commands.rs`
- **Verification:** Re-ran `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features`; all passed before commit.
- **Committed in:** `fb8de6d`

### Process Adjustments

**AGENTS.md precedence over TDD RED commits**
- **Found during:** Task 1 and Task 2
- **Issue:** The plan requested TDD RED flow, but repo instructions require the full Rust verification gate to pass before every commit.
- **Adjustment:** Ran RED failures locally, then committed only passing GREEN task states after the required Rust gate.
- **Impact:** Preserved higher-priority repo policy without changing command behavior.

***

**Total deviations:** 1 auto-fixed (1 blocking), 1 process adjustment.
**Impact on plan:** The auto-fix was limited to test fixture shape. Scope stayed within pure command route planning and safety regressions.

## Issues Encountered

- Task 1 RED failed as intended on unimplemented command planner `todo!()` calls.
- Task 2 RED failed as intended on a missing `apply_block_found_dismiss_effect` helper before the helper was implemented.

## Known Stubs

None. Stub scan found no `TODO`, `FIXME`, placeholder text, `todo!()`, or UI-flowing hardcoded empty values in the files created or modified by this plan.

## Threat Flags

None. The planned trust surfaces were HTTP command route input into pure planning and pure plans into firmware effects; no unplanned network endpoint, auth path, file access pattern, schema change, raw ASIC command, UART call, ESP-IDF import, NVS write, or direct restart execution was introduced.

## Authentication Gates

None.

## User Setup Required

None - no external service configuration required.

## Verification

- `bazel test //crates/bitaxe-api:tests --test_filter=commands` - failed in Task 1 RED, then passed
- `bazel test //crates/bitaxe-api:tests --test_filter=commands` - failed in Task 2 RED, then passed
- `bazel test //crates/bitaxe-api:tests` - passed after each task and as plan quick verification
- `just test` - passed as full plan verification
- `cargo fmt --all` - passed before each task commit
- `cargo clippy --all-targets --all-features -- -D warnings` - passed before each task commit
- `cargo build --all-targets --all-features` - passed before each task commit
- `cargo test --all-features` - passed before each task commit
- `rg -n "CommandFrame|JobFrame|uart|esp_idf|Nvs|NVS|nvs|allow_work_submission|esp_restart" crates/bitaxe-api/src/commands.rs` - clean output
- `git status --short reference/esp-miner` - clean output

## Next Phase Readiness

Ready for firmware route wiring and comparison evidence. The API crate now exposes pure command plans that later firmware adapters can call without duplicating public response copy or mining safety transition rules.

## Self-Check: PASSED

- Found `.planning/phases/05-axeos-api-logs-and-telemetry/05-06-SUMMARY.md`
- Found `crates/bitaxe-api/src/commands.rs`
- Found `crates/bitaxe-api/fixtures/api/command-responses.json`
- Found task commit `fb8de6d`
- Found task commit `d13ec80`
- Reference implementation remains unmodified.

***
*Phase: 05-axeos-api-logs-and-telemetry*
*Completed: 2026-06-27*
