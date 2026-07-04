# Phase 21 Bounded Soak Evidence

bounded_soak_status: approved_controlled_no_share_soak
duration_seconds: 300
live_smoke_prerequisite: controlled-no-share
controlled_package_boot_status: trusted
controlled_runtime_harness_status: observed
controlled_run_provenance: actual-controlled-run-or-harness
pool_input_bridge_status: applied
pool_lifecycle_status: active
subscribe_status: sent
authorize_status: sent
notify_job_status: accepted work_enqueued=true
bm1366_work_dispatch_status: typed_action_ready
result_receive_status: bounded_no_result
share_outcome: bounded no-share
share_submission_status: bounded_no_share
runtime_snapshot_status: updated
api_websocket_telemetry_update_status: ready
accepted_shares_observed: none
rejected_shares_observed: none
thermal_power_status: bounded_safe_fixture_values
watchdog_responsiveness_status: passed
watchdog_yield_checkpoint_count: 14
api_snapshot_count: 1
api_snapshot_status: redacted_sample_captured
websocket_frame_status: passed frames=5
safe_stop_status: complete mining=disabled hardware_control=disabled work_submission=disabled
redaction_status: passed
network_scan: disabled
hardware_command_status: run through allow-manifest-validated wrapper
allow_manifest: docs/parity/evidence/phase-21-live-mining-and-soak-evidence/bounded-soak/allow-bounded-soak.json
detector_log: docs/parity/evidence/phase-21-live-mining-and-soak-evidence/bounded-soak/detect-ultra205.log
soak_log: docs/parity/evidence/phase-21-live-mining-and-soak-evidence/bounded-soak/bounded-soak.log
conclusion: approved bounded controlled no-share soak recorded

## Scope

The bounded soak ran for `duration_seconds: 300` after the live-smoke tier had produced controlled no-share evidence with the controlled package, pool input bridge, runtime markers, telemetry captures, watchdog checkpoints, and safe-stop markers. The soak command was validated by the Phase 21 mining allow manifest before execution.

The run remained an approved controlled no-share soak. It did not observe accepted or rejected shares and does not claim unbounded production mining, active voltage/fan/fault control, frequency transition, thermal sensor parity, power sensor parity, OTA/recovery behavior, or non-205 board behavior.

## Evidence Artifacts

| Artifact | Path | Result |
|----------|------|--------|
| Allow manifest | `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/bounded-soak/allow-bounded-soak.json` | passed `mining-allow` validation |
| Soak log | `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/bounded-soak/bounded-soak.log` | approved controlled no-share soak |
| Watchdog observations | `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/bounded-soak/watchdog-observations.md` | passed bounded observation review |
| API snapshot | `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/bounded-soak/api-system-info-snapshots.redacted.jsonl` | redacted sample captured |
| WebSocket capture | `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/bounded-soak/websocket/api-ws-live.txt` | `/api/ws/live` frames captured |
