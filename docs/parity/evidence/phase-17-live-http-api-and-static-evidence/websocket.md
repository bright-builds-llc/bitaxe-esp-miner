# Phase 17 WebSocket Evidence

websocket_status: blocked
device_url_status: blocked - missing explicit origin-only DEVICE_URL
target_lock_status: absent - Plan 17-03 did not create target-lock.json because no explicit origin-only DEVICE_URL was available
network_scan: disabled
websocket_live_frame_status: blocked - missing explicit DEVICE_URL; artifact absent - not cited
websocket_raw_log_frame_status: blocked - missing explicit DEVICE_URL; artifact absent - not cited
redaction_status: passed - blocked WebSocket capture log reviewed; frame artifacts absent - not cited

## Identity Context

| Field | Value |
| --- | --- |
| source_commit | `d9e471c9699eb0140749127416640aa1bf077d26` |
| reference_commit | `c1915b0a63bfabebdb95a515cedfee05146c1d50` |
| board | `205` |
| selected_port | `/dev/cu.usbmodem1101` |
| package_manifest | `docs/parity/evidence/phase-17-live-http-api-and-static-evidence/package-release-gate/bitaxe-ultra205-package.json` |
| flash_evidence_json | `docs/parity/evidence/phase-17-live-http-api-and-static-evidence/serial-boot/flash-command-evidence.json` |
| serial_evidence | `docs/parity/evidence/phase-17-live-http-api-and-static-evidence/serial-boot.md` |
| http_evidence | `docs/parity/evidence/phase-17-live-http-api-and-static-evidence/http-static-api.md` |
| websocket_capture_log | `docs/parity/evidence/phase-17-live-http-api-and-static-evidence/websocket/websocket-capture.log` |

Plan 17-02 captured package and wrapper-owned flash identity for the current
Ultra 205. Plan 17-03 then recorded that no explicit origin-only `DEVICE_URL`
was available and intentionally left `target-lock.json` absent. Plan 17-04
therefore preserved the same no-scan boundary and did not infer a target from
serial logs, AP UI, router UI, mDNS, ARP, local network state, or operator
observations.

## Command Plan

The helper syntax and smoke-test checks were run before any WebSocket capture
decision:

```bash
node --check scripts/phase17-websocket-capture.mjs
bazel test //scripts:phase17_live_http_api_smoke_test
```

The bounded WebSocket commands that would run only with an explicit origin-only
`DEVICE_URL` are:

```bash
node scripts/phase17-websocket-capture.mjs --device-url "$DEVICE_URL" --path /api/ws/live --out docs/parity/evidence/phase-17-live-http-api-and-static-evidence/websocket/api-ws-live.txt --duration-ms 5000 --max-frames 3
node scripts/phase17-websocket-capture.mjs --device-url "$DEVICE_URL" --path /api/ws --out docs/parity/evidence/phase-17-live-http-api-and-static-evidence/websocket/api-ws.txt --duration-ms 5000 --max-frames 3
```

Both commands were not run because the only allowed target sources were absent:
no explicit `DEVICE_URL` was present in the execution environment, and no
explicit-input `target-lock.json` existed.

## Frame Evidence

| WebSocket path | Status | Artifact status | Claim boundary |
| --- | --- | --- | --- |
| `/api/ws/live` | blocked - missing DEVICE_URL | absent - not cited | Live connect or cadence frame proof is not claimed. |
| `/api/ws` | blocked - missing DEVICE_URL | absent - not cited | Raw-log frame proof is not claimed. An open timeout without raw log frame was not observed because the capture did not run. |

`/api/ws/live` may be promoted only when
`docs/parity/evidence/phase-17-live-http-api-and-static-evidence/websocket/api-ws-live.txt`
contains at least one redacted connect or cadence frame with
`websocket_frame_status=passed`.

`/api/ws` may be promoted only when
`docs/parity/evidence/phase-17-live-http-api-and-static-evidence/websocket/api-ws.txt`
contains at least one redacted raw-log frame with
`websocket_frame_status=passed`. If a future bounded capture opens but records
no frame before timeout, the required status is
`websocket_raw_log_frame_status: pending - open timeout without raw log frame`,
not verified raw-log streaming.

## Frame Snippet Policy

Committed WebSocket frame artifacts must contain redacted metadata or snippets
only. They must redact raw `DEVICE_URL`, private endpoints, IP addresses, MAC
addresses, Wi-Fi credentials, pool credentials, worker secrets, API tokens, NVS
secret values, and local terminal secrets before citation.

Absent frame artifacts are not cited by this ledger. The redaction review marks
both planned frame outputs as `absent - not cited`.

## Explicit Claims And Non-Claims

websocket_no_upgrade_claim: route-coexistence-only - not observed in Plan 17-04 because DEVICE_URL was missing
websocket_live_frame_claim: not claimed
websocket_raw_log_frame_claim: not claimed
websocket_promotion_status: blocked - no frame artifacts generated

ota_route_presence_claim: not claimed in Plan 17-04
valid_ota_upload_claim: not claimed
invalid_ota_rejection_claim: not claimed
rollback_claim: not claimed
boot_validation_claim: not claimed
otawww_update_claim: not claimed

## Conclusion

conclusion: blocked - Plan 17-04 preserved the explicit-target and no-scan
boundary. No live WebSocket frame, raw-log stream, HTTP/static/API, valid OTA,
rollback, boot-validation, or OTAWWW update claim is promoted from these
blocked artifacts.
