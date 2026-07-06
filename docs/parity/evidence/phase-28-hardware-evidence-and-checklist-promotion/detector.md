# Phase 28 detector Evidence

slot: detector
slot_status: passed
board: 205
source_commit: d497b33c9dcb17c1040d5c67e91eccfa0ec49b8a
reference_commit: c1915b0a63bfabebdb95a515cedfee05146c1d50
package_identity: bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json
evidence_mode: phase28-hardware-evidence-and-checklist-promotion
source_phase27_root: docs/parity/evidence/phase-27-live-hardware-asic-and-stratum-bridge/
consolidation_status: cross_linked
source_slot: ../phase-27-live-hardware-asic-and-stratum-bridge/detector.md
detector_evidence: just detect-ultra205
detector_status: passed
command_category: repo-owned-phase27-live-hardware-bridge-evidence
redaction_status: passed
share_outcome: blocked_safe_prerequisite
asic_bridge_status: blocked
safe_stop_status: blocked
raw_artifacts_committed: no
raw_pool_values_committed: no
pool_config: local-owner-supplied
wifi_config: local-owner-supplied
network_scan: disabled

## observed_behavior

Phase 28 cross-links the Phase 27 detector slot without duplicating raw local artifacts. Phase 27 recorded `detector_status: passed` for exactly one Ultra 205 session before the bounded live capture attempt.

## conclusion

Consolidation preserves Phase 27 detector-gated workflow categories only. Accepted/rejected shares remain non-claims.

## exact_non_claims

- accepted/rejected shares remain non-claims unless detector-gated live proof exists with ASIC bridge correlation markers.
- Full active voltage, fan, thermal, fault, and self-test safety closure remains a non-claim.
- OTAWWW/recovery destructive evidence, non-205 boards, Stratum v2, UI/BAP, and unbounded stress mining remain non-claims.
