---
phase: 03-bm1366-asic-protocol-and-safe-initialization
verified: 2026-06-27T02:08:06Z
status: passed
score: 13/13 must-haves verified
generated_by: gsd-verifier
lifecycle_mode: yolo
phase_lifecycle_id: 3-2026-06-26T22-40-40
generated_at: 2026-06-27T02:08:06Z
lifecycle_validated: true
overrides_applied: 0
requirements_checked:
  - ASIC-01
  - ASIC-02
  - ASIC-03
  - ASIC-04
  - ASIC-05
  - ASIC-06
  - ASIC-07
  - ASIC-08
residual_hardware_evidence_debt:
  - "Live Ultra 205 BM1366 chip-detect was not run; docs/parity/evidence/phase-03-ultra-205-bm1366-chip-detect.md remains not run - hardware verification pending."
  - "ASIC initialization, serial transport, diagnostic work-send/result-receive, frequency transition, voltage, fan, thermal, power, and production mining remain below verified until board-named hardware evidence exists."
---

# Phase 03: BM1366 ASIC Protocol And Safe Initialization Verification Report

**Phase Goal:** Firmware can communicate with BM1366 through typed pure protocol logic and a narrow UART adapter, with live Ultra 205 initialization guarded and fail-closed.
**Verified:** 2026-06-27T02:08:06Z
**Status:** passed
**Re-verification:** No - initial verification

## Goal Achievement

Phase 03 achieved the safe non-live goal. The implementation provides typed BM1366 protocol logic, exact-fault parsing, active Ultra 205 BM1366 dispatch, fail-closed init planning, and a gated firmware UART/reset/status adapter. The live Ultra 205 chip-detect smoke was intentionally not run and is not overclaimed.

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | Developer can run BM1366 packet, register, CRC, work-encoding, result-parsing, nonce, domain, and error-case fixtures with upstream-compatible outputs. | VERIFIED | `cargo test -p bitaxe-asic --all-features` passed 48 tests, including CRC, packet, register, work, result, transcript, chip-detect, init-plan, and frequency/voltage tests. Fixture metadata exists under `crates/bitaxe-asic/fixtures/bm1366/`. |
| 2 | Firmware can reset, preflight, and stage BM1366 initialization on Ultra 205 only when required board, power, thermal, safety, and config gates pass. | VERIFIED | `Bm1366InitPlan::chip_detect_only` and `full_init` gate board/config/power/thermal/safety evidence and emit typed adapter actions. Full init fails before effectful stages when power/thermal/safety evidence is missing. |
| 3 | Unsafe or incomplete ASIC initialization conditions fail closed with visible logs/status instead of enabling mining or hardware control. | VERIFIED | Default firmware logs `asic_status=preflight_missing reason=hardware_evidence_ack_missing initialized=false mining=disabled work_submission=disabled`; adapter setup/I/O/chip-id validation failures publish fail-closed status and attempt reset-low when reset is available. |
| 4 | Firmware translates typed ASIC commands and observations through a narrow UART adapter without leaking raw protocol details into user-facing control logic. | VERIFIED | `firmware/bitaxe/src/asic_adapter.rs` interprets `Bm1366AdapterAction`; raw preamble/CRC constants are absent from `firmware/bitaxe/src`. `rg -n "0x55|0xaa|0x1366|crc5|crc16" firmware/bitaxe/src` returned no matches. |
| 5 | Developer can inspect reference breadcrumbs and parity checklist rows for BM1366 behavior, including hardware-smoke evidence before release parity is claimed. | VERIFIED | BM1366 modules include reference breadcrumbs; `docs/parity/checklist.md` rows `ASIC-001` through `ASIC-007` cite implementation/evidence and keep live rows below `verified`; chip-detect evidence remains `not run - hardware verification pending`. |

**Score:** 13/13 must-haves verified across 5 roadmap success criteria and ASIC-01 through ASIC-08.

### Required Artifacts

