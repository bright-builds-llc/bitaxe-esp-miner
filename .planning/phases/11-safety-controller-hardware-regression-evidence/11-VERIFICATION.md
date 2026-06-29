---
phase: 11
phase_slug: 11-safety-controller-hardware-regression-evidence
verified: 2026-06-29T21:41:49Z
status: passed
score: "12/12 must-haves verified"
generated_by: gsd-verify-work
lifecycle_mode: yolo
phase_lifecycle_id: 11-2026-06-29T20-23-34
generated_at: 2026-06-29T21:41:49Z
lifecycle_validated: true
overrides_applied: 0
---

# Phase 11: Safety Controller Hardware Regression Evidence Verification Report

**Phase Goal:** Ultra 205 safety-critical hardware surfaces have documented hardware-regression evidence before they are promoted beyond implemented or safe-unavailable status.
**Verified:** 2026-06-29T21:41:49Z
**Status:** passed
**Re-verification:** No - initial verification

## Goal Achievement

Phase 11 achieved the evidence-governance goal. The codebase now has a board-205-only safety evidence ledger, a component evidence-pack contract, generated wrapper-owned Ultra 205 flash-monitor evidence, redaction review, conservative checklist citations, and a parity validator that prevents active safety-control rows from being marked `verified` without `hardware-regression`.

This pass does not claim active-control parity. Active DS4432U voltage writes, fan duty effects, live thermal/fault paths, self-test hardware submodes, runtime input/display parity, INA260/EMC2101 freshness, API/WebSocket live telemetry, and mining/load stress remain below verified unless later exact evidence exists.

### Observable Truths

| # | Truth | Status | Evidence |
| --- | --- | --- | --- |
| 1 | The phase plan documents the recovery path before destructive, fault-injection, or hardware-actuation verification runs. | VERIFIED | `11-01-PLAN.md` requires the runbook first; the ledger has `Hardware Gate`, `Recovery Protocol`, `Allowed Command Set`, and `Prohibited Commands Unless A Later Task Adds Bounded Recovery`. |
| 2 | Ultra 205 safety evidence records exact commands, board, port, commits, logs, observed behavior, and conclusion, or records pending status for unobserved active claims. | VERIFIED | `flash-command-evidence.json` records board `205`, `/dev/cu.usbmodem1101`, firmware commit `048e765e854d24ac87b711c1dfe92bcaa2d4c085`, reference commit `c1915b0a63bfabebdb95a515cedfee05146c1d50`, commands, trusted output, and conclusion. |
| 3 | Self-test, display/input status, watchdog, and load responsiveness evidence is captured or explicitly remains below verified with follow-up. | VERIFIED | `flash-monitor.log` records display/input runtime-gap and watchdog supervisor start/yield markers; checklist rows `UI-001`, `UI-002`, `UI-003`, `SELF-001`, and `STAT-002` remain below verified for runtime/hardware/load subclaims. |
| 4 | Parity tooling continues to reject safety-critical `verified` rows without hardware-smoke or hardware-regression evidence. | VERIFIED | `validate_rows` keeps the safety-critical hardware evidence guard and `cargo test -p bitaxe-parity --all-features` passed 49 tests. |
| 5 | A board-205-only Phase 11 runbook exists before live safety evidence work. | VERIFIED | `docs/parity/evidence/phase-11-safety-controller-hardware-regression-evidence.md` starts from `just detect-ultra205` and stops on zero ports, multiple ports, board-info failure, non-205 target, missing recovery, missing evidence procedure, or redaction uncertainty. |
| 6 | Every safety surface has claim type, allowed probe, pass criteria, failure criteria, evidence token, and promotion boundary. | VERIFIED | The `Surface Evidence Matrix` includes `PWR-001`, `PWR-002`, `PWR-003`, `PWR-005`, `PWR-006`, `THR-001`, `THR-002`, `THR-003`, `IO-001`, `UI-001`, `UI-002`, `UI-003`, `SELF-001`, `STAT-002`, `API-002`, `API-006`, and `EVD-05`. |
| 7 | Actuation, stress, destructive, fault-injection, and self-test hardware paths stop unless bounded steps, abort conditions, and recovery are documented. | VERIFIED | The ledger prohibits active DS4432U voltage writes, fan duty actuation, overheat stimulus, fault injection, self-test hardware submodes, mining stress, raw I2C writes, raw flash writes, erase, rollback, and interrupted-update procedures. |
| 8 | Evidence artifacts require board, port, source commit, reference commit, command/probe, logs, conclusion, and redaction review. | VERIFIED | The ledger and artifact README require those fields; `redaction-review.md` reviewed generated JSON/log/terminal output and concluded no redaction was required. |
| 9 | Parity tooling rejects active safety-control rows marked verified without hardware-regression evidence. | VERIFIED | `is_active_safety_control` covers `PWR-001`, `PWR-002`, `PWR-003`, `PWR-005`, `THR-001`, `THR-002`, `SELF-001`, and `UI-003`; validation emits `requires hardware-regression evidence`. |
| 10 | Read-only or safe-unavailable hardware-smoke evidence remains allowed only for exact narrow observations. | VERIFIED | `active_safety_control_allows_read_only_hardware_smoke_rows` covers `PWR-006`; checklist notes cite narrow safe boot/runtime-gap/watchdog observations without promoting mixed rows. |
| 11 | Checklist updates cannot accidentally verify voltage writes, fan actuation, self-test hardware, runtime input, or overheat/fault paths from broad smoke evidence. | VERIFIED | Affected active rows stay `implemented` or `in-progress`; `just parity` passed with `validation_errors: none`. |
| 12 | Final verification proves safety tests, parity tests, aggregate tests, parity report, and reference read-only status remain green. | VERIFIED | Local verification passed: safety cargo tests, parity cargo tests, `just parity`, `just test`, `git diff -- reference/esp-miner --exit-code`, and `git diff --check`. |

