# Phase 28 log Evidence

slot: log
slot_status: blocked
board: 205
source_commit: d497b33c9dcb17c1040d5c67e91eccfa0ec49b8a
reference_commit: c1915b0a63bfabebdb95a515cedfee05146c1d50
package_identity: bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json
evidence_mode: phase28-hardware-evidence-and-checklist-promotion
source_phase27_root: docs/parity/evidence/phase-27-live-hardware-asic-and-stratum-bridge/
consolidation_status: blocked
source_reference: ../phase-27-live-hardware-asic-and-stratum-bridge/command.md
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

Phase 28 does not commit raw monitor logs. Log slot remains blocked with category labels only; Phase 27 command slot records repo-owned capture workflow without raw log duplication.

## conclusion

Consolidation preserves redacted log non-claims. No raw serial log content is committed in Phase 28.

## exact_non_claims

- Raw monitor logs, device targets, IPs, MACs, and Wi-Fi values are not committed.
- accepted/rejected shares remain non-claims unless detector-gated live proof exists with ASIC bridge correlation markers.
- Full active voltage, fan, thermal, fault, and self-test safety closure remains a non-claim.
