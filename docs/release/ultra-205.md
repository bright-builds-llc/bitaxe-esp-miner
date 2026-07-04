# Ultra 205 Release Operator Guide

This guide is the Phase 7 operator contract for Bitaxe Ultra 205 release
candidates. It gives the commands, artifacts, evidence gates, and recovery
paths a developer needs before using package, flash, OTA, recovery, rollback,
or erase workflows on connected hardware.

Do not treat package generation or serial route registration as live HTTP,
WebSocket, OTA, rollback, recovery, large erase, failed update, or interrupted
update verification. Phase 16 records the prior release-evidence status in
`docs/parity/evidence/phase-16-current-commit-release-evidence-completion.md`.
Phase 17 records the current live HTTP/API/static/WebSocket evidence status in
`docs/parity/evidence/phase-17-live-http-api-and-static-evidence/summary.md`.
Phase 18 records the current firmware OTA evidence status in
`docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/summary.md`.
Phase 19 records the current recovery-regression and OTAWWW closure status in
`docs/parity/evidence/phase-19-recovery-regression-and-otawww-evidence/summary.md`.
Use those ledgers' blocker and pending language and keep affected parity rows
below `verified`.

## Phase 19 Recovery Regression And OTAWWW Evidence Status

Phase 19 evidence is recorded in
`docs/parity/evidence/phase-19-recovery-regression-and-otawww-evidence/summary.md`,
with package, serial boot, target-lock, recovery-regression, OTAWWW gap, and
redaction artifacts under
`docs/parity/evidence/phase-19-recovery-regression-and-otawww-evidence/`.

Current conclusion: package generation, manifest-backed release gate, Ultra
205 detector evidence, wrapper-owned factory flash-monitor evidence, and final
redaction review passed for board `205`, port `/dev/cu.usbmodem1101`, source
commit `6842d7a6d3d4fc64d93900a9847c8a0b97edc16d`, and reference commit
`c1915b0a63bfabebdb95a515cedfee05146c1d50`.

The Phase 19 target lock remains blocked because no trusted raw origin-only
target path was available. Network scanning stayed disabled, and no device URL
was inferred from committed redacted serial evidence. Recovery-regression
operations that require explicit destructive or fault-injection permission did
not run.

Supported Phase 19 claims are limited to:

- package and release-gate identity from
  `docs/parity/evidence/phase-19-recovery-regression-and-otawww-evidence/package-release-gate.md`;
- detector and wrapper-owned flash-monitor identity from
  `docs/parity/evidence/phase-19-recovery-regression-and-otawww-evidence/serial-boot.md`;
- blocked target provenance from
  `docs/parity/evidence/phase-19-recovery-regression-and-otawww-evidence/target-lock.json`;
- recovery-regression pending status from
  `docs/parity/evidence/phase-19-recovery-regression-and-otawww-evidence/recovery-regression.md`;
- explicit OTAWWW REL-03 gap documentation from
  `docs/parity/evidence/phase-19-recovery-regression-and-otawww-evidence/otawww.md`;
- final redaction status from
  `docs/parity/evidence/phase-19-recovery-regression-and-otawww-evidence/redaction-review.md`.

Phase 19 does not claim valid firmware OTA verification, post-OTA reboot
identity, selected next partition, post-OTA boot-validation, rollback,
destructive rollback, failed-update recovery beyond previously recorded invalid
image rejection, large erase, interrupted update, whole-`www` OTAWWW update
behavior, production mining, pool behavior, active safety telemetry, or long
soak behavior. Keep `OTA-001`, `OTA-002`, `REL-001`, `REL-002`, and `REL-003`
below `verified` unless a later artifact records the specific missing behavior
required by the parity checklist.

The Phase 19 package/release-gate command sequence was:

```bash
just package
bazel run //tools/parity:report -- release-gate --manifest docs/parity/evidence/phase-19-recovery-regression-and-otawww-evidence/package-release-gate/bitaxe-ultra205-package.json
```

The Phase 19 serial evidence command was:

```bash
just flash-monitor board=205 port=/dev/cu.usbmodem1101 manifest=bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json wifi-credentials=wifi-credentials.json evidence-dir=docs/parity/evidence/phase-19-recovery-regression-and-otawww-evidence/serial-boot capture-timeout-seconds=45 redact-evidence=true
```

