---
phase: 27-live-hardware-asic-and-stratum-bridge
verified: 2026-07-05T15:30:00Z
status: passed
score: 8/8 must-haves verified
generated_by: gsd-verifier
lifecycle_mode: yolo
phase_lifecycle_id: 27-2026-07-05T14-51-50
generated_at: 2026-07-05T15:30:00Z
lifecycle_validated: true
---

# Phase 27: Live Hardware ASIC And Stratum Bridge Verification Report

**Phase Goal:** Close the live-hardware integration gap with opt-in Phase 27 bridge dispatch, observation correlation, submit emission, and detector-gated evidence.
**Verified:** 2026-07-05T15:30:00Z
**Status:** passed

## Goal Achievement

| # | Truth | Status | Evidence |
| --- | --- | --- | --- |
| 1 | Phase 27 live bridge is distinct compile-time opt-in; fail-closed on mismatch | VERIFIED | `mining_evidence_mode.rs` tests; `PHASE27_LIVE_HARDWARE_BRIDGE_MODE/ACK` |
| 2 | Production UART survives boot gate for Phase 27 | VERIFIED | `asic_adapter/production.rs` `store_production_peripherals` |
| 3 | Live runtime dispatches `SendProductionWork` after notify | VERIFIED | `live_stratum_runtime.rs` ASIC bridge step with `GuardedMiningLoopInputs` |
| 4 | Observations stamped with dispatch generation | VERIFIED | `ProductionNonceObservation { observed_generation }` in bridge read path |
| 5 | Submit queued only after correlation | VERIFIED | `apply_bridge_observation` + unit tests |
| 6 | Evidence wrapper with blocked/hardware modes | VERIFIED | `scripts/phase27-live-hardware-bridge-evidence.sh`; Bazel test passed |
| 7 | Committed share-outcome category only | VERIFIED | `share_outcome: blocked_safe_prerequisite` in committed evidence |
| 8 | Mining-allow and checklist conservative promotion | VERIFIED | `mining_allow.rs` Phase 27 tier; checklist rows at implemented/workflow |

**Score:** 8/8 truths verified

## Blockers / Non-Claims

- No detector-gated hardware run in this session; `share_outcome` remains `blocked_safe_prerequisite`.
- STR-09 `verified` promotion requires accepted/rejected hardware evidence — not claimed.
- Phase 28 full checklist promotion explicitly deferred.

## Final Gate

All final gate commands in `27-VALIDATION.md` passed.
