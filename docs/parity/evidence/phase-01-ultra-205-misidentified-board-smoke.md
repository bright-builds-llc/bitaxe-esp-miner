# Phase 01 Ultra 205 Misidentified Board Smoke Report

## Summary

Date: 2026-06-25 America/Chicago

Result: hardware smoke passed only as a safe-state ESP32-S3 boot/flash check on a board later identified by the user as an Ultra 205.

This report is not valid Gamma 601 evidence. The Bitaxe was visible at `/dev/cu.usbmodem1101`, flashed successfully with the Phase 1 Rust firmware, and emitted boot and safe-state log lines after a monitor-attached `CTRL+R` reset, but the physical board was not the Phase 1 target.

Important caveats:

- The physical board was later identified by the user as an Ultra 205, not a Gamma 601.
- `reference/esp-miner/config-205.cvs` identifies Ultra 205 defaults as `devicemodel=ultra`, `boardversion=205`, `asicmodel=BM1366`, `asicfrequency=485`, and `asicvoltage=1200`.
- The flashed Phase 1 firmware is safe-state only and logged `mining=disabled`, `asic_work_submission=disabled`, and `hardware_control=disabled`, so it did not intentionally drive ASIC or power-control surfaces on the Ultra 205.
- The firmware logged `board=Gamma 601 asic=BM1370` because the current Phase 1 firmware is hard-coded for Gamma 601. On an Ultra 205, that identity log is incorrect and must not be used as parity evidence.
- `just flash-monitor` entered monitor mode successfully but did not emit boot logs until `CTRL+R` was sent while the monitor was attached.
- The first flash used a stale cached Bazel firmware image with `firmware_commit=f6f5daa9b079`; `bazel clean` forced a rebuild, and the final verified boot log shows `firmware_commit=9a833407a57f`.
- After `bazel clean`, `just package` rebuilt the firmware ELF but the package genrule failed because it could not find the executable reference guard path. The final flash used the freshly rebuilt ELF via an absolute `image=` override.

## Board

User-corrected board identity: Ultra 205 with BM1366 ASIC.

Observed firmware identity log: Gamma 601 with BM1370 ASIC.

Conclusion: the observed firmware identity log is incorrect for this physical board.

## Port

`/dev/cu.usbmodem1101`

`espflash list-ports` identified:

```text
/dev/cu.usbmodem1101   1001:303A  Espressif  USB JTAG/serial debug unit
/dev/tty.usbmodem1101  1001:303A  Espressif  USB JTAG/serial debug unit
```

## Firmware Commit

Repository HEAD during final flash:

```text
9a833407a57f
```

Observed firmware log line:

```text
firmware_commit=9a833407a57f
```

## Reference Commit

```text
c1915b0a63bfabebdb95a515cedfee05146c1d50
```

## Command Report

| Command | Outcome | Notes |
| --- | --- | --- |
| `ls -l /dev/cu.usbmodem1101 /dev/tty.usbmodem1101 2>&1` | passed | Both macOS serial device nodes existed. |
| `espflash list-ports` | passed | Found Espressif USB JTAG/serial debug unit at `/dev/cu.usbmodem1101` and `/dev/tty.usbmodem1101`. |
| `system_profiler SPUSBDataType` | passed | Produced no useful text output in this shell, so `ioreg` was used for USB detail. |
| `espflash --version && espflash --help \| sed -n '1,180p'` | passed | `espflash 4.0.1`; confirmed available commands. |
| `espflash board-info --port /dev/cu.usbmodem1101` | failed | Initial attempt failed because Chrome had the serial device open. |
| `lsof /dev/cu.usbmodem1101 /dev/tty.usbmodem1101 2>&1` | diagnostic | Showed Google Chrome PID 894 holding `/dev/cu.usbmodem1101`. |
| `ioreg -p IOUSB -l -w 0 \| rg -n "303A\|1001\|Espressif\|usbmodem1101\|JTAG\|serial\|Serial" -C 3` | passed | Confirmed Espressif USB device and serial `F0:F5:BD:4A:AB:CC`. |
| `just monitor dry-run=true port=/dev/cu.usbmodem1101` | passed | Rendered `espflash monitor --port /dev/cu.usbmodem1101`. |
| `just flash dry-run=true board=601 port=/dev/cu.usbmodem1101` | passed | Rendered `espflash flash --chip esp32s3 --port /dev/cu.usbmodem1101 .../bitaxe-gamma601.elf`. |
| `just flash-monitor dry-run=true board=601 port=/dev/cu.usbmodem1101 evidence-dir=docs/parity/evidence/phase-01-gamma-601-boot-log` | passed | Rendered flash and monitor commands without touching hardware. |
| `espflash board-info --port /dev/cu.usbmodem1101` | passed | Confirmed ESP32-S3, 16 MB flash, MAC `f0:f5:bd:4a:ab:cc`, secure boot disabled, flash encryption disabled. |
| `timeout 15s just monitor port=/dev/cu.usbmodem1101` | failed | Non-TTY monitor failed with `Failed to initialize input reader`. |
| `timeout 15s just monitor port=/dev/cu.usbmodem1101` in a PTY | timed out | Monitor attached, but no boot log arrived before timeout. |
| `timeout 20s espflash monitor --chip esp32s3 --before usb-reset --port /dev/cu.usbmodem1101` in a PTY | timed out | Monitor attached, but no boot log arrived before timeout. |
| `timeout 120s just flash-monitor board=601 port=/dev/cu.usbmodem1101` in a PTY | flashed, then timed out | Flash verified successfully, but monitor produced no log before timeout. Boot log was later captured by manually sending `CTRL+R`. |
| `espflash monitor --port /dev/cu.usbmodem1101` in a PTY, then `CTRL+R` | passed | Captured expected boot log, but from stale cached firmware commit `f6f5daa9b079`. |
| `bazel clean` | passed | Cleared stale Bazel action cache. |
| `just package` | failed after firmware rebuild | Firmware ELF rebuilt successfully, but package genrule failed: `bazel-out/darwin_arm64-opt-exec/bin/scripts/verify_reference_clean: No such file or directory`. |
| `strings bazel-bin/firmware/bitaxe/bitaxe-firmware.elf \| rg "9a833407a57f\|f6f5daa9b079\|firmware_commit\|bitaxe-rust boot" -n` | passed | Confirmed rebuilt ELF contains `9a833407a57f`. |
| `just flash-monitor board=601 port=/dev/cu.usbmodem1101 image=bazel-bin/firmware/bitaxe/bitaxe-firmware.elf` | failed | Relative `image=` path was not found under `bazel run` execroot. |
| `just flash-monitor board=601 port=/dev/cu.usbmodem1101 image=/Users/peterryszkiewicz/Repos/bitaxe-esp-miner/bazel-bin/firmware/bitaxe/bitaxe-firmware.elf` in a PTY, then `CTRL+R` | passed | Flashed the current-commit ELF and captured the required boot log. |

