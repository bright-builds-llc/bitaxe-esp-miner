# Phase 23 Log Slot

slot: log
slot_status: blocked
board: 205
source_commit: current-phase-source
reference_commit: current-reference
package_identity: blocked-mode-workflow
detector_evidence: `just detect-ultra205` required for hardware logs
command_category: redacted-log-capture
redaction_status: passed
safe_stop_status: not-run-static-workflow-slot
raw_artifacts_committed: no
pool_config: local-owner-supplied
wifi_config: local-owner-supplied
raw_pool_values_committed: no
network_scan: disabled

## Observed Behavior

Committed logs must contain redacted lifecycle/status markers only. Raw pool values, raw Stratum payloads, raw share payloads, socket errors, device URLs, IP addresses, MAC addresses, Wi-Fi values, NVS secrets, API tokens, and raw BM1366 frames are not accepted.

## Conclusion

The log slot is blocked until a repo-owned capture produces redacted committed output or an explicit blocked artifact.

## exact_non_claims

- This slot does not verify trusted BM1366 production work.
- This slot does not verify live Stratum socket success.
- This slot does not verify accepted/rejected share outcomes.
- This slot does not verify Phase 26 telemetry promotion.
