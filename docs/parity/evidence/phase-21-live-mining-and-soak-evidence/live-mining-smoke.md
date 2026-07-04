# Phase 21 Live Mining Smoke Evidence

live_mining_smoke_status: blocked
blocker: missing_live_prerequisites
missing_live_prerequisites: DEVICE_URL-or-pool-input-category
missing_live_prerequisite_categories: DEVICE_URL,BITAXE_POOL_URL,BITAXE_POOL_USER,BITAXE_POOL_PASSWORD
enablement_status: ready-not-run
controlled_runtime_harness_status: ready-not-run
controlled_package_boot_status: not-run
pool_input_bridge_status: not-run - missing_live_prerequisites
pool_lifecycle_status: not-run
subscribe_status: not-run
authorize_status: not-run
notify_job_status: not-run
bm1366_work_dispatch_status: not-run
result_receive_status: not-run
share_submission_status: not-run
runtime_snapshot_status: not-run
api_websocket_telemetry_update_status: not-run
share_outcome: not-run
accepted_shares_observed: none
rejected_shares_observed: none
hashrate_inputs_status: not-run
watchdog_status: not-run
api_telemetry_status: not-run
websocket_frame_status: not-run
safe_stop_status: not-run
redaction_status: passed
network_scan: disabled
hardware_command_status: not-run
detector_status: passed
board: 205
port: /dev/cu.usbmodem1101
board_info_status: passed
chip_detect_summary: docs/parity/evidence/phase-21-live-mining-and-soak-evidence/bm1366-init-work-result.md
work_result_summary: docs/parity/evidence/phase-21-live-mining-and-soak-evidence/bm1366-init-work-result.md
conclusion: blocked - missing_live_prerequisites

## Scope

Task 1 ran a fresh detector gate before evaluating live smoke prerequisites.
The detector found the Ultra 205 on the selected USB serial port and board-info
passed. The preflight, enablement, and diagnostic work-result prerequisite
ledgers are present, but the live target and disposable pool input categories
were not present in the executor environment.

Because `DEVICE_URL`, `BITAXE_POOL_URL`, `BITAXE_POOL_USER`, and
`BITAXE_POOL_PASSWORD` were absent, this plan did not create a
`live-pool-smoke` allow manifest, did not flash the controlled live-mining
package, did not PATCH pool settings, did not run the mining wrapper, and did
not infer a device target from serial logs or network state.

## Evidence Artifacts

| Artifact | Path | Result |
|----------|------|--------|
| Fresh detector log | `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/live-mining-smoke/detect-ultra205.log` | redacted detector pass |
| Pool input bridge | `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/live-mining-smoke/pool-input-bridge.md` | blocked before pool patch |
| Preflight ledger | `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/preflight.md` | passed |
| Enablement ledger | `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/live-mining-enablement.md` | ready |
| BM1366 diagnostic prerequisite ledger | `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/bm1366-init-work-result.md` | complete diagnostic prerequisite only |

## Non-Claims

- live pool connectivity: not run
- production mining: not run
- controlled no share: not claimed
- accepted shares: not observed
- rejected shares: not observed
- successful BM1366 initialization: not claimed
- production work dispatch: not claimed
- live API/WebSocket telemetry freshness: not claimed
- bounded soak stability: not claimed
- watchdog responsiveness under mining load: not claimed
- frequency transition, active control, OTA, erase, or release recovery behavior: not claimed

## Conclusion

The live mining smoke tier is precisely blocked by missing live prerequisites.
This is not controlled no share evidence and must not be cited as live pool,
share, telemetry freshness, watchdog, or soak proof.