The Phase 19 recovery/OTAWWW helper recorded pending and blocked evidence only:

```bash
scripts/phase19-recovery-otawww-evidence.sh \
  --manifest docs/parity/evidence/phase-19-recovery-regression-and-otawww-evidence/package-release-gate/bitaxe-ultra205-package.json \
  --factory-image bazel-bin/firmware/bitaxe/bitaxe-ultra205-factory.bin \
  --ota-image bazel-bin/firmware/bitaxe/esp-miner.bin \
  --port /dev/cu.usbmodem1101 \
  --out-dir docs/parity/evidence/phase-19-recovery-regression-and-otawww-evidence \
  --target-lock-out docs/parity/evidence/phase-19-recovery-regression-and-otawww-evidence/target-lock.json
```

## Phase 18 Firmware OTA And Rollback Evidence Status

Phase 18 evidence is recorded in
`docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/summary.md`,
with package, serial boot, target-lock, firmware OTA, and redaction artifacts
under `docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/`.

Current conclusion: package, manifest-backed release gate, Ultra 205 detector,
wrapper-owned factory flash-monitor evidence, sanitized target lock, invalid
image rejection, final redaction review, and a valid upload HTTP response were
captured for board `205`, port `/dev/cu.usbmodem1101`, source commit
`22d02f8e97928f1ec29360552179380b92582e6a`, and reference commit
`c1915b0a63bfabebdb95a515cedfee05146c1d50`.

The target was accepted from trusted local USB flash-monitor evidence and
committed only as sanitized provenance in
`docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/target-lock.json`.
Network scanning remained disabled. The final redaction review records
`redaction_status: passed` in
`docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/redaction-review.md`.

Supported Phase 18 claims are limited to:

- package and release-gate identity from
  `docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/package-release-gate.md`;
- detector and wrapper-owned flash-monitor identity from
  `docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/serial-boot.md`;
- sanitized target provenance from
  `docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/target-lock.json`;
- invalid image rejection for `POST /api/system/OTA` from
  `docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/firmware-ota.md`;
- valid upload response only: HTTP 200 with `Firmware update complete, rebooting now!`;
- final redaction status from
  `docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/redaction-review.md`.

Phase 18 does not claim valid OTA verification, post-OTA reboot identity,
selected next partition, post-OTA boot-validation, rollback, destructive
rollback, failed-update recovery beyond invalid image rejection, large erase,
interrupted update, OTAWWW whole-`www` update behavior, production mining, pool
behavior, active safety telemetry, or long soak behavior. Keep `OTA-001`,
`REL-001`, `REL-002`, and `REL-003` below `verified` unless a later artifact
records the specific missing behavior required by the parity checklist.

The Phase 18 package/release-gate command sequence was:

```bash
just package
bazel run //tools/parity:report -- release-gate --manifest docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/package-release-gate/bitaxe-ultra205-package.json
```

The Phase 18 serial evidence command was:

```bash
just flash-monitor board=205 port=/dev/cu.usbmodem1101 manifest=bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json wifi-credentials=wifi-credentials.json evidence-dir=docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/serial-boot capture-timeout-seconds=45 redact-evidence=true
```

The Phase 18 firmware OTA helper recorded invalid rejection, valid upload
response, and blocked post-OTA marker evidence:

```bash
scripts/phase18-firmware-ota-evidence.sh --manifest docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/package-release-gate/bitaxe-ultra205-package.json --ota-image bazel-bin/firmware/bitaxe/esp-miner.bin --port /dev/cu.usbmodem1101 --out-dir docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence --target-lock-out docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/target-lock.json --monitor-seconds 45 --use-flash-log-device-url target/phase18-firmware-ota-and-rollback-evidence-dev-raw/flash-command-evidence.json
```

## Phase 17 Live HTTP API And Static Evidence Status

Phase 17 evidence is recorded in
`docs/parity/evidence/phase-17-live-http-api-and-static-evidence/summary.md`,
with package, serial, HTTP, WebSocket, and redaction artifacts under
`docs/parity/evidence/phase-17-live-http-api-and-static-evidence/`.

