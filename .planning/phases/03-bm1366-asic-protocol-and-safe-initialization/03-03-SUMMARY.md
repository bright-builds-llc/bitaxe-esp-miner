---
phase: 03-bm1366-asic-protocol-and-safe-initialization
plan: "03"
subsystem: asic
tags: [rust, bm1366, dispatch, uart-transcript, adapter-boundary, fixtures]

requires:
  - phase: 03-bm1366-asic-protocol-and-safe-initialization
    provides: BM1366 packet, work, result, register, and protocol fault foundations from Plans 03-01 and 03-02
provides:
  - Active Ultra 205 BM1366 dispatch with deferred non-V1 ASIC states
  - Semantic BM1366 command, adapter-action, observation, and init-status contracts
  - Fake UART transcript harness for chip detect and result-frame fault paths
  - Reference-derived transcript fixture metadata for adapter-boundary cases
affects: [phase-03, phase-04, asic, firmware-uart-adapter, init-planning, parity-evidence]

tech-stack:
  added: []
  patterns:
    - "Catalog-gated ASIC dispatch keeps only Ultra 205 BM1366 active"
    - "Semantic command/action/observation boundary wraps raw frame bytes inside bitaxe-asic"
    - "Fake UART transcripts convert malformed input into typed fail-closed observations"

key-files:
  created:
    - crates/bitaxe-asic/src/dispatch.rs
    - crates/bitaxe-asic/src/bm1366/command.rs
    - crates/bitaxe-asic/src/bm1366/observation.rs
    - crates/bitaxe-asic/src/bm1366/transcript.rs
    - crates/bitaxe-asic/fixtures/bm1366/transcript-cases.json
  modified:
    - Cargo.lock
    - MODULE.bazel.lock
    - crates/bitaxe-asic/Cargo.toml
    - crates/bitaxe-asic/BUILD.bazel
    - crates/bitaxe-asic/src/lib.rs
    - crates/bitaxe-asic/src/bm1366.rs
    - crates/bitaxe-asic/src/error.rs

key-decisions:
  - "Only board version 205, family Ultra, ASIC model BM1366, count 1, and ActiveUltra205 scope return ActiveBm1366."
  - "Firmware-facing ASIC behavior is expressed as Bm1366Command, Bm1366AdapterAction, Bm1366Observation, and AsicInitStatus while raw frames stay inside bitaxe-asic."
  - "Fake UART transcripts fail closed on timeout, partial read, bad preamble, bad CRC, unknown register, invalid job ID, and chip-count mismatch."

patterns-established:
  - "Dispatch consumes Phase 2 catalog facts instead of duplicating board/ASIC scope constants in firmware."
  - "Transcript fixture metadata records checklist IDs, pinned reference commit, source behavior, license posture, and derivation per case."

requirements-completed: [ASIC-03, ASIC-04, ASIC-08]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 3-2026-06-26T22-40-40
generated_at: 2026-06-27T00:35:56Z

duration: 16 min
completed: 2026-06-27
---

# Phase 03 Plan 03: BM1366 Dispatch And Transcript Boundary Summary

**Ultra 205-only BM1366 dispatch, semantic adapter contracts, and fail-closed fake UART transcripts**

## Performance

- **Duration:** 16 min
- **Started:** 2026-06-27T00:19:50Z
- **Completed:** 2026-06-27T00:35:56Z
- **Tasks:** 2
- **Files modified:** 12

## Accomplishments

- Added `AsicDispatch::ActiveBm1366` gated to exact Ultra 205 catalog facts, with BM1370, BM1368, BM1397, and other paths explicitly deferred.
- Added semantic BM1366 commands, adapter actions, observations, chip identifiers, addresses, and init status values.
- Added a fake UART transcript harness with exact chip-ID reads and fail-closed coverage for timeout, partial reads, bad preambles, bad CRCs, unknown registers, invalid job IDs, and chip-count mismatch.
- Added transcript fixture metadata with pinned reference commit and provenance fields.

## Task Commits

Each task was committed atomically:

1. **Task 1: Add active BM1366 dispatch and semantic command surface** - `fcf9124` (feat)
2. **Task 2: Add fake UART transcript coverage for adapter faults** - `445b3c8` (feat)
3. **Generated build metadata: Refresh Bazel module lock** - `774a116` (chore)

_Note: TDD RED failures were run and recorded before implementation, but failing intermediate states were not committed because the repo Rust pre-commit rule requires passing format, clippy, build, and tests before every commit._

## Files Created/Modified

