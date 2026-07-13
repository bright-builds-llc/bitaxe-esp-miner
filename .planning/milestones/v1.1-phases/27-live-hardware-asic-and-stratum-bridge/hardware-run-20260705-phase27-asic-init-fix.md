# Phase 27 ASIC Init Fix — Hardware Runs 2026-07-05

## Software changes

- Added `AsicAdapterMode::Phase27ProductionBridge` gated by `BITAXE_MINING_EVIDENCE_MODE` + Phase 27 ack (resolves ack collision with work-result diagnostic mode).
- Phase 27 boot runs chip-detect prelude then work-result UART bootstrap; retains production UART on diagnostic parse success.
- Phase 27 `mining_loop_gate` uses `production_ready()` instead of hardcoded `asic_initialized: true`.

## Retry 3 (boot path only, no chip-detect prelude)

- Boot: `asic_work_result_diagnostic=started`, `bm1366_diagnostic_work=dispatched`
- Boot: `bm1366_diagnostic_result=timeout` → no UART retention
- Bridge: `phase25_prerequisite_status=asic_initialized_gate_missing` (not `production_asic_init_failed`)
- Outcome: `blocked_safe_prerequisite`, `safe_stop_status=complete`

## Retry 4 (chip-detect prelude + diagnostic)

- Boot: `asic_status=chip_detect_only` then `chip_detect_adapter_error` (partial UART read 9/11 bytes)
- Bridge: `asic_initialized_gate_missing`
- Outcome: `blocked_safe_prerequisite`, `safe_stop_status=complete`

## Conclusion

The `production_asic_init_failed` software gap is fixed: Phase 27 firmware no longer FailClosed at boot without attempting ASIC init, and the bridge gates on real production readiness. Remaining blocker is BM1366 UART/chip-detect on this hardware session (same partial-read pattern as Phase 12 chip-detect evidence). Committed parity evidence not promoted (no accepted/rejected share).
