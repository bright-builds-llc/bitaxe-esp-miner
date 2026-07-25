# Quick Task 260627-b0q Summary: Display startup debug text on Ultra 205 OLED

Date: 2026-06-27

## Result

Implemented a startup-only OLED debug screen for the Ultra 205/BM1366 bring-up path. Firmware now builds the exact four-line startup text in `bitaxe-core`, renders it once on the SSD1306 128x32 display over I2C0 SDA GPIO47/SCL GPIO48 at address `0x3c`, flushes the display buffer, and continues the existing fail-closed ASIC boot gate.

Display initialization or render failure is non-fatal and logs `display_status=unavailable ...`.

## Files Changed

- `crates/bitaxe-core/src/lib.rs`
- `firmware/bitaxe/src/display_adapter.rs`
- `firmware/bitaxe/src/main.rs`
- `firmware/bitaxe/src/asic_adapter.rs`
- `Cargo.toml`
- `Cargo.lock`
- `firmware/bitaxe/Cargo.toml`
- `docs/parity/checklist.md`
- `docs/parity/evidence/ultra-205-startup-display-debug-2026-06-27.md`

## Verification

| Command | Result |
| --- | --- |
| `cargo test -p bitaxe-core startup_debug --all-features` | passed |
| `cargo check -p bitaxe-firmware --target xtensa-esp32s3-espidf` | passed |
| `cargo fmt --all -- --check` | passed |
| `cargo clippy --all-targets --all-features -- -D warnings` | passed |
| `cargo build --all-targets --all-features` | passed |
| `cargo test --all-features` | passed |
| `just build` | passed |
| `just test` | passed |
| `just package` | passed |
| `just verify-reference` | passed |
| `just parity` | passed |
| `just flash-monitor board=205 port=/dev/cu.usbmodem1101` | passed serial smoke |
| User visual confirmation | passed | User confirmed the OLED text is visible after flashing. |

## Hardware Evidence

Serial boot log contained:

```text
display_status=startup_text_rendered model=SSD1306 size=128x32 address=0x3c
```

The monitor stayed attached through the dwell window without display failure logs. The user confirmed the OLED text is visible after flashing.

## Residual Risk

This is intentionally not full upstream display parity. LVGL, screen carousel, display timeout, rotation/config from NVS, runtime screen updates, and shared I2C peripheral behavior remain future work.
