# Phase 27 share-outcome Evidence

slot: share-outcome
slot_status: blocked
board: 205
source_commit: 4df61908f1d31d7cb3b72b77deb461351d0901e9
reference_commit: c1915b0a63bfabebdb95a515cedfee05146c1d50
package_identity: bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json
evidence_mode: phase27-live-hardware-asic-stratum-bridge
evidence_ack: ultra205-phase27-live-hardware-bridge-safe-stop
detector_evidence: just detect-ultra205
board_info_evidence: espflash board-info
command_category: repo-owned-phase27-live-hardware-bridge-evidence
redaction_status: passed
share_outcome: blocked_safe_prerequisite
asic_bridge_status: blocked
safe_stop_status: complete
raw_artifacts_committed: no
pool_config: local-owner-supplied
wifi_config: local-owner-supplied
port_source: explicit
duration_seconds: 120
redact_evidence: true
raw_pool_values_committed: no
network_scan: disabled

## observed_behavior

No live pool response tied to live ASIC-derived submit intent was observed. asic_production_status category markers remain blocked. accepted/rejected shares remain non-claims.

## conclusion

Phase 27 records an exact blocked safe-prerequisite non-claim: board_info_failure.

## exact_non_claims

- accepted/rejected shares remain non-claims unless a detector-gated live pool response is tied to live ASIC-derived submit intent with ASIC bridge correlation markers.
- Phase 28 checklist promotion remains a non-claim except where this evidence root explicitly supports category labels only.
- Full active voltage, fan, thermal, fault, and self-test safety closure remains a non-claim.
