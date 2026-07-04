# Phase 22 Blocker Reasons

## Status

phase22_blocker_reason_status: implemented
board: 205
redaction_status: safe_reason_strings_only
evidence_class: unit,workflow

## Purpose

This ledger records the stable, redaction-safe blocker reason strings used by the Phase 22 production-mining prerequisite contract. These strings are safe to log, serialize, cite in evidence, and expose through operator/API surfaces because they name a blocker category without including pool credentials, device endpoints, Wi-Fi data, NVS values, raw Stratum payloads, raw share payloads, or raw BM1366 frames.

## Stable Reasons

| Reason | Category | Meaning | Primary Rust-owned target |
| --- | --- | --- | --- |
| `power_sample_stale` | power | A power sample exists but is too old for the mining attempt. | `crates/bitaxe-safety/src/power.rs`, `crates/bitaxe-safety/src/mining_preconditions.rs` |
| `power_sample_unavailable` | power | Power telemetry is unavailable. | `crates/bitaxe-safety/src/power.rs`, `crates/bitaxe-safety/src/mining_preconditions.rs` |
| `input_voltage_unsafe` | power, voltage | The observed input voltage is outside the safe operating window. | `crates/bitaxe-safety/src/power.rs`, `crates/bitaxe-safety/src/mining_preconditions.rs` |
| `thermal_reading_unavailable` | thermal | Thermal telemetry is unavailable. | `crates/bitaxe-safety/src/thermal.rs`, `crates/bitaxe-safety/src/mining_preconditions.rs` |
| `thermal_reading_invalid` | thermal | Thermal telemetry exists but is invalid for a safe decision. | `crates/bitaxe-safety/src/thermal.rs`, `crates/bitaxe-safety/src/mining_preconditions.rs` |
| `fan_observation_unavailable` | fan | Fan observation data is unavailable for the mining attempt. | `crates/bitaxe-safety/src/mining_preconditions.rs` |
| `fan_observation_stale` | fan | Fan observation data exists but is too old for the mining attempt. | `crates/bitaxe-safety/src/mining_preconditions.rs` |
| `voltage_observation_unavailable` | voltage | Voltage observation data is unavailable for the mining attempt. | `crates/bitaxe-safety/src/mining_preconditions.rs` |
| `voltage_observation_stale` | voltage | Voltage observation data exists but is too old for the mining attempt. | `crates/bitaxe-safety/src/mining_preconditions.rs` |
| `bounded_observation_ambiguous` | bounded evidence | Bounded evidence exists but has no positive validity window. | `crates/bitaxe-safety/src/mining_preconditions.rs` |
| `bounded_observation_undocumented` | bounded evidence | Bounded evidence lacks source, evidence id, or reason. | `crates/bitaxe-safety/src/mining_preconditions.rs` |
| `bounded_observation_board_mismatch` | bounded evidence | Bounded evidence is not scoped to board `205`. | `crates/bitaxe-safety/src/mining_preconditions.rs` |
| `hardware_evidence_ack_missing` | safety | The mining loop lacks the explicit hardware-evidence acknowledgement required before dispatch. | `crates/bitaxe-stratum/src/v1/mining_loop.rs` |
| `safety_preflight_evidence_missing` | safety | Overall safety evidence is missing or unsafe for work submission. | `crates/bitaxe-safety/src/mining_preconditions.rs`, `crates/bitaxe-stratum/src/v1/mining_loop.rs` |

## Operator Visibility

Phase 22 carries exact blocker reasons through these Rust-owned surfaces:

| Surface | Behavior | Target |
| --- | --- | --- |
| Safety decision | A blocked prerequisite returns `ProductionMiningPreconditionDecision::Blocked { reason, plan }`. | `crates/bitaxe-safety/src/mining_preconditions.rs` |
| Mining loop | A blocked production precondition returns `MiningLoopDecision::Blocked { reason }` before dispatch. | `crates/bitaxe-stratum/src/v1/mining_loop.rs` |
| Runtime state | Safe-blocked work submission stores `maybe_blocked_reason`. | `crates/bitaxe-stratum/src/v1/state.rs` |
| API projection | Safe-blocked mining output renders `blockedReason` from runtime state. | `crates/bitaxe-api/src/mining.rs` |

## Redaction Boundary

Stable reason strings are category labels only. They must not be extended with raw pool URLs, ports, workers, owner addresses, passwords, tokens, device URLs, IPs, MACs, Wi-Fi values, NVS secret values, raw Stratum payloads, raw share payloads, or raw BM1366 frames.

## Evidence Level

This ledger supports `SAFE-11` at `implemented` with `unit,workflow` evidence. It proves stable reason taxonomy and exact fail-closed reason propagation, but it does not verify live hardware observations or active safety-control behavior.

