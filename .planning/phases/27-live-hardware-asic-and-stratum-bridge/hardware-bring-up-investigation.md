# Phase 27 Hardware Bring-Up Investigation

## Boot log markers (retry 4)

- `safety_fan_effect=write` — absent (fan suppressed)
- `safety_voltage_effect=write` — absent (voltage suppressed)
- `asic_status=chip_detect_only` → `chip_detect_adapter_error` (partial UART 9/11)
- `asic_production_status=initialized` — absent

## Upstream bring-up order (pinned `c1915b0a63bfabebdb95a515cedfee05146c1d50`)

1. `i2c_bitaxe_init()` — I2C0 GPIO47/48 @ 400 kHz
2. `asic_hold_reset_low()` — GPIO1 reset low
3. NVS / device_config
4. `VCORE_init()` — DS4432U + INA260; GPIO10 enable active-low when plugged in
5. `Thermal_init()` / EMC2101 — fan driver enabled
6. Fan controller task — startup 70% duty
7. `VCORE_set_voltage(1.2V)` before `asic_initialize()`

## Ultra 205 I2C map (shared I2C0)

| Device | Address | GPIO |
|--------|---------|------|
| SSD1306 | 0x3C | SDA 47, SCL 48 |
| INA260 | 0x40 | same bus |
| DS4432U | 0x48 | same bus |
| EMC2101 | 0x4C | same bus |

GPIO10 ASIC enable: active-low (0 = power on). Default voltage: 1200 mV. Startup fan: 70%.

## Phase 27 integration decision

- Defer display init when Phase 27 mode is active (avoid I2C0 contention during safety bring-up).
- Run safety bring-up inside the ASIC boot gate with a shared reset driver before BM1366 UART chip-detect.
- Hold reset low during voltage/fan enable, then pulse reset before chip-detect (upstream order).

## Retry4 hardware result (2026-07-05)

Safety bring-up markers observed on Ultra 205 (`/dev/cu.usbmodem1101`):

- `phase27_safety_bring_up=complete`
- `safety_power_status=observed`, `safety_thermal_status=observed`, `safety_fan_status=startup_duty`
- `asic_enable_status=active`, `asic_reset_status=post_bring_up_pulse`

Chip-detect still fails with partial UART read 9/11 after safety bring-up. Share proof remains `blocked_safe_prerequisite`.
