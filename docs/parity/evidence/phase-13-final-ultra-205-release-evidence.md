# Phase 13 Final Ultra 205 Release Evidence

This ledger summarizes the Phase 13 Ultra 205 release-candidate evidence that was
captured before final checklist and release-document updates. It intentionally
keeps package, USB serial boot, live HTTP, firmware OTA, rollback, recovery,
large erase, failed-update, interrupted-update, and OTAWWW evidence as separate
classes.

## Package Identity

Source evidence:
`docs/parity/evidence/phase-13-final-ultra-205-release-evidence/package-release-gate.md`

Package command:
`just package`

Manifest-backed release-gate command:
`bazel run //tools/parity:report -- release-gate --manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json`

Result: `release_gate: passed`

Package status: `package_status: passed`

Manifest path: `bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json`

Live-flashed package source commit:
`190849539700b8f9a7909fd2b6ebd84142557968`

Reference commit:
`c1915b0a63bfabebdb95a515cedfee05146c1d50`

ESP-IDF version: `v5.5.4`

Rust target: `xtensa-esp32s3-espidf`

Default flash image: `bitaxe-ultra205.elf`

| Artifact | Kind | Offset | SHA-256 |
| --- | --- | --- | --- |
| `bitaxe-ultra205.elf` | firmware_elf | `Unavailable` | `dbe8c778ede1b721a06f44c4d5ab4a1a7558439dd2e6446d97e668e0b5fb9735` |
| `esp-miner.bin` | firmware_ota_image | `0x10000` | `e55e22da45f510b124beba62f56425fd468a95b1efd17949cfc140e15f220c42` |
| `www.bin` | www_spiffs_image | `0x410000` | `0dbb0eba0cc4198186d0175557ec134d7829f3426faf35d8baf263ee0a7c65a0` |
| `bitaxe-ultra205-factory.bin` | factory_merged_image | `0x0` | `b354279c76d6ab05741c4444eb525beeef9be27c7285da394f5ee5396256d37a` |
| `otadata-initial.bin` | otadata_initial | `0xf10000` | `7d2c7ac4888bfd75cd5f56e8d61f69595121183afc81556c876732fd3782c62f` |

Release document inputs named by the manifest:
`docs/release/ultra-205.md`, `docs/release/license-inventory.md`, and
`docs/release/provenance-manifest.md`.

## Detector And Serial Boot

Detector evidence:
`docs/parity/evidence/phase-13-final-ultra-205-release-evidence/hardware-detection.md`
and
`docs/parity/evidence/phase-13-final-ultra-205-release-evidence/detect-ultra205.log`

Detector command:
`just detect-ultra205 > docs/parity/evidence/phase-13-final-ultra-205-release-evidence/detect-ultra205.log 2>&1`

Detector result: `detector_status: passed`

Board: `205`

Selected port: `/dev/cu.usbmodem1101`

Board-info command:
`espflash board-info --chip esp32s3 --port /dev/cu.usbmodem1101 --non-interactive`

Serial boot command:
`just flash-monitor board=205 port=/dev/cu.usbmodem1101 evidence-dir=docs/parity/evidence/phase-13-final-ultra-205-release-evidence/serial-boot capture-timeout-seconds=25`

Serial evidence:
`docs/parity/evidence/phase-13-final-ultra-205-release-evidence/serial-boot.md`,
`docs/parity/evidence/phase-13-final-ultra-205-release-evidence/serial-boot/flash-command-evidence.json`,
and
`docs/parity/evidence/phase-13-final-ultra-205-release-evidence/serial-boot/flash-monitor.log`

Serial result: `serial_boot_status: passed`

Wrapper trust: `trusted_output: true`

Capture status: `timed_out_after_trusted_output`

Observed firmware commit: `190849539700`