## Board Info

```text
Chip type:         esp32s3 (revision v0.2)
Crystal frequency: 40 MHz
Flash size:        16MB
Features:          WiFi, BLE, Embedded Flash
MAC address:       f0:f5:bd:4a:ab:cc

Security Information:
=====================
Flags: 0x00000000 (0)
Key Purposes: [0, 0, 0, 0, 0, 0, 12]
Chip ID: 9
API Version: 0
Secure Boot: Disabled
Flash Encryption: Disabled
SPI Boot Crypt Count (SPI_BOOT_CRYPT_CNT): 0x0
```

## Final Flash Command

```bash
just flash-monitor board=601 port=/dev/cu.usbmodem1101 image=/Users/peterryszkiewicz/Repos/bitaxe-esp-miner/bazel-bin/firmware/bitaxe/bitaxe-firmware.elf
```

Rendered `espflash` command:

```text
espflash flash --chip esp32s3 --port /dev/cu.usbmodem1101 /Users/peterryszkiewicz/Repos/bitaxe-esp-miner/bazel-bin/firmware/bitaxe/bitaxe-firmware.elf
```

Flash result excerpt:

```text
Chip type:         esp32s3 (revision v0.2)
Crystal frequency: 40 MHz
Flash size:        16MB
Features:          WiFi, BLE, Embedded Flash
MAC address:       f0:f5:bd:4a:ab:cc
App/part. size:    447,872/16,384,000 bytes, 2.73%
0x0      Skipped! (checksum matches)
0x8000   Skipped! (checksum matches)
0x10000  Verifying... OK!
monitor_command: espflash monitor --port /dev/cu.usbmodem1101
```

## Captured Boot Log

Captured by sending `CTRL+R` while `espflash monitor --port /dev/cu.usbmodem1101` was attached through the repo `flash-monitor` command.

