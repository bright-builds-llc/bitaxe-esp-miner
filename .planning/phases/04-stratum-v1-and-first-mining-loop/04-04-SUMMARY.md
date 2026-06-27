---
phase: 04-stratum-v1-and-first-mining-loop
plan: "04"
subsystem: stratum
tags: [rust, stratum-v1, mining-loop, bm1366, firmware-status, parity-evidence]

requires:
  - phase: 04-stratum-v1-and-first-mining-loop
    provides: "Plans 04-01 through 04-03 typed Stratum v1 messages, fake-pool state, mining work, share submission, and bounded work queue"
  - phase: 03-bm1366-asic-protocol-and-safe-initialization
    provides: "Typed BM1366 init status, work fields, command boundary, nonce results, and valid-job tracking"
provides:
  - "Fail-closed pure mining-loop gate requiring ASIC initialization, safety evidence, and hardware-evidence acknowledgement"
  - "Guarded typed BM1366 work dispatch planning and valid-job share-submission mapping"
  - "Firmware-visible blocked mining-loop status with live work submission disabled"
  - "Phase 4 parity evidence and checklist rows that separate pure/fake-pool proof from live hardware mining proof"
affects: [phase-04-stratum, phase-05-api-telemetry, phase-06-safety, parity-evidence]

tech-stack:
  added: []
  patterns:
    - "Gate firmware-visible mining-loop readiness through a pure fail-closed decision object"
    - "Expose blocked firmware status without live sockets or BM1366 production work submission"
    - "Record hardware smoke/soak criteria separately from unit and fake-pool evidence"

key-files:
  created:
    - crates/bitaxe-stratum/src/v1/mining_loop.rs
    - docs/parity/evidence/phase-04-stratum-v1-mining-loop.md
  modified:
    - MODULE.bazel.lock
    - crates/bitaxe-stratum/BUILD.bazel
    - crates/bitaxe-stratum/src/v1.rs
    - firmware/bitaxe/src/asic_adapter.rs
    - firmware/bitaxe/src/asic_adapter/status.rs
    - firmware/bitaxe/src/main.rs
    - docs/parity/checklist.md

key-decisions:
  - "Mining-loop work submission defaults to `hardware_evidence_ack_missing` and reaches `Ready` only when ASIC initialization, safety evidence, and hardware-evidence acknowledgement are all present."
  - "Firmware publishes `mining_loop_status=blocked reason=hardware_evidence_ack_missing work_submission=disabled` while `main.rs` remains free of live pool sockets and BM1366 work submission."
  - "Phase 4 checklist rows advance pure Stratum v1, fake-pool, job, queue, and fail-closed coordination surfaces to implemented, while live hardware mining smoke and soak remain `not run - hardware evidence pending`."
  - "Stratum v2 remains deferred by Phase 4 scope; this plan records the v1 first-loop boundary only."
  - "TDD RED failures were run but not committed because AGENTS.md requires passing Rust verification before every commit."

patterns-established:
  - "MiningLoopGate owns pure readiness decisions and applies them to MiningRuntimeState."
  - "GuardedMiningLoopInputs returns Bm1366WorkFields and optional share submissions without raw ASIC frame construction."
  - "Firmware status helpers publish disabled mining-loop state through the ASIC adapter facade."

requirements-completed: [STR-01, STR-02, STR-03, STR-04, STR-05, STR-06, STR-07]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 04-2026-06-27T13-17-33
generated_at: 2026-06-27T15:04:46Z

duration: 10 min
completed: 2026-06-27
---

# Phase 04 Plan 04: Fail-Closed Mining Loop Gate Summary

**Fail-closed Stratum v1 mining-loop gate with firmware blocked status and non-overclaiming Phase 4 evidence**

## Performance

- **Duration:** 10 min
- **Started:** 2026-06-27T14:53:47Z
- **Completed:** 2026-06-27T15:04:46Z
- **Tasks:** 3
- **Files modified:** 10

## Accomplishments

- Added `MiningLoopGate`, `MiningLoopDecision`, and guarded planning that keeps work submission blocked unless ASIC initialization, safety evidence, and hardware-evidence acknowledgement all pass.
- Added guarded typed BM1366 work dispatch planning and share-submission mapping from valid tracked nonce results without creating raw ASIC frames in Stratum.
- Added firmware boot logging for `mining_loop_status=blocked reason=hardware_evidence_ack_missing work_submission=disabled` without adding live pool sockets or work submission.
- Added Phase 4 evidence and checklist updates that mark pure/fake-pool progress without claiming live mining proof.

## Task Commits

Each task was committed atomically:

1. **Task 1: Add fail-closed mining-loop gate** - `b32316b` (feat)
2. **Task 2: Add visible firmware status without enabling live mining** - `2710317` (feat)
3. **Task 3: Update parity checklist and Phase 4 evidence** - `5274071` (docs)

## Files Created/Modified

- `crates/bitaxe-stratum/src/v1/mining_loop.rs` - Pure fail-closed mining-loop gate, guarded work dispatch planning, and tests.
- `crates/bitaxe-stratum/src/v1.rs` - Exports the new `mining_loop` module.
- `crates/bitaxe-stratum/BUILD.bazel` - Adds `mining_loop.rs` to the Bazel Rust library target.
- `firmware/bitaxe/src/asic_adapter/status.rs` - Adds the visible blocked mining-loop status helper.
- `firmware/bitaxe/src/asic_adapter.rs` - Re-exports the status helper through the firmware ASIC adapter facade.
- `firmware/bitaxe/src/main.rs` - Publishes the blocked mining-loop status after the ASIC boot gate.
- `docs/parity/checklist.md` - Updates STR-001 through STR-007 and STAT-004 with implementation and evidence pointers.
- `docs/parity/evidence/phase-04-stratum-v1-mining-loop.md` - Records pure/fake-pool evidence and hardware smoke/soak criteria with hardware run status pending.
- `MODULE.bazel.lock` - Refreshes Bazel module metadata after verification.

