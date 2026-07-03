# Phase 17 Live HTTP API And Static Evidence Summary

## Scope

Phase 17 records current Ultra 205 package, release-gate, detector,
flash-monitor, HTTP/static/API, WebSocket, and redaction evidence under
`docs/parity/evidence/phase-17-live-http-api-and-static-evidence/`.

The source commit for the package and flashed firmware evidence is
`9a2bf5850ea042731f6a7947cc7eb04dc4589e90`; the reference commit is
`c1915b0a63bfabebdb95a515cedfee05146c1d50`. Package, serial identity,
sanitized target lock, live HTTP/static/API probes, bounded WebSocket frame
captures, and final redaction review passed. Network scanning remained
disabled.

## Exact Commands

Package and release-gate commands:

```bash
just package
bazel run //tools/parity:report -- release-gate --manifest docs/parity/evidence/phase-17-live-http-api-and-static-evidence/package-release-gate/bitaxe-ultra205-package.json
```

Detector and commit-ready flash-monitor commands:

```bash
just detect-ultra205
just flash-monitor board=205 port=/dev/cu.usbmodem1101 manifest=bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json wifi-credentials=wifi-credentials.json evidence-dir=docs/parity/evidence/phase-17-live-http-api-and-static-evidence/serial-boot capture-timeout-seconds=45 redact-evidence=true
```

The local developer-raw USB flash-monitor run used the same board, port,
manifest, and Wi-Fi credential file only to obtain the raw `device_url` in
memory. That local evidence is not commit-ready and remains under `target/`.

HTTP helper command:

```bash
scripts/phase17-live-http-api-smoke.sh --use-flash-log-device-url --manifest docs/parity/evidence/phase-17-live-http-api-and-static-evidence/package-release-gate/bitaxe-ultra205-package.json --flash-evidence-json target/phase17-gap-current-dev-raw/flash-command-evidence.json --out-dir docs/parity/evidence/phase-17-live-http-api-and-static-evidence/http-static-api --target-lock-out docs/parity/evidence/phase-17-live-http-api-and-static-evidence/target-lock.json
```

WebSocket capture commands:

```bash
node scripts/phase17-websocket-capture.mjs --device-url-from-flash-evidence target/phase17-gap-current-dev-raw/flash-command-evidence.json --path /api/ws/live --out docs/parity/evidence/phase-17-live-http-api-and-static-evidence/websocket/api-ws-live.txt --duration-ms 5000 --max-frames 3
node scripts/phase17-websocket-capture.mjs --device-url-from-flash-evidence target/phase17-gap-current-dev-raw/flash-command-evidence.json --path /api/ws --out docs/parity/evidence/phase-17-live-http-api-and-static-evidence/websocket/api-ws.txt --duration-ms 5000 --max-frames 3
```

## Artifact Status Matrix

| Artifact | Status | Citation boundary |
| --- | --- | --- |
| `package-release-gate.md` | present | Package and release-gate identity only. |
| `package-release-gate/bitaxe-ultra205-package.json` copied package manifest | present | Source/reference commits and artifact checksums only. |
| `package-release-gate/package-command.log` | present | `just package` command result only. |
| `package-release-gate/release-gate.log` | present | Release-gate command result only. |
| `serial-boot/detect-ultra205.log` detector log | present | USB board identity and board-info gate, MAC redacted. |
| `serial-boot/flash-command-evidence.json` flash JSON | present | Commit-ready wrapper-owned flash-monitor command identity only. |
| `serial-boot/flash-monitor.log` flash-monitor log | present | Redacted serial boot, Wi-Fi join, SPIFFS mount, and route registration. |
| `target-lock.json` | present | Sanitized target provenance with `created_from_explicit_input: true`, `device_url_source: usb_flash_monitor_log`, and `network_scan: disabled`. |
| `http-static-api/http-static-api.log` HTTP route log | present | Live route transcript with redacted target only. |
| `http-static-api/*.headers.txt` HTTP headers | present | Selected headers for the D-08 route set. |
| `http-static-api/*.body.txt` HTTP bodies | present | Redacted body snippets for the D-08 route set. |
| `http-static-api/*.curl-error.txt` HTTP curl errors | present | Empty or redacted curl-error artifacts for the D-08 route set. |
| `websocket/websocket-capture.log` | present | Bounded capture command shapes and conclusions. |
| `websocket/api-ws-live.txt` | present | `/api/ws/live` redacted frame capture. |
| `websocket/api-ws.txt` | present | `/api/ws` raw-log connection frame capture. |
| `redaction-review.md` | present | Final redaction status for cited Phase 17 artifacts. |

## Package And Flash Identity