**Score:** 12/12 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
| --- | --- | --- | --- |
| `docs/parity/evidence/phase-11-safety-controller-hardware-regression-evidence.md` | Phase 11 ledger, runbook, matrix, execution log, and final conclusions | VERIFIED | Exists, substantive, cites detector gate, exact safe boot artifacts, pending active-control boundaries, and final verification. |
| `docs/parity/evidence/phase-11-safety-controller-hardware-regression-evidence/README.md` | Component evidence pack contract | VERIFIED | Defines `safe-baseline`, `power-telemetry`, `voltage-control`, `thermal-fan`, `self-test-watchdog-load`, `display-input`, and `parity-guard`. |
| `docs/parity/evidence/phase-11-safety-controller-hardware-regression-evidence/redaction-review.md` | Secret-redaction review | VERIFIED | Reviewed generated artifacts and terminal output; conclusion passed with no required redaction. |
| `docs/parity/evidence/phase-11-safety-controller-hardware-regression-evidence/flash-command-evidence.json` | Wrapper-generated machine evidence | VERIFIED | Records `trusted_output=true`, `capture_status=timed_out_after_trusted_output`, board `205`, port, commits, manifest/image paths, commands, and conclusion. |
| `docs/parity/evidence/phase-11-safety-controller-hardware-regression-evidence/flash-monitor.log` | Wrapper-generated serial log | VERIFIED | Contains boot identity, safe state, runtime display/input gap, OTA boot-validation state, and watchdog supervisor markers. |
| `docs/parity/checklist.md` | Conservative Phase 11 checklist citations | VERIFIED | Affected safety rows cite Phase 11 evidence while leaving active/mixed claims below verified. |
| `tools/parity/src/main.rs` | Active safety-control validation | VERIFIED | Implements safety-critical and active-control guards with unit coverage. |

### Key Link Verification

