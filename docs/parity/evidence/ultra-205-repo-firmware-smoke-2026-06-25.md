# Ultra 205 Repo Firmware Smoke Report

## Summary

Date: 2026-06-25 22:50:53 CDT

Result: repo firmware flash and safe-state boot smoke passed on the connected Ultra 205.

The connected Bitaxe at `/dev/cu.usbmodem1101` was intentionally flashed from the official ESP-Miner Ultra 205 factory image back to the repo's Phase 1 Rust firmware. This is only an ESP32-S3 flash/boot/safe-state smoke for the physical Ultra 205. It is not Ultra 205 parity evidence because the current repo command surface and firmware identity are still hard-coded for `board=601`, Gamma 601, and BM1370.

## Board Caveats

- Physical board: Ultra 205 with BM1366 ASIC.
- Repo command used: `board=601`, because `tools/flash` currently rejects `board=205`.
- Firmware identity log: `board=Gamma 601 asic=BM1370`, which is incorrect for this physical board.
- Safety reason this run was acceptable: the Phase 1 firmware logs `mining=disabled`, `asic_work_submission=disabled`, and `hardware_control=disabled`; it does not intentionally drive ASIC, fan, voltage, thermal, power, or mining behavior.

## Port And Device

`espflash list-ports` identified:

```text
/dev/cu.usbmodem1101   1001:303A  Espressif  USB JTAG/serial debug unit
/dev/tty.usbmodem1101  1001:303A  Espressif  USB JTAG/serial debug unit
```

`lsof /dev/cu.usbmodem1101 /dev/tty.usbmodem1101` returned no holders.

`espflash board-info --port /dev/cu.usbmodem1101` reported:

```text
Chip type:         esp32s3 (revision v0.2)
Crystal frequency: 40 MHz
Flash size:        16MB
Features:          WiFi, BLE, Embedded Flash
MAC address:       f0:f5:bd:4a:ab:cc
Secure Boot: Disabled
Flash Encryption: Disabled
```

## Firmware Artifacts

Repository HEAD:

```text
9a833407a57f
```

Package manifest:

```text
bazel-bin/firmware/bitaxe/bitaxe-gamma601-package.json
```

Firmware ELF:

```text
bazel-bin/firmware/bitaxe/bitaxe-firmware.elf
sha256 cb354590949bc81c8e4c83420295bd38587c11eab4ae0d38b4bba266a27bf624
```

Gamma 601 package ELF:

```text
bazel-bin/firmware/bitaxe/bitaxe-gamma601.elf
sha256 cb354590949bc81c8e4c83420295bd38587c11eab4ae0d38b4bba266a27bf624
```

Gamma 601 factory package image:

```text
bazel-bin/firmware/bitaxe/bitaxe-gamma601-factory.bin
sha256 9a452765d555df6b6018fde02bd866b29dadc2f7733ee542f6c18d7acd8b5ae0
```

Manifest identity:

```text
"board": "601"
"device_model": "Gamma 601"
"asic": "BM1370"
"firmware_commit": "9a833407a57f9f89220beb8eed3f576976d94558"
"reference_commit": "c1915b0a63bfabebdb95a515cedfee05146c1d50"
"esp_idf_version": "v5.5.4"
"rust_target": "xtensa-esp32s3-espidf"
```

## Commands

Pre-flash checks:

```bash
espflash list-ports
lsof /dev/cu.usbmodem1101 /dev/tty.usbmodem1101
espflash board-info --port /dev/cu.usbmodem1101
git rev-parse --short=12 HEAD
```

Build and package:

```bash
just build
just package
```

Dry-run command rendering:

```bash
just monitor dry-run=true port=/dev/cu.usbmodem1101
just flash dry-run=true board=601 port=/dev/cu.usbmodem1101 \
  image=/Users/peterryszkiewicz/Repos/bitaxe-esp-miner/bazel-bin/firmware/bitaxe/bitaxe-firmware.elf
```

