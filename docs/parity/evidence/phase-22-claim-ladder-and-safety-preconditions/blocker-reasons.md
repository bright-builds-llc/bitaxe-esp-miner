# Production Mining Blocker Reasons

## Status

phase22_blocker_reason_status: implemented
board: 205
redaction_status: safe_reason_strings_only
evidence_class: unit,workflow

## Purpose

This ledger records the closed, redaction-safe production-session reason
vocabulary that reaches Rust operator/API surfaces. The original Phase 22
ledger described a pre-production mining-loop seam that no longer exists. The
current source of truth is the typed `ProductionSessionBlocker` model used by
the production readiness, recovery, runtime-state, and API projection chain.

These strings are category labels only. They contain no pool credentials,
device endpoints, Wi-Fi data, NVS values, raw Stratum/share payloads, raw
BM1366 frames, telemetry values, or runtime identifiers.

## Stable Reasons

| Reason | Class | Observable behavior |
| --- | --- | --- |
| `operator_paused` | operator intent | Work submission is disabled; mining activity is `paused`; API `blockedReason` is empty because this is not a failure. |
| `network_unavailable` | network readiness | Work submission is disabled; mining activity is `safe_blocked`; API exposes the exact reason. |
| `stratum_v1_unsupported` | protocol readiness | Work submission is disabled; mining activity is `safe_blocked`; API exposes the exact reason. |
| `safety_prerequisites_stale` | safety readiness | Work submission is disabled; mining activity is `safe_blocked`; API exposes the exact reason. |
| `campaign_lease_unavailable` | campaign admission | Work submission is disabled; mining activity is `safe_blocked`; API exposes the exact reason. |
| `campaign_lease_consumed` | campaign lifecycle | Work submission is disabled; mining activity is `safe_blocked`; API exposes the exact reason. |
| `campaign_activation_timed_out` | campaign lifecycle | Work submission is disabled; mining activity is `safe_blocked`; API exposes the exact reason. |
| `production_asic_unavailable` | ASIC readiness | Work submission is disabled; mining activity is `safe_blocked`; API exposes the exact reason. |
| `production_asic_version_mask_unavailable` | ASIC capability | Work submission is disabled; mining activity is `safe_blocked`; API exposes the exact reason. |
| `production_asic_dispatch_unavailable` | ASIC dispatch | Work submission is disabled; mining activity is `safe_blocked`; API exposes the exact reason. |
| `production_asic_poll_unavailable` | ASIC receive | Work submission is disabled; mining activity is `safe_blocked`; API exposes the exact reason. |
| `production_asic_queue_full` | ASIC dispatch | Work submission is disabled; mining activity is `safe_blocked`; API exposes the exact reason. |
| `production_asic_worker_unavailable` | ASIC ownership | Work submission is disabled; mining activity is `safe_blocked`; API exposes the exact reason. |
| `job_transition_protocol_inconsistent` | work lifecycle | Work submission is disabled; mining activity is `safe_blocked`; API exposes the exact reason. |
| `actuation_unqualified` | safety actuation | Work submission is disabled; mining activity is `safe_blocked`; API exposes the exact reason. |
| `pool_configuration_unavailable` | pool configuration | Work submission is disabled; mining activity is `safe_blocked`; API exposes the exact reason. |
| `pools_exhausted` | pool recovery | Work submission is disabled; mining activity is `safe_blocked`; API exposes the exact reason. |

## Current Production Chain

| Surface | Behavior | Rust-owned target |
| --- | --- | --- |
| Closed vocabulary and readiness precedence | `ProductionSessionBlocker` owns all labels; `ProductionReadiness::maybe_blocker()` chooses the first admission failure before effects. | `crates/bitaxe-stratum/src/v1/recovery_policy.rs` |
| Production snapshot | A typed blocker calls `block_work_submission(blocker.label())`; only `OperatorPaused` is then represented as `Paused`, while failures remain `SafeBlocked`. | `crates/bitaxe-stratum/src/v1/production_session/runtime.rs` |
| Runtime state | Blocking disables work, records the exact stable label, and defaults mining activity to `SafeBlocked`; allowing work clears the prior reason. | `crates/bitaxe-stratum/src/v1/state.rs` |
| API projection | `blockedReason` is emitted only when work is blocked and mining activity is `SafeBlocked`; paused/ready shapes cannot expose stale failure text. | `crates/bitaxe-api/src/mining.rs` |
| Firmware readiness input | The owner reads current operator, network, protocol, safety, lease, and actuation facts into typed `ProductionReadiness`. | `firmware/bitaxe/src/production_mining_session.rs` |
| Reference behavior | Upstream pauses mining and marks pools unavailable after configured pool retries are exhausted; system power management consumes fail-closed system state. | `reference/esp-miner/main/tasks/protocol_coordinator.c`, `reference/esp-miner/main/system.c` |

## Redaction Boundary

The exact production label set is limited to lowercase ASCII letters, digits,
and underscores, and every label is unique. Regressions enumerate the complete
current set and join each failure label through the same runtime-state operation
used by production into the API projection. Labels must never be extended with
raw pool URLs, ports, workers, owner addresses, passwords, tokens, device URLs,
IPs, MACs, Wi-Fi values, NVS secrets, transport payloads, share payloads, ASIC
frames, telemetry values, or runtime identities.

## Evidence Level

This ledger supports `SAFE-11` at `implemented` with `unit,workflow` evidence.
Promotion additionally requires the accepted SAFE-10 detector-gated live safety
proof, current source/reference binding, independent validation, and the
privacy/non-claim checks frozen in the active SAFE-11 plan.

No wire-identical upstream reason strings are claimed. Live fault injection,
individual active control effects, self-test, other boards/ASICs, arbitrary
profiles/pools, unbounded mining, OTA/recovery, and release readiness remain
outside this evidence.
