---
status: diagnosed
phase: 27-live-hardware-asic-and-stratum-bridge
source: 27-01-SUMMARY.md, 27-02-SUMMARY.md, 27-03-SUMMARY.md, 27-04-SUMMARY.md
started: 2026-07-06T16:30:00Z
updated: 2026-07-06T16:45:00Z
---

## Current Test

[testing complete]

## Tests

### 1. Phase 27 compile-time mode/ack opt-in
expected: Distinct `PHASE27_LIVE_HARDWARE_BRIDGE_MODE` + ACK enables Phase 27 bridge; mismatched or missing values fail closed (no silent Phase 25 fallback).
result: pass
verified_by: agent
evidence: `bazel test //crates/bitaxe-stratum:tests //crates/bitaxe-asic:tests` passed; `mining_evidence_mode.rs` unit tests for `Phase27LiveHardwareBridge` and fail-closed paths.

### 2. Automated Phase 27 test gate
expected: Stratum, ASIC, parity mining-allow, and Phase 27 evidence script tests pass without errors.
result: pass
verified_by: agent
evidence: `bazel test //crates/bitaxe-stratum:tests //crates/bitaxe-asic:tests //tools/parity:tests //scripts:phase27_live_hardware_bridge_evidence_test --test_output=errors` — 4/4 passed (2026-07-06).

### 3. Production UART retained for bridge
expected: After work-result bootstrap, production UART executor is stored in `OnceLock` and reused for bridge dispatch (not re-created per job).
result: pass
verified_by: agent
evidence: `firmware/bitaxe/src/asic_adapter/production.rs` `store_production_peripherals`; Phase 27 VERIFICATION.md truth #2; hardware runs show sustained 1M baud TX after bootstrap.

### 4. Live bridge dispatches pool work after Stratum notify
expected: Phase 27 live capture logs `asic_bridge_status=work_dispatched` after pool settings consumed and Stratum runtime active.
result: pass
verified_by: agent
evidence: `b4-init-state-20260706-retry-H3b/share-outcome.md` `asic_bridge_status: work_dispatched`; B2 pool consumed marker fixed in retry matrix.

### 5. Observation-to-submit correlation (host)
expected: `apply_bridge_observation` queues submit intent only when nonce observation matches active dispatch generation; unit tests cover correlation paths.
result: pass
verified_by: agent
evidence: `crates/bitaxe-stratum/src/v1/live_runtime.rs` `apply_bridge_observation`; `27-02-SUMMARY.md` self-check; stratum tests passed.

### 6. Evidence wrapper and mining-allow governance
expected: Repo-owned `phase27-live-hardware-bridge-evidence.sh` produces redacted blocked-mode artifacts; mining-allow accepts `blocked_safe_prerequisite` and rejects overbroad verified share claims.
result: pass
verified_by: agent
evidence: `scripts/phase27-live-hardware-bridge-evidence-test` passed; `docs/parity/evidence/phase-27-live-hardware-asic-and-stratum-bridge/share-outcome.md`; checklist rows at `implemented` not `verified` for share claims.

### 7. Production result read uses ~10s poll window
expected: After production work dispatch, bridge polls results for ~10s (not ~1s double-read bug); logs show ~47 `result_read_attempt` markers over ~9960ms.
result: pass
verified_by: agent
evidence: B3 P0 fix; `b3-production-read-20260706-retry/` and `b4-init-state-20260706-retry-G3.md` (~9960ms window, 47 polls).

### 8. Post-dispatch UART proof before production success
expected: Within 10s after pool production work TX, serial logs show `asic_uart_trace=rx_chunk`, `register_read_parsed`, or equivalent nonce parse — production does not fail closed on timeout alone when ASIC is healthy.
result: issue
reported: "All B3/B4 hardware matrix runs (G1, G3, H1, H3b) show zero post-dispatch RX; `production_result_timeout` after ~10s poll despite correct TX framing and init fixes."
severity: blocker
verified_by: agent
evidence: `work-result-hypothesis-results.md` B3/B4 tables; canonical `b4-init-state-20260706-retry-H3b/conclusion.md`.

