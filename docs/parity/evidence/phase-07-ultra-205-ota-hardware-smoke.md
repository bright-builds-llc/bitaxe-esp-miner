# Phase 7 Ultra 205 OTA And Recovery Hardware Smoke

This document records the Phase 7 Ultra 205 hardware verification continuation
run on 2026-06-28 after the operator recovered the board with a manual reflash.
It replaces the earlier bootloader-connection blocker with fresh serial
evidence from a corrected factory flash.

The run proves the factory image can be written at `0x0`, the Phase 7 partition
table is present on the device, SPIFFS mounts from the `www` partition, and the
HTTP route shell registers on hardware. It does not prove live HTTP, OTA,
static-route, recovery-page, rollback, large-erase, or interrupted-update
behavior because the firmware did not expose a reachable network address in the
captured boot logs.

## Run Identity

| Field | Value |
| --- | --- |
| board | Ultra 205 |
| port | `/dev/cu.usbmodem1101` |
| detector result | exactly one ESP USB candidate; `espflash board-info --chip esp32s3 --port /dev/cu.usbmodem1101 --non-interactive` passed |
| detected chip | ESP32-S3 rev v0.2, 16 MB flash |
| detected MAC | `f0:f5:bd:4a:ab:cc` |
| firmware commit | `dc9266e34b975db7fbf2ef02041c3410173802e2` with local Phase 7 changes |
| reference commit | `c1915b0a63bfabebdb95a515cedfee05146c1d50` |
| package manifest path | `bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json` |
| app OTA image path | `bazel-bin/firmware/bitaxe/esp-miner.bin` |
| www.bin path | `bazel-bin/firmware/bitaxe/www.bin` |
| factory image path | `bazel-bin/firmware/bitaxe/bitaxe-ultra205-factory.bin` |
| conclusion | passed for Phase 7 serial scope - corrected factory boot and route registration are proven; live HTTP/OTA checks are deferred to Phase 8 because no reachable device URL was exposed |

## Commands Run

Commands were run from `/Users/peterryszkiewicz/Repos/bitaxe-esp-miner`.

| Command | Exit | Result |
| --- | --- | --- |
| `just detect-ultra205` | 0 | Selected `/dev/cu.usbmodem1101`; board-info identified one ESP32-S3 candidate. |
| `just package` | 0 | Produced `bitaxe-ultra205.elf`, `esp-miner.bin`, `www.bin`, `otadata-initial.bin`, `bitaxe-ultra205-factory.bin`, and `bitaxe-ultra205-package.json`. |
| `just flash dry-run=true board=205 port=/dev/cu.usbmodem1101` | 0 | Confirmed the flash command renders `espflash write-bin --chip esp32s3 --port /dev/cu.usbmodem1101 0x0 .../bitaxe-ultra205-factory.bin`. |
| `just flash-monitor board=205 port=/dev/cu.usbmodem1101 evidence-dir=docs/parity/evidence/phase-07-ultra-205-ota-hardware-smoke` | 0 | Wrote and verified the merged factory image, then captured boot logs through HTTP route registration. |
| `cargo test -p bitaxe-api --all-features update_plan` | 0 | Firmware OTA planner tests passed after the hardware fixes. |
| `cargo test -p bitaxe-api --all-features static_plan` | 0 | Static route and recovery resolver tests passed after the hardware fixes. |
| `cargo test -p bitaxe-api --all-features logs` | 0 | Retained log tests passed after the hardware fixes. |
| `just package` | 0 | Rebuilt the final package after the HTTP stack and thread-stack fixes. |

The final hardware command evidence recorded:

```text
flash: espflash write-bin --chip esp32s3 --port /dev/cu.usbmodem1101 0x0 .../bitaxe-ultra205-factory.bin
monitor: espflash monitor --port /dev/cu.usbmodem1101
```

## Package Manifest

Values from the regenerated `bitaxe-ultra205-package.json` used for the final
hardware flash:

