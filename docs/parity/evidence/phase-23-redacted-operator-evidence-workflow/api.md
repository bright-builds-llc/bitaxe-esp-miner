# Phase 23 API Slot

slot: api
slot_status: blocked
board: 205
source_commit: current-phase-source
reference_commit: current-reference
package_identity: blocked-mode-workflow
detector_evidence: `just detect-ultra205` required before API evidence
command_category: redacted-api-capture
redaction_status: passed
safe_stop_status: not-run-static-workflow-slot
raw_artifacts_committed: no
pool_config: local-owner-supplied
wifi_config: local-owner-supplied
raw_pool_values_committed: no
network_scan: disabled

## Observed Behavior

API capture is blocked unless the workflow has a current valid target. stale DEVICE_URL, mDNS, ARP, router state, network scan, and unrelated evidence are invalid target sources.

## Conclusion

The API slot proves only that the workflow has a redaction-safe place to record API evidence or target blockers.

## exact_non_claims

- This slot does not verify trusted BM1366 production work.
- This slot does not verify live Stratum socket success.
- This slot does not verify accepted/rejected share outcomes.
- This slot does not verify Phase 26 telemetry promotion.
