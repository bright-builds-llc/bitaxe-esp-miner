# Phase 8 Ultra 205 Release Gate Evidence

This ledger records Phase 8 package, hardware detection, and live release-gate
preconditions for the Ultra 205. It intentionally avoids checklist promotion
language until each live surface has the required evidence class.

## Run Identity

| Field | Value |
| --- | --- |
| board | Ultra 205 |
| port | port=/dev/cu.usbmodem1101 |
| source commit | `11374c334aafef3b0b8e5b04cb7810e7e823208f` |
| reference commit | `c1915b0a63bfabebdb95a515cedfee05146c1d50` |
| conclusion | package and hardware detection preconditions passed; live HTTP/OTA evidence pending |

## Hardware Detection Gate

| Field | Value |
| --- | --- |
| detector result | detector result: passed - exactly one likely ESP USB serial port detected |
| board-info output | board-info output: `Chip type: esp32s3 (revision v0.2); Flash size: 16MB; Features: WiFi, BLE, Embedded Flash; MAC address: f0:f5:bd:4a:ab:cc; Secure Boot: Disabled; Flash Encryption: Disabled` |
| selected port | port=/dev/cu.usbmodem1101 |
| command | `just detect-ultra205` |
| board-info command | `espflash board-info --chip esp32s3 --port /dev/cu.usbmodem1101 --non-interactive` |
| detected chip | ESP32-S3 rev v0.2 |
| flash size | 16MB |
| detected MAC | `f0:f5:bd:4a:ab:cc` |
| conclusion | passed - hardware detection gate allows Task 3 flash-monitor evidence attempt |

## Package Manifest

| Field | Value |
| --- | --- |
| package manifest path | `bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json` |
| factory image path | `bazel-bin/firmware/bitaxe/bitaxe-ultra205-factory.bin` |
| firmware OTA image path | `bazel-bin/firmware/bitaxe/esp-miner.bin` |
| www.bin path | `bazel-bin/firmware/bitaxe/www.bin` |
| otadata path | `bazel-bin/firmware/bitaxe/otadata-initial.bin` |
| artifact SHA-256 values | see artifact table below |
| command | `just package` |
| conclusion | package preflight passed and produced the Phase 8 evidence baseline |

| Artifact | Kind | Offset | sha256 |
| --- | --- | --- | --- |
| `bitaxe-ultra205.elf` | firmware_elf | Unavailable | `39f35e5fbadb724b4b2194dfd700d91f2d7a7c2d4af383227f46475537b45cfb` |
| `esp-miner.bin` | firmware_ota_image | 0x10000 | `28af3f014328748977d446cff86a70d9c8c2773eece14a32b058abe723b99197` |
| `www.bin` | www_spiffs_image | 0x410000 | `0dbb0eba0cc4198186d0175557ec134d7829f3426faf35d8baf263ee0a7c65a0` |
| `bitaxe-ultra205-factory.bin` | factory_merged_image | 0x0 | `9ba7f0171382b51733fe894705d31a25840b0cb3d5dfaf4ba392361733e3b169` |
| `firmware/bitaxe/partitions-ultra205.csv` | partition_table | Unavailable | `19f4fe9b96e6807566dcde496697dde11a8c4258f8c74d3439aaee114a33bba5` |
| `otadata-initial.bin` | otadata_initial | 0xf10000 | `7d2c7ac4888bfd75cd5f56e8d61f69595121183afc81556c876732fd3782c62f` |

## DEVICE_URL Discovery

| Field | Value |
| --- | --- |
| DEVICE_URL status | not run - Phase 8 evidence pending |
| DEVICE_URL source | not run - Phase 8 evidence pending |
| sanitized URL evidence | not run - Phase 8 evidence pending |
| exact private URL committed | no |
| conclusion | not run - Phase 8 evidence pending |

## Static And Recovery HTTP Smoke

| Field | Value |
| --- | --- |
| `/` HTTP status | not run - Phase 8 evidence pending |
| `/assets/app.css.gz` HTTP status | not run - Phase 8 evidence pending |
| missing static redirect behavior | not run - Phase 8 evidence pending |
| `/recovery` HTTP status | not run - Phase 8 evidence pending |
| conclusion | not run - Phase 8 evidence pending |

## Firmware OTA Accepted Upload

| Field | Value |
| --- | --- |
| route | `/api/system/OTA` |
| upload artifact | `esp-miner.bin` |
| upload response | not run - Phase 8 evidence pending |
| reboot observed | not run - Phase 8 evidence pending |
| post-reboot identity | not run - Phase 8 evidence pending |
| conclusion | not run - Phase 8 evidence pending |

## Invalid Image Rejection

| Field | Value |
| --- | --- |
| route | `/api/system/OTA` |
| invalid artifact | not run - Phase 8 evidence pending |
| public response | not run - Phase 8 evidence pending |
| device remained operable | not run - Phase 8 evidence pending |
| conclusion | not run - Phase 8 evidence pending |

## Failed Update Recovery

| Field | Value |
| --- | --- |
| failure class | not run - Phase 8 evidence pending |
| running partition after failure | not run - Phase 8 evidence pending |
| recovery procedure | not run - Phase 8 evidence pending |
| conclusion | not run - Phase 8 evidence pending |

## Rollback And Boot Validation

| Field | Value |
| --- | --- |
| pending image state | not run - Phase 8 evidence pending |
| boot-validation output | not run - Phase 8 evidence pending |
| marked-valid observation | not run - Phase 8 evidence pending |
| marked-invalid observation | not run - Phase 8 evidence pending |
| conclusion | not run - Phase 8 evidence pending |

## OTAWWW Gap Response

| Field | Value |
| --- | --- |
| route | `/api/system/OTAWWW` |
| upload artifact | `www.bin` |
| expected public gap response | `Wrong API input` |
| observed response | not run - Phase 8 evidence pending |
| conclusion | not run - Phase 8 evidence pending |

## Large Erase Recovery

| Field | Value |
| --- | --- |
| erase command | not run - Phase 8 evidence pending |
| recovery flash command | not run - Phase 8 evidence pending |
| post-recovery boot | not run - Phase 8 evidence pending |
| conclusion | not run - Phase 8 evidence pending |

## Interrupted Update Recovery

| Field | Value |
| --- | --- |
| interrupted route | not run - Phase 8 evidence pending |
| interruption point | not run - Phase 8 evidence pending |
| post-interruption reachability | not run - Phase 8 evidence pending |
| recovery procedure | not run - Phase 8 evidence pending |
| conclusion | not run - Phase 8 evidence pending |

## Deferred Scope Review

| Field | Value |
| --- | --- |
| non-205 boards | deferred - no Ultra 205 evidence applies to other boards |
| deferred protocols/accessories/UI | deferred - Stratum v2, BAP, all-board images, and Angular UI rewrite remain outside V1 |
| checklist promotion | none in this plan |
| conclusion | not run - Phase 8 evidence pending |

## Secret Redaction Review

| Field | Value |
| --- | --- |
| exact private URLs | not committed |
| Wi-Fi credentials | not committed |
| pool credentials | not committed |
| private endpoints | not committed |
| NVS secret values | not committed |
| conclusion | not run - Phase 8 evidence pending |

## Final Conclusion

not run - Phase 8 evidence pending
