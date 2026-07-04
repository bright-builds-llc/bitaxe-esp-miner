# Phase 21 Readiness Audit

firmware_live_mining_status: blocked_by_default
observed_marker: mining_loop_status=blocked reason=hardware_evidence_ack_missing
controlled_enablement_required: true
enablement_plan: 21-02-PLAN.md
network_scan: disabled

This audit records the pre-enable state before Phase 21 live mining or soak evidence may run. The firmware currently publishes a blocked mining-loop marker during normal startup, and the live wrapper must require a separate enablement pack before any live mining smoke or bounded soak command proceeds.

## Static Audit Command

```bash
rg -n "mining_loop_status|work_submission|hardware_evidence_ack|BITAXE_ASIC_DIAGNOSTIC" firmware/bitaxe crates/bitaxe-stratum scripts tools
```

## Observations

| Surface | Evidence | Conclusion |
|---------|----------|------------|
| Firmware startup | `firmware/bitaxe/src/main.rs` calls `asic_adapter::publish_mining_loop_blocked_status("hardware_evidence_ack_missing")`. | Startup remains fail-closed for live mining. |
| Firmware status logging | `firmware/bitaxe/src/asic_adapter/status.rs` logs `mining_loop_status=blocked reason={reason} work_submission=disabled`. | User-visible marker matches the blocked-by-default claim. |
| Diagnostic packaging | `scripts/phase15-bm1366-diagnostic-package.sh` gates chip-detect and work-result builds behind `BITAXE_ASIC_DIAGNOSTIC` and matching hardware evidence acknowledgement values. | Existing BM1366 diagnostic evidence remains package-scoped, not live mining enablement. |
| Stratum runtime core | `crates/bitaxe-stratum/src/v1/mining_loop.rs` models `hardware_evidence_ack` and keeps work submission blocked when evidence is missing. | Runtime has the pure gate, but firmware still needs the controlled runtime/harness enablement pack. |
| Phase 21 wrapper | `scripts/phase21-live-mining-evidence.sh` requires readiness, enablement, chip-detect, and work-result summaries before live commands. | Missing prerequisites produce blocked or pending evidence, not live mining claims. |

## Current Claim Boundary

The repository can record a software readiness audit and gated evidence contract in Plan 21-01. It cannot claim live production mining, accepted or rejected share behavior, bounded soak, live telemetry freshness, or watchdog behavior until later plans provide redaction-reviewed hardware and runtime evidence.

## Required Next Artifact

Plan `21-02-PLAN.md` must produce a live-mining enablement pack with both:

- `controlled_live_mining_package_status: ready`
- `controlled_runtime_harness_status: ready`

Until both markers exist and redaction passes, Phase 21 live mining and bounded soak wrappers must stop before hardware/network mining actions.
