# Phase 17 Live HTTP API And Static Evidence Summary

## Scope

Phase 17 records current Ultra 205 package, release-gate, detector,
flash-monitor, HTTP/static/API, WebSocket, and redaction evidence under
`docs/parity/evidence/phase-17-live-http-api-and-static-evidence/`.

The source commit for the package and flashed firmware evidence is
`d9e471c9699eb0140749127416640aa1bf077d26`; the reference commit is
`c1915b0a63bfabebdb95a515cedfee05146c1d50`. Package and serial identity
passed, but live HTTP/static/API and WebSocket probes were blocked because no
explicit origin-only `DEVICE_URL` or explicit-input `target-lock.json` was
available. Network scanning remained disabled.

## Exact Commands

Plan 17-02 package and release-gate commands:

```bash
just package
bazel run //tools/parity:report -- release-gate --manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json
```

Plan 17-02 detector and flash-monitor commands:

```bash
just detect-ultra205
just flash-monitor board=205 port=/dev/cu.usbmodem1101 manifest=bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json evidence-dir=docs/parity/evidence/phase-17-live-http-api-and-static-evidence/serial-boot capture-timeout-seconds=35
```

Plan 17-03 Phase 17 HTTP helper command:

```bash
scripts/phase17-live-http-api-smoke.sh --manifest docs/parity/evidence/phase-17-live-http-api-and-static-evidence/package-release-gate/bitaxe-ultra205-package.json --flash-evidence-json docs/parity/evidence/phase-17-live-http-api-and-static-evidence/serial-boot/flash-command-evidence.json --out-dir docs/parity/evidence/phase-17-live-http-api-and-static-evidence/http-static-api --target-lock-out docs/parity/evidence/phase-17-live-http-api-and-static-evidence/target-lock.json
```

Plan 17-04 WebSocket capture commands that would run only after an explicit
origin-only target exists:

```bash
node scripts/phase17-websocket-capture.mjs --device-url "$DEVICE_URL" --path /api/ws/live --out docs/parity/evidence/phase-17-live-http-api-and-static-evidence/websocket/api-ws-live.txt --duration-ms 5000 --max-frames 3
node scripts/phase17-websocket-capture.mjs --device-url "$DEVICE_URL" --path /api/ws --out docs/parity/evidence/phase-17-live-http-api-and-static-evidence/websocket/api-ws.txt --duration-ms 5000 --max-frames 3
```

Task 1 redaction scan command:

```bash
rg -n -i 'ssid|wifi|password|pool|token|device_url|nvs|stratum|https?://|wss?://|([0-9]{1,3}\.){3}[0-9]{1,3}|([[:xdigit:]]{2}:){5}[[:xdigit:]]{2}|secret' docs/parity/evidence/phase-17-live-http-api-and-static-evidence
```

## Artifact Status Matrix

| Artifact | Status | Citation boundary |
| --- | --- | --- |
| `package-release-gate.md` | present | Package and release-gate identity only. |
| `package-release-gate/bitaxe-ultra205-package.json` copied package manifest | present | Source/reference commits and artifact checksums only. |
| `package-release-gate/package-command.log` | present | `just package` command result only. |
| `package-release-gate/release-gate.log` | present | Release-gate command result only. |
| `serial-boot/detect-ultra205.log` detector log | present | USB board identity and board-info gate only. |
| `serial-boot/flash-command-evidence.json` flash JSON | present | Wrapper-owned flash-monitor command identity only. |
| `serial-boot/flash-monitor.log` flash-monitor log | present | Serial boot, SPIFFS mount, and route registration only. |
| `target-lock.json` | absent - not cited | No explicit origin-only `DEVICE_URL` was accepted. |
| `http-static-api/http-static-api.log` HTTP route log | present | Blocked no-target helper transcript only. |
| `http-static-api/*.headers.txt` HTTP headers | absent - not cited | No live route probes ran. |
| `http-static-api/*.body.txt` HTTP bodies | absent - not cited | No live route probes ran. |
| `http-static-api/*.curl-error.txt` HTTP curl errors | absent - not cited | No live route probes ran. |
| `websocket/websocket-capture.log` | present | Blocked no-target capture transcript only. |
| `websocket/api-ws-live.txt` | absent - not cited | No `/api/ws/live` frame capture ran. |
| `websocket/api-ws.txt` | absent - not cited | No `/api/ws` frame capture ran. |
| `redaction-review.md` | present | Final redaction status for cited Phase 17 artifacts. |

## Package And Flash Identity

`package_status: passed` and `release_gate_status: passed` are recorded in
`package-release-gate.md`. The package manifest copy and wrapper-owned serial
evidence agree on source commit `d9e471c9699eb0140749127416640aa1bf077d26`
and reference commit `c1915b0a63bfabebdb95a515cedfee05146c1d50`.

`detector_status: passed` and `flash_monitor_status: passed` are recorded in
`serial-boot.md`. The detector selected board `205` on
`/dev/cu.usbmodem1101`, and `flash-command-evidence.json` has
`trusted_output: true` with `capture_status: timed_out_after_trusted_output`.
Serial evidence shows factory boot, SPIFFS mounted, and HTTP route shell
registration, but it does not prove live HTTP responses or WebSocket frames.

## HTTP Static API Evidence

`http_static_api_status: blocked` is recorded in `http-static-api.md` because
Plan 17-03 did not receive an explicit origin-only `DEVICE_URL`. The helper
recorded `network_scan: disabled`, did not infer a target from serial logs or
local network state, and did not create `target-lock.json`.

The D-08 route set remains below verified in Phase 17:

