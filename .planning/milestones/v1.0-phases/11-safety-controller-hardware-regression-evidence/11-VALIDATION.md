---
phase: 11
slug: safety-controller-hardware-regression-evidence
status: passed
nyquist_compliant: true
wave_0_complete: true
created: 2026-06-29T20:51:50Z
generated_by: gsd-plan-phase
lifecycle_mode: yolo
phase_lifecycle_id: 11-2026-06-29T20-23-34
---

# Phase 11 - Validation Strategy

Per-phase validation contract for feedback sampling during execution.

## Test Infrastructure

| Property | Value |
| --- | --- |
| Framework | Rust unit tests, Bazel test targets, just command surface, and hardware evidence artifacts |
| Config file | `Cargo.toml`, `MODULE.bazel`, `Justfile` |
| Quick run command | `cargo test -p bitaxe-safety --all-features` |
| Full suite command | `just test` |
| Evidence command | `just detect-ultra205`; if exactly one Ultra 205 is confirmed, `just flash-monitor board=205 port=<port> evidence-dir=<dir>` |
| Estimated runtime | Quick unit feedback is expected to be under 60 seconds; full Bazel feedback and firmware hardware evidence are environment-dependent. |

## Sampling Rate

- After every task commit: run the narrow automated command named in that task's acceptance criteria.
- After every plan wave: run `cargo test -p bitaxe-safety --all-features`, `cargo test -p bitaxe-parity --all-features`, and `just parity` when the touched surface includes safety or checklist evidence.
- Before verification: run `just test`, `just parity`, lifecycle validation, and hardware evidence commands when the Ultra 205 detector succeeds.
- Max feedback latency: no three consecutive implementation tasks may proceed without automated feedback or an explicit manual-only evidence note.

## Per-Requirement Verification Map

| Requirement | Behavior | Test Type | Automated Command | Manual Or Hardware Evidence | Status |
| --- | --- | --- | --- | --- | --- |
| SAFE-01 | Voltage and power-control decisions fail closed and separate DS4432U write claims from INA260 read-only telemetry. | unit plus hardware evidence | `cargo test -p bitaxe-safety --all-features` | Phase 11 ledger keeps active DS4432U writes and INA260 freshness pending unless exact evidence exists. | passed with active-control evidence pending |
| SAFE-02 | Thermal and fan readings, duty behavior, RPM behavior, and failures are evidenced or held pending. | unit plus hardware evidence | `cargo test -p bitaxe-safety --all-features` | Phase 11 ledger keeps EMC2101, fan RPM, and fan duty evidence pending unless exact evidence exists. | passed with thermal/fan evidence pending |
| SAFE-03 | PID and thermal-control decisions stay covered before hardware effects are enabled. | unit | `cargo test -p bitaxe-safety --all-features` | Pure PID remains unit-tested; no fan actuation was run. | passed |
| SAFE-04 | Overheat, fan, power, thermal, and ASIC fault paths enter safe states and expose status. | unit plus guarded manual/hardware evidence | `cargo test -p bitaxe-safety --all-features` | Fault-injection and failure-path evidence remains pending without bounded recovery. | passed with fault-path evidence pending |
| SAFE-05 | Self-test lifecycle covers factory flags, start, pass, fail, restart, cancel, and visible result reporting. | unit plus guarded manual/hardware evidence | `cargo test -p bitaxe-safety --all-features` | Self-test hardware submodes remain pending; watchdog supervisor markers were captured. | passed with self-test hardware evidence pending |
| SAFE-06 | Display and input status needed for Ultra 205 administration is preserved or documented as deferred. | artifact review plus parity | `just parity` | Runtime display/input gap marker was captured; runtime input/display parity remains pending. | passed with runtime parity pending |
| SAFE-07 | Power, current, voltage, fan, and temperature telemetry are captured where hardware exposes them. | hardware evidence | `just detect-ultra205` | Detector and wrapper safe boot evidence captured board, port, commits, commands, board-info, logs, behavior, and conclusion. | passed for safe boot evidence |
| SAFE-08 | Safety-critical verified rows require hardware-smoke or hardware-regression evidence. | unit plus parity report | `cargo test -p bitaxe-parity --all-features`; `just parity` | Parity guard now requires hardware-regression for active safety-control verified rows. | passed |
| SAFE-09 | Mining, control, API, and telemetry tasks avoid watchdog starvation and preserve responsiveness under load. | unit plus guarded manual/hardware evidence | `cargo test -p bitaxe-safety --all-features` | Watchdog startup/yield markers captured; mining/load stress remains pending. | passed with load evidence pending |
| EVD-05 | Evidence layers include unit tests, golden fixtures, API comparison, hardware smoke, and hardware regression or soak evidence where appropriate. | integration plus artifact review | `just test`; `just parity` | Phase 11 evidence ledger, wrapper artifacts, and redaction review completed. | passed |

## Wave 0 Requirements

- [x] `docs/parity/evidence/phase-11-safety-controller-hardware-regression-evidence.md` - create the Phase 11 ledger/runbook/matrix for SAFE-01 through SAFE-09 and EVD-05.
- [x] Official DS4432U documentation check - active voltage-register behavior stayed out of execution because official documentation was not verified.
- [x] Hardware availability gate - `just detect-ultra205` ran before live board interaction and detector output was recorded.

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
| --- | --- | --- | --- |
| Ultra 205 hardware presence and board-info capture | SAFE-07, EVD-05 | Requires connected hardware over USB. | Run `just detect-ultra205`; if exactly one port is confirmed, record port, board-info, source commit, and reference commit. |
| DS4432U active voltage write behavior | SAFE-01 | Actuation can affect hardware safety and requires a recovery path. | Do not run unless the executable plan documents bounded values, abort conditions, and recovery instructions. |
| Fan duty actuation and physical fan response | SAFE-02, SAFE-04 | Physical actuator behavior cannot be proven by pure tests. | Prefer read-only temperature/RPM first; only attempt duty actuation with bounded probe steps and recovery. |
| Overheat, ASIC fault, and self-test failure paths | SAFE-04, SAFE-05 | Fault injection can be destructive or disruptive. | Record pending unless the plan includes phase-gated recovery and expected safe-state observations. |
| Runtime display/input administration | SAFE-06 | Current startup OLED evidence does not prove runtime input/display parity. | Review existing gap evidence and keep runtime display/input below verified unless a safe live route exists. |

## Security Threat Coverage

| Threat Ref | Surface | Secure Behavior | Validation |
| --- | --- | --- | --- |
| T-11-01 | Hardware evidence files | Do not record Wi-Fi credentials, pool credentials, private endpoints, or NVS secret values. | Redaction review before verification and commit. |
| T-11-02 | Actuator commands | Unsafe voltage/fan/fault/self-test actions remain gated behind documented recovery. | Plan review plus hardware command review before execution. |
| T-11-03 | Parity checklist promotion | Safety-critical rows cannot become verified without hardware-smoke or hardware-regression evidence. | `cargo test -p bitaxe-parity --all-features`; `just parity`. |

## Validation Sign-Off

- [x] All tasks have automated verification or a documented manual-only evidence path.
- [x] Sampling continuity: no three consecutive implementation tasks without automated verification or explicit evidence-gated pending status.
- [x] Wave 0 covers all missing evidence references from research.
- [x] No watch-mode flags in verification commands.
- [x] Hardware interaction follows the Ultra 205 detector and evidence rules.
- [x] `nyquist_compliant: true` set in frontmatter when the executed plan satisfies this contract.

Approval: approved 2026-06-29
