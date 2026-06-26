# Prioritize Gamma 601 BM1370 for Hardware Bring-Up

Status: Superseded by [ADR-0014](0014-pivot-to-ultra-205-bm1366-first-parity.md).

The first hardware bring-up target is Bitaxe Gamma 601 with BM1370 ASIC, matching upstream `reference/esp-miner/config-601.cvs` and the Gamma/BM1370 paths in `reference/esp-miner/main/device_config.h` and `reference/esp-miner/components/asic/bm1370.c`. Device-user parity still includes all upstream-supported Bitaxe boards and configs, but early `just flash`, smoke tests, and hardware acceptance should optimize for the user's available Gamma 601 device, with Bitaxe 205 treated as a secondary available target.
