# Phase 23 WebSocket Slot

slot: websocket
slot_status: blocked
board: 205
source_commit: current-phase-source
reference_commit: current-reference
package_identity: blocked-mode-workflow
detector_evidence: `just detect-ultra205` required before WebSocket evidence
command_category: redacted-websocket-capture
redaction_status: passed
safe_stop_status: not-run-static-workflow-slot
raw_artifacts_committed: no
pool_config: local-owner-supplied
wifi_config: local-owner-supplied
raw_pool_values_committed: no
network_scan: disabled

## Observed Behavior

WebSocket capture is blocked unless the workflow has a current valid target. stale DEVICE_URL, mDNS, ARP, router state, network scan, and unrelated evidence are invalid target sources.

## Conclusion

The WebSocket slot proves only that the workflow has a redaction-safe place to record WebSocket evidence or target blockers.

## exact_non_claims

- This slot does not verify trusted BM1366 production work.
- This slot does not verify live Stratum socket success.
- This slot does not verify accepted/rejected share outcomes.
- This slot does not verify Phase 26 telemetry promotion.
