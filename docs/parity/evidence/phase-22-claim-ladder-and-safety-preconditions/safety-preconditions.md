# Phase 22 Safety Preconditions

## Status

phase22_safety_preconditions_status: implemented
precondition_contract: fresh_or_explicitly_bounded
board: 205
evidence_class: unit,workflow
redaction_status: safe_reason_strings_only

## Purpose

This ledger records the Phase 22 production-mining prerequisite contract for Ultra 205. It is evidence for prerequisite readiness semantics only: it explains which typed inputs must exist before BM1366 production work dispatch can be considered, how stale or unsafe inputs fail closed, and which Rust-owned targets enforce the contract.

Phase 22 did not run a new detector-gated hardware mining session. It does not claim accepted shares, rejected shares, unbounded production mining, full active voltage/fan/thermal/self-test/fault-stimulus closure, active DS4432U voltage control, fan actuation, thermal fault stimulus, self-test hardware closure, fault-stimulus closure, non-205 boards, Stratum v2, OTA/recovery trust, runtime display/input parity, or BAP behavior.

## Contract

`precondition_contract: fresh_or_explicitly_bounded`

Production mining readiness for board `205` requires every prerequisite category to be represented as a typed `ProductionMiningPrerequisite` before BM1366 work dispatch:

| Category | Accepted evidence | Blocking examples | Rust-owned target |
| --- | --- | --- | --- |
| `power` | Fresh `PowerObservation` or valid `BoundedObservationEvidence` scoped to board `205` | `power_sample_unavailable`, `power_sample_stale`, `input_voltage_unsafe`, `bounded_observation_undocumented`, `bounded_observation_ambiguous`, `bounded_observation_board_mismatch` | `crates/bitaxe-safety/src/mining_preconditions.rs` |
| `thermal` | Fresh safe `ThermalObservation` or valid `BoundedObservationEvidence` scoped to board `205` | `thermal_reading_unavailable`, `thermal_reading_invalid`, `bounded_observation_undocumented`, `bounded_observation_ambiguous`, `bounded_observation_board_mismatch` | `crates/bitaxe-safety/src/mining_preconditions.rs` |
| `fan` | Fresh fan observation from the shell or valid `BoundedObservationEvidence` scoped to board `205` | `fan_observation_unavailable`, `fan_observation_stale`, `bounded_observation_undocumented`, `bounded_observation_ambiguous`, `bounded_observation_board_mismatch` | `crates/bitaxe-safety/src/mining_preconditions.rs` |
| `voltage` | Fresh voltage observation from the shell or valid `BoundedObservationEvidence` scoped to board `205` | `voltage_observation_unavailable`, `voltage_observation_stale`, `input_voltage_unsafe`, `bounded_observation_undocumented`, `bounded_observation_ambiguous`, `bounded_observation_board_mismatch` | `crates/bitaxe-safety/src/mining_preconditions.rs` |
| `safety` | Fresh overall safe state or valid `BoundedObservationEvidence` scoped to board `205` | `safety_preflight_evidence_missing`, `hardware_evidence_ack_missing`, `bounded_observation_undocumented`, `bounded_observation_ambiguous`, `bounded_observation_board_mismatch` | `crates/bitaxe-safety/src/mining_preconditions.rs`, `crates/bitaxe-stratum/src/v1/mining_loop.rs` |

Bounded observations are accepted only when they include a nonempty source, board `205`, a nonempty evidence id, a nonzero validity window, and a nonempty reason. Missing source, evidence id, or reason is `bounded_observation_undocumented`; a zero validity window is `bounded_observation_ambiguous`; any board other than `205` is `bounded_observation_board_mismatch`.

## Dispatch Gate

The safety contract is consumed before BM1366 work dispatch:

| Step | Behavior | Rust-owned target |
| --- | --- | --- |
| Typed prerequisite decision | `ProductionMiningPreconditions::decision()` returns `Ready` only when power, thermal, fan, voltage, and safety are fresh or explicitly bounded. | `crates/bitaxe-safety/src/mining_preconditions.rs` |
| Fail-closed effect plan | Blocked decisions carry the stable reason and `SafetyEffectPlan::fail_closed(reason)`, including work-submission blocking and hardware-control suppression. | `crates/bitaxe-safety/src/mining_preconditions.rs` |
| Mining loop gate | `MiningLoopGate::decision()` checks `production_preconditions` before legacy power, thermal, safety, hardware-ack, and ASIC-init gates. | `crates/bitaxe-stratum/src/v1/mining_loop.rs` |
| Runtime state projection | `MiningRuntimeState::block_work_submission(reason)` stores the exact fail-closed reason for operator/API visibility. | `crates/bitaxe-stratum/src/v1/state.rs` |
| API visibility | `mining_state_from_runtime()` maps the exact runtime blocker to `blockedReason`. | `crates/bitaxe-api/src/mining.rs` |

## Evidence Level

This ledger supports `SAFE-10` at `implemented` with `unit,workflow` evidence. It proves the typed prerequisite model, the fail-closed decision shape, and exact blocker propagation through Rust-owned pure/runtime state surfaces.

This ledger does not support `SAFE-10` or `SAFE-11` at `verified`, because Phase 22 did not run detector-gated hardware evidence proving the exact production-mining prerequisite behavior on live power, thermal, fan, voltage, and safety observations.

## Verification

- `bazel test //crates/bitaxe-safety:tests //crates/bitaxe-stratum:tests //crates/bitaxe-api:tests`
- `just parity`
- `just verify-reference`