## Decisions Made

- Kept mining-loop readiness pure and fail-closed by default; parser and fake-pool tests cannot unlock firmware work submission.
- Used typed `Bm1366WorkFields` as the guarded dispatch output and kept raw ASIC frame construction outside `bitaxe-stratum`.
- Published a firmware status line only; no live pool socket, Stratum task, credentials, or BM1366 production work submission was added.
- Kept STR-006 and STR-007 below `verified` because no live Ultra 205 mining smoke or soak ran.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Kept TDD RED failures out of git history**
- **Found during:** Task 1 and Task 2 TDD execution
- **Issue:** The generic GSD TDD flow calls for RED commits, but AGENTS.md requires passing Rust format, clippy, build, and tests before every commit.
- **Fix:** Ran RED failures locally, implemented GREEN behavior, and committed only passing task states.
- **Files modified:** Task-owned Stratum and firmware files.
- **Verification:** RED failures were observed before implementation; final task commits passed targeted checks and the full Rust pre-commit sequence.
- **Committed in:** `b32316b`, `2710317`

**2. [Rule 3 - Blocking] Exposed firmware status helper through the ASIC adapter facade**
- **Found during:** Task 2 (Add visible firmware status without enabling live mining)
- **Issue:** `firmware/bitaxe/src/asic_adapter/status.rs` is a private submodule, so `main.rs` could not call the new helper directly.
- **Fix:** Added a narrow `pub use status::publish_mining_loop_blocked_status;` in `firmware/bitaxe/src/asic_adapter.rs`.
- **Files modified:** `firmware/bitaxe/src/asic_adapter.rs`
- **Verification:** `cargo build -p bitaxe-firmware --target xtensa-esp32s3-espidf`; full Rust pre-commit sequence.
- **Committed in:** `2710317`

**3. [Rule 3 - Blocking] Refreshed tracked Bazel module lock metadata**
- **Found during:** Task 3 verification
- **Issue:** `bazel test //crates/bitaxe-stratum:tests` refreshed tracked `MODULE.bazel.lock` metadata after the Phase 4 Stratum dependency graph changed.
- **Fix:** Committed the generated lock refresh with the evidence update so the worktree remained clean and Bazel metadata stayed reproducible.
- **Files modified:** `MODULE.bazel.lock`
- **Verification:** `bazel test //crates/bitaxe-stratum:tests`; `just parity`; full Rust pre-commit sequence.
- **Committed in:** `5274071`

**Total deviations:** 3 auto-fixed (1 missing critical, 2 blocking)
**Impact on plan:** All deviations preserved repo rules, module visibility, and reproducible Bazel verification. No live mining behavior or reference-tree mutation was introduced.

## Issues Encountered

None beyond the auto-fixed deviations above.

## Known Stubs

None.

## Authentication Gates

None.

## User Setup Required

None - no external service configuration required.

## Verification

- `cargo test -p bitaxe-stratum mining_loop --all-features` - passed
- `cargo build -p bitaxe-firmware --target xtensa-esp32s3-espidf` - passed
- `rg -n "struct MiningLoopGate|enum MiningLoopDecision|HARDWARE_EVIDENCE_ACK_MISSING|SAFETY_PREFLIGHT_EVIDENCE_MISSING|ASIC_INITIALIZED_GATE_MISSING" crates/bitaxe-stratum/src/v1/mining_loop.rs` - passed
- `rg -n "publish_mining_loop_blocked_status|mining_loop_status=blocked reason=\{reason\} work_submission=disabled" firmware/bitaxe/src/asic_adapter/status.rs` - passed
- `! rg -n "esp_transport|STRATUM_V1|submit_share|SendDiagnosticWork" firmware/bitaxe/src/main.rs` - passed
- `rg -n "STR-001|STR-002|STR-003|STR-004|STR-006|STAT-004|phase-04-stratum-v1-mining-loop" docs/parity/checklist.md docs/parity/evidence/phase-04-stratum-v1-mining-loop.md` - passed
- `cargo test -p bitaxe-stratum --all-features` - passed
- `cargo test -p bitaxe-asic --all-features` - passed
- `bazel test //crates/bitaxe-stratum:tests` - passed
- `just verify-reference` - passed, `reference clean: c1915b0a63bfabebdb95a515cedfee05146c1d50`
- `just parity` - passed with `validation_errors: none`
- `cargo fmt --all` - passed before task commits
- `cargo clippy --all-targets --all-features -- -D warnings` - passed before task commits
- `cargo build --all-targets --all-features` - passed before task commits
- `cargo test --all-features` - passed before task commits
- `git status --short reference/esp-miner` - clean

## Next Phase Readiness

Phase 04 is complete. Phase 05 can consume typed Stratum runtime state, share counters, work queue status, and fail-closed mining-loop status for API and telemetry mapping without enabling live mining.

## Self-Check: PASSED

- Found `crates/bitaxe-stratum/src/v1/mining_loop.rs`
- Found `docs/parity/evidence/phase-04-stratum-v1-mining-loop.md`
- Found `.planning/phases/04-stratum-v1-and-first-mining-loop/04-04-SUMMARY.md`
- Found task commit `b32316b`
- Found task commit `2710317`
- Found task commit `5274071`

---
*Phase: 04-stratum-v1-and-first-mining-loop*
*Completed: 2026-06-27*
