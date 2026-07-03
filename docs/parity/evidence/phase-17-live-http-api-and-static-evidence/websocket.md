# Phase 17 WebSocket Evidence

websocket_status: passed
device_url_status: accepted - trusted USB flash-monitor source, committed target redacted
target_lock_status: present
network_scan: disabled
websocket_live_frame_status: passed
websocket_raw_log_frame_status: passed
redaction_status: passed - WebSocket frame artifacts reviewed

## Identity Context

| Field | Value |
| --- | --- |
| source_commit | `9a2bf5850ea042731f6a7947cc7eb04dc4589e90` |
| reference_commit | `c1915b0a63bfabebdb95a515cedfee05146c1d50` |
| board | `205` |
| selected_port | `/dev/cu.usbmodem1101` |
| package_manifest | `docs/parity/evidence/phase-17-live-http-api-and-static-evidence/package-release-gate/bitaxe-ultra205-package.json` |
| target_lock | `docs/parity/evidence/phase-17-live-http-api-and-static-evidence/target-lock.json` |
| http_evidence | `docs/parity/evidence/phase-17-live-http-api-and-static-evidence/http-static-api.md` |
| websocket_capture_log | `docs/parity/evidence/phase-17-live-http-api-and-static-evidence/websocket/websocket-capture.log` |

The WebSocket helper used the same local developer-raw USB flash-monitor source
as the HTTP helper and wrote redacted frame/open artifacts only. Both captures
were bounded with `--duration-ms 5000 --max-frames 3`.

## Command Evidence

The helper syntax check passed:

```bash
node --check scripts/phase17-websocket-capture.mjs
```

The bounded capture commands were:

```bash
node scripts/phase17-websocket-capture.mjs --device-url-from-flash-evidence target/phase17-gap-current-dev-raw/flash-command-evidence.json --path /api/ws/live --out docs/parity/evidence/phase-17-live-http-api-and-static-evidence/websocket/api-ws-live.txt --duration-ms 5000 --max-frames 3
node scripts/phase17-websocket-capture.mjs --device-url-from-flash-evidence target/phase17-gap-current-dev-raw/flash-command-evidence.json --path /api/ws --out docs/parity/evidence/phase-17-live-http-api-and-static-evidence/websocket/api-ws.txt --duration-ms 5000 --max-frames 3
```

## Frame Evidence

| WebSocket path | Status | Artifact | Claim boundary |
| --- | --- | --- | --- |
| `/api/ws/live` | passed | `websocket/api-ws-live.txt` | Redacted live update frames observed; frame artifact records `websocket_frame_status=passed frames=3`. |
| `/api/ws` | passed | `websocket/api-ws.txt` | Raw-log WebSocket opened and emitted `axeos_websocket_logs=connected`; frame artifact records `websocket_frame_status=passed frames=1`. |

`/api/ws/live` includes redacted update frames with Wi-Fi/network identifiers
redacted. `/api/ws` has a raw-log connection frame, so it is not merely an
open/timeout pending result.

## Frame Snippet Policy

Committed WebSocket frame artifacts contain redacted metadata or snippets only.
They redact raw `DEVICE_URL`, private endpoints, IP addresses, MAC addresses,
Wi-Fi credentials, pool credentials, worker secrets, API tokens, NVS secret
values, and local terminal secrets before citation.

## Explicit Claims And Non-Claims

websocket_no_upgrade_claim: route-coexistence-only - HTTP no-upgrade checks are
cited in `http-static-api.md` only as route coexistence.

websocket_live_frame_claim: passed - `/api/ws/live` frame proof is based on
`websocket/api-ws-live.txt`.

websocket_raw_log_frame_claim: passed - `/api/ws` frame proof is based on
`websocket/api-ws.txt`.

ota_route_presence_claim: not claimed by WebSocket evidence.
valid_ota_upload_claim: not claimed.
invalid_ota_rejection_claim: not claimed.
rollback_claim: not claimed.
boot_validation_claim: not claimed.
otawww_update_claim: not claimed.

## Conclusion

conclusion: passed - Phase 17 has bounded, redacted WebSocket frame/open
artifacts for `/api/ws/live` and `/api/ws` from the same just-flashed Ultra 205
identity used by the HTTP/static/API evidence.
