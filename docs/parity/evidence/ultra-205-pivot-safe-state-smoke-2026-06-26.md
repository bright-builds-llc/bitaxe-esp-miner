# Ultra 205 Pivot Safe-State Smoke Report

## Summary

Date: 2026-06-26 08:37:28 CDT

Result: post-pivot Ultra 205 safe-state flash and boot smoke passed on the connected Bitaxe at `/dev/cu.usbmodem1101`.

This run validates the initial Ultra 205/BM1366 parity pivot for identity, packaging, flash workflow, and safe boot logging. It does not validate BM1366 initialization, mining, voltage, fan, thermal, power, display, Wi-Fi, API, OTA, or NVS runtime behavior.

## Board And Port

Physical board: Bitaxe Ultra 205 with BM1366 ASIC.

Serial port:

```text
/dev/cu.usbmodem1101   1001:303A  Espressif  USB JTAG/serial debug unit
/dev/tty.usbmodem1101  1001:303A  Espressif  USB JTAG/serial debug unit
```

`lsof /dev/cu.usbmodem1101` returned no port holders before flashing.

Board info:

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
9a833407a57f9f89220beb8eed3f576976d94558
```

The working tree contained the uncommitted Ultra 205 pivot changes during this run; the firmware commit log line records the repository HEAD, while the boot identity proves the uncommitted Ultra 205 image was the flashed artifact.

Package manifest:

```text
bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json
sha256 4334f4afd4f7ec143021a56f8457f9c0b97b708887e7ce14bb59ef8b1dbd9581
```

Firmware ELF:

```text
bazel-bin/firmware/bitaxe/bitaxe-ultra205.elf
sha256 8b2a7d4370c48a67a2924106dbd6c67df15dd0fb4dd47c331fe9d1d18066c490
```

Factory image:

```text
bazel-bin/firmware/bitaxe/bitaxe-ultra205-factory.bin
sha256 28af0fdce20ce9be1e25c0440a3849abbaf5f13b12e76014075f185f86beb663
```

Manifest identity:

```text
"board": "205"
"device_model": "Ultra 205"
"asic": "BM1366"
"firmware_commit": "9a833407a57f9f89220beb8eed3f576976d94558"
"reference_commit": "c1915b0a63bfabebdb95a515cedfee05146c1d50"
"esp_idf_version": "v5.5.4"
"rust_target": "xtensa-esp32s3-espidf"
"default_flash_image": "bitaxe-ultra205.elf"
```

## Command Report

| Command | Outcome | Notes |
| --- | --- | --- |
| `node "$HOME/.codex/get-shit-done/bin/gsd-tools.cjs" init quick "Pivot V1 parity target from Gamma 601 to Ultra 205 safe-state boot/flash"` | passed | Created quick task `260626-bnt`. |
| `cargo fmt --all` | passed | Formatted Rust workspace. |
| `mdformat --check .planning/ROADMAP.md .planning/STATE.md` | failed | Pre-existing/new Markdown formatting drift was found before formatting. |
| `mdformat .planning/ROADMAP.md .planning/STATE.md` | passed | Rewrote the two planning files. |
| `cargo clippy --all-targets --all-features -- -D warnings` | failed initially | Plain host Cargo tried to build `esp-idf-sys` for `aarch64-apple-darwin`; firmware Cargo checks require the ESP target wrapper. |
| `cargo clippy --workspace --exclude bitaxe-firmware --all-targets --all-features -- -D warnings` | passed | Host-side crates and tools linted cleanly. |
| `cargo build --workspace --exclude bitaxe-firmware --all-targets --all-features` | passed | Host-side crates and tools built cleanly. |
| `cargo test --workspace --exclude bitaxe-firmware --all-features` | passed | Host-side unit tests passed, including Ultra 205 board, config, manifest, CLI parsing, and deferred `board=601` coverage. |
| Add Cargo workspace `default-members` for host-checkable crates/tools | passed | Root Cargo pre-commit commands no longer try to build the ESP-IDF firmware crate for the macOS host target; firmware remains a workspace member and is verified through `just build`. |
| `cargo clippy --all-targets --all-features -- -D warnings` | passed | Exact repo pre-commit lint command passed after `default-members` was set. |
| `cargo build --all-targets --all-features` | passed | Exact repo pre-commit build command passed after `default-members` was set. |
| `cargo test --all-features` | passed | Exact repo pre-commit test command passed after `default-members` was set. |
| `cargo fmt --all -- --check` | passed | Rust formatting check passed after edits. |
| `just verify-reference` | passed | Reported `reference clean: c1915b0a63bfabebdb95a515cedfee05146c1d50`. |
| `just build` | passed | Built the ESP-IDF firmware target through Bazel. |
| `just test` | passed | Ran all Bazel tests and firmware/package checks. |
| `just package` | passed | Produced `bitaxe-ultra205.elf`, `bitaxe-ultra205-factory.bin`, and `bitaxe-ultra205-package.json`. |
| `just parity` | failed during checklist tightening | The guard rejected `CFG-001` as `verified` with `unit` evidence because that row includes the voltage default. The row was lowered to `implemented`/`unit` because hardware use of frequency and voltage remains safety-critical and unverified. |
| `just parity` | passed | Final parity checklist validation reported no validation errors. |
| `just flash dry-run=true board=205 port=/dev/cu.usbmodem1101` | passed | Rendered `espflash flash --chip esp32s3 --port /dev/cu.usbmodem1101 .../bitaxe-ultra205.elf`. |
| `just monitor dry-run=true port=/dev/cu.usbmodem1101` | passed | Rendered `espflash monitor --port /dev/cu.usbmodem1101`. |
| `just flash-monitor dry-run=true board=205 port=/dev/cu.usbmodem1101` | passed | Rendered flash and monitor commands with the Ultra 205 package. |
| `just flash dry-run=true board=601 port=/dev/cu.usbmodem1101` | failed as expected | Rejected deferred Gamma 601 with `Phase 1 supports board=205 only`. |
| `espflash list-ports` | passed | Found `/dev/cu.usbmodem1101` and `/dev/tty.usbmodem1101`. |
| `lsof /dev/cu.usbmodem1101` | passed | No process held the serial port. |
| `espflash board-info --port /dev/cu.usbmodem1101` | passed | Confirmed ESP32-S3, 16 MB flash, MAC `f0:f5:bd:4a:ab:cc`, secure boot disabled, and flash encryption disabled. |
| `just flash-monitor board=205 port=/dev/cu.usbmodem1101` | passed | Flashed the Ultra 205 package, attached monitor, then captured boot after `CTRL+R`. |
| `CTRL+R` in monitor | passed | Reset the chip to capture a clean boot log. |
| `CTRL+C` in monitor | passed | Stopped monitor after required lines were captured. |

## Dry-Run Evidence

Flash dry-run rendered:

```text
manifest: .../firmware/bitaxe/bitaxe-ultra205-package.json
default_flash_image: .../firmware/bitaxe/bitaxe-ultra205.elf
flash_command: espflash flash --chip esp32s3 --port /dev/cu.usbmodem1101 .../firmware/bitaxe/bitaxe-ultra205.elf
```

Monitor dry-run rendered:

```text
monitor_command: espflash monitor --port /dev/cu.usbmodem1101
```

Deferred board check rendered:

```text
Error: error: invalid value '601' for '--board <BOARD>': board 601 is deferred after the Ultra 205 pivot; Phase 1 supports board=205 only
```

## Flash Evidence

Actual flash command rendered:

```text
flash_command: espflash flash --chip esp32s3 --port /dev/cu.usbmodem1101 .../firmware/bitaxe/bitaxe-ultra205.elf
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

## Boot Evidence

Captured after sending `CTRL+R` while `espflash monitor --port /dev/cu.usbmodem1101` was attached through `just flash-monitor board=205`.

```text
ESP-ROM:esp32s3-20210327
Build:Mar 27 2021
rst:0x15 (USB_UART_CHIP_RESET),boot:0x28 (SPI_FAST_FLASH_BOOT)
I (27) boot: ESP-IDF v5.4.1-426-g3ad36321ea 2nd stage bootloader
I (29) boot: chip revision: v0.2
I (43) boot.esp32s3: SPI Flash Size : 16MB
I (240) app_init: Project name:     libespidf
I (244) app_init: App version:      9a83340
I (257) app_init: ESP-IDF:          v5.5.4
I (329) bitaxe_firmware: bitaxe-rust boot: board=Ultra 205 asic=BM1366
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

The connected Ultra 205 successfully flashed and booted the post-pivot Rust safe-state firmware. The live boot log proves Ultra 205/BM1366 identity plus disabled mining, ASIC work submission, and hardware control. Safety-critical and mining surfaces remain unverified and intentionally disabled.