Current conclusion: package, manifest-backed release gate, Ultra 205 detector,
wrapper-owned factory flash-monitor evidence, sanitized target lock, live
HTTP/static/API route evidence, bounded WebSocket frame evidence, and final
redaction review passed for board `205`, port `/dev/cu.usbmodem1101`, source
commit `9a2bf5850ea042731f6a7947cc7eb04dc4589e90`, and reference commit
`c1915b0a63bfabebdb95a515cedfee05146c1d50`.

The live target was accepted from trusted local USB flash-monitor evidence and
committed only as sanitized provenance in
`docs/parity/evidence/phase-17-live-http-api-and-static-evidence/target-lock.json`.
Network scanning remained disabled. The final redaction review records
`redaction_status: passed` in
`docs/parity/evidence/phase-17-live-http-api-and-static-evidence/redaction-review.md`.

Supported Phase 17 claims are limited to:

- package and release-gate identity from
  `docs/parity/evidence/phase-17-live-http-api-and-static-evidence/package-release-gate.md`;
- detector and wrapper-owned flash-monitor identity from
  `docs/parity/evidence/phase-17-live-http-api-and-static-evidence/serial-boot.md`;
- live HTTP/static/API route evidence from
  `docs/parity/evidence/phase-17-live-http-api-and-static-evidence/http-static-api.md`;
- bounded WebSocket frame evidence from
  `docs/parity/evidence/phase-17-live-http-api-and-static-evidence/websocket.md`;
- final redaction status from
  `docs/parity/evidence/phase-17-live-http-api-and-static-evidence/redaction-review.md`.

Phase 17 does not claim valid OTA upload, invalid OTA rejection, reboot,
rollback, selected partition, boot validation, whole-`www` OTAWWW update
behavior, production mining, pool behavior, active safety telemetry, or long
soak behavior. Phase 17 supports live-evidence promotion for `FS-001`,
`API-004`, `API-005`, `API-006`, `API-007`, and `API-008` only to the extent
recorded in the cited artifacts. Keep `OTA-001`, `OTA-002`, and `REL-003`
below `verified` unless a later artifact records the specific OTA, rollback,
boot-validation, or whole-`www` behavior required by the parity checklist.

The Phase 17 package/release-gate command sequence was:

```bash
just package
bazel run //tools/parity:report -- release-gate --manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json
```

The Phase 17 serial evidence command was:

```bash
just flash-monitor board=205 port=/dev/cu.usbmodem1101 manifest=bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json wifi-credentials=wifi-credentials.json evidence-dir=docs/parity/evidence/phase-17-live-http-api-and-static-evidence/serial-boot capture-timeout-seconds=45 redact-evidence=true
```

The Phase 17 HTTP helper command recorded live route evidence and a sanitized
target lock:

```bash
scripts/phase17-live-http-api-smoke.sh --use-flash-log-device-url --manifest docs/parity/evidence/phase-17-live-http-api-and-static-evidence/package-release-gate/bitaxe-ultra205-package.json --flash-evidence-json target/phase17-gap-current-dev-raw/flash-command-evidence.json --out-dir docs/parity/evidence/phase-17-live-http-api-and-static-evidence/http-static-api --target-lock-out docs/parity/evidence/phase-17-live-http-api-and-static-evidence/target-lock.json
```

The Phase 17 WebSocket capture commands recorded bounded frame artifacts:

```bash
node scripts/phase17-websocket-capture.mjs --device-url-from-flash-evidence target/phase17-gap-current-dev-raw/flash-command-evidence.json --path /api/ws/live --out docs/parity/evidence/phase-17-live-http-api-and-static-evidence/websocket/api-ws-live.txt --duration-ms 5000 --max-frames 3
node scripts/phase17-websocket-capture.mjs --device-url-from-flash-evidence target/phase17-gap-current-dev-raw/flash-command-evidence.json --path /api/ws --out docs/parity/evidence/phase-17-live-http-api-and-static-evidence/websocket/api-ws.txt --duration-ms 5000 --max-frames 3
```

## Phase 16 Current-Commit Evidence Status

Phase 16 current-commit release evidence is recorded in
`docs/parity/evidence/phase-16-current-commit-release-evidence-completion.md`, with
component evidence under
`docs/parity/evidence/phase-16-current-commit-release-evidence-completion/`.

