---
phase: 04-stratum-v1-and-first-mining-loop
plan: "02"
subsystem: stratum
tags: [rust, stratum-v1, fake-pool, lifecycle-state, telemetry]

requires:
  - phase: 04-stratum-v1-and-first-mining-loop
    provides: "Plan 04-01 typed Stratum v1 JSON-RPC client/server messages, parser, and protocol fixtures"
provides:
  - "Typed mining runtime state with pool lifecycle, work-submission gate, share counters, hashrate inputs, and mining activity status"
  - "Deterministic fake-pool transcript runner for accepted, rejected, reconnect, fallback, and unexpected-client behavior"
  - "Provenance-rich fake-pool transcript fixture metadata using synthetic credentials only"
affects: [phase-04-stratum, phase-05-api-telemetry, parity-evidence]

tech-stack:
  added: []
  patterns:
    - "Keep Stratum lifecycle and counters in pure state types for later firmware/API adapters"
    - "Drive fake-pool behavior from typed Stratum client/server messages instead of raw JSON"

key-files:
  created:
    - crates/bitaxe-stratum/src/v1/state.rs
    - crates/bitaxe-stratum/src/v1/fake_pool.rs
    - crates/bitaxe-stratum/fixtures/v1/fake-pool-transcripts.json
  modified:
    - crates/bitaxe-stratum/BUILD.bazel
    - crates/bitaxe-stratum/src/v1.rs

key-decisions:
  - "Mining runtime state remains pure and telemetry-ready; no Phase 5 HTTP/WebSocket handlers, firmware sockets, live pool calls, or hardware side effects were introduced."
  - "Fake-pool transcripts update state only from typed Stratum client/server messages and fail on unexpected client messages instead of advancing silently."
  - "Timeout transitions represent fallback activation in the deterministic fake-pool harness, while disconnect and client.reconnect map to Reconnecting lifecycle state."
  - "TDD RED failures were run but not committed because AGENTS.md requires passing Rust verification before every commit."

patterns-established:
  - "Runtime counters use ShareDifficulty and retained rejected reasons so later API/telemetry surfaces can map them without re-parsing raw pool responses."
  - "FakePoolTranscript owns scripted ExpectClient, SendServer, Disconnect, and Timeout events and returns MiningRuntimeState as its only output."

requirements-completed: [STR-02, STR-04, STR-05]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 04-2026-06-27T13-17-33
generated_at: 2026-06-27T14:33:02Z

duration: 8 min
completed: 2026-06-27
---

# Phase 04 Plan 02: Fake Pool And Runtime State Summary

**Typed mining runtime state and deterministic Stratum v1 fake-pool lifecycle runner**

## Performance

- **Duration:** 8 min
- **Started:** 2026-06-27T14:24:13Z
- **Completed:** 2026-06-27T14:33:02Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments

- Added typed `MiningRuntimeState` with lifecycle status, work-submission gate, accepted/rejected share counters, rejected reasons, pool difficulty, hashrate inputs, and mining activity status.
- Added `FakePoolTranscript` and `FakePoolEvent` to deterministically cover subscribe, authorize, set-difficulty, notify, accepted submit, rejected submit, reconnect, fallback, and unexpected-client paths.
- Added fake-pool fixture metadata with reference commit, source file, checklist IDs, derivation, and synthetic credential values only.
- Wired the new Stratum modules into `v1.rs` and the Bazel `bitaxe_stratum` target.

## Task Commits

Each task was committed atomically:

1. **Task 1: Add runtime state and share counters** - `022343f` (feat)
2. **Task 2: Add deterministic fake-pool transcript runner** - `a323836` (feat)

## Files Created/Modified

- `crates/bitaxe-stratum/src/v1/state.rs` - Defines pool lifecycle, work-submission, mining activity, share counters, hashrate inputs, share difficulty, and runtime state tests.
- `crates/bitaxe-stratum/src/v1/fake_pool.rs` - Defines deterministic fake-pool events/transcripts and maps typed protocol outcomes into runtime state.
- `crates/bitaxe-stratum/fixtures/v1/fake-pool-transcripts.json` - Records fake-pool scenario names and provenance metadata for STR-002, STR-004, and STR-005.
- `crates/bitaxe-stratum/src/v1.rs` - Exports `state` and `fake_pool`.
- `crates/bitaxe-stratum/BUILD.bazel` - Adds the new Stratum source files to the Bazel Rust target.

## Decisions Made

- Kept the runtime state and fake-pool harness pure: no firmware sockets, ESP-IDF networking, live pool calls, API handlers, WebSocket handlers, ASIC work dispatch, voltage, fan, thermal, or power side effects.
- Used typed `StratumV1ClientMessage` and `StratumV1ServerMessage` values as the fake-pool boundary so malformed raw JSON cannot bypass Plan 04-01 parsing.
- Treated `Timeout` as deterministic fallback activation and `Disconnect`/`ClientReconnect` as reconnecting lifecycle transitions.
- Used synthetic usernames/passwords in tests and fixtures only.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Kept TDD RED failures out of git history**
- **Found during:** Task 1 and Task 2 TDD execution
- **Issue:** The generic GSD TDD flow calls for RED commits, but AGENTS.md requires passing Rust format, clippy, build, and tests before every commit.
- **Fix:** Ran RED failures locally, implemented GREEN behavior, and committed only passing task states.
- **Files modified:** Task-owned Stratum files only.
- **Verification:** Task-scoped RED failures were observed before implementation; final task commits were made only after passing `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features`.
- **Committed in:** `022343f`, `a323836`

---

**Total deviations:** 1 auto-fixed (1 missing critical)
**Impact on plan:** The adjustment enforces repo commit safety without changing the planned runtime state or fake-pool behavior.

## Issues Encountered

None.

## Known Stubs

None.

## Authentication Gates

None.

## User Setup Required

None - no external service configuration required.

## Verification

- `cargo test -p bitaxe-stratum runtime_state --all-features` - passed
- `cargo test -p bitaxe-stratum fake_pool --all-features` - passed
- `cargo test -p bitaxe-stratum pool_lifecycle --all-features` - passed
- `cargo test -p bitaxe-stratum --all-features` - passed
- `bazel test //crates/bitaxe-stratum:tests` - passed
- `cargo fmt --all` - passed before task commits and Task 2 amend
- `cargo clippy --all-targets --all-features -- -D warnings` - passed before task commits and Task 2 amend
- `cargo build --all-targets --all-features` - passed before task commits and Task 2 amend
- `cargo test --all-features` - passed before task commits and Task 2 amend
- `git status --short reference/esp-miner` - clean

## Next Phase Readiness

Ready for Plan 04-03. The Stratum v1 protocol core now has deterministic runtime state and fake-pool lifecycle coverage for the mining job and work queue bridge.

## Self-Check: PASSED

- Found `crates/bitaxe-stratum/src/v1/state.rs`
- Found `crates/bitaxe-stratum/src/v1/fake_pool.rs`
- Found `crates/bitaxe-stratum/fixtures/v1/fake-pool-transcripts.json`
- Found `.planning/phases/04-stratum-v1-and-first-mining-loop/04-02-SUMMARY.md`
- Found task commit `022343f`
- Found task commit `a323836`

---
*Phase: 04-stratum-v1-and-first-mining-loop*
*Completed: 2026-06-27*