| Artifact | Expected | Status | Details |
|---|---|---|---|
| `crates/bitaxe-asic/src/bm1366.rs` and `bm1366/` children | BM1366 public facade and pure protocol modules | VERIFIED | Facade exports `adapter_gate`, `chip_detect`, `command`, `crc`, `frequency_voltage`, `init_plan`, `observation`, `packet`, `registers`, `result`, `transcript`, and `work`. |
| `crates/bitaxe-asic/src/bm1366/crc.rs`, `packet.rs`, `registers.rs` | CRC5, CRC16-FALSE, command/job frame, register codecs | VERIFIED | Command frames use CRC5, job frames use CRC16-FALSE, register IDs are typed, and CRC16 is computed without copying the upstream table. |
| `crates/bitaxe-asic/src/bm1366/work.rs`, `result.rs` | Diagnostic work payloads and result parsing | VERIFIED | Work payload is fixed 82 bytes and diagnostic-only; result parsing validates exact 11-byte frames, preamble, CRC, job IDs, core bounds, register IDs, and version bits before observations. |
| `crates/bitaxe-asic/src/dispatch.rs`, `command.rs`, `observation.rs`, `transcript.rs`, `chip_detect.rs` | Active dispatch, semantic adapter boundary, transcript and chip-detect fault handling | VERIFIED | BM1366 is the only active V1 path; non-205 ASICs are deferred. Transcript/chip-detect paths cover timeout, partial read, bad preamble, bad CRC, unknown register, invalid job, chip-count mismatch, and setup/I/O fail-closed behavior. |
| `crates/bitaxe-asic/src/bm1366/init_plan.rs`, `frequency_voltage.rs` | Fail-closed staged init and pure transition decisions | VERIFIED | Board/config gates reuse Phase 2 facts; full init needs power, thermal, and safety evidence. Frequency/voltage decisions reuse validation types and carry `MissingHardwareEvidence`/`ImplementedNotVerified`. |
| `firmware/bitaxe/src/asic_adapter.rs`, `asic_adapter/uart.rs`, `reset.rs`, `status.rs` | Narrow firmware UART/reset/status adapter | VERIFIED | Firmware compiles for `xtensa-esp32s3-espidf`, defaults fail-closed, and only enters chip-detect mode when both compile-time evidence gates are present. |
| `docs/parity/checklist.md` and `docs/parity/evidence/phase-03-ultra-205-bm1366-chip-detect.md` | Checklist/evidence boundary without false verified hardware claims | VERIFIED | `just parity` passed with `validation_errors: none`; chip-detect file concludes `not run - hardware verification pending`. |

### Key Link Verification

| From | To | Via | Status | Details |
|---|---|---|---|---|
| `crates/bitaxe-asic/src/lib.rs` | `crates/bitaxe-asic/src/bm1366.rs` | `pub mod bm1366` | VERIFIED | gsd-tools key-link check passed for Plan 03-01. |
| `packet.rs` | `crc.rs` | CommandFrame uses CRC5; JobFrame uses CRC16-FALSE | VERIFIED | gsd-tools key-link check passed for Plan 03-01. |
| `work.rs` | `packet.rs` | JobFrame wraps BM1366 work payload | VERIFIED | gsd-tools key-link check passed for Plan 03-02. |
| `result.rs` | `registers.rs` and `error.rs` | Register mapping and typed faults | VERIFIED | gsd-tools key-link check passed for Plan 03-02. |
| `dispatch.rs` | `bitaxe-config/src/catalog.rs` | Active/deferred dispatch consumes catalog facts | VERIFIED | gsd-tools key-link check passed for Plan 03-03. |
| `command.rs` | `packet.rs` | Commands encode through crate-owned frame bytes | VERIFIED | gsd-tools key-link check passed for Plan 03-03. |
| `transcript.rs` | `result.rs` | Transcript bytes decode to observations/faults | VERIFIED | gsd-tools key-link check passed for Plan 03-03. |
| `init_plan.rs` | `bitaxe-config/src/catalog.rs` and `command.rs` | Init gates and adapter actions | VERIFIED | gsd-tools key-link check passed for Plan 03-04. |
| `frequency_voltage.rs` | `bitaxe-config/src/validation.rs` | Frequency/voltage constructors | VERIFIED | gsd-tools key-link check passed for Plan 03-04. |
| `firmware/bitaxe/src/asic_adapter.rs` | `command.rs`, `adapter_gate.rs`, `status.rs` | Typed action interpretation and fail-closed status | VERIFIED | gsd-tools key-link check passed for Plan 03-05. |
| `docs/parity/checklist.md` | Phase 03 evidence file | Checklist cites evidence without false verified status | VERIFIED | gsd-tools key-link check passed for Plan 03-05. |

### Data-Flow Trace

