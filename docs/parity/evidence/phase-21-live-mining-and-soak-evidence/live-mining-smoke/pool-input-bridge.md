# Phase 21 Pool Input Bridge

pool_input_bridge_status: applied
pool_settings_consumed_by_runtime: true
pool_config: local-owner-supplied
network_scan: disabled
settings_patch_status: http_status_200_curl_0
runtime_refresh_status: observed
controlled_runtime_harness_status: observed
raw_pool_values_committed: no
redaction_status: passed

## Scope

The bridge applied local owner-supplied pool settings through the firmware-owned `PATCH /api/system` settings route using an explicit same-session device target. The committed artifacts keep only redacted categories and status labels; no pool endpoint, worker, password, private target, Wi-Fi credential, API token, or NVS secret value is committed.

## Runtime Markers

The redacted `/api/system/logs` capture records `phase21_pool_settings_consumed=true source=settings_patch`, `phase21_controlled_runtime_status=ready`, Stratum subscribe/authorize/notify markers, typed BM1366 work dispatch, bounded no-result/no-share markers, runtime snapshot update, API/WebSocket telemetry readiness, watchdog yield checkpoints, and final safe-stop.

## Conclusion

Pool settings reached the controlled runtime through the firmware settings path and the runtime consumed them without exposing raw pool values.
