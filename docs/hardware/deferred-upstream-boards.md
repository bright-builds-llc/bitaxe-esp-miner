# Deferred upstream boards

Upstream `v2.15.4` adds two boards that we do not support. The owner decided on
2026-10-30 to document them and defer the work. Nothing here is a parity claim.

`task-parity-other-boards-scope` owns any later decision to take them up. Each
board then needs its own contract and hardware evidence, as ADR-0014 requires,
and registered identity, as ADR-0041 requires.

All values come from upstream's `main/device_config.h` and the `config-1201.cvs`
and `config-1300.cvs` files at `v2.15.4`, added by `3cfc082` and `cc667af`. None
were checked on hardware.

## The boards

| | Naja Duo | Gamma Hex |
| --- | --- | --- |
| Board version | 1201 | 1300 |
| Device model | `NajaDuo` | `GammaHex` |
| ASIC | BM1372/BM1373 (`BM1373` in config, chip id 1372) | BM1370 |
| ASIC count | 2 | 6 |
| Default frequency | 327 MHz | 690 MHz |
| Frequency options | 327, 350, 375, 380, 400, 410 MHz | 400, 490, 525, 550, 600, 625, 690 MHz |
| Default core voltage | 1000 mV | 1200 mV |
| Input voltage | 12 V | 12 V |
| Maximum power | 60 W | 180 W |
| Voltage domains | 2 | 2 |
| Power controller | TPS546, `TPS546_CONFIG_NAJA_DUO` | TPS546, `TPS546_CONFIG_GAMMA_HEX` |
| Fan and thermal | EMC2103 | EMC2103, direct PWM |
| Pins | `BITAXE_COLOR_PINS` | `BITAXE_COLOR_PINS` |
| ASIC enable | Active high | Active high |

Both are the "Bitaxe Color" boards from `cc667af`, which also adds an ST7789
320x170 display path over I80.

## What supporting them would need

- **BM1372/BM1373:** a new protocol core, since we have none. Upstream's driver
  is `components/asic/bm1373.c`, with chip detection through an ID alias.
- **Gamma Hex:** six BM1370 chips, so multi-chip coverage our single-chip 601 does
  not provide, and the `BM1370_init` changes in
  `task-upstream-v2154-asic-power-sequencing`.
- **Both:**
  - EMC2103 support, multiple voltage domains and the Color pin map;
  - the ST7789 display;
  - their TPS546 configurations, which need a safety review before any use;
  - their own seeded defaults, power and thermal limits, and self-test targets.

## Until then

- Do not build, flash or claim either board. The board identity guard in ADR-0041
  stays the boundary.
- When the pin moves past these commits, CFG-006 must list both boards as
  non-claims rather than imply coverage.
- Upstream's `master` has moved its config files to `configs/`, so look there if
  reading a later commit.