| Artifact | Data Variable | Source | Produces Real Data | Status |
|---|---|---|---|---|
| `firmware/bitaxe/src/main.rs` | safe-state and ASIC status logs | `Phase1SafeState::default()` plus `asic_adapter::run_boot_gate()` | Yes - logs safe no-mining/no-work state and default fail-closed ASIC status | VERIFIED |
| `adapter_gate.rs` -> `asic_adapter.rs` | `AsicAdapterMode` | `BITAXE_ASIC_DIAGNOSTIC` and `BITAXE_HARDWARE_EVIDENCE_ACK` compile-time envs | Yes - default is `FailClosed`; exact chip-detect ack is required for diagnostic effects | VERIFIED |
| `init_plan.rs` -> `asic_adapter.rs` | `Bm1366AdapterAction` sequence | Phase 2 board/config facts plus explicit preflight evidence tokens | Yes - chip-detect-only actions are emitted; full init blocks without evidence | VERIFIED |
| `uart.rs` -> `chip_detect.rs` -> `status.rs` | chip-id response bytes and status | UART exact 11-byte read validated by preamble, CRC, chip ID, and expected count | Yes in typed code path; live hardware data not captured yet | VERIFIED for code path; hardware evidence pending |
| `frequency_voltage.rs` | transition evidence status | Phase 2 validation newtypes | Yes - accepted pure decisions carry `MissingHardwareEvidence`, not verified hardware status | VERIFIED |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|---|---|---|---|
| BM1366 pure protocol, init, transcript, and adapter gate tests | `cargo test -p bitaxe-asic --all-features` | 48 passed, 0 failed | PASS |
| Bazel-visible ASIC tests | `bazel test //crates/bitaxe-asic:tests` | passed, cached | PASS |
| Firmware adapter compiles for ESP32-S3 ESP-IDF target | `source "$HOME/export-esp.sh" && cargo check -p bitaxe-firmware --target xtensa-esp32s3-espidf` | finished successfully | PASS |
| Checklist validity and false verified guard | `just parity` | `validation_errors: none` | PASS |
| Reference implementation remains clean | `git status --short reference/esp-miner` | no output | PASS |
| Full pre-commit Rust gate | main-context checks | `cargo fmt --all`, clippy, build, and `cargo test --all-features` were reported passed before verification | PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|---|---|---|---|---|
| ASIC-01 | 03-01 | BM1366 packet, register, and CRC codecs as pure Rust with fixtures | SATISFIED | `crc.rs`, `packet.rs`, `registers.rs`, `protocol-cases.json`, and unit tests for CRC/packet/register behavior. |
| ASIC-02 | 03-02 | Work encoding and result parsing for payloads, nonces, domains, and error cases | SATISFIED | `work.rs` encodes diagnostic work; `result.rs` parses 11-byte result frames and rejects invalid job/core/register/preamble/CRC cases. |
| ASIC-03 | 03-03 | BM1366 active path and other ASIC families deferred/not verified | SATISFIED | `dispatch.rs` returns `ActiveBm1366` only for Ultra 205/BM1366/count 1/ActiveUltra205; BM1370/BM1368/BM1397 are deferred. |
| ASIC-04 | 03-03, 03-05 | Narrow UART adapter boundary between typed logic and ESP-IDF serial I/O | SATISFIED | Firmware interprets `Bm1366AdapterAction`; raw BM1366 preamble/CRC constants do not appear in firmware source. |
| ASIC-05 | 03-04, 03-05 | Reset, preflight, and staged init fail closed unless gates pass | SATISFIED | `init_plan.rs` blocks missing board/config/power/thermal/safety evidence; firmware setup/I/O errors publish fail-closed status. Live init remains unverified. |
| ASIC-06 | 03-01, 03-04 | Frequency/voltage decisions range-checked and require hardware evidence before verified | SATISFIED | `frequency_voltage.rs` reuses Phase 2 validation and stores missing evidence status; checklist keeps frequency row `implemented`, not `verified`. |
| ASIC-07 | 03-05 | Hardware-smoke evidence required before release parity is claimed | SATISFIED AS GATE | Evidence file explicitly says `not run - hardware verification pending`; checklist rows `ASIC-002` through `ASIC-007` stay below `verified`, so no release parity is claimed. |
| ASIC-08 | 03-01 through 03-05 | Reference breadcrumbs and parity checklist rows | SATISFIED | BM1366 modules and fixture metadata point to pinned reference paths/commit; checklist rows cover BM1366 behavior and deferred BM1370 scope. |

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|---|---:|---|---|---|
| None blocking | - | - | - | No TODO/FIXME/placeholder, hardcoded empty output, raw protocol leakage into firmware, or false hardware-verified checklist rows were found. `_ => {}` matches in `firmware/bitaxe/src/asic_adapter.rs` are constrained to fail-closed helper loops that intentionally ignore non-reset/non-status actions. |

### Human Verification Required

None blocking Phase 03's safe non-live goal.

Future human-approved hardware verification remains before promoting live BM1366 rows to `verified`: run the command template in `docs/parity/evidence/phase-03-ultra-205-bm1366-chip-detect.md` on a safe bench Ultra 205, capture board/port/commit/log evidence, and only then consider promoting live chip-detect/serial/init rows.

### Gaps Summary

No blocking gaps found. Phase 03 is complete for typed pure BM1366 protocol logic, fail-closed staged init decisions, and the narrow default-off firmware adapter. Residual debt is intentionally preserved as evidence debt, not hidden: live chip-detect, ASIC initialization, serial transport, diagnostic work-send/result-receive, frequency transition, voltage, fan, thermal, power, and production mining are not hardware-verified.

### Disconfirmation Pass

- Partial requirement checked: ASIC-07 is only satisfied as a gate. The implementation does not contain hardware smoke, and the checklist correctly prevents release parity claims.
- Potentially misleading test checked: unit and golden tests prove packet/result/init logic, not physical UART timing or reset electrical behavior. The report keeps those as residual hardware evidence debt.
- Error path checked: review fixes added fail-closed setup and I/O paths; current tests cover reset unavailable, UART unavailable, chip-detect response invalid, and adapter I/O failure behavior.

---

_Verified: 2026-06-27T02:08:06Z_
_Verifier: the agent (gsd-verifier)_
