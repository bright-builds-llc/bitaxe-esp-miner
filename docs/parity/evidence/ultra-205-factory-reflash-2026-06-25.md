# Ultra 205 Factory Reflash Report

## Summary

Date: 2026-06-25 22:45:56 CDT

Result: Ultra 205 factory reflash succeeded.

The connected Bitaxe at `/dev/cu.usbmodem1101` was restored from the repo's Phase 1 Rust safe-state firmware to the official ESP-Miner Ultra 205 factory image.

## Firmware Image

Release: ESP-Miner `v2.14.1`

Asset:

```text
https://github.com/bitaxeorg/ESP-Miner/releases/download/v2.14.1/esp-miner-factory-205-v2.14.1.bin
```

Verified SHA-256:

```text
e04413321ca879b41c2f950ca64da1828dd69bfc85daa20085b158340b0ff48f
```

Local downloaded path:

```text
/tmp/bitaxe-ultra205-reflash/esp-miner-factory-205-v2.14.1.bin
```

## Commands

Confirm latest release asset:

```bash
curl -fsSL https://api.github.com/repos/bitaxeorg/ESP-Miner/releases/latest \
  | jq -r '.tag_name, .html_url, (.assets[] | select(.name=="esp-miner-factory-205-v2.14.1.bin") | [.name, .browser_download_url, .digest, (.size|tostring)] | @tsv)'
```

Download and verify:

```bash
mkdir -p /tmp/bitaxe-ultra205-reflash
curl -fL --retry 3 \
  -o /tmp/bitaxe-ultra205-reflash/esp-miner-factory-205-v2.14.1.bin \
  https://github.com/bitaxeorg/ESP-Miner/releases/download/v2.14.1/esp-miner-factory-205-v2.14.1.bin
shasum -a 256 /tmp/bitaxe-ultra205-reflash/esp-miner-factory-205-v2.14.1.bin
```

Pre-flash port checks:

```bash
espflash list-ports
espflash board-info --port /dev/cu.usbmodem1101
```

Factory flash:

```bash
espflash write-bin --chip esp32s3 --port /dev/cu.usbmodem1101 \
  0x0 /tmp/bitaxe-ultra205-reflash/esp-miner-factory-205-v2.14.1.bin
```

Post-flash monitor:

```bash
espflash monitor --port /dev/cu.usbmodem1101
```

Then `CTRL+R` was sent in the attached monitor to capture a clean boot log.

## Flash Result

Factory image write completed and verified:

```text
Chip type:         esp32s3 (revision v0.2)
Crystal frequency: 40 MHz
Flash size:        16MB
Features:          WiFi, BLE, Embedded Flash
MAC address:       f0:f5:bd:4a:ab:cc
0x0      Verifying... OK!
```

## Boot Evidence

Key post-flash boot log lines:

```text
I (915) app_init: Project name:     esp-miner
I (920) app_init: App version:      v2.14.1
I (936) app_init: ESP-IDF:          v5.5.3
I (1187) device_config: Device Model: Ultra
I (1187) device_config: Board Version: 205
I (1189) device_config: ASIC: 1x BM1366 (112 cores)
I (2044) vcore: Set ASIC voltage = 1.200V
I (2049) power_management: ASIC Frequency: 485 MHz, Expected hashrate: 433.59MH/s
I (2062) system: Firmware Version: v2.14.1
I (2063) system: AxeOS Version: v2.14.1
I (10521) asic_init: ASIC initialized successfully with 1 chip(s) (cold boot mode)
I (10529) create_jobs_task: ASIC Job Interval: 2000 ms
I (10535) create_jobs_task: ASIC Ready!
```

## Notes

- Chrome/Web Serial initially held `/dev/cu.usbmodem1101`, causing `espflash` open failures. Closing or disconnecting the Chrome serial session released the port.
- The factory image was written at address `0x0`.
- The device is no longer running the repo's Phase 1 Rust firmware.
