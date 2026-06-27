---
phase: 03-bm1366-asic-protocol-and-safe-initialization
plan: "05"
subsystem: asic
tags: [rust, bm1366, firmware-adapter, uart, fail-closed, parity-evidence, hardware-gate]

requires:
  - phase: 03-bm1366-asic-protocol-and-safe-initialization
    provides: Safe BM1366 init planning, typed adapter actions, and unverified frequency/voltage decisions from Plans 03-03 and 03-04
provides:
  - Fail-closed BM1366 firmware adapter gate
  - Typed firmware boundary for BM1366 UART/reset/status effects
  - Host-testable diagnostic compile-env gate
  - Phase 3 Ultra 205 BM1366 chip-detect evidence record
  - Checklist updates that keep ASIC live behavior below verified without hardware evidence
affects: [phase-03, phase-04, firmware-uart-adapter, hardware-evidence, parity-checklist, ultra-205]

tech-stack:
  added: []
  patterns:
    - "Firmware interprets typed BM1366 adapter actions and keeps raw protocol bytes inside bitaxe-asic"
    - "Default firmware ASIC behavior is fail-closed unless both diagnostic compile-time gates are present"
    - "Human-gated chip-detect evidence must remain not run until a named Ultra 205 hardware-smoke log exists"

key-files:
  created:
    - crates/bitaxe-asic/src/bm1366/adapter_gate.rs
    - firmware/bitaxe/src/asic_adapter.rs
    - firmware/bitaxe/src/asic_adapter/uart.rs
    - firmware/bitaxe/src/asic_adapter/reset.rs
    - firmware/bitaxe/src/asic_adapter/status.rs
    - docs/parity/evidence/phase-03-ultra-205-bm1366-chip-detect.md
  modified:
    - Cargo.lock
    - MODULE.bazel.lock
    - crates/bitaxe-asic/BUILD.bazel
    - crates/bitaxe-asic/src/bm1366.rs
    - firmware/bitaxe/Cargo.toml
    - firmware/bitaxe/sdkconfig.defaults
    - firmware/bitaxe/src/main.rs
    - docs/parity/checklist.md

key-decisions:
  - "Use the safe skip path for the human-gated checkpoint because no live Ultra 205 flashing/monitoring approval or port was provided."
  - "Do not run flash, monitor, chip-detect, or port detection during continuation."
  - "Keep `phase-03-ultra-205-bm1366-chip-detect.md` concluded as `not run - hardware verification pending`."
  - "Keep ASIC-002 through ASIC-007 below `verified` until board-named Ultra 205 chip-detect hardware-smoke evidence exists."

patterns-established:
  - "AdapterGate requires `BITAXE_ASIC_DIAGNOSTIC=chip-detect` plus `BITAXE_HARDWARE_EVIDENCE_ACK=ultra205-chip-detect-safe-bench` before diagnostic hardware effects are allowed."
  - "Parity checklist rows cite evidence files while preserving a status/evidence distinction between pure unit coverage and live hardware proof."

requirements-completed: [ASIC-04, ASIC-05, ASIC-07, ASIC-08]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 3-2026-06-26T22-40-40
generated_at: 2026-06-27T01:19:47Z

duration: 5 min continuation
completed: 2026-06-27
---

# Phase 03 Plan 05: BM1366 Firmware Adapter Summary

**Fail-closed BM1366 firmware adapter with explicit Ultra 205 chip-detect evidence gating**

## Performance

- **Duration:** 5 min continuation from checkpoint
- **Started:** 2026-06-27T01:19:11Z
- **Completed:** 2026-06-27T01:19:47Z
- **Tasks:** 3
- **Files modified:** 14

## Accomplishments

