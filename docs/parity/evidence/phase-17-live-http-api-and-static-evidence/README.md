# Phase 17 Live HTTP API And Static Evidence

## Scope

This directory is the Phase 17 evidence root for live HTTP, static asset,
recovery page, API route coexistence, WebSocket frame capture, and redaction
review artifacts for a just-flashed Ultra 205 board `205`.

This scaffold is created before live capture. It does not claim valid firmware
OTA upload, invalid image rejection, reboot, rollback, selected partition, boot
validation, large erase, failed-update recovery, interrupted-update recovery,
whole-`www` OTAWWW update behavior, non-205 boards, production mining, or soak
behavior.

## Required Command Order

Run commands in this order when Phase 17 live evidence is collected:

1. `just package`
2. `bazel run //tools/parity:report -- release-gate --manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json`
3. `just detect-ultra205`
4. `just flash-monitor board=205 port=<port> manifest=bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json evidence-dir=docs/parity/evidence/phase-17-live-http-api-and-static-evidence/serial-boot capture-timeout-seconds=35`
5. `scripts/phase17-live-http-api-smoke.sh --device-url <origin> --manifest docs/parity/evidence/phase-17-live-http-api-and-static-evidence/package-release-gate/bitaxe-ultra205-package.json --flash-evidence-json docs/parity/evidence/phase-17-live-http-api-and-static-evidence/serial-boot/flash-command-evidence.json --out-dir docs/parity/evidence/phase-17-live-http-api-and-static-evidence/http-static-api`
6. `node scripts/phase17-websocket-capture.mjs --device-url <origin> --path /api/ws/live --out docs/parity/evidence/phase-17-live-http-api-and-static-evidence/websocket/api-ws-live.txt`
7. `node scripts/phase17-websocket-capture.mjs --device-url <origin> --path /api/ws --out docs/parity/evidence/phase-17-live-http-api-and-static-evidence/websocket/api-ws.txt`
8. `rg -n -i "ssid|wifi|password|pool|token|device_url|nvs|stratum|https?://|wss?://|([0-9]{1,3}\.){3}[0-9]{1,3}|([[:xdigit:]]{2}:){5}[[:xdigit:]]{2}|secret" docs/parity/evidence/phase-17-live-http-api-and-static-evidence`
9. `just parity`
10. `just verify-reference`

## Target And Identity Gate

Live route probes require an explicit origin-only `--device-url <origin>` or
explicit `DEVICE_URL`. Helpers must not scan, use mDNS, inspect ARP tables,
scrape routers, parse serial logs, or guess targets.

`scripts/phase17-live-http-api-smoke.sh` must block before curl unless the
package manifest and flash evidence JSON agree on source commit, reference
commit, board `205`, trusted `flash-monitor` output, and observed commit
markers. The generated `target-lock.json` stores only sanitized target
provenance and package/flash identity.

## HTTP Static API Evidence

The HTTP helper records per-route `*.headers.txt`, `*.body.txt`, and
`*.curl-error.txt` artifacts under `http-static-api/`, plus
`http-static-api.log`.

Required route coverage:

- `GET /`
- `GET /assets/app.css.gz`
- `GET /phase17-missing-static`
- `GET /recovery`
- `GET /api/system/info`
- `GET /api/phase17-unknown`
- `GET /api/ws`
- `GET /api/ws/live`
- `POST /api/system/OTA`
- `POST /api/system/OTAWWW`

HTTP WebSocket no-upgrade responses are route coexistence only. Empty firmware
OTA POST evidence is route presence only. OTAWWW `Wrong API input` evidence is
the current fail-closed gap response only.

## WebSocket Evidence

Frame-level WebSocket proof is collected separately with
`scripts/phase17-websocket-capture.mjs`.

`/api/ws/live` can support live WebSocket frame evidence only when the bounded
capture records at least one redacted frame. `/api/ws` can support raw log frame
evidence only when the bounded capture records a redacted log frame or accepted
raw-log stream frame. An opened `/api/ws` socket with no frame before timeout is
recorded as pending, not failed static routing and not verified frame behavior.

## Redaction Rules

Only redacted micro-artifacts and reviewed snippets may be cited. Redact private
`DEVICE_URL` values, private endpoints, IP addresses, MAC addresses, Wi-Fi
credentials, pool credentials, worker secrets, API tokens, NVS secret values,
local terminal secrets, and raw URLs.

The redaction review must list generated artifacts and mark missing artifacts as
`absent - not cited` before any checklist or release doc promotion.

## Checklist Promotion Rules

Promote only rows supported by exact Phase 17 artifacts. Do not promote from
helper existence, route registration, package existence, or no-upgrade
WebSocket responses alone.

`FS-001` requires live static behavior evidence naming `/assets/app.css.gz`,
missing static redirect, and `/recovery` with no blocked/pending language.
WebSocket checklist rows require bounded frame artifacts, not only HTTP
coexistence responses.

## Explicit Non-Claims

Phase 17 does not claim:

- valid OTA upload
- invalid image rejection
- reboot after OTA
- rollback
- selected partition
- boot validation
- large erase recovery
- failed-update recovery
- interrupted-update recovery
- whole-`www` OTAWWW update parity
- production mining or soak behavior
- non-205 board verification

## Blocked Evidence Rules

If `DEVICE_URL` is missing, invalid, unreachable, or not the just-flashed
device, write blocked evidence and keep affected rows below verified. If the
package manifest or flash evidence JSON is missing, stale, mismatched, or not
trusted, write blocked evidence and do not run curl or WebSocket capture.

Absent artifacts must be listed as `absent - not cited`; blocked artifacts may
be cited only for the blocker they prove.
