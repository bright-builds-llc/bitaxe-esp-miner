---
phase: 03-bm1366-asic-protocol-and-safe-initialization
plan: "02"
subsystem: asic
tags: [rust, bm1366, work-encoding, result-parsing, nonce, fixtures]

requires:
  - phase: 03-bm1366-asic-protocol-and-safe-initialization
    provides: BM1366 CRC, packet frame, register codec, and protocol fault foundation from Plan 03-01
provides:
  - Diagnostic-only BM1366 82-byte work payload encoding
  - BM1366 job-id advance, lookup-key, and small-core semantics
  - Exact 11-byte BM1366 result frame validation and parsing
  - Typed job nonce, register read, invalid job, invalid core, preamble, length, and CRC faults
  - Reference-derived work/result fixture metadata
affects: [phase-03, phase-04, asic, firmware-uart-adapter, stratum-work-queue]

tech-stack:
  added: []
  patterns:
    - "Fixed-size ASIC payload arrays wrapped by typed Rust domain values"
    - "Typed valid-job set gates result parsing before downstream observations"
    - "Reference-derived fixture metadata remains separate from MIT source expression"

key-files:
  created:
    - crates/bitaxe-asic/src/bm1366/work.rs
    - crates/bitaxe-asic/src/bm1366/result.rs
    - crates/bitaxe-asic/fixtures/bm1366/work-result-cases.json
  modified:
    - crates/bitaxe-asic/BUILD.bazel
    - crates/bitaxe-asic/src/bm1366.rs
    - crates/bitaxe-asic/src/error.rs

key-decisions:
  - "Keep work construction explicitly diagnostic through diagnostic_job_frame."
  - "Reject stale or unknown result job IDs through Bm1366ValidJobIds before producing nonce observations."
  - "Add InvalidCoreId so nonce-derived core IDs outside the BM1366 normal-core range fail as typed protocol faults."

patterns-established:
  - "BM1366 result parsing validates length, receive preamble, CRC5 residue, register mapping, job validity, and core bounds before returning observations."
  - "BM1366 fixture objects carry checklist_ids, source_file, source_behavior, reference_commit, license_posture, and derivation."

requirements-completed: [ASIC-02, ASIC-08]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 3-2026-06-26T22-40-40
generated_at: 2026-06-27T00:15:05Z

duration: 10 min
completed: 2026-06-27
---

# Phase 03 Plan 02: BM1366 Work And Result Parsing Summary

**Diagnostic BM1366 work payload encoding and fail-closed result frame parsing in pure Rust**

## Performance

- **Duration:** 10 min
- **Started:** 2026-06-27T00:05:11Z
- **Completed:** 2026-06-27T00:15:05Z
- **Tasks:** 2
- **Files modified:** 6

## Accomplishments

- Added `Bm1366JobId`, `Bm1366WorkFields`, `Bm1366WorkPayload`, and `DiagnosticWorkFrame` for fixed 82-byte BM1366 diagnostic work payloads and 88-byte job frames.
- Added `ResultFrameBytes`, `Bm1366ValidJobIds`, `Bm1366ParsedResult`, `Bm1366NonceResult`, and `Bm1366RegisterRead` for exact 11-byte receive parsing.
- Extended reference-derived fixture metadata for work layout, job nonce results, register reads, invalid jobs, timeout, partial length, bad preamble, and bad CRC cases.

## Task Commits

Each task was committed atomically:

1. **Task 1: Encode diagnostic BM1366 work payloads and job IDs** - `d108081` (feat)
2. **Task 2: Parse BM1366 result frames and nonce-derived fields** - `c55aa69` (feat)

_Note: TDD RED failures were run and recorded before implementation, but failing intermediate states were not committed because the repo Rust pre-commit rule requires passing format, clippy, build, and tests before any commit._

## Files Created/Modified