| Artifact | SHA-256 | Offset | Notes |
| --- | --- | --- | --- |
| `bitaxe-ultra205.elf` | `39f35e5fbadb724b4b2194dfd700d91f2d7a7c2d4af383227f46475537b45cfb` | `Unavailable` | App ELF artifact. |
| `esp-miner.bin` | `28af3f014328748977d446cff86a70d9c8c2773eece14a32b058abe723b99197` | `0x10000` | App OTA image. |
| `www.bin` | `0dbb0eba0cc4198186d0175557ec134d7829f3426faf35d8baf263ee0a7c65a0` | `0x410000` | Static filesystem image. |
| `otadata-initial.bin` | `7d2c7ac4888bfd75cd5f56e8d61f69595121183afc81556c876732fd3782c62f` | `0xf10000` | OTA data image. |
| `bitaxe-ultra205-factory.bin` | `9ba7f0171382b51733fe894705d31a25840b0cb3d5dfaf4ba392361733e3b169` | `0x0` | Merged factory image selected by `just flash`. |
| `partitions-ultra205.csv` | `19f4fe9b96e6807566dcde496697dde11a8c4258f8c74d3439aaee114a33bba5` | `Unavailable` | CSV partition contract. |

## Corrective Changes Made During This Continuation

The hardware run found three firmware defects after the board was recovered:

- The HTTP server rejected `max_open_sockets = 8`; the ESP-IDF server reported a
  maximum of 7 after its internal sockets. The route shell now configures
  `max_open_sockets = 7`.
- Starting the HTTP server without first initializing ESP-IDF netif/event-loop
  state caused an LwIP `Invalid mbox` assertion in `httpd_start`. The route
  shell now calls `esp_netif_init` and `esp_event_loop_create_default` before
  constructing `EspHttpServer`.
- The live telemetry thread overflowed the default pthread stack immediately
  after route registration. The thread now starts with a 16 KiB stack.

After those fixes, the final serial run reached stable route registration and
did not repeat the retained-log allocation abort, LwIP assertion, or pthread
stack-overflow failure during the observation window.

## Hardware Boot Evidence

The final serial log showed the corrected partition table:

```text
I (51) boot: Partition Table:
I (60) boot:  0 nvs              WiFi data        01 02 00009000 00006000
I (73) boot:  2 factory          factory app      00 00 00010000 00400000
I (79) boot:  3 www              Unknown data     01 82 00410000 00300000
I (86) boot:  4 ota_0            OTA app          00 10 00710000 00400000
I (92) boot:  5 ota_1            OTA app          00 11 00b10000 00400000
I (99) boot:  6 otadata          OTA data         01 00 00f10000 00002000
I (105) boot:  7 coredump         Unknown data     01 03 00f12000 00010000
```

The same boot showed the expected safe-state and startup signals:

```text
I (966) bitaxe_firmware: safe_state: mining=disabled asic_work_submission=disabled hardware_control=disabled
I (986) bitaxe_firmware::display_adapter: display_status=startup_text_rendered model=SSD1306 size=128x32 address=0x3c
I (1006) bitaxe_firmware::boot_validation: ota_boot_validation=not_pending state=factory
I (1226) bitaxe_firmware::filesystem: spiffs_mount=available partition=www total_bytes=2884241 used_bytes=4518
I (1426) bitaxe_firmware::http_api: axeos_api_route_shell=started registered_routes=15
I (1436) bitaxe_firmware: partition=factory
I (1436) bitaxe_firmware: psram_status=available
```

Boot evidence conclusion: passed for factory image installation, partition
layout, SPIFFS mount, boot validation entry, PSRAM availability, display startup,
safe-state logging, and HTTP route registration.

## HTTP URL Discovery

No reachable `DEVICE_URL` was discovered from the allowed sources:

- The serial log contains no IP, DHCP, Wi-Fi association, AP address, mDNS, or
  hostname announcement.