```text
ESP-ROM:esp32s3-20210327
Build:Mar 27 2021
rst:0x15 (USB_UART_CHIP_RESET),boot:0x28 (SPI_FAST_FLASH_BOOT)
Saved PC:0x40378ca5
SPIWP:0xee
mode:DIO, clock div:2
load:0x3fce2810,len:0x15a0
load:0x403c8700,len:0x4
load:0x403c8704,len:0xd24
load:0x403cb700,len:0x2f04
entry 0x403c8928
I (27) boot: ESP-IDF v5.4.1-426-g3ad36321ea 2nd stage bootloader
I (27) boot: compile time Apr 24 2025 15:55:11
I (28) boot: Multicore bootloader
I (29) boot: chip revision: v0.2
I (32) boot: efuse block revision: v1.3
I (35) boot.esp32s3: Boot SPI Speed : 40MHz
I (39) boot.esp32s3: SPI Mode       : DIO
I (43) boot.esp32s3: SPI Flash Size : 16MB
I (47) boot: Enabling RNG early entropy source...
I (51) boot: Partition Table:
I (54) boot: ## Label            Usage          Type ST Offset   Length
I (60) boot:  0 nvs              WiFi data        01 02 00009000 00006000
I (67) boot:  1 phy_init         RF data          01 01 0000f000 00001000
I (73) boot:  2 factory          factory app      00 00 00010000 00fa0000
I (80) boot: End of partition table
I (83) esp_image: segment 0: paddr=00010020 vaddr=3c050020 size=13694h ( 79508) map
I (110) esp_image: segment 1: paddr=000236bc vaddr=3fc91900 size=034b8h ( 13496) load
I (114) esp_image: segment 2: paddr=00026b7c vaddr=40374000 size=0949ch ( 38044) load
I (125) esp_image: segment 3: paddr=00030020 vaddr=42000020 size=490f4h (299252) map
I (199) esp_image: segment 4: paddr=0007911c vaddr=4037d49c size=0440ch ( 17420) load
I (205) esp_image: segment 5: paddr=0007d530 vaddr=50000000 size=00020h (    32) load
I (211) boot: Loaded app from partition at offset 0x10000
I (211) boot: Disabling RNG early entropy source...
I (225) cpu_start: Multicore app
I (233) cpu_start: GPIO 44 and 43 are used as console UART I/O pins
I (235) cpu_start: Pro cpu start user code
I (235) cpu_start: cpu freq: 160000000 Hz
I (236) app_init: Application information:
I (240) app_init: Project name:     libespidf
I (244) app_init: App version:      9a83340
I (248) app_init: Compile time:     Jun 25 2026 22:21:38
I (253) app_init: ELF file SHA256:  000000000...
I (257) app_init: ESP-IDF:          v5.5.4
I (261) efuse_init: Min chip rev:     v0.0
I (265) efuse_init: Max chip rev:     v0.99
I (269) efuse_init: Chip rev:         v0.2
I (273) heap_init: Initializing. RAM available for dynamic allocation:
I (279) heap_init: At 3FC95780 len 00053F90 (335 KiB): RAM
I (284) heap_init: At 3FCE9710 len 00005724 (21 KiB): RAM
I (289) heap_init: At 3FCF0000 len 00008000 (32 KiB): DRAM
I (294) heap_init: At 600FE000 len 00001FE8 (7 KiB): RTCRAM
I (301) spi_flash: detected chip: gd
I (303) spi_flash: flash io: dio
I (307) sleep_gpio: Configure to isolate all GPIO pins in sleep state
I (312) sleep_gpio: Enable automatic switching of GPIO sleep configuration
I (319) main_task: Started on CPU0
I (329) main_task: Calling app_main()
I (329) bitaxe_firmware: bitaxe-rust boot: board=Gamma 601 asic=BM1370
I (329) bitaxe_firmware: safe_state: mining=disabled asic_work_submission=disabled hardware_control=disabled
I (339) bitaxe_firmware: reset_reason=11
I (349) bitaxe_firmware: partition=factory
I (349) bitaxe_firmware: psram_status=unavailable
I (349) bitaxe_firmware: firmware_commit=9a833407a57f
I (359) bitaxe_firmware: reference_commit=c1915b0a63bfabebdb95a515cedfee05146c1d50
I (369) bitaxe_firmware: esp_idf_version=v5.5.4
I (369) bitaxe_firmware: rust_target=xtensa-esp32s3-espidf
I (379) main_task: Returned from app_main()
```

## Observed Firmware Log Lines

- `bitaxe-rust boot: board=Gamma 601 asic=BM1370`
- `safe_state: mining=disabled asic_work_submission=disabled hardware_control=disabled`
- `reset_reason=11`
- `partition=factory`
- `psram_status=unavailable`
- `firmware_commit=9a833407a57f`
- `reference_commit=c1915b0a63bfabebdb95a515cedfee05146c1d50`
- `esp_idf_version=v5.5.4`
- `rust_target=xtensa-esp32s3-espidf`

## Conclusion

Conclusion: hardware-smoke evidence captured for a misidentified Ultra 205 only.

The Ultra 205 accepted the Phase 1 Rust firmware, booted from the `factory` partition, logged the safe no-mining/no-hardware-control state, and reported the expected ESP-IDF target and reference commit.

This evidence must not be used to mark Gamma 601 parity rows verified. It proves that the ESP32-S3 serial flash path and safe-state firmware can run on the connected board, but it also proves the current Phase 1 firmware identity is wrong for Ultra 205.

Follow-up issues found during evidence capture:

- Do not flash the Gamma 601-only Phase 1 image to Ultra 205 as functional firmware. It is safe-state only and not a correct Ultra 205 mining firmware.
- The Bazel firmware genrule does not track git HEAD as an action input, so cached firmware artifacts can embed stale `BITAXE_FIRMWARE_COMMIT` values after rebases.
- `//firmware/bitaxe:firmware_image` can fail after a clean build because `scripts/package-firmware.sh` receives a Bazel execpath for `//scripts:verify_reference_clean` that is not available at runtime.
- `espflash monitor` requires a PTY with this setup. Non-TTY capture mode fails with `Failed to initialize input reader`.
- `just flash-monitor` attaches to the monitor after flashing, but no boot log was observed until `CTRL+R` was sent manually while attached.