Current conclusion: package, manifest-backed release gate, Ultra 205 detector,
factory flash, and serial boot evidence passed for board `205`, port
`/dev/cu.usbmodem1101`, source commit
`8490118a7e7f6fc1a9ac2e4025d983b0f402c8ca`, and reference commit
`c1915b0a63bfabebdb95a515cedfee05146c1d50`.

Historical Phase 16 live network gate: no reachable just-flashed device URL was
available for that earlier evidence set.

Because no reachable just-flashed device URL was available, live HTTP, static,
recovery, firmware OTA, invalid image rejection, rollback, failed update
recovery, large erase, and interrupted-update checks remain blocked or pending.
No network scan, upload, erase, rollback, or destructive recovery operation ran.

Keep `FS-001`, `OTA-001`, `REL-001`, `REL-002`, and `REL-003` below
`verified` until a later evidence record includes the required live observations.
Keep the OTAWWW REL-03 gap and public response `Wrong API input` until
whole-`www` hardware-regression and interrupted-update evidence exists.

The Phase 16 package/release-gate command sequence was:

```bash
just package
bazel run //tools/parity:report -- release-gate --manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json
```

The Phase 16 serial evidence command was:

```bash
just flash-monitor board=205 port=/dev/cu.usbmodem1101 manifest=bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json evidence-dir=docs/parity/evidence/phase-16-current-commit-release-evidence-completion/serial-boot capture-timeout-seconds=35
```

The Phase 16 JSON source of truth is
`docs/parity/evidence/phase-16-current-commit-release-evidence-completion/serial-boot/flash-command-evidence.json`.
The wrapper-owned serial log is
`docs/parity/evidence/phase-16-current-commit-release-evidence-completion/serial-boot/flash-monitor.log`.

## Historical Phase 13 Evidence Status

Phase 13 release evidence is historical evidence for source commit
`190849539700b8f9a7909fd2b6ebd84142557968`; see
`docs/parity/evidence/phase-13-final-ultra-205-release-evidence.md`.

## Build And Package

Run the canonical command surface from the repository root:

```bash
just build
just package
```

`just package` writes the Ultra 205 package manifest at:

```text
bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json
```

The package manifest must name the release artifacts directly:

- `bitaxe-ultra205.elf` - USB flashing default image for `tools/flash`.
- `esp-miner.bin` - firmware OTA image for `/api/system/OTA`.
- `www.bin` - static filesystem image built from the Rust-owned `www` tree.
- `bitaxe-ultra205-factory.bin` - merged factory/recovery image.
- `otadata-initial.bin` - OTA data initial image or erased-flash fallback.

Record the package manifest path, source commit, reference commit, artifact
SHA-256 values, ESP-IDF version, Rust target, and package command output before
using the artifacts in release evidence.

The historical Phase 13 package/release-gate command sequence was:

```bash
just package
bazel run //tools/parity:report -- release-gate --manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json
```

## USB Flash

Flash the Ultra 205 with an explicit serial port:

```bash
just flash board=205 port=<port>
```

Current Rust firmware does not start the upstream Bitaxe setup AP or captive
portal. For Phase 17 live HTTP/WebSocket bring-up, seed router credentials at
flash time by copying the checked-in example to a local, untracked JSON file:

```bash
cp wifi-credentials.json.example wifi-credentials.json
```

Edit `wifi-credentials.json` with your router credentials:

```json
{"ssid":"<router-ssid>","wifiPass":"<router-password>"}
```

Pass that file to `just flash` or `just flash-monitor`:

```bash
just flash-monitor board=205 port=<port> wifi-credentials=wifi-credentials.json evidence-dir=<path>
```

Developer USB evidence is raw by default: the persisted flash-monitor artifacts
may keep SSID, IP address, MAC address, and `device_url` values so live
HTTP/WebSocket UAT can use the just-flashed target. Hard secrets such as
`wifiPass`, pool credentials, tokens, API keys, and NVS secret values remain
redacted. For commit-ready or shareable artifacts, request full redaction:

```bash
just flash-monitor board=205 port=<port> wifi-credentials=wifi-credentials.json evidence-dir=<path> redact-evidence=true
```

