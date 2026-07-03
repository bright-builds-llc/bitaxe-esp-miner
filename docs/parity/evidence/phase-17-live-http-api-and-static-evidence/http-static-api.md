# Phase 17 HTTP Static API Evidence

http_static_api_status: passed
device_url_status: accepted - trusted USB flash-monitor source, committed target redacted
target_lock_status: present
identity_status: passed - package manifest and flash evidence match current source/reference commits
network_scan: disabled
redaction_status: passed - target lock and route artifacts reviewed

## Identity Context

| Field | Value |
| --- | --- |
| source_commit | `9a2bf5850ea042731f6a7947cc7eb04dc4589e90` |
| reference_commit | `c1915b0a63bfabebdb95a515cedfee05146c1d50` |
| board | `205` |
| selected_port | `/dev/cu.usbmodem1101` |
| package_manifest | `docs/parity/evidence/phase-17-live-http-api-and-static-evidence/package-release-gate/bitaxe-ultra205-package.json` |
| commit-ready flash evidence | `docs/parity/evidence/phase-17-live-http-api-and-static-evidence/serial-boot/flash-command-evidence.json` |
| local live target source | `target/phase17-gap-current-dev-raw/flash-command-evidence.json` |
| target_lock | `docs/parity/evidence/phase-17-live-http-api-and-static-evidence/target-lock.json` |
| helper_log | `docs/parity/evidence/phase-17-live-http-api-and-static-evidence/http-static-api/http-static-api.log` |

The helper used `--use-flash-log-device-url` against the local developer-raw
USB evidence under `target/phase17-gap-current-dev-raw/` to read the raw
`device_url` in memory. The committed artifacts contain only
`device_url_redacted: http://[redacted]`, `device_url_source:
usb_flash_monitor_log`, `network_scan: disabled`, and the matching
source/reference identity.

## Route Evidence

| D-08 route | Status | Artifacts | Claim boundary |
| --- | --- | --- | --- |
| GET / | passed | `root.headers.txt`, `root.body.txt`, `root.curl-error.txt` | Root static entry markers observed. |
| GET /assets/app.css.gz | passed | `app-css-gz.headers.txt`, `app-css-gz.body.txt`, `app-css-gz.curl-error.txt` | CSS gzip/cache headers observed. |
| GET /phase17-missing-static | passed | `missing-static.headers.txt`, `missing-static.body.txt`, `missing-static.curl-error.txt` | Missing-static redirect to `/` observed. |
| GET /recovery | passed | `recovery.headers.txt`, `recovery.body.txt`, `recovery.curl-error.txt` | Recovery page route loaded. |
| GET /api/system/info | passed | `system-info.headers.txt`, `system-info.body.txt`, `system-info.curl-error.txt` | Redacted current-device API body contains Ultra 205/BM1366 identity. |
| GET /api/phase17-unknown | passed | `unknown-api.headers.txt`, `unknown-api.body.txt`, `unknown-api.curl-error.txt` | Unknown API JSON 404 observed. |
| GET /api/ws | passed | `api-ws.headers.txt`, `api-ws.body.txt`, `api-ws.curl-error.txt` | HTTP no-upgrade route-coexistence only; not frame proof. |
| GET /api/ws/live | passed | `api-ws-live.headers.txt`, `api-ws-live.body.txt`, `api-ws-live.curl-error.txt` | HTTP no-upgrade route-coexistence only; not frame proof. |
| POST /api/system/OTA | passed | `firmware-ota.headers.txt`, `firmware-ota.body.txt`, `firmware-ota.curl-error.txt` | Empty POST route-presence/validation-path only. |
| POST /api/system/OTAWWW | passed | `otawww.headers.txt`, `otawww.body.txt`, `otawww.curl-error.txt` | Live fail-closed `Wrong API input` gap response observed; whole-`www` update still deferred. |

## Helper Evidence

The helper transcript records:

- `phase17_live_http_api_smoke`
- `network_scan: disabled`
- `DEVICE_URL status: provided`
- `DEVICE_URL source: usb_flash_monitor_log`
- `identity_status: passed`
- `target_status: passed`
- `http_static_api_status: passed`
- `websocket_no_upgrade_claim: route-coexistence-only`
- `ota_route_presence_claim: route-presence-only`
- `otawww_rel03_status: deferred`
- `conclusion: passed - all Phase 17 HTTP/static/API route probes matched expected live evidence markers`

## Explicit Claims And Non-Claims

websocket_no_upgrade_claim: route-coexistence-only - HTTP no-upgrade responses
were observed for `/api/ws` and `/api/ws/live`, but frame proof comes only from
`docs/parity/evidence/phase-17-live-http-api-and-static-evidence/websocket/`.

ota_route_presence_claim: route-presence-only - `POST /api/system/OTA` accepted
the empty validation-path probe. Phase 17 does not claim valid OTA upload,
invalid OTA rejection, reboot, rollback, selected partition, or boot validation.

otawww_rel03_status: deferred - `POST /api/system/OTAWWW` returned the expected
fail-closed gap response. Phase 17 does not claim whole-`www` OTAWWW update
behavior.

## Conclusion

conclusion: passed - Phase 17 now has live HTTP/static/API route evidence from
a just-flashed Ultra 205 with a sanitized target lock and no network scanning.
Unsupported valid OTA, rollback, boot-validation, OTAWWW update, mining,
safety telemetry, and soak claims remain outside this evidence set.
