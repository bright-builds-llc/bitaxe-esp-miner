# Pivot To Ultra 205 BM1366 For First Parity

## Status

Accepted.

Supersedes [ADR-0007](0007-prioritize-gamma-601-bm1370-bring-up.md).

## Decision

The first hardware bring-up and V1 parity target is now Bitaxe Ultra 205 with BM1366 ASIC, matching upstream `reference/esp-miner/config-205.cvs`, the Ultra 205 entry in `reference/esp-miner/main/device_config.h`, and the BM1366 path in `reference/esp-miner/components/asic/bm1366.c`.

Gamma 601 with BM1370 remains in the project parity scope, but it is deferred until after the Ultra 205 path has evidence-backed boot, config, ASIC, mining, safety, OTA, and release behavior.

## Rationale

The available connected hardware is an Ultra 205. Recent hardware evidence restored the board to official ESP-Miner `v2.14.1`, confirmed `Device Model: Ultra`, `Board Version: 205`, and `ASIC: 1x BM1366`, then proved that the repo's safe-state firmware can be flashed and booted on that physical board.

Continuing to optimize the first V1 path for Gamma 601 would keep the project blocked on unavailable hardware and would encourage misleading evidence reuse. The Ultra 205 pivot lets implementation and hardware evidence move together while preserving the same device-user parity and safety standards.

## Consequences

- `board=205` is the default and only supported first-path board for package, flash, and monitor workflows.
- Firmware safe boot logs identify `Ultra 205` and `BM1366`.
- Phase 2 and later roadmap work should target Ultra 205 defaults, BM1366 protocol behavior, DS4432U/INA260 power telemetry, and board-specific hardware evidence.
- Gamma 601, BM1370, and TPS546 behavior remain deferred parity surfaces and must not inherit Ultra 205 verification.
- Safety-critical surfaces still require hardware smoke or regression evidence before `verified`.
