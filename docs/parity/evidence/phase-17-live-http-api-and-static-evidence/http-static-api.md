# Phase 17 HTTP Static API Evidence

http_static_api_status: blocked
device_url_status: blocked - missing explicit origin-only DEVICE_URL
target_lock_status: absent - not cited
identity_status: available - package and flash identity captured by Plan 17-02, but live route probes were not run because DEVICE_URL was missing
network_scan: disabled
redaction_status: passed - blocked HTTP artifacts reviewed; missing route artifacts are absent - not cited

## Identity Context

| Field | Value |
| --- | --- |
| source_commit | `d9e471c9699eb0140749127416640aa1bf077d26` |
| reference_commit | `c1915b0a63bfabebdb95a515cedfee05146c1d50` |
| board | `205` |
| selected_port | `/dev/cu.usbmodem1101` |
| package_manifest | `docs/parity/evidence/phase-17-live-http-api-and-static-evidence/package-release-gate/bitaxe-ultra205-package.json` |
| flash_evidence_json | `docs/parity/evidence/phase-17-live-http-api-and-static-evidence/serial-boot/flash-command-evidence.json` |
| helper_log | `docs/parity/evidence/phase-17-live-http-api-and-static-evidence/http-static-api/http-static-api.log` |

The package and flash evidence are present and still useful as identity context,
but Plan 17-03 did not receive an explicit origin-only `DEVICE_URL`. The helper
therefore stopped before live HTTP requests and did not infer a target from
serial logs, AP UI, router UI, mDNS, ARP, local network state, or operator
observations.

## Route Evidence

| D-08 route | Status | Artifact status | Claim boundary |
| --- | --- | --- | --- |
| GET / | blocked - missing DEVICE_URL | absent - not cited | Root static entry markers not claimed. |
| GET /assets/app.css.gz | blocked - missing DEVICE_URL | absent - not cited | CSS/gzip/cache header evidence not claimed. |
| GET /phase17-missing-static | blocked - missing DEVICE_URL | absent - not cited | Missing-static redirect evidence not claimed. |
| GET /recovery | blocked - missing DEVICE_URL | absent - not cited | Recovery page evidence not claimed. |
| GET /api/system/info | blocked - missing DEVICE_URL | absent - not cited | Live current-device API body not claimed. |
| GET /api/phase17-unknown | blocked - missing DEVICE_URL | absent - not cited | Unknown API JSON 404 evidence not claimed. |
| GET /api/ws | blocked - missing DEVICE_URL | absent - not cited | No-upgrade route-coexistence response not observed. |
| GET /api/ws/live | blocked - missing DEVICE_URL | absent - not cited | No-upgrade route-coexistence response not observed. |
| POST /api/system/OTA | blocked - missing DEVICE_URL | absent - not cited | Empty OTA POST route-presence evidence not claimed. |
| POST /api/system/OTAWWW | blocked - missing DEVICE_URL | absent - not cited | OTAWWW fail-closed live response not claimed. |

## Explicit Claims And Non-Claims

websocket_no_upgrade_claim: route-coexistence-only - not observed in Plan 17-03 because DEVICE_URL was missing
websocket_frame_claim: not claimed in Plan 17-03

ota_route_presence_claim: route-presence-only - not observed in Plan 17-03 because DEVICE_URL was missing
ota_non_claims: valid OTA upload, invalid image rejection, reboot, rollback, selected partition, boot validation not claimed

otawww_rel03_status: blocked - live fail-closed response not observed without DEVICE_URL
otawww_update_claim: not claimed

## Helper Evidence

The helper transcript records:

- `phase17_live_http_api_smoke`
- `network_scan: disabled`
- `DEVICE_URL status: blocked - missing DEVICE_URL`
- `target_status: blocked`
- `http_static_api_status: blocked`
- `conclusion: blocked - live HTTP/static/API evidence requires an explicit origin-only DEVICE_URL`

No `target-lock.json` exists because no explicit target was accepted. No
per-route header, body, or curl-error artifacts were generated; each is
therefore `absent - not cited`.

## Conclusion

conclusion: blocked - Plan 17-03 preserved the explicit-target and no-scan
boundary. No live HTTP/static/API, WebSocket frame, valid OTA, rollback,
boot-validation, or OTAWWW update claim is promoted from these blocked artifacts.
