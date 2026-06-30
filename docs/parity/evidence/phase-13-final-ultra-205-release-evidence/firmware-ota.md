# Phase 13 Firmware OTA Evidence

## Command Log

- command: `scripts/phase13-firmware-ota-smoke.sh --device-url "${DEVICE_URL:-}" --manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --ota-image bazel-bin/firmware/bitaxe/esp-miner.bin --port /dev/cu.usbmodem1101 --out-dir docs/parity/evidence/phase-13-final-ultra-205-release-evidence/firmware-ota --monitor-seconds 45`
- firmware_ota_status: blocked - DEVICE_URL unavailable
- board: `205`
- selected port from Plan 13-02: `/dev/cu.usbmodem1101`
- package manifest: `bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json`
- firmware OTA image: `bazel-bin/firmware/bitaxe/esp-miner.bin`
- firmware OTA artifact kind: `firmware_ota_image`
- firmware OTA artifact offset: `0x10000`
- firmware OTA artifact SHA-256: `e55e22da45f510b124beba62f56425fd468a95b1efd17949cfc140e15f220c42`
- manifest source commit: `190849539700b8f9a7909fd2b6ebd84142557968`
- manifest reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- helper source commit: `4e7bd620ee98fe086fc1be3cb84a961e4e9c5979`
- DEVICE_URL status: blocked - DEVICE_URL unavailable
- smoke log: `docs/parity/evidence/phase-13-final-ultra-205-release-evidence/firmware-ota/firmware-ota-smoke.log`
- post-OTA monitor log: `docs/parity/evidence/phase-13-final-ultra-205-release-evidence/firmware-ota/post-ota-monitor.log`

## Gate Decision

Valid firmware OTA was not run. Plan 13-02 detector evidence passed, and Plan
13-05 created the current recovery runbook, but Plan 13-03 recorded
`DEVICE_URL status: blocked - missing DEVICE_URL` and the current environment
does not provide a real `DEVICE_URL`. Per D-06 and D-13, the helper did not scan
the network, infer a target from serial output, or run a valid OTA upload.

The generated helper log records the matching blocked status:

```text
firmware_ota_status: blocked - DEVICE_URL unavailable
```

## Firmware OTA Route Contract

| Field | Evidence |
| --- | --- |
| upload route | `POST /api/system/OTA` |
| required artifact | manifest `esp-miner.bin` |
| required checksum | `e55e22da45f510b124beba62f56425fd468a95b1efd17949cfc140e15f220c42` |
| helper validation | `scripts/phase13-firmware-ota-smoke.sh` checks the manifest `firmware_ota_image` path is `esp-miner.bin` and the file SHA-256 matches the manifest before any valid upload |
| expected valid OTA public response | `Firmware update complete, rebooting now!` |
| expected post-reboot marker | `ota_boot_validation=` in `post-ota-monitor.log` |
| selected next app partition | unavailable - valid OTA did not run and the public route does not expose this state |

## Invalid Image Rejection

Invalid image rejection was not run because live HTTP/OTA probes require an
explicit reachable `DEVICE_URL` for the just-flashed Ultra 205. The helper is
ready to create `firmware-ota/invalid-firmware.bin`, upload it to
`/api/system/OTA`, record the public status/body, and keep that result separate
from the valid OTA upload.

A rejected invalid image is not rollback proof. Rollback and boot-validation
parity require post-update bootloader or `ota_boot_validation=` state from a
valid OTA attempt.

## Valid OTA And Boot Validation

Valid OTA was skipped because `DEVICE_URL` is unavailable. The helper did not
upload `esp-miner.bin`, did not observe the public success body
`Firmware update complete, rebooting now!`, did not run post-OTA monitor capture,
and did not observe `firmware_commit=`, `reference_commit=`, or
`ota_boot_validation=` after reboot.

The expected success path remains:

1. Upload manifest `esp-miner.bin` to `POST /api/system/OTA`.
1. Record public status `200` and body `Firmware update complete, rebooting now!`.
1. Run `scripts/phase13-monitor-capture.sh --port /dev/cu.usbmodem1101 --out docs/parity/evidence/phase-13-final-ultra-205-release-evidence/firmware-ota/post-ota-monitor.log --seconds 45 --no-reset`.
1. Require `firmware_commit=`, `reference_commit=`, and `ota_boot_validation=` markers before any passed firmware OTA conclusion.

## Redaction

redaction: passed for `firmware-ota.md`,
`firmware-ota/firmware-ota-smoke.log`, and
`firmware-ota/post-ota-monitor.log`. No private `DEVICE_URL`, HTTP headers,
HTTP response bodies, Wi-Fi credentials, pool credentials, API tokens, NVS
secret values, private endpoints, or raw terminal secrets were generated because
the helper stopped before curl and monitor capture.

## Conclusion

Conclusion: firmware_ota_status: blocked - DEVICE_URL unavailable. Firmware OTA
valid upload, invalid image rejection, reboot identity, rollback, selected
partition, and boot-validation evidence remain below verified until a reachable
`DEVICE_URL` for the just-flashed Ultra 205 is explicitly provided.