- `Cargo.lock` - Records the new local `bitaxe-config` dependency edge for `bitaxe-asic`.
- `MODULE.bazel.lock` - Refreshes crate-universe file hashes after the Cargo dependency change.
- `crates/bitaxe-asic/Cargo.toml` - Adds the local `bitaxe-config` dependency used by dispatch.
- `crates/bitaxe-asic/BUILD.bazel` - Exposes dispatch, command, observation, and transcript modules to Bazel.
- `crates/bitaxe-asic/src/lib.rs` - Exports dispatch and adds unit coverage for dispatch, commands, and transcripts.
- `crates/bitaxe-asic/src/bm1366.rs` - Exports command, observation, and transcript modules.
- `crates/bitaxe-asic/src/error.rs` - Adds timeout and transcript write-mismatch protocol faults.
- `crates/bitaxe-asic/src/dispatch.rs` - Implements Ultra 205-only active dispatch and deferred ASIC reasons.
- `crates/bitaxe-asic/src/bm1366/command.rs` - Implements semantic commands and adapter actions over existing frame constructors.
- `crates/bitaxe-asic/src/bm1366/observation.rs` - Implements observations, chip/address newtypes, and init statuses.
- `crates/bitaxe-asic/src/bm1366/transcript.rs` - Implements fake UART transcript execution and fail-closed outcomes.
- `crates/bitaxe-asic/fixtures/bm1366/transcript-cases.json` - Records provenance metadata for transcript cases.

## Decisions Made

- Dispatch depends on `bitaxe-config` catalog facts so the active ASIC path cannot drift from Phase 2 board evidence scope.
- The command enum uses `SendDiagnosticWork`, not a production work variant, preserving the Phase 3 no-mining boundary.
- Transcript failures use typed `Bm1366ProtocolFault` observations and publish fail-closed status actions rather than silently dropping malformed UART input.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Preserved Rust pre-commit requirements during TDD**
- **Found during:** Tasks 1 and 2
- **Issue:** The generic TDD flow allows failing RED commits, but repo-local Rust rules require `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features` before every commit.
- **Fix:** Ran RED tests to prove failure, then committed only passing task outcomes after the full Rust gate.
- **Files modified:** Task files only
- **Verification:** Full Rust pre-commit sequence passed before task commits.
- **Committed in:** `fcf9124`, `445b3c8`

**2. [Rule 2 - Missing Critical] Added transcript-specific protocol faults**
- **Found during:** Task 2
- **Issue:** The transcript harness needed typed faults for UART timeout and expected-write mismatch to satisfy timeout handling and ordered write verification.
- **Fix:** Added `Bm1366ProtocolFault::Timeout` and `Bm1366ProtocolFault::TranscriptWriteMismatch`, and mapped them into fail-closed transcript observations.
- **Files modified:** `crates/bitaxe-asic/src/error.rs`, `crates/bitaxe-asic/src/bm1366/transcript.rs`
- **Verification:** `cargo test -p bitaxe-asic transcript --all-features` passed with timeout and write-verification paths covered.
- **Committed in:** `445b3c8`

___

**Total deviations:** 2 auto-fixed (2 missing critical)
**Impact on plan:** Both deviations preserve stricter repo rules or required fail-closed behavior. No mining, firmware side effects, or hardware verification claims were added.

## Issues Encountered

- Expected TDD RED failures occurred for missing dispatch/command/observation modules and then missing transcript/timeout surfaces.
- Bazel verification refreshed `MODULE.bazel.lock` after the `bitaxe-asic` Cargo dependency change; committed separately as `774a116`.

## User Setup Required

None - no external service configuration required.

## Known Stubs

None - source and fixture files touched by this plan have no placeholder or stub data that blocks the plan goal.

## Verification

- `cargo test -p bitaxe-asic dispatch --all-features` - passed, 2 dispatch tests.
- `cargo test -p bitaxe-asic bm1366_read_chip_id --all-features` - passed, command-to-frame adapter test.
- `cargo test -p bitaxe-asic transcript --all-features` - passed, 7 transcript tests.
- `cargo test -p bitaxe-asic --all-features` - passed, 29 tests.
- `bazel test //crates/bitaxe-asic:tests` - passed.
- `rg -n "ActiveBm1366|SendDiagnosticWork|FakeUartTranscript" crates/bitaxe-asic/src` - passed.
- `git status --short reference/esp-miner` - clean, no output.
- `cargo fmt --all` - passed.
- `cargo clippy --all-targets --all-features -- -D warnings` - passed.
- `cargo build --all-targets --all-features` - passed.
- `cargo test --all-features` - passed.

## Next Phase Readiness

Ready for Plan 03-04 to consume the active dispatch and semantic adapter contracts while building fail-closed staged init and frequency/voltage transition decisions.

___
*Phase: 03-bm1366-asic-protocol-and-safe-initialization*
*Completed: 2026-06-27*

## Self-Check: PASSED

- Created files verified on disk.
- Summary file verified on disk.
- Task and generated metadata commits verified in git history: `fcf9124`, `445b3c8`, `774a116`.
