# Phase 27 detector Evidence

slot: detector
slot_status: blocked
board: 205
source_commit: 5e8461ccd64d2aab163c956d1b089511bda499c8
reference_commit: c1915b0a63bfabebdb95a515cedfee05146c1d50
package_identity: /tmp/phase27-manifest.json
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
pool_config: not-supplied
wifi_config: not-supplied
port_source: not-supplied
duration_seconds: not-requested
redact_evidence: not-requested
raw_pool_values_committed: no
network_scan: disabled

## observed_behavior

Detector status is blocked; hardware promotion requires just detect-ultra205 before any flash-monitor work.

## conclusion

Phase 27 records an exact blocked safe-prerequisite non-claim: blocked_mode_static_workflow.

## exact_non_claims

- accepted/rejected shares remain non-claims unless a detector-gated live pool response is tied to live ASIC-derived submit intent with ASIC bridge correlation markers.
- Phase 28 checklist promotion remains a non-claim except where this evidence root explicitly supports category labels only.
- Full active voltage, fan, thermal, fault, and self-test safety closure remains a non-claim.
