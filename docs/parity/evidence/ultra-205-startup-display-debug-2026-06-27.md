# Ultra 205 Startup Display Debug Evidence

## Summary

Date: 2026-06-27 08:08:48 CDT

Result: serial hardware smoke passed on the connected Ultra 205 at `/dev/cu.usbmodem1101`. The Rust firmware rendered the startup-only SSD1306 128x32 debug screen before the safe ASIC boot gate and logged `display_status=startup_text_rendered`.

Visual display presence was confirmed by the user after flashing. Codex cannot directly inspect the physical OLED, so the evidence remains user-observed rather than camera-captured.

This run covers only the minimal startup debug text path. It does not validate upstream LVGL display parity, carousel behavior, screen tasks, display config from NVS, rotation settings, timeout handling, mining, voltage, fan, thermal, power, Wi-Fi, API, OTA, or NVS runtime behavior.

## Expected OLED Text

```text
Bitaxe Rust
Ultra 205 BM1366
SAFE no mining
fw <12-char commit>
```

If the firmware commit is unavailable, the fourth line is:

```text
fw Unavailable
```

## Firmware And Reference

Repository HEAD at implementation time:

```text
496138949f3f
```

Reference commit:

```text
c1915b0a63bfabebdb95a515cedfee05146c1d50
```

## Command Evidence

| Command | Outcome | Notes |
| --- | --- | --- |
| `cargo test -p bitaxe-core startup_debug --all-features` | passed | Proved exact four startup debug lines, unavailable fallback, 12-character commit truncation, and 128x32 geometry fit. |
| `cargo check -p bitaxe-firmware --target xtensa-esp32s3-espidf` | passed | Proved the firmware SSD1306 display adapter and single-ownership peripheral boot wiring compile for the ESP32-S3 target. |
| `cargo fmt --all -- --check` | passed | Rust formatting check passed. |
| `cargo clippy --all-targets --all-features -- -D warnings` | passed | Host/default workspace lint passed with warnings denied. |
| `cargo build --all-targets --all-features` | passed | Host/default workspace build passed. |
| `cargo test --all-features` | passed | Host/default workspace tests passed, including the startup debug tests. |
| `just build` | passed | Built `//firmware/bitaxe:firmware`. |
| `just test` | passed | Ran Bazel tests and package checks. |
| `just package` | passed | Produced `bitaxe-ultra205.elf`, `bitaxe-ultra205-factory.bin`, and `bitaxe-ultra205-package.json`. |
| `just verify-reference` | passed | Reported `reference clean: c1915b0a63bfabebdb95a515cedfee05146c1d50`. |
| `just parity` | passed | Reported `validation_errors: none`. |
| `espflash list-ports` | passed | Found `/dev/cu.usbmodem1101` and `/dev/tty.usbmodem1101` as ESP32-S3 USB JTAG serial ports. |
| `lsof /dev/cu.usbmodem1101` | passed | No process held the serial port. |
| `just flash-monitor board=205 port=/dev/cu.usbmodem1101` | passed | Flashed the Ultra 205 firmware, attached monitor, reset the chip with `CTRL+R`, captured boot logs, waited through a 60-second serial dwell window, and exited with `CTRL+C`. |

## Serial Log Acceptance

Required log line after flashing:

```text
display_status=startup_text_rendered model=SSD1306 size=128x32 address=0x3c
```

If the display is missing or unavailable, the firmware must keep safe boot running and log:

```text
display_status=unavailable reason=startup_text_render_failed ...
```

## Hardware Smoke

Status: serial smoke passed; user confirmed the OLED text is visible.

Command:

```text
just flash-monitor board=205 port=/dev/cu.usbmodem1101
```

Port:

```text
/dev/cu.usbmodem1101   1001:303A  Espressif  USB JTAG/serial debug unit
/dev/tty.usbmodem1101  1001:303A  Espressif  USB JTAG/serial debug unit
```

Flash excerpt:

```text
flash_command: espflash flash --chip esp32s3 --port /dev/cu.usbmodem1101 .../bitaxe-ultra205.elf
Chip type:         esp32s3 (revision v0.2)
Crystal frequency: 40 MHz
Flash size:        16MB
Features:          WiFi, BLE, Embedded Flash
MAC address:       f0:f5:bd:4a:ab:cc
App/part. size:    527,024/16,384,000 bytes, 3.22%
0x0      Skipped! (checksum matches)
0x8000   Skipped! (checksum matches)
0x10000  Verifying... OK!
monitor_command: espflash monitor --port /dev/cu.usbmodem1101
```

Boot excerpt captured after sending `CTRL+R`:

```text
I (357) bitaxe_firmware: bitaxe-rust boot: board=Ultra 205 asic=BM1366
I (357) bitaxe_firmware: safe_state: mining=disabled asic_work_submission=disabled hardware_control=disabled
I (397) bitaxe_firmware::display_adapter: display_status=startup_text_rendered model=SSD1306 size=128x32 address=0x3c
I (397) bitaxe_firmware::asic_adapter::status: asic_status=preflight_missing reason=hardware_evidence_ack_missing initialized=false mining=disabled work_submission=disabled
I (407) bitaxe_firmware: reset_reason=11
I (407) bitaxe_firmware: partition=factory
I (417) bitaxe_firmware: psram_status=unavailable
I (417) bitaxe_firmware: firmware_commit=496138949f3f
I (427) bitaxe_firmware: reference_commit=c1915b0a63bfabebdb95a515cedfee05146c1d50
I (427) bitaxe_firmware: esp_idf_version=v5.5.4
I (437) bitaxe_firmware: rust_target=xtensa-esp32s3-espidf
I (437) main_task: Returned from app_main()
```

Serial dwell: monitor remained attached for roughly 60 seconds after reset without additional failure logs.

Visual observation: user confirmed the OLED shows the startup debug text. Codex did not directly inspect the physical display. Expected text for this flashed image:

```text
Bitaxe Rust
Ultra 205 BM1366
SAFE no mining
fw 496138949f3f
```

The monitor remained attached for roughly 60 seconds after reset without additional failure logs.

## Conclusion

The startup debug display path is implemented below full display parity. Checklist rows `IO-001`, `UI-001`, and `UI-002` remain `in-progress` because this is only startup debug text, not the complete upstream display and screen flow.