### 9. Live accepted/rejected share with ASIC bridge correlation
expected: Detector-gated hardware capture records `share_outcome: accepted` or `rejected` tied to live ASIC-derived submit intent and bridge correlation markers (STR-09 verified tier).
result: issue
reported: "Committed and retry hardware evidence remains `share_outcome: blocked_safe_prerequisite`; no pool accepted/rejected response correlated to ASIC nonce observation."
severity: blocker
verified_by: agent
evidence: `b4-init-state-20260706-retry-H3b/share-outcome.md`; `phase27-04-defer-note.md` share tier NOT MET.

### 10. Phase 27-04 checklist promotion integrity
expected: Checklist promotion stays conservative — STR-09 and ASIC-11 do not claim verified live share proof while blocked; defer note documents blocked tiers honestly.
result: pass
verified_by: agent
evidence: `docs/parity/checklist.md` rows at `implemented`; `phase27-04-defer-note.md` explicit deferral; `27-04-SUMMARY.md` self-check passed.

## Summary

total: 10
passed: 8
issues: 2
pending: 0
skipped: 0
blocked: 0

## Gaps

- truth: "Post-dispatch UART shows rx_chunk or register_read_parsed within 10s after pool production work"
  status: failed
  reason: "Hardware matrix (B3 F1 retry, B4 G1/G3/H1/H3b): work_dispatched then production_result_timeout; zero post-dispatch asic_uart_trace=rx_chunk despite ~9960ms polled read loop and golden-matched TX."
  severity: blocker
  test: 8
  root_cause: "Init/timing/read-window tuning exhausted (W1–W10, W8 ramp, W9 delay, skip_boot_diagnostic). Remaining upstream divergences: notify-driven bridge vs continuous ASIC_result_task + create_jobs_task orchestration (H4); W13 mining-enable semantics (InitializedNoMining bootstrap without UART proof); possible ASIC mining-enable state not equivalent to upstream task-start path."
  artifacts:
    - path: ".planning/phases/27-live-hardware-asic-and-stratum-bridge/work-result-upstream-diff-v5.md"
      issue: "Steps 11–14 diverge from upstream task orchestration; register init matched"
    - path: ".planning/phases/27-live-hardware-asic-and-stratum-bridge/b4-init-state-20260706-retry-H3b/live-capture-runtime/flash-monitor.log"
      issue: "production_result_timeout at ~38940ms; no post-dispatch rx_chunk"
    - path: "firmware/bitaxe/src/live_stratum_runtime.rs"
      issue: "Notify-driven dispatch; no upstream-style ASIC_result_task loop"
  missing:
    - "H4: Align result receive + job pump with upstream ASIC_result_task / create_jobs_task timing"
    - "Optional Wave 4: upstream baseline UART capture on same hardware for diff"
    - "Require UART proof tier before checklist verified promotion"
  debug_session: ".planning/phases/27-live-hardware-asic-and-stratum-bridge/work-result-hypothesis-results.md"

- truth: "Detector-gated live share accepted or rejected with ASIC bridge correlation markers"
  status: failed
  reason: "Downstream of test 8: no nonce observation → no correlated submit → share_outcome remains blocked_safe_prerequisite across all committed hardware evidence."
  severity: blocker
  test: 9
  root_cause: "Blocked by missing post-dispatch UART/nonce (gap test 8). STR-09 verified tier correctly not promoted."
  artifacts:
    - path: "docs/parity/evidence/phase-27-live-hardware-asic-and-stratum-bridge/share-outcome.md"
      issue: "share_outcome blocked_safe_prerequisite"
    - path: ".planning/phases/27-live-hardware-asic-and-stratum-bridge/phase27-04-defer-note.md"
      issue: "Share tier NOT MET; 27-04 promotion deferred"
  missing:
    - "Resolve test 8 UART/nonce gap first"
    - "Re-run detector-gated phase27-evidence --mode hardware after fix"
  debug_session: ".planning/phases/27-live-hardware-asic-and-stratum-bridge/phase27-04-defer-note.md"