The flash wrapper generates and writes a replacement NVS partition at `0x9000`
using the upstream keys `wifissid` and `wifipass`. Missing non-Wi-Fi settings
load firmware defaults. If credentials are absent or wrong, recover by USB
reseeding with another `wifi-credentials=<local-wifi-json>` flash run. Do not
commit the credential file, passwords, pool credentials, tokens, API keys,
private endpoints, or NVS secret values. Raw developer artifacts containing
local SSIDs, IP addresses, MAC addresses, or `device_url` values must be
redacted before commit or release citation.

For Phase 21 live mining verification, keep owner pool input in a local,
untracked JSON file by copying the checked-in example:

```bash
cp pool-credentials.json.example pool-credentials.json
```

Edit `pool-credentials.json` so `poolUser` is your BTC-address-derived worker,
such as `<owner-btc-address>.bitaxe`. The sample uses the upstream Ultra 205
default pool endpoint shape:

```json
{
  "poolURL": "public-pool.io",
  "poolPort": 3333,
  "poolUser": "<owner-btc-address>.bitaxe",
  "poolPassword": "x"
}
```

Pass the file only to repo-owned local live mining verification commands:

```bash
scripts/phase21-live-mining-evidence.sh ... --pool-credentials pool-credentials.json
```

The `.env` shape remains available as a fallback for commands that need direct
`BITAXE_POOL_*` environment variables:

```bash
cp pool-credentials.env.example pool-credentials.env
set -a
. ./pool-credentials.env
set +a
```

Do not commit real `pool-credentials*.json` or `pool-credentials*.env` files,
paste their contents into chat, or copy raw pool values into evidence.
Commit-ready evidence should use category labels such as
`pool_config: local-owner-supplied` and must redact or omit raw pool URLs,
ports, users, workers, addresses, passwords, endpoints, tokens, and NVS secret
values.

Use the factory image path from
`bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json` when recovery requires
a full USB flash baseline. Record the exact port, package manifest, source
commit, reference commit, and observed flash result.

## Monitor

Monitor an already-flashed device with:

```bash
just monitor port=<port>
```

Logs should record firmware identity, board/ASIC target, reset reason, package
or partition identity when available, SPIFFS status, OTA status, rollback boot
validation status, and recovery-relevant errors.

## Flash And Monitor Evidence Capture

For the Phase 9 wrapper-owned USB flash and boot-log capture, run:

```bash
just flash-monitor board=205 port=<port> evidence-dir=docs/parity/evidence/phase-09-flash-monitor-evidence-wrapper-hardening
```

The optional timeout override is:

```bash
just flash-monitor board=205 port=<port> evidence-dir=docs/parity/evidence/phase-09-flash-monitor-evidence-wrapper-hardening capture-timeout-seconds=25
```

The JSON source of truth is
`docs/parity/evidence/phase-09-flash-monitor-evidence-wrapper-hardening/flash-command-evidence.json`.
The wrapper-owned serial log is
`docs/parity/evidence/phase-09-flash-monitor-evidence-wrapper-hardening/flash-monitor.log`.

Passing capture statuses are `completed` and
`timed_out_after_trusted_output`. Any other status means evidence capture failed and is not trusted.
The JSON record must name the board, port, source commit,
reference commit, package manifest path, flash image path, exact flash command,
exact monitor command, monitor log path, capture mode, capture status, timeout,
trusted-output flag, and conclusion.

This serial evidence proves only the wrapper-owned flash-monitor boot-log path.
It does not verify live HTTP, static, recovery, firmware OTA, invalid image
rejection, rollback, failed update recovery, large erase, interrupted update, or
OTAWWW behavior.

For the historical Phase 13 release-evidence capture, the exact command was:

```bash
just flash-monitor board=205 port=/dev/cu.usbmodem1101 evidence-dir=docs/parity/evidence/phase-13-final-ultra-205-release-evidence/serial-boot capture-timeout-seconds=25
```

The historical Phase 13 JSON source of truth is
`docs/parity/evidence/phase-13-final-ultra-205-release-evidence/serial-boot/flash-command-evidence.json`.
The wrapper-owned serial log is
`docs/parity/evidence/phase-13-final-ultra-205-release-evidence/serial-boot/flash-monitor.log`.

### Flash-Monitor Recovery

If evidence capture fails, use repo-owned recovery and diagnostic steps:

```bash
just detect-ultra205
just flash-monitor board=205 port=<port> evidence-dir=<path>
just monitor port=<port>
```

