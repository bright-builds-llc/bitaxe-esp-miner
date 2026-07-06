# Phase 28 safe-stop Evidence

slot: safe-stop
slot_status: blocked
board: 205
source_commit: d497b33c9dcb17c1040d5c67e91eccfa0ec49b8a
reference_commit: c1915b0a63bfabebdb95a515cedfee05146c1d50
package_identity: bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json
evidence_mode: phase28-hardware-evidence-and-checklist-promotion
source_phase27_root: docs/parity/evidence/phase-27-live-hardware-asic-and-stratum-bridge/
consolidation_status: cross_linked
source_reference: ../phase-25-live-stratum-runtime-and-safe-stop/safe-stop.md
detector_evidence: not-run-consolidation-only
command_category: repo-owned-phase28-consolidation
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

Phase 28 cross-links Phase 25 safe-stop workflow categories and inherits Phase 27 `safe_stop_status: blocked`. Hardware-level live runtime stop proof remains below verified.

## conclusion

Safe-stop consolidation preserves blocked categories from Phase 27 and workflow categories from Phase 25 without claiming hardware-verified live stop proof.

## exact_non_claims

- Hardware-level live runtime stop remains below verified until detector-gated live evidence exists.
- accepted/rejected shares remain non-claims unless detector-gated live proof exists with ASIC bridge correlation markers.
- Full active voltage, fan, thermal, fault, and self-test safety closure remains a non-claim.
- OTAWWW/recovery destructive evidence, non-205 boards, Stratum v2, UI/BAP, and unbounded stress mining remain non-claims.
