---
phase: 04-stratum-v1-and-first-mining-loop
plan: "03"
subsystem: stratum
tags: [rust, stratum-v1, mining-job, bm1366, work-queue, sha2]

requires:
  - phase: 04-stratum-v1-and-first-mining-loop
    provides: "Plans 04-01 and 04-02 typed Stratum v1 messages, fake-pool transcripts, and runtime state"
  - phase: 03-bm1366-asic-protocol-and-safe-initialization
    provides: "Typed BM1366 work fields, job IDs, nonce results, and valid-job tracking"
provides:
  - "Pure Stratum notify/extranonce/merkle bridge to typed BM1366 work fields"
  - "Share-submission data mapped from typed BM1366 nonce results"
  - "Bounded Stratum work queue with upstream capacity 12 and clean-jobs valid-job reset"
affects: [phase-04-stratum, phase-05-api-telemetry, phase-06-safety, parity-evidence]

tech-stack:
  added: [sha2, bitaxe-asic, bitaxe-config]
  patterns:
    - "Decode pool-provided hex into typed byte arrays before building mining work"
    - "Keep raw ASIC frame construction owned by bitaxe-asic while Stratum produces Bm1366WorkFields only"
    - "Pair queued mining work with Bm1366ValidJobIds and reset both on clean-jobs"

key-files:
  created:
    - crates/bitaxe-stratum/src/v1/coinbase.rs
    - crates/bitaxe-stratum/src/v1/mining.rs
    - crates/bitaxe-stratum/src/v1/queue.rs
    - crates/bitaxe-stratum/fixtures/v1/mining-job-cases.json
  modified:
    - Cargo.lock
    - crates/bitaxe-stratum/Cargo.toml
    - crates/bitaxe-stratum/BUILD.bazel
    - crates/bitaxe-stratum/src/error.rs
    - crates/bitaxe-stratum/src/v1.rs

key-decisions:
  - "Stratum mining job construction produces typed Bm1366WorkFields and never constructs raw ASIC JobFrame or CommandFrame values."
  - "Malformed hex and oversized extranonce2 lengths fail with StratumV1Error before pool data can become mining work."
  - "Clean-jobs behavior is explicit through MiningWorkQueue::clear_jobs, which clears both queued work and Bm1366ValidJobIds."
  - "TDD RED failures were run but not committed because AGENTS.md requires passing Rust verification before every commit."

patterns-established:
  - "Coinbase and merkle helpers are pure functions with reference breadcrumbs and focused mining_job tests."
  - "MiningWorkBuilder owns Stratum notify/extranonce state and returns a MiningWork wrapper around typed BM1366 work fields."
  - "BoundedWorkQueue<T, N> uses VecDeque with fixed capacity and typed QueueFull/QueueEmpty errors."

requirements-completed: [STR-03, STR-06]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 04-2026-06-27T13-17-33
generated_at: 2026-06-27T14:48:16Z

duration: 10 min
completed: 2026-06-27
---

# Phase 04 Plan 03: Mining Job And Work Queue Bridge Summary

**Pure Stratum v1 mining job construction and bounded BM1366 work queue with clean-jobs invalidation**

## Performance

- **Duration:** 10 min
- **Started:** 2026-06-27T14:37:51Z
- **Completed:** 2026-06-27T14:48:16Z
- **Tasks:** 2
- **Files modified:** 9

## Accomplishments

- Added coinbase/extranonce/double-SHA/merkle helpers with strict hex validation and upstream little-endian extranonce behavior.
- Added `MiningWorkBuilder`, `MiningWork`, and `ShareSubmission` to turn typed Stratum notify/extranonce data and BM1366 nonce results into pure mining-loop data.
- Added `BoundedWorkQueue` and `MiningWorkQueue` with fixed capacity 12, FIFO dequeue behavior, typed queue errors, and `Bm1366ValidJobIds` reset on clean-jobs.
- Added provenance-rich mining job fixture metadata for STR-003 and STR-006.

## Task Commits

Each task was committed atomically:

1. **Task 1: Add coinbase and mining job construction helpers** - `ac915c7` (feat)
2. **Task 2: Add bounded work queue and clean-jobs behavior** - `f6b62fa` (feat)