Use `just monitor port=<port>` as a diagnostic-only follow-up after the wrapper
failure has been recorded. raw `espflash monitor` output must not be used as the trusted Phase 9 proof.

## Firmware OTA

Firmware OTA uses the AxeOS firmware update route:

- Upload target: `/api/system/OTA`
- Expected upload file name: `esp-miner.bin`
- Success response: `Firmware update complete, rebooting now!`
- Visible AxeOS surface: `Update Firmware`

Before claiming firmware OTA verified, capture these facts on Ultra 205:

1. Upload accepted for `esp-miner.bin`.
1. Invalid image rejection or validation failure behavior.
1. AP/APSTA rejection behavior.
1. Progress/status labels and final public response.
1. Selected next app partition.
1. Reboot scheduling and post-reboot firmware identity.
1. Rollback and boot-validation evidence.
1. Recovery procedure if boot validation fails.

The success response proves only that the upload path reached the reboot step.
It does not prove rollback, boot validation, or return-to-operable-state parity
without matching hardware logs.

Phase 16 firmware OTA status is conservative:
`firmware_ota_status: blocked`. The helper did not upload `esp-miner.bin`,
did not run invalid image rejection, did not observe post-reboot identity, and
did not observe boot-validation or rollback state because `DEVICE_URL` was
missing.

Phase 18 firmware OTA status remains conservative. It captured invalid image
rejection with HTTP 500 and `Write Error`, and it captured a valid upload HTTP
200 response with `Firmware update complete, rebooting now!`. It did not
capture post-OTA `firmware_commit=`, `reference_commit=`, or
`ota_boot_validation=` markers, so valid OTA, selected next partition,
post-reboot identity, boot-validation, rollback, and recovery remain below
verified.

## AxeOS Static Update Gap

The upstream AxeOS static update route is:

- Upload target: `/api/system/OTAWWW`
- Expected upload file name: `www.bin`
- Visible AxeOS surface: `Update AxeOS`
- Current public gap response: `Wrong API input`

AxeOS update is not available in this release candidate. Use `just package` to
create `www.bin` and flash the factory image, or use `/recovery` only after the
documented evidence gate is complete.

This is the OTAWWW REL-03 gap. Phase 16 did not observe the public `Wrong API input` response because `DEVICE_URL` was missing. Do not claim static update
parity from `www.bin` package generation alone. REL-03 remains an explicit
release gap until evidence proves whole-`www` partition write behavior, recovery
access, and interrupted-update recovery on Ultra 205.

## Static And Recovery Smoke

Static filesystem smoke must cover:

- `/` serving the packaged AxeOS-compatible static entry point.
- Representative gzip smoke path `/assets/app.css.gz`.
- Missing static asset redirect behavior.
- API route coexistence for `/api/*`, `/api/ws`, `/api/ws/live`,
  `/api/system/OTA`, and `/api/system/OTAWWW`.
- `/recovery` availability when normal static assets are unavailable or
  damaged.

Static smoke is live firmware evidence only when the record names the connected
Ultra 205 board, serial port, firmware commit, reference commit, package
manifest path, and observed HTTP responses.

Phase 16 static/recovery status is conservative: the historical static smoke
was blocked because no reachable just-flashed device URL was available.
No live `/`, `/assets/app.css.gz`, missing static redirect, `/recovery`, API
coexistence, or WebSocket coexistence response was captured.

## Recovery Page

Use the recovery route when AxeOS static assets are missing, corrupt, not
mounted, or intentionally being restored:

- Route: `/recovery`
- Upload file: `www.bin`
- Recovery surface: `AxeOS Recovery`

Recovery evidence must capture the `/recovery` page load, upload attempt,
public response, response body, restart step, post-restart static route state,
and whether the device returned to an operable state. Do not mark recovery
verified until the hardware-smoke record contains these observations.

## Large Erase

Large Erase is destructive. Use it only when the device can be recovered
through USB factory flashing with the current package manifest and a reachable
`DEVICE_URL` for post-restore HTTP/static proof.

Safe procedure:

1. Record the board, port, source commit, reference commit, and package
   manifest path.
1. Confirm `bitaxe-ultra205-factory.bin` is present in
   `bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json`.
