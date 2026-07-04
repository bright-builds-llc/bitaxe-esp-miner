# Phase 21 Live API And WebSocket Telemetry

telemetry_correlation_status: passed
network_scan: disabled
api_system_info_status: http_status_200_curl_0
websocket_frame_status: passed frames=4
statistics_correlation_status: controlled-zero-hashrate-snapshot
pool_lifecycle_correlation: active
share_counter_correlation: zero accepted and zero rejected in controlled no-share window
hashrate_input_correlation: bounded_zero_hashrate_inputs
watchdog_correlation: watchdog_yield_checkpoints_observed
same_run_source: live-mining-smoke
redaction_status: passed
api_snapshot: docs/parity/evidence/phase-21-live-mining-and-soak-evidence/live-api-websocket-telemetry/api-system-info.redacted.json
websocket_capture: docs/parity/evidence/phase-21-live-mining-and-soak-evidence/live-api-websocket-telemetry/websocket/api-ws-live.txt

## Scope

The telemetry pack cites the explicit-target API and WebSocket captures from the same Phase 21 live-smoke controlled no-share run. The API body and WebSocket frames are committed only after redaction of target, network, Wi-Fi, pool, worker, password, token, IP, MAC, and hostname fields.

## Conclusion

The live `/api/system/info` and `/api/ws/live` surfaces reflected the controlled runtime state during the no-share window: board `205`, BM1366 identity, active pool lifecycle, zero share counters, zero hashrate, redacted network fields, and WebSocket live update frames.