Observed reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`

Observed boot markers include board `Ultra 205`, ASIC `BM1366`,
`ota_boot_validation=not_pending state=factory`, `spiffs_mount=available`,
route shell startup, `reset_reason=11`, `esp_idf_version=v5.5.4`, and
`safe_state: mining=disabled asic_work_submission=disabled hardware_control=disabled`.

Conclusion: USB detector and wrapper-owned serial boot evidence passed for the
package source commit above. This does not prove live HTTP/static/recovery,
firmware OTA, rollback, failed-update, large erase, interrupted-update, or
OTAWWW parity.

## DEVICE_URL Status

`DEVICE_URL status: blocked - missing DEVICE_URL`

The Phase 13 helpers did not scan the network, infer a target from serial route
registration, or promote package or serial evidence into live HTTP claims.

Affected evidence classes remain below verified: live static, recovery page,
API/WebSocket coexistence, firmware OTA valid upload, invalid image rejection,
post-OTA boot validation, rollback, failed-update recovery, large erase
recovery, interrupted-update recovery, and OTAWWW live response.

## HTTP Static Recovery

Evidence:
`docs/parity/evidence/phase-13-final-ultra-205-release-evidence/http-static-recovery.md`
and
`docs/parity/evidence/phase-13-final-ultra-205-release-evidence/http-static-recovery/http-static-smoke.log`

Command:
`scripts/phase13-http-static-smoke.sh --device-url "${DEVICE_URL:-}" --manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --out-dir docs/parity/evidence/phase-13-final-ultra-205-release-evidence/http-static-recovery`

Result:

```text
DEVICE_URL status: blocked - missing DEVICE_URL
http_static_status: blocked
conclusion: blocked - live HTTP/static/recovery evidence requires an explicit DEVICE_URL
```

| Request | Expected observation | Actual observation |
| --- | --- | --- |
| `GET /` | `200` fallback static page with admin/release markers | not run - blocked before curl |
| `GET /assets/app.css.gz` | `200` static gzip asset response | not run - blocked before curl |
| `GET /phase13-missing-static` | missing static redirect body `Redirect to the captive portal` | not run - blocked before curl |
| `GET /recovery` | `200` recovery page with `AxeOS Recovery` markers | not run - blocked before curl |
| `GET /api/system/info` | API route coexists with static wildcard | not run - blocked before curl |
| `GET /api/phase13-unknown` | `404` JSON body `{"error":"unknown route"}` | not run - blocked before curl |
| `GET /api/ws` | WebSocket route response, not static wildcard | not run - blocked before curl |
| `GET /api/ws/live` | Live WebSocket route response, not static wildcard | not run - blocked before curl |
| `POST /api/system/OTAWWW` | `400` body `Wrong API input` | not run - blocked before curl |

Checklist conclusion: API route, static, recovery, WebSocket coexistence, and
OTAWWW live response evidence remains below verified because no live HTTP
response was captured.

## Firmware OTA And Invalid Rejection

Evidence:
`docs/parity/evidence/phase-13-final-ultra-205-release-evidence/firmware-ota.md`,
`docs/parity/evidence/phase-13-final-ultra-205-release-evidence/firmware-ota/firmware-ota-smoke.log`,
and
`docs/parity/evidence/phase-13-final-ultra-205-release-evidence/firmware-ota/post-ota-monitor.log`

Command:
`scripts/phase13-firmware-ota-smoke.sh --device-url "${DEVICE_URL:-}" --manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --ota-image bazel-bin/firmware/bitaxe/esp-miner.bin --port /dev/cu.usbmodem1101 --out-dir docs/parity/evidence/phase-13-final-ultra-205-release-evidence/firmware-ota --monitor-seconds 45`

Result:

```text
DEVICE_URL status: blocked - DEVICE_URL unavailable
firmware_ota_status: blocked - DEVICE_URL unavailable
network_scan: disabled - DEVICE_URL must be explicit
conclusion: blocked - firmware OTA evidence requires a reachable just-flashed Ultra 205 DEVICE_URL
```

The helper validated the manifest contract for `esp-miner.bin` and recorded the
OTA artifact checksum, but it did not upload `esp-miner.bin`.

Expected valid OTA public response if a live run is later possible:
`Firmware update complete, rebooting now!`

Invalid image rejection: not run because live HTTP/OTA probes require an
explicit reachable `DEVICE_URL`.

Checklist conclusion: `OTA-001` remains below verified. A valid OTA upload,
invalid image rejection, and post-reboot `boot-validation` evidence are all
required before verified firmware OTA parity.

## Rollback And Boot Validation

Rollback status:
`pending - Plan 04 OTA evidence not run yet`

Boot-validation status:
`pending - Plan 04 OTA evidence not run yet`

The serial boot log captured `ota_boot_validation=not_pending state=factory`.
That is factory boot-state evidence, not rollback proof. A rejected invalid
upload is also not rollback proof. Rollback and boot-validation parity require
post-update bootloader or `ota_boot_validation=` state from a valid OTA attempt.

Checklist conclusion: rollback and boot-validation-sensitive release rows remain
below verified.

## Large Erase

Evidence:
`docs/parity/evidence/phase-13-final-ultra-205-release-evidence/recovery-regression.md`,
`docs/parity/evidence/phase-13-final-ultra-205-release-evidence/recovery-regression/large-erase.log`,
and
`docs/parity/evidence/phase-13-final-ultra-205-release-evidence/recovery-regression/large-erase-post-restore-monitor.log`

Approved erase command shape:
`espflash erase-flash --chip esp32s3 --port <path> --non-interactive`

Selected erase command that would be used only after gates clear:
`espflash erase-flash --chip esp32s3 --port /dev/cu.usbmodem1101 --non-interactive`

Result:

```text
large_erase_status: pending - allow flag not provided
large_erase_post_restore_monitor_status: pending - allow flag not provided
capture_status=pending
```

Factory restore command recorded by the helper:
`just flash board=205 port=/dev/cu.usbmodem1101 image=bazel-bin/firmware/bitaxe/bitaxe-ultra205-factory.bin manifest=bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json evidence-dir=docs/parity/evidence/phase-13-final-ultra-205-release-evidence/recovery-regression/large-erase-restore`

Checklist conclusion: large erase recovery remains pending and below verified.

## Failed Update

Evidence:
`docs/parity/evidence/phase-13-final-ultra-205-release-evidence/recovery-regression.md`
and
`docs/parity/evidence/phase-13-final-ultra-205-release-evidence/recovery-regression/recovery-regression.log`

Failed-update route: `POST /api/system/OTA`

Result:
`failed_update_status: pending - allow flag not provided`

No invalid firmware upload was attempted through live HTTP, no public response
was captured, and no post-failure partition/static/API state exists for this
run.

Checklist conclusion: failed-update recovery remains pending and below verified.

## Interrupted Update

Evidence:
`docs/parity/evidence/phase-13-final-ultra-205-release-evidence/recovery-regression.md`
and
`docs/parity/evidence/phase-13-final-ultra-205-release-evidence/recovery-regression/interrupted-ota.log`

Interrupted-update route: `POST /api/system/OTA`

Interrupted-update artifact: `bazel-bin/firmware/bitaxe/esp-miner.bin`

Result:
`interrupted_update_status: pending - allow flag not provided`

No bounded upload interruption was attempted, no live HTTP status/body was
captured, and no post-interruption recovery check exists because `DEVICE_URL`
was missing and the allow flag was not provided.

Checklist conclusion: interrupted-update recovery remains pending and below
verified.

## OTAWWW REL-03 Gap

Expected route: `POST /api/system/OTAWWW`

Expected public response: `Wrong API input`

Observed live response: not run - blocked before curl because `DEVICE_URL` was
missing.

`www.bin` package generation, serial route registration, and recovery-page
availability do not prove whole-`www` update parity. `OTA-002` remains deferred
unless whole-`www` interrupted-update hardware-regression evidence exists.

Owner: `phase-07-release`

## Checklist Conclusions

Rows that may cite Phase 13 package/serial evidence without new live HTTP
claims:

- `WF-005`: may continue to be `verified` with `hardware-smoke` because Phase
  13 wrapper-owned `just flash-monitor` evidence passed on board `205`, port
  `/dev/cu.usbmodem1101`.
- `SYS-003`: may continue to be `verified` with `hardware-smoke` because the
  serial log observed `safe_state: mining=disabled asic_work_submission=disabled hardware_control=disabled`.

Rows that must stay below verified unless a future run supplies a reachable
`DEVICE_URL` and the required live observations:

- `API-004`, `API-007`, and `API-008` for live HTTP/static/recovery/WebSocket
  behavior.
- `FS-001` for live static, `/assets/app.css.gz`, missing static redirect, and
  `/recovery`.
- `OTA-001` for valid OTA, invalid image rejection, and boot-validation.
- `REL-001` and `REL-002` for live OTA/recovery/rollback-sensitive partition
  and SDK behavior.
- `REL-003` for rollback, recovery, large erase, failed update, and
  interrupted-update evidence.
- `OTA-002` remains deferred with public response `Wrong API input` as the
  expected gap behavior, not observed live evidence.

No row should contain blocker language while marked `verified`.

## Final Verification

Status before Task 3: pending.

The final verification section will be updated after the plan runs the required
script syntax checks, Bazel script tests, Rust pre-commit sequence, package
build, release gate, parity guard, full test command, reference guard, and
reference-tree diff guard.

## Redaction Review

Review file:
`docs/parity/evidence/phase-13-final-ultra-205-release-evidence/redaction-review.md`

Result: passed for generated Phase 13 package, detector, serial, HTTP blocker,
firmware OTA blocker, recovery pending, and final ledger artifacts.

The redaction scan found only expected category labels, Wi-Fi capability text,
NVS partition labels, PSRAM memory-pool log text, route/gap wording, USB/MAC
bench evidence, package paths, commits, and checksums. No Wi-Fi credentials,
pool credentials, API tokens, private endpoints, private `DEVICE_URL` values,
NVS secret values, worker secrets, or raw terminal secrets were present.

## Non-Claims

This ledger does not claim:

- Live `/`, `/assets/app.css.gz`, missing static redirect, `/recovery`, API, or
  WebSocket HTTP behavior.
- Valid firmware OTA, invalid image rejection, selected next partition,
  post-OTA identity, rollback, or boot-validation parity.
- Large erase recovery, failed-update recovery, interrupted-update recovery, or
  post-failure operability.
- OTAWWW whole-`www` update parity.
- Active ASIC initialization, active mining, voltage, fan, thermal, power, or
  broader safety-critical hardware-control parity.
- Hardware evidence for source commits created after the Plan 13-02 live flash.

## Residual Risks

- `DEVICE_URL` is still unavailable, so every live network and OTA evidence
  class remains blocked or pending.
- The live-flashed package identity is source commit
  `190849539700b8f9a7909fd2b6ebd84142557968`. Later documentation-only commits
  require repackaging before publication and do not by themselves create new
  hardware evidence.
- Destructive and fault-injection evidence did not run because the required
  allow flags and live prerequisites were absent.
- OTAWWW remains the explicit REL-03 gap until whole-`www` interrupted-update
  hardware-regression evidence exists.
- Release artifacts remain GPL-risk-reviewed and publication still requires the
  documented release approval posture in `docs/release/provenance-manifest.md`.