1. Rerun `just detect-ultra205`, require exactly one `port=`, require it to
   match the selected port, and run immediate `espflash board-info`.
1. Record the exact erase command and tool version before running it.
1. Flash the factory image with `just flash board=205 port=<port>`.
1. Monitor with `just monitor port=<port>`.
1. Record post-erase boot identity, `safe_state: mining=disabled`,
   `spiffs_mount=available`, filesystem, recovery, and API reachability.
1. Require post-restore HTTP/static smoke to pass.

Do not describe Large Erase as verified unless that full sequence is captured
in hardware evidence.

Phase 16 large erase status is conservative:
`large_erase_status: pending - allow flag not provided`. No erase, restore, or
post-restore monitor command ran.

## Failed Firmware Update

For a failed firmware update, capture:

- Request target `/api/system/OTA`.
- File name and checksum for `esp-miner.bin`.
- Whether the failure is invalid image rejection, upload interruption, write
  error, validation failure, activation failure, or post-reboot validation.
- Public response and internal update status.
- Running partition after reboot or failed activation.
- Rollback logs when a pending image is marked valid or invalid.
- Recovery procedure used to return the Ultra 205 to an operable state.

Do not treat a rejected upload as rollback proof. Rollback requires bootloader
or boot-validation evidence from the actual post-update state.
A failed-update capture must prove invalid-image rejection; a `200` response,
curl failure, wrong-route response, or server error is blocked evidence.

Phase 16 failed-update status is conservative:
`failed_update_status: pending - allow flag not provided`. No invalid firmware
upload was attempted through live HTTP.

Phase 18 captured invalid image rejection through `/api/system/OTA`, but that
record is failed-update rejection evidence only. It does not prove recovery,
rollback, running partition after reboot, interrupted upload behavior, or
return-to-operable-state parity.

## Interrupted Static Update

For interrupted static update evidence, capture:

- Request target `/api/system/OTAWWW`.
- File name and checksum for `www.bin`.
- Point of interruption and observed device state.
- Whether `/`, `/assets/app.css.gz`, missing static redirect behavior, and
  `/recovery` remain reachable.
- Recovery procedure that restores AxeOS or the factory image.
- Final conclusion.

Until the Phase 8 record exists, interrupted static update remains deferred and
OTAWWW remains the REL-03 gap.
For the historical Phase 13 interrupted firmware OTA helper, a completed `200` OTA response
is blocked evidence and post-interruption HTTP/static smoke must pass before a
captured conclusion is valid.

Phase 16 interrupted-update status is conservative:
`interrupted_update_status: pending - allow flag not provided`. No bounded
upload interruption was attempted.

## Rollback And Boot Validation

Rollback evidence must distinguish:

- Upload accepted.
- Next app partition selected.
- Device rebooted.
- New app booted.
- Boot validation marked the pending app valid.
- Boot validation marked a bad pending app invalid and rebooted.
- Rollback path observed or ruled out by captured logs.

Successful `/api/system/OTA` response text is not enough to claim Rollback or
boot-validation parity.

Phase 16 rollback and boot-validation status remains:
`blocked - Plan 16-04 OTA did not run`. Serial factory boot observed
`ota_boot_validation=not_pending state=factory`, which is not rollback proof.

Phase 18 rollback and boot-validation status remains below verified. The
pre-OTA factory boot log observed `ota_boot_validation=not_pending state=factory`,
but the post-OTA monitor captured no `firmware_commit=`, `reference_commit=`,
or `ota_boot_validation=` marker after the valid upload response.

## Evidence Required Before Verified Claims

Before claiming verified release or update parity, evidence must exist for:

- Package artifact generation and manifest checksums.
- Live firmware OTA success and invalid-image rejection on Ultra 205.
- Live static filesystem and `/recovery` smoke on Ultra 205.
- Rollback and boot-validation behavior.
- Failed firmware update recovery.
- Interrupted static update behavior, or the explicit REL-03 gap.
- License inventory and provenance review covering Cargo and non-Cargo inputs.

Use `docs/parity/evidence/phase-07-ota-filesystem-release.md` for the Phase 7
rollup and
`docs/parity/evidence/phase-07-ultra-205-ota-hardware-smoke.md` for the manual
Ultra 205 OTA/recovery smoke capture.