| From | To | Via | Status | Details |
| --- | --- | --- | --- | --- |
| Phase 11 ledger | `AGENTS.md` | Ultra 205 detector and evidence rules | VERIFIED | Ledger requires `just detect-ultra205` before live evidence. |
| Phase 11 ledger | `docs/parity/checklist.md` | Surface matrix row IDs and checklist citations | VERIFIED | Ledger includes every required safety row; checklist cites the Phase 11 evidence path. |
| `tools/parity/src/main.rs` | `docs/parity/checklist.md` | Checklist row ID validation | VERIFIED | Parser validates checklist rows and active-control IDs. |
| Phase 11 ledger | wrapper-generated artifacts | `flash-command-evidence.json` and `flash-monitor.log` | VERIFIED | Manual `rg` verified both artifact names and observed markers in the ledger and generated files. |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
| --- | --- | --- | --- | --- |
| `tools/parity/src/main.rs` | `ChecklistRow` list | `parse_checklist` reads `docs/parity/checklist.md`, then `validate_rows` enforces evidence tokens | Yes | FLOWING |
| `docs/parity/checklist.md` | Phase 11 row notes and evidence tokens | Human-edited checklist rows citing Phase 11 ledger | Yes | FLOWING |
| Phase 11 ledger | Execution log and conclusions | Wrapper JSON/log artifacts plus redaction review | Yes | FLOWING |
| `flash-command-evidence.json` and `flash-monitor.log` | Captured board metadata and serial markers | `just flash-monitor board=205 port=/dev/cu.usbmodem1101 evidence-dir=...` | Yes | FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
| --- | --- | --- | --- |
| Pure safety decisions remain covered | `cargo test -p bitaxe-safety --all-features` | 32 tests plus doc-test harness passed | PASS |
| Parity validator enforces active safety-control guard | `cargo test -p bitaxe-parity --all-features` | 49 tests passed | PASS |
| Checklist has no invalid verified overclaims | `just parity` | Passed with `validation_errors: none` | PASS |
| Aggregate repo test surface remains green | `just test` | 13 Bazel test targets passed | PASS |
| Reference tree remains read-only | `git diff -- reference/esp-miner --exit-code` | No diff | PASS |
| Diff has no whitespace errors | `git diff --check` | Passed | PASS |
| Evidence markers are present | `rg "trusted_output|safe_state:|display_input_status|safety_supervisor"` on Phase 11 artifacts | Expected JSON/log markers found | PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
| --- | --- | --- | --- | --- |
| SAFE-01 | 11-01, 11-02, 11-03 | Voltage and power-control surfaces fail closed and are not overclaimed | SATISFIED for Phase 11 evidence posture | Ledger records active DS4432U/VCORE pending; parity guard requires `hardware-regression`; checklist stays below verified. |
| SAFE-02 | 11-01, 11-02, 11-03 | Thermal and fan surfaces expose readings/failure reporting or stay pending | SATISFIED for Phase 11 evidence posture | Thermal/fan matrix rows and checklist notes keep live sensor/fan duty evidence pending. |
| SAFE-03 | 11-01, 11-03 | PID and thermal-control decisions covered by pure unit tests | SATISFIED | `cargo test -p bitaxe-safety --all-features` passed; `THR-003` remains unit-evidenced only. |
| SAFE-04 | 11-01, 11-02, 11-03 | Fault paths fail closed and expose status without unsafe claims | SATISFIED for Phase 11 evidence posture | Fault/overheat rows require bounded recovery and `hardware-regression`; no fault injection was run. |
| SAFE-05 | 11-01, 11-02, 11-03 | Self-test lifecycle covered or held pending for hardware submodes | SATISFIED for Phase 11 evidence posture | Pure self-test tests pass; `SELF-001` remains below verified for hardware submodes. |
| SAFE-06 | 11-01, 11-02, 11-03 | Display/input preserved or explicitly documented as deferred gap | SATISFIED | Log contains `display_input_status=runtime_gap reason=hardware_evidence_pending startup_only=true`; checklist keeps runtime parity pending. |
| SAFE-07 | 11-01, 11-03 | Power/current/voltage/fan/temperature telemetry captured where exposed, or held pending | SATISFIED for Phase 11 evidence posture | Detector-gated safe boot evidence captured; INA260, EMC2101, fan RPM, and live telemetry freshness remain pending instead of overclaimed. |
| SAFE-08 | 11-01, 11-02, 11-03 | Safety-critical surfaces cannot be verified without hardware evidence | SATISFIED | Safety-critical guard plus active-control `hardware-regression` guard are implemented and tested; `just parity` passes. |
| SAFE-09 | 11-01, 11-03 | Watchdog/load responsiveness avoids starvation and remains observable | SATISFIED for Phase 11 evidence posture | Watchdog supervisor start/yield markers captured; bounded load/stress remains pending. |
| EVD-05 | 11-01, 11-02, 11-03 | Evidence layers include tests, hardware smoke, and regression/soak evidence where appropriate | SATISFIED | Evidence ledger, wrapper artifacts, redaction review, cargo tests, `just parity`, `just test`, and reference diff check are recorded. |

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| --- | --- | --- | --- | --- |
| None | N/A | No TODO/FIXME/placeholder/stub markers found in Phase 11 touched scopes | N/A | No blocker anti-patterns found. |

### Human Verification Required

None for the Phase 11 evidence-governance goal. Live active-control and stress/fault scenarios are intentionally not verified by this phase; they require future bounded recovery procedures before they can be claimed.

### Gaps Summary

No blocking gaps found for Phase 11. The phase passed because it produced detector-gated Ultra 205 safe boot evidence, redaction review, conservative checklist posture, and machine-enforced parity guards. The unverified active-control and runtime/live-telemetry surfaces remain explicitly pending and are not treated as Phase 11 failures.

_Verified: 2026-06-29T21:41:49Z_
_Verifier: the agent (gsd-verifier)_