| Route | Phase 17 status |
| --- | --- |
| `GET /` | blocked - missing `DEVICE_URL`; absent - not cited |
| `GET /assets/app.css.gz` | blocked - missing `DEVICE_URL`; absent - not cited |
| `GET /phase17-missing-static` | blocked - missing `DEVICE_URL`; absent - not cited |
| `GET /recovery` | blocked - missing `DEVICE_URL`; absent - not cited |
| `GET /api/system/info` | blocked - missing `DEVICE_URL`; absent - not cited |
| `GET /api/phase17-unknown` | blocked - missing `DEVICE_URL`; absent - not cited |
| `GET /api/ws` | blocked - missing `DEVICE_URL`; no no-upgrade route-coexistence response observed |
| `GET /api/ws/live` | blocked - missing `DEVICE_URL`; no no-upgrade route-coexistence response observed |
| `POST /api/system/OTA` | blocked - missing `DEVICE_URL`; route-presence evidence not claimed |
| `POST /api/system/OTAWWW` | blocked - missing `DEVICE_URL`; fail-closed response not observed |

## WebSocket Evidence

`websocket_status: blocked` is recorded in `websocket.md`. Plan 17-04 did not
run a live WebSocket capture because both allowed target sources were absent:
no explicit origin-only `DEVICE_URL` and no explicit-input `target-lock.json`.

`/api/ws/live` and `/api/ws` are separate proof surfaces. `/api/ws/live` may be
promoted only when `websocket/api-ws-live.txt` contains a redacted connect or
cadence frame. `/api/ws` may be promoted only when `websocket/api-ws.txt`
contains a redacted raw-log frame. If a future `/api/ws` capture opens but
records no frame before timeout, the correct status is `pending - open timeout
without raw log frame`, not verified raw-log streaming. Phase 17 has no
no-upgrade or frame-level WebSocket claim.

## Redaction Status

redaction_status: passed

`redaction-review.md` records every cited Phase 17 artifact as reviewed or
`absent - not cited`. The final redaction scan found only allowed matches:
policy labels, absent-artifact statements, command examples, USB identity
metadata, MAC address retained for board identity, WiFi/BLE feature labels,
ESP-IDF NVS boot labels, PSRAM memory-pool log text, and Rust/Cargo version
strings that resemble IP addresses. No Wi-Fi credentials, pool credentials,
worker secrets, API tokens, private endpoints, NVS secret values, raw target
URLs, or local terminal secrets were found.

## Checklist Promotion Matrix

| Checklist row | Phase 17 action |
| --- | --- |
| `FS-001` | Keep implemented, below verified. Live static behavior, `/assets/app.css.gz`, missing static redirect, and `/recovery` are blocked by missing `DEVICE_URL`. |
| `API-004` | Keep implemented, below verified for live HTTP behavior. Serial route registration is not live route proof. |
| `API-005` | Keep implemented, below verified for `/api/ws` raw-log frames. No frame artifact exists. |
| `API-006` | Keep implemented, below verified for `/api/ws/live` telemetry frames. No frame artifact exists. |
| `API-007` | Keep implemented, below verified for live `/recovery` page load. No route artifact exists. |
| `API-008` | Keep implemented, below verified for live static assets and API/WebSocket coexistence. No route artifact exists. |
| `OTA-001` | Keep implemented, below verified. Phase 17 does not claim valid OTA upload, invalid image rejection, reboot, rollback, selected partition, or boot validation. |
| `OTA-002` | Keep deferred. Phase 17 does not claim whole-`www` OTAWWW update behavior. |
| `REL-003` | Keep implemented, below verified. Phase 17 does not prove release image behavior that depends on valid OTA, rollback, recovery, or OTAWWW regression evidence. |

## Explicit Non-Claims

Phase 17 does not claim valid OTA upload, invalid OTA rejection, reboot,
rollback, selected partition, boot validation, whole-`www` OTAWWW update
behavior, production mining, pool behavior, active safety telemetry, or long
soak behavior.

Package generation, release-gate success, detector output, serial boot,
SPIFFS mount, and route registration are cited only for the identity and
workflow facts they directly prove.

## Blocked And Pending States

- `DEVICE_URL status: blocked - missing DEVICE_URL`
- `target-lock.json`: absent - not cited
- Live HTTP/static/API route artifacts: absent - not cited
- `/api/ws/live` frame artifact: absent - not cited
- `/api/ws` frame artifact: absent - not cited
- `/api/ws` open timeout without raw log frame: not observed; pending rule retained for future captures
- Valid OTA, invalid image rejection, rollback, boot validation, whole-`www` OTAWWW, mining, safety telemetry, and soak evidence: not claimed in Phase 17

## Conclusions

Phase 17 closes with a conservative ledger: package, release-gate, detector,
flash-monitor, and redaction review passed for the cited artifacts, while live
HTTP/static/API and WebSocket evidence remains blocked by missing explicit
`DEVICE_URL`. The release docs, parity checklist, and requirements traceability
must cite these exact Phase 17 artifacts and preserve blocked or pending status
for unsupported live surfaces.

## Residual Risks

- A future explicit origin-only `DEVICE_URL` run is still required before live
  static, recovery, API route, WebSocket route-coexistence, or frame-level
  claims can move above implemented.
- Firmware OTA success, invalid image rejection, selected partition, reboot,
  rollback, boot validation, failed-update recovery, large erase, interrupted
  update, and whole-`www` OTAWWW behavior remain later-phase evidence work.
- Active safety telemetry, production mining, pool behavior, share behavior,
  and soak evidence remain outside Phase 17.