- Current firmware startup initializes the ESP-IDF network stack enough for
  `httpd_start`, but it does not yet join Wi-Fi or start an AP network
  interface before the HTTP server starts.
- No user-provided address was available for this run.

HTTP discovery conclusion: deferred to Phase 8 - HTTP surface was not reachable
because no device URL was available from logs, known local network state, or
user input.

## Firmware OTA Accepted Upload

| Field | Value |
| --- | --- |
| request route | `/api/system/OTA` |
| upload file | `esp-miner.bin` |
| upload checksum | `28af3f014328748977d446cff86a70d9c8c2773eece14a32b058abe723b99197` |
| public response | not run |
| expected success response | `Firmware update complete, rebooting now!` |
| reboot observed | not run |
| post-reboot identity | not run |
| running partition | not run |
| conclusion | deferred to Phase 8 - no reachable HTTP URL was available |

## Invalid Image Rejection

| Field | Value |
| --- | --- |
| route | `/api/system/OTA` |
| invalid artifact | not run |
| public response | not run |
| firmware logs | not run |
| device remained operable | serial boot remained operable; HTTP rejection not exercised |
| conclusion | deferred to Phase 8 - no reachable HTTP URL was available |

## Rollback And Boot Validation

| Field | Value |
| --- | --- |
| pending image state before reboot | not run |
| boot-validation log lines | `ota_boot_validation=not_pending state=factory` observed |
| marked valid log observed | not run |
| marked invalid/reboot log observed | not run |
| rollback observed | not run |
| running partition after rollback decision | factory |
| conclusion | Phase 7 serial boot-validation entry observed; OTA rollback behavior deferred to Phase 8 |

## Static Filesystem Smoke

| Surface | Expected | Observed |
| --- | --- | --- |
| `/` | HTTP success for static entry point | not run |
| gzip asset | `/assets/app.css.gz` served with gzip behavior | not run |
| `/assets/app.css.gz` | representative gzip smoke path reachable | not run |
| missing static redirect | missing static path redirects to `/` | not run |
| API coexistence | `/api/*` routes are not captured by static wildcard | route order registered on hardware; HTTP requests not run |

Static route conclusion: deferred to Phase 8 - SPIFFS mounted and routes
registered, but no reachable HTTP URL was available.

## Recovery Page

| Field | Value |
| --- | --- |
| route | `/recovery` |
| recovery page reachable | not run |
| upload file | `www.bin` |
| upload response | not run |
| restart response | not run |
| post-restart `/` result | not run |
| conclusion | deferred to Phase 8 - no reachable HTTP URL was available |

## OTAWWW Gap Response

| Field | Value |
| --- | --- |
| route | `/api/system/OTAWWW` |
| upload file | `www.bin` |
| expected public response | `Wrong API input` |
| firmware gap log | route registered; request not run |
| conclusion | deferred to Phase 8 - no reachable HTTP URL was available |

## Large Erase And Recovery Steps

No erase command was run. The recovered board accepted the corrected factory
`write-bin` flash, the resulting partition table was correct, and boot reached
SPIFFS and HTTP route registration. A full erase was not needed under the run
plan.

Large erase conclusion: not run - no inconsistent factory boot or partition
state remained after the corrected factory flash.

## Interrupted Update

No interrupted firmware or static update was attempted.

Interrupted-update conclusion: not run - fault-injection recovery was out of
scope for this continuation pass.

## Final Conclusion

passed for Phase 7 serial scope - the recovered Ultra 205 proves corrected
factory flashing, partition layout, SPIFFS mount, PSRAM availability, safe
startup, boot-validation entry, and HTTP route-shell registration. Live `/`,
gzip asset, missing-static redirect, `/recovery`, valid OTA, invalid OTA,
OTAWWW response, rollback, large-erase, and interrupted-update behavior remain
unverified on hardware and are deferred to the Phase 8 release evidence gate.
