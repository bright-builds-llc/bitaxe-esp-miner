---
phase: 03-bm1366-asic-protocol-and-safe-initialization
plan: "01"
subsystem: asic
tags: [rust, bm1366, crc, packet-framing, registers, fixtures]

requires:
  - phase: 02-ultra-205-config-and-nvs-model
    provides: Ultra 205 BM1366 target identity and evidence boundaries
provides:
  - Pure BM1366 CRC5 and CRC16-FALSE helpers
  - BM1366 command and job frame constructors
  - Typed BM1366 register IDs and register payload builders
  - Reference-derived BM1366 protocol fixture metadata
affects: [phase-03, phase-04, asic, firmware-uart-adapter, parity-evidence]

tech-stack:
  added: [thiserror]
  patterns:
    - "foo.rs plus foo/ Rust module layout for bm1366 child modules"
    - "Independent bitwise CRC16-FALSE implementation instead of copied upstream table"
    - "Metadata-rich reference-derived fixture JSON"

key-files:
  created:
    - crates/bitaxe-asic/src/error.rs
    - crates/bitaxe-asic/src/bm1366.rs
    - crates/bitaxe-asic/src/bm1366/crc.rs
    - crates/bitaxe-asic/src/bm1366/packet.rs
    - crates/bitaxe-asic/src/bm1366/registers.rs
    - crates/bitaxe-asic/fixtures/bm1366/protocol-cases.json
  modified:
    - Cargo.lock
    - MODULE.bazel.lock
    - crates/bitaxe-asic/Cargo.toml
    - crates/bitaxe-asic/BUILD.bazel
    - crates/bitaxe-asic/src/lib.rs

key-decisions:
  - "Keep raw BM1366 frame construction inside bitaxe-asic through CommandFrame, JobFrame, and FrameBytes."
  - "Compute CRC16-FALSE bitwise to avoid copying the upstream GPL CRC table into MIT source."
  - "Run TDD RED failures but avoid committing failing RED states because AGENTS.md requires passing Rust checks before every commit."

patterns-established:
  - "BM1366 protocol files carry module-level reference breadcrumbs to the pinned reference tree and checklist rows."
  - "Reference-derived fixture cases include checklist_ids, source_file, source_behavior, reference_commit, license_posture, and derivation."

requirements-completed: [ASIC-01, ASIC-08]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 3-2026-06-26T22-40-40
generated_at: 2026-06-27T00:02:04Z

duration: 11 min
completed: 2026-06-27
---

# Phase 03 Plan 01: BM1366 Protocol Foundation Summary

**Pure BM1366 CRC, packet framing, register payloads, and provenance fixtures in bitaxe-asic**

## Performance

- **Duration:** 11 min
- **Started:** 2026-06-26T23:50:18Z
- **Completed:** 2026-06-27T00:02:04Z
- **Tasks:** 2
- **Files modified:** 11

## Accomplishments

- Replaced the ASIC crate's protocol-only placeholder surface with a BM1366 module facade, typed protocol fault enum, and public exports.
- Implemented CRC5, CRC16-FALSE, command frame, job frame, and BM1366 register payload construction as pure Rust.
- Added reference-derived fixture metadata with pinned commit, checklist IDs, license posture, and exact command/register-write frame bytes.

## Task Commits

Each task was committed atomically:

1. **Task 1: Establish BM1366 module graph and error contracts** - `73bcf40` (feat)
2. **Task 2: Implement CRC, packet, and register codecs** - `2846870` (feat)
3. **Generated build metadata: Refresh Bazel crate lock** - `bfef44e` (chore)

_Note: TDD RED failures were run and recorded before implementation, but failing intermediate states were not committed because the repo Rust pre-commit rule requires passing format, clippy, build, and tests before any commit._

## Files Created/Modified

- `Cargo.lock` - Records the new `bitaxe-asic` dependency on workspace `thiserror`.
- `MODULE.bazel.lock` - Records the generated crate mirror metadata after the new `thiserror` dependency.
- `crates/bitaxe-asic/Cargo.toml` - Adds `thiserror.workspace = true`.
- `crates/bitaxe-asic/BUILD.bazel` - Includes new BM1366 source files, fixture compile data, and `@crates//:thiserror`.
- `crates/bitaxe-asic/src/lib.rs` - Exports BM1366 modules and tests the public protocol contract.
- `crates/bitaxe-asic/src/error.rs` - Defines `Bm1366ProtocolFault`.
- `crates/bitaxe-asic/src/bm1366.rs` - Provides the BM1366 facade, constants, child modules, and breadcrumbs.
- `crates/bitaxe-asic/src/bm1366/crc.rs` - Implements CRC5 and CRC16-FALSE.
- `crates/bitaxe-asic/src/bm1366/packet.rs` - Implements frame bytes plus command/job frame constructors.
- `crates/bitaxe-asic/src/bm1366/registers.rs` - Implements typed register IDs and payload builders.
- `crates/bitaxe-asic/fixtures/bm1366/protocol-cases.json` - Stores BM1366 protocol fixture provenance and exact frame bytes.

## Decisions Made

- Raw transmit preamble, header, length, payload, and CRC construction now lives in `bitaxe-asic`, not firmware.
- CRC16-FALSE is implemented bitwise from the documented polynomial and initial value rather than by copying the upstream table.
- RED failures are recorded in execution evidence but not committed when doing so would violate AGENTS.md Rust commit requirements.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Preserved Rust pre-commit requirements during TDD**
- **Found during:** Tasks 1 and 2
- **Issue:** The generic TDD flow allows failing RED commits, but repo-local Rust rules require `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features` before every commit.
- **Fix:** Ran the RED tests to prove failure, then committed only passing task outcomes after the full Rust gate.
- **Files modified:** Task files only
- **Verification:** Full Rust pre-commit sequence passed before both task commits.
- **Committed in:** `73bcf40`, `2846870`

---

**Total deviations:** 1 instruction-driven adjustment
**Impact on plan:** No scope change. The implementation still followed TDD behavior while respecting stricter repo commit rules.

## Issues Encountered

None beyond expected TDD RED failures.

## User Setup Required

None - no external service configuration required.

## Verification

- `cargo test -p bitaxe-asic bm1366_contract --all-features` - passed
- `cargo test -p bitaxe-asic bm1366_crc --all-features` - passed
- `cargo test -p bitaxe-asic bm1366_packet --all-features` - passed
- `cargo test -p bitaxe-asic bm1366_register --all-features` - passed
- `cargo fmt --all` - passed before each task commit
- `cargo clippy --all-targets --all-features -- -D warnings` - passed before each task commit
- `cargo build --all-targets --all-features` - passed before each task commit
- `cargo test --all-features` - passed before each task commit
- `cargo test -p bitaxe-asic --all-features` - passed, 8 tests
- `bazel test //crates/bitaxe-asic:tests` - passed
- `git status --short reference/esp-miner` - clean, no output

## Next Phase Readiness

Ready for Plan 03-02 to build BM1366 work encoding, job ID semantics, result parsing, and nonce/register fault handling on top of the protocol foundation.

---

*Phase: 03-bm1366-asic-protocol-and-safe-initialization*
*Completed: 2026-06-27*

## Self-Check: PASSED

- Created files verified on disk.
- Task commits verified in git history: `73bcf40`, `2846870`, `bfef44e`.
