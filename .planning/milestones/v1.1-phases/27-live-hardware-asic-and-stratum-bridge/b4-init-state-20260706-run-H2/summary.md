# Phase 27 Evidence Summary

board: 205
source_commit: 4df61908f1d31d7cb3b72b77deb461351d0901e9
reference_commit: c1915b0a63bfabebdb95a515cedfee05146c1d50
package_identity: bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json
evidence_mode: phase27-live-hardware-asic-stratum-bridge
evidence_ack: ultra205-phase27-live-hardware-bridge-safe-stop
package_artifact_status: blocked
detector_status: passed
board_info_status: blocked
share_outcome: blocked_safe_prerequisite
asic_bridge_status: blocked
safe_stop_status: complete
redaction_status: passed
raw_artifacts_committed: no
raw_pool_values_committed: no
network_scan: disabled
pool_config: local-owner-supplied
wifi_config: local-owner-supplied
port_source: explicit
safety_bring_up_status: not-run

## Supported Claim

Phase 27 records an exact blocked safe-prerequisite non-claim: board_info_failure.

## exact_non_claims

- accepted/rejected shares remain non-claims unless detector-gated live proof exists with ASIC bridge correlation markers.
- Raw credentials, endpoints, target data, socket details, device targets, IPs, MACs, and raw BM1366 frames are not committed.
- Phase 28 checklist promotion remains deferred except where this evidence root explicitly supports category labels only.
