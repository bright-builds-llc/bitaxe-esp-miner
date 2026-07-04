# Phase 23 Detector Slot

slot: detector
slot_status: blocked
board: 205
source_commit: current-phase-source
reference_commit: current-reference
package_identity: not-required-for-detector-slot
detector_evidence: `just detect-ultra205`
command_category: detector-gate
redaction_status: passed
safe_stop_status: not-run-static-workflow-slot
raw_artifacts_committed: no
pool_config: not-read
wifi_config: not-read
raw_pool_values_committed: no
network_scan: disabled

## Observed Behavior

Hardware-capable evidence starts with `just detect-ultra205`. The workflow must block when detection is absent, ambiguous, not board `205`, or board-info fails.

## Conclusion

Detector evidence is a required gate before any hardware slot can be marked passed.

## exact_non_claims

- This slot does not verify trusted BM1366 production work.
- This slot does not verify live Stratum socket success.
- This slot does not verify accepted/rejected share outcomes.
- This slot does not verify Phase 26 telemetry promotion.