`package_status: passed` and `release_gate_status: passed` are recorded in
`package-release-gate.md`. The package manifest copy and wrapper-owned serial
evidence agree on source commit `9a2bf5850ea042731f6a7947cc7eb04dc4589e90`
and reference commit `c1915b0a63bfabebdb95a515cedfee05146c1d50`.

`detector_status: passed` and `flash_monitor_status: passed` are recorded in
`serial-boot.md`. The detector selected board `205` on
`/dev/cu.usbmodem1101`, and `flash-command-evidence.json` has
`trusted_output: true`, `redaction_mode: commit-redacted`, `commit_ready:
true`, and `capture_status: timed_out_after_trusted_output`. Serial evidence
shows factory boot, Wi-Fi join, SPIFFS mounted, and HTTP route shell
registration.

## HTTP Static API Evidence

http_static_api_status: passed
target_lock_status: present
network_scan: disabled

The helper accepted a trusted USB flash-monitor `device_url` source and wrote a
sanitized `target-lock.json`; no raw target is committed.

| Route | Phase 17 status |
| --- | --- |
| `GET /` | passed - root static entry markers observed |
| `GET /assets/app.css.gz` | passed - gzip/cache selected headers observed |
| `GET /phase17-missing-static` | passed - missing-static redirect observed |
| `GET /recovery` | passed - recovery page loaded |
| `GET /api/system/info` | passed - redacted current-device API body observed |
| `GET /api/phase17-unknown` | passed - unknown API JSON 404 observed |
| `GET /api/ws` | passed - no-upgrade route-coexistence only |
| `GET /api/ws/live` | passed - no-upgrade route-coexistence only |
| `POST /api/system/OTA` | passed - route-presence/validation-path only |
| `POST /api/system/OTAWWW` | passed - fail-closed `Wrong API input` gap response |

## WebSocket Evidence

websocket_status: passed
websocket_live_frame_status: passed
websocket_raw_log_frame_status: passed

`websocket/api-ws-live.txt` records `websocket_frame_status=passed frames=3`
for `/api/ws/live` redacted live update frames.

`websocket/api-ws.txt` records `websocket_frame_status=passed frames=1` for
`/api/ws` raw-log connection frame evidence. This is frame proof and not merely
an open/timeout pending result.

## Redaction Status

redaction_status: passed

`redaction-review.md` records every cited Phase 17 artifact as reviewed. The
committed evidence contains no Wi-Fi credential value, pool credential, worker
secret, API token, private endpoint, NVS secret value, raw target URL, raw IP
address, raw MAC address, or local terminal secret.

## Checklist Promotion Matrix

| Checklist row | Phase 17 action |
| --- | --- |
| `FS-001` | Promote to live HTTP evidence support for SPIFFS/static route behavior: `/`, `/assets/app.css.gz`, missing-static redirect, and `/recovery` were observed. |
| `API-004` | Promote live HTTP route evidence for the D-08 API/static route set while preserving OTA and WebSocket frame boundaries. |
| `API-005` | Promote `/api/ws` raw-log frame evidence based on `websocket/api-ws.txt`. |
| `API-006` | Promote `/api/ws/live` telemetry frame evidence based on `websocket/api-ws-live.txt`. |
| `API-007` | Promote live `/recovery` page load evidence. |
| `API-008` | Promote live static assets and API/WebSocket route coexistence evidence. |
| `OTA-001` | Keep below verified for valid OTA upload, invalid OTA rejection, reboot identity, selected partition, rollback, and boot validation. |
| `OTA-002` | Keep deferred. Phase 17 observed only fail-closed `Wrong API input`, not whole-`www` OTAWWW update behavior. |
| `REL-003` | Keep below verified for release image behavior that depends on valid OTA, rollback, recovery update, or OTAWWW regression evidence. |

## Explicit Non-Claims

Phase 17 does not claim valid OTA upload, invalid OTA rejection, reboot,
rollback, selected partition, boot validation, whole-`www` OTAWWW update
behavior, production mining, pool behavior, active safety telemetry, or long
soak behavior.

Package generation, release-gate success, detector output, serial boot,
Wi-Fi join, SPIFFS mount, live HTTP/static/API probes, and bounded WebSocket
frames are cited only for the facts they directly prove.

## Conclusions

Phase 17 closes with live package, flash, target, HTTP/static/API, and
WebSocket evidence from a just-flashed Ultra 205. The release docs, parity
checklist, and requirements traceability may cite these exact artifacts while
preserving valid OTA, rollback, boot-validation, whole-`www` OTAWWW, mining,
safety telemetry, and soak as below verified or out of scope.

## Residual Risks

- Firmware OTA success, invalid image rejection, selected partition, reboot,
  rollback, boot validation, failed-update recovery, large erase, interrupted
  update, and whole-`www` OTAWWW behavior remain later-phase evidence work.
- Active safety telemetry, production mining, pool behavior, share behavior,
  and soak evidence remain outside Phase 17.