## Files Created/Modified

- `Cargo.lock` - Records new `bitaxe-stratum` dependency edges for `bitaxe-asic`, `bitaxe-config`, and `sha2`.
- `crates/bitaxe-stratum/Cargo.toml` - Adds planned workspace/path dependencies for mining job construction.
- `crates/bitaxe-stratum/BUILD.bazel` - Adds new Stratum source files and Bazel deps.
- `crates/bitaxe-stratum/src/error.rs` - Adds `QueueFull` and `QueueEmpty`.
- `crates/bitaxe-stratum/src/v1.rs` - Exports `coinbase`, `mining`, and `queue`.
- `crates/bitaxe-stratum/src/v1/coinbase.rs` - Implements strict hex decoding, double SHA-256, merkle folding, and little-endian extranonce2 generation.
- `crates/bitaxe-stratum/src/v1/mining.rs` - Implements Stratum notify/extranonce to `Bm1366WorkFields` and nonce-result share submissions.
- `crates/bitaxe-stratum/src/v1/queue.rs` - Implements bounded FIFO work queue and clean-jobs valid-job reset.
- `crates/bitaxe-stratum/fixtures/v1/mining-job-cases.json` - Adds mining job fixture provenance and cases.

## Decisions Made

- `bitaxe-stratum` produces `Bm1366WorkFields` only; raw ASIC frame bytes remain in `bitaxe-asic`.
- Version rolling mask data is accepted by the builder, while the base work fields keep the notify version and later nonce-result version bits flow into share submission data.
- Queue overflow is a typed `QueueFull` error instead of an unbounded allocation or blocking wait in the pure host-testable layer.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Kept TDD RED failures out of git history**
- **Found during:** Task 1 and Task 2 TDD execution
- **Issue:** The generic GSD TDD flow calls for RED commits, but AGENTS.md requires passing Rust format, clippy, build, and tests before every commit.
- **Fix:** Ran RED failures locally, implemented GREEN behavior, and committed only passing task states.
- **Files modified:** Task-owned Stratum files only.
- **Verification:** RED failures were observed for missing mining helpers and queue types; final task commits passed targeted tests and the full Rust pre-commit sequence.
- **Committed in:** `ac915c7`, `f6b62fa`

**Total deviations:** 1 auto-fixed (1 missing critical)
**Impact on plan:** The adjustment enforces repo commit safety without changing the planned Stratum mining job or queue behavior.

## Issues Encountered

None.

## Known Stubs

None.

## Authentication Gates

None.

## User Setup Required

None - no external service configuration required.

## Verification

- `cargo test -p bitaxe-stratum mining_job --all-features` - passed
- `cargo test -p bitaxe-stratum work_queue --all-features` - passed
- `cargo test -p bitaxe-stratum mining_loop --all-features` - passed
- `cargo test -p bitaxe-asic --all-features` - passed
- `cargo fmt --all` - passed before both task commits
- `cargo clippy --all-targets --all-features -- -D warnings` - passed before both task commits
- `cargo build --all-targets --all-features` - passed before both task commits
- `cargo test --all-features` - passed before both task commits
- `rg -n "JobFrame|CommandFrame|diagnostic_job_frame|CommandFrame::new|JobFrame::new" crates/bitaxe-stratum` - no matches
- `git status --short reference/esp-miner` - clean

## Next Phase Readiness

Ready for Plan 04-04. Pure Stratum mining job construction, share submission data, bounded queue behavior, and clean-jobs invalidation are available for the firmware gated mining-loop shell without bypassing ASIC safety boundaries.

## Self-Check: PASSED

- Found `.planning/phases/04-stratum-v1-and-first-mining-loop/04-03-SUMMARY.md`
- Found `crates/bitaxe-stratum/src/v1/coinbase.rs`
- Found `crates/bitaxe-stratum/src/v1/mining.rs`
- Found `crates/bitaxe-stratum/src/v1/queue.rs`
- Found `crates/bitaxe-stratum/fixtures/v1/mining-job-cases.json`
- Found task commit `ac915c7`
- Found task commit `f6b62fa`

---
*Phase: 04-stratum-v1-and-first-mining-loop*
*Completed: 2026-06-27*
