# Phase 23 Board-Info Slot

slot: board-info
slot_status: blocked
board: 205
source_commit: current-phase-source
reference_commit: current-reference
package_identity: not-required-for-board-info-slot
detector_evidence: `just detect-ultra205`
command_category: espflash-board-info
redaction_status: passed
safe_stop_status: not-run-static-workflow-slot
raw_artifacts_committed: no
pool_config: not-read
wifi_config: not-read
raw_pool_values_committed: no
network_scan: disabled

## Observed Behavior

Board-info evidence is valid only when the detector finds exactly one likely ESP32-S3 Ultra 205 port and `espflash board-info --chip esp32s3 --port <detected-port> --non-interactive` succeeds.

## Conclusion

Board-info is blocked in static workflow mode and must be refreshed in the current hardware session before hardware evidence can pass.

## exact_non_claims

- This slot does not verify trusted BM1366 production work.
- This slot does not verify live Stratum socket success.
- This slot does not verify accepted/rejected share outcomes.
- This slot does not verify Phase 26 telemetry promotion.
