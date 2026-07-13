---
phase: 21-live-mining-and-soak-evidence
status: passed
generated_by: gsd-yolo-discuss-plan-execute-commit-and-push
lifecycle_mode: yolo
phase_lifecycle_id: 21-2026-07-04T01-35-47
generated_at: 2026-07-04T19:05:00Z
lifecycle_validated: true
requirements:
  - ASIC-07
  - STR-06
  - STR-07
  - SAFE-09
  - EVD-05
---

# Phase 21 Verification

final_phase_status: passed
phase21_evidence_closure: approved_controlled_no_share_soak
controlled_runtime_harness_status: observed
controlled_runtime_harness_observation_status: observed in live smoke and bounded soak
redaction_review: passed
reference_clean: passed
just_test: passed
just_parity: passed
just_verify_reference: passed
lifecycle_validation: passed
source_commit_after_traceability: b5502a6
reference_commit: c1915b0
summary_artifact: docs/parity/evidence/phase-21-live-mining-and-soak-evidence/summary.md
commit_push_gate: clean - commit and push allowed

Phase 21 now has redaction-reviewed controlled no-share live smoke and approved bounded controlled no-share soak evidence. The live-smoke wrapper records pool input bridge application, controlled runtime markers, subscribe/authorize/notify flow, typed BM1366 work dispatch, bounded no-result/no-share, runtime snapshot/API/WebSocket telemetry updates, watchdog checkpoints, and final safe-stop. The bounded soak records a 300-second approved controlled no-share window, watchdog responsiveness, redacted telemetry captures, and final safe-stop.

Accepted and rejected shares were not observed and remain non-claims. ASIC frequency transition, active voltage/fan/fault controls, OTA/recovery, destructive/fault-injection, unbounded stress, and non-205 board behavior remain outside the verified Phase 21 closure.

## Verification Commands

| Command | Result |
| --- | --- |
| `bash -n scripts/phase21-live-mining-evidence.sh scripts/phase21-live-mining-evidence-test.sh` | passed |
| `bash scripts/phase21-live-mining-evidence-test.sh` | passed |
| `bazel run //tools/parity:report -- mining-allow --manifest docs/parity/evidence/phase-21-live-mining-and-soak-evidence/live-mining-smoke/allow-live-mining-smoke.json --surface mining-smoke --allowed-command <manifest allowed_command>` | passed |
| `scripts/phase21-live-mining-evidence.sh --surface mining-smoke ... --device-url <redacted> --pool-credentials pool-credentials.json` | passed; controlled no-share evidence recorded |
| `bazel run //tools/parity:report -- mining-allow --manifest docs/parity/evidence/phase-21-live-mining-and-soak-evidence/bounded-soak/allow-bounded-soak.json --surface bounded-soak --allowed-command <manifest allowed_command>` | passed |
| `scripts/phase21-live-mining-evidence.sh --surface bounded-soak --duration-seconds 300 ... --device-url <redacted> --pool-credentials pool-credentials.json` | passed; approved bounded controlled no-share soak recorded |
| `just test` | passed |
| `just parity` | passed |
| `just verify-reference` | passed |
| `git diff --check` | passed |
| `git diff -- reference/esp-miner --exit-code` | passed |

## Evidence Closure

| Artifact | Status | Closure impact |
| --- | --- | --- |
| `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/summary.md` | passed | Final ledger records `phase21_status: passed` and `phase21_evidence_closure: approved_controlled_no_share_soak`. |
| `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/redaction-review.md` | passed | Redaction passed for committed artifacts. |
| `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/live-mining-smoke.md` | controlled-no-share | Live smoke records controlled no-share runtime markers and safe-stop. |
| `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/live-api-websocket-telemetry.md` | passed | Explicit-target API/WebSocket correlation captured and redacted. |
| `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/bounded-soak.md` | approved_controlled_no_share_soak | 300-second bounded controlled no-share soak recorded. |
| `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/bounded-soak/watchdog-observations.md` | passed | Bounded watchdog responsiveness recorded. |

rejected_overclaims: accepted_shares, rejected_shares, unbounded_production_mining, ASIC_frequency_transition, active_voltage_fan_fault_control, OTA_recovery, non_205_board_behavior