- `crates/bitaxe-asic/src/bm1366/work.rs` - Encodes fixed-size diagnostic work payloads and job frames with job-id semantics.
- `crates/bitaxe-asic/src/bm1366/result.rs` - Validates and parses BM1366 result frames into typed nonce/register observations or typed faults.
- `crates/bitaxe-asic/fixtures/bm1366/work-result-cases.json` - Records work/result fixture metadata and provenance.
- `crates/bitaxe-asic/BUILD.bazel` - Exposes new BM1366 work/result source files to Bazel.
- `crates/bitaxe-asic/src/bm1366.rs` - Exports the new `work` and `result` modules.
- `crates/bitaxe-asic/src/error.rs` - Adds `InvalidCoreId` for out-of-range nonce-derived core IDs.

## Decisions Made

- Work frame construction stays named and scoped as diagnostic through `diagnostic_job_frame`, preserving the Phase 3 boundary before live mining integration.
- Result parsing requires a typed valid-job set so stale or untracked job IDs return `InvalidJobId` instead of producing observations.
- Out-of-range BM1366 core IDs use a dedicated `InvalidCoreId` fault instead of overloading job-id or CRC errors.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Preserved Rust pre-commit requirements during TDD**
- **Found during:** Tasks 1 and 2
- **Issue:** The generic TDD flow allows failing RED commits, but repo-local Rust rules require `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features` before every commit.
- **Fix:** Ran the RED tests to prove failure, then committed only passing task outcomes after the full Rust gate.
- **Files modified:** Task files only
- **Verification:** Full Rust pre-commit sequence passed before both task commits.
- **Committed in:** `d108081`, `c55aa69`

**2. [Rule 2 - Missing Critical] Added a typed invalid-core fault**
- **Found during:** Task 2
- **Issue:** The plan required rejecting nonce-derived `core_id` values outside `0..112`, but the existing fault enum did not have a precise fault for that invariant.
- **Fix:** Added `Bm1366ProtocolFault::InvalidCoreId` and used it when result parsing sees a core ID outside the BM1366 normal-core range.
- **Files modified:** `crates/bitaxe-asic/src/error.rs`, `crates/bitaxe-asic/src/bm1366/result.rs`
- **Verification:** `cargo test -p bitaxe-asic bm1366_result --all-features` passed and includes the out-of-range core case.
- **Committed in:** `c55aa69`

___

**Total deviations:** 2 auto-fixed (2 missing critical)
**Impact on plan:** Both deviations preserve stricter repo rules or required fail-closed behavior. No scope was added beyond pure BM1366 protocol handling.

## Issues Encountered

None beyond expected TDD RED failures.

## User Setup Required

None - no external service configuration required.

## Verification

- `cargo test -p bitaxe-asic bm1366_work --all-features` - passed, 4 tests.
- `cargo test -p bitaxe-asic bm1366_result --all-features` - passed, 6 tests.
- `cargo test -p bitaxe-asic --all-features` - passed, 18 tests.
- `bazel test //crates/bitaxe-asic:tests` - passed.
- `rg -n "ASIC-003|ASIC-004|ASIC-006" crates/bitaxe-asic/fixtures/bm1366/work-result-cases.json` - passed.
- `git status --short reference/esp-miner` - clean, no output.
- `cargo fmt --all` - passed before each task commit.
- `cargo clippy --all-targets --all-features -- -D warnings` - passed before each task commit.
- `cargo build --all-targets --all-features` - passed before each task commit.
- `cargo test --all-features` - passed before each task commit.

## Next Phase Readiness

Ready for Plan 03-03 to add active BM1366 dispatch, semantic command/observation types, and fake UART transcript coverage on top of the work/result codecs.

___

*Phase: 03-bm1366-asic-protocol-and-safe-initialization*
*Completed: 2026-06-27*

## Self-Check: PASSED

- Created files verified on disk.
- Task commits verified in git history: `d108081`, `c55aa69`.
