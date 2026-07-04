# Phase 21 Live Mining Smoke Evidence

live_mining_smoke_status: controlled-no-share
controlled_package_boot_status: trusted
controlled_runtime_harness_status: observed
controlled_run_provenance: actual-controlled-run-or-harness
pool_input_bridge_status: applied
pool_settings_consumed_by_runtime: true
pool_lifecycle_status: active
subscribe_status: sent
authorize_status: sent
notify_job_status: accepted work_enqueued=true
bm1366_work_dispatch_status: typed_action_ready
result_receive_status: bounded_no_result
share_submission_status: bounded_no_share
runtime_snapshot_status: updated
api_websocket_telemetry_update_status: ready
share_outcome: bounded no-share
accepted_shares_observed: none
rejected_shares_observed: none
hashrate_inputs_status: bounded_zero_hashrate_inputs
watchdog_status: bounded observations present
watchdog_yield_checkpoint_count: 14
api_telemetry_status: http_status_200_curl_0
websocket_frame_status: passed frames=4
safe_stop_status: complete mining=disabled hardware_control=disabled work_submission=disabled
redaction_status: passed
network_scan: disabled
hardware_command_status: run through allow-manifest-validated wrapper
detector_status: passed
board: 205
port: /dev/cu.usbmodem1101
board_info_status: passed
allow_manifest: docs/parity/evidence/phase-21-live-mining-and-soak-evidence/live-mining-smoke/allow-live-mining-smoke.json
smoke_log: docs/parity/evidence/phase-21-live-mining-and-soak-evidence/live-mining-smoke/mining-smoke.log
controlled_package_boot_log: docs/parity/evidence/phase-21-live-mining-and-soak-evidence/live-mining-smoke/controlled-package-boot/flash-monitor.log
pool_input_bridge_log: docs/parity/evidence/phase-21-live-mining-and-soak-evidence/live-mining-smoke/pool-input-bridge/logs.redacted.txt
conclusion: controlled no-share evidence recorded

## Scope

The live-smoke tier ran only after detector, package, readiness, chip-detect, work-result, explicit-target, pool-input, mining-allow, and redaction gates passed. The committed manifest records the command with `[redacted-url]`; the actual target stayed local runtime input derived from the same controlled-package boot session.

The controlled runtime consumed pool settings through the firmware settings path, emitted Stratum subscribe, authorize, notify/job, typed BM1366 work dispatch, bounded result/no-share, runtime snapshot, API/WebSocket telemetry, watchdog-yield, and safe-stop markers. No accepted or rejected share was observed, so this artifact supports only a bounded controlled no-share conclusion.

## Evidence Artifacts

| Artifact | Path | Result |
|----------|------|--------|
| Allow manifest | `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/live-mining-smoke/allow-live-mining-smoke.json` | passed `mining-allow` validation |
| Wrapper log | `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/live-mining-smoke/mining-smoke.log` | controlled no-share conclusion |
| Pool input bridge | `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/live-mining-smoke/pool-input-bridge.md` | applied and consumed by runtime |
| API snapshot | `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/live-mining-smoke/api-system-info.redacted.json` | HTTP 200, redacted mining telemetry |
| WebSocket capture | `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/live-mining-smoke/websocket-live.redacted.log` | `/api/ws/live` frames captured |

## Non-Claims

This is not accepted-share proof, rejected-share proof, active voltage/fan/fault-control proof, frequency-transition proof, non-205 board proof, OTA/recovery proof, or unbounded production-mining soak proof. Raw pool values, device URL, Wi-Fi values, private endpoints, worker strings, passwords, tokens, IP addresses, and MAC addresses are not committed.