Actual flash and monitor:

```bash
just flash-monitor board=601 port=/dev/cu.usbmodem1101 \
  image=/Users/peterryszkiewicz/Repos/bitaxe-esp-miner/bazel-bin/firmware/bitaxe/bitaxe-firmware.elf
```

Then `CTRL+R` was sent in the attached monitor to capture a clean boot log, and `CTRL+C` was sent after the expected lines were captured.

## Command Results

| Command | Outcome | Notes |
| --- | --- | --- |
| `espflash list-ports` | passed | Found `/dev/cu.usbmodem1101` and `/dev/tty.usbmodem1101`. |
| `lsof /dev/cu.usbmodem1101 /dev/tty.usbmodem1101` | passed | No process held the serial port. |
| `espflash board-info --port /dev/cu.usbmodem1101` | passed | Confirmed ESP32-S3, 16 MB flash, MAC `f0:f5:bd:4a:ab:cc`, secure boot disabled, flash encryption disabled. |
| `just build` | passed | Built `bazel-bin/firmware/bitaxe/bitaxe-firmware.elf`. |
| `just package` | passed | Produced `bitaxe-gamma601.elf`, `bitaxe-gamma601-factory.bin`, and `bitaxe-gamma601-package.json`. |
| `just monitor dry-run=true port=/dev/cu.usbmodem1101` | passed | Rendered `espflash monitor --port /dev/cu.usbmodem1101`. |
| `just flash dry-run=true board=601 port=/dev/cu.usbmodem1101 image=/Users/peterryszkiewicz/Repos/bitaxe-esp-miner/bazel-bin/firmware/bitaxe/bitaxe-firmware.elf` | passed | Rendered the exact `espflash flash` command. |
| `just flash-monitor board=601 port=/dev/cu.usbmodem1101 image=/Users/peterryszkiewicz/Repos/bitaxe-esp-miner/bazel-bin/firmware/bitaxe/bitaxe-firmware.elf` | passed | Flash wrote and verified all segments, then monitor attached. |
| `CTRL+R` in monitor | passed | Captured clean boot log. |

Flash result excerpt:

```text
Chip type:         esp32s3 (revision v0.2)
Crystal frequency: 40 MHz
Flash size:        16MB
Features:          WiFi, BLE, Embedded Flash
MAC address:       f0:f5:bd:4a:ab:cc
App/part. size:    447,872/16,384,000 bytes, 2.73%
0x0      Verifying... OK!
0x8000   Verifying... OK!
0x10000  Verifying... OK!
monitor_command: espflash monitor --port /dev/cu.usbmodem1101
```

## Boot Evidence

Captured after `CTRL+R` while `espflash monitor --port /dev/cu.usbmodem1101` was attached through the repo `flash-monitor` command.

```text
ESP-ROM:esp32s3-20210327
Build:Mar 27 2021
rst:0x15 (USB_UART_CHIP_RESET),boot:0x28 (SPI_FAST_FLASH_BOOT)
I (27) boot: ESP-IDF v5.4.1-426-g3ad36321ea 2nd stage bootloader
I (29) boot: chip revision: v0.2
I (43) boot.esp32s3: SPI Flash Size : 16MB
I (236) app_init: Application information:
I (240) app_init: Project name:     libespidf
I (244) app_init: App version:      9a83340
I (248) app_init: Compile time:     Jun 25 2026 22:21:38
I (257) app_init: ESP-IDF:          v5.5.4
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

## Conclusion

The repo's current Phase 1 Rust firmware can be built, packaged, flashed, and booted on the connected Ultra 205 as a safe-state ESP32-S3 smoke test. The test confirms the flash workflow and boot log path, but not Ultra 205 board parity. Actual Ultra 205 support still requires `board=205`, Ultra/BM1366 identity, config defaults, and hardware-control evidence before any 205-specific rows should be marked verified.