- Added a host-tested `AsicAdapterMode` gate and exact fail-closed ASIC status string.
- Added firmware UART/reset/status adapter modules that interpret typed `Bm1366AdapterAction` values while defaulting to no UART/GPIO effects.
- Recorded the Phase 3 Ultra 205 BM1366 chip-detect evidence boundary with live execution explicitly pending.
- Updated ASIC checklist rows so ASIC-002 through ASIC-007 remain below `verified` without board-named Ultra 205 chip-detect hardware-smoke evidence.

## Task Commits

Each implementation task was committed atomically:

1. **Task 1: Implement narrow firmware ASIC adapter with default fail-closed status** - `525f1b8` (feat)
2. **Task 2: Record Phase 3 parity evidence boundaries and checklist updates** - `d8e1208` (docs)
3. **Task 3: Human-gated Ultra 205 chip-detect smoke review** - safe skip path, no implementation commit

## Files Created/Modified

- `crates/bitaxe-asic/src/bm1366/adapter_gate.rs` - Pure diagnostic mode gate and default fail-closed status tests.
- `firmware/bitaxe/src/asic_adapter.rs` - Firmware interpreter for typed BM1366 adapter actions.
- `firmware/bitaxe/src/asic_adapter/uart.rs` - UART1 TX17/RX18 constants and chip-detect/result timeout behavior.
- `firmware/bitaxe/src/asic_adapter/reset.rs` - Reset/hold-low adapter constants for the fail-closed boundary.
- `firmware/bitaxe/src/asic_adapter/status.rs` - Visible ASIC status publication.
- `firmware/bitaxe/src/main.rs` - Preserves safe-state boot log and emits the fail-closed ASIC status by default.
- `docs/parity/evidence/phase-03-ultra-205-bm1366-chip-detect.md` - Chip-detect command template and skipped-gate evidence record.
- `docs/parity/checklist.md` - ASIC rows updated to distinguish unit/workflow evidence from live hardware verification.

## Decisions Made

- The checkpoint was resolved through the safe skip path from the orchestrator. No flash, monitor, chip-detect, port detection, or hardware assumption was performed.
- The evidence conclusion remains `not run - hardware verification pending`.
- ASIC initialization, serial transport, result receive, frequency transition, and diagnostic work behavior remain below `verified` until a future run captures board-named Ultra 205 hardware-smoke logs.

## Deviations from Plan

None - the plan explicitly allowed the live run to be skipped when no approval or port was provided. The continuation completed that branch without changing evidence status.

## Issues Encountered

- Task 3 reached a human-verification checkpoint with no live hardware approval. The orchestrator selected the safe skip branch, so the plan completed without hardware actions.

## User Setup Required

None for this plan completion. Future verification still requires a human-approved safe bench Ultra 205 setup and an explicit port before chip-detect can run.

## Known Stubs

None. The `not run - hardware verification pending` text is the intentional evidence state for the skipped live checkpoint, not a stub.

## Verification

- `rg -n "Conclusion|not run - hardware verification pending|hardware-smoke|Ultra 205|BM1366" docs/parity/evidence/phase-03-ultra-205-bm1366-chip-detect.md docs/parity/checklist.md` - passed; evidence remains pending and checklist rows preserve the hardware boundary.
- `just parity` - passed with `validation_errors: none`.
- `cargo test -p bitaxe-asic --all-features` - passed, 44 tests.
- `bazel test //crates/bitaxe-asic:tests` - passed.
- `git status --short reference/esp-miner` - clean, no output.

## Next Phase Readiness

Phase 03 is complete for safe, non-live work. The residual risk is hardware evidence: ASIC init, serial chip-detect, frequency transition, diagnostic work-send, result receive, voltage, fan, thermal, power, and production mining remain unverified until a future approved Ultra 205 hardware-smoke or regression run records logs.

---

*Phase: 03-bm1366-asic-protocol-and-safe-initialization*
*Completed: 2026-06-27*

## Self-Check: PASSED

- Summary file verified on disk.
- Task commits verified in git history: `525f1b8`, `d8e1208`.
- Safe-skip evidence boundary verified: chip-detect conclusion remains `not run - hardware verification pending`.
