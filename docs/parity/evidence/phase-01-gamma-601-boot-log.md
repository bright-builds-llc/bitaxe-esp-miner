# Phase 01 Gamma 601 Boot/Log Evidence

## Summary

Date: 2026-06-25 America/Chicago

Result: no valid Gamma 601 hardware-smoke evidence is currently captured.

An ESP32-S3 Bitaxe at `/dev/cu.usbmodem1101` was flashed and monitored, but the user later corrected the physical board identity to Ultra 205. The detailed command report was moved to `docs/parity/evidence/phase-01-ultra-205-misidentified-board-smoke.md`.

## Board

Expected Phase 1 target: Gamma 601 with BM1370 ASIC.

Actual board used in the smoke run: Ultra 205 with BM1366 ASIC.

## Conclusion

Conclusion: missing valid Gamma 601 hardware-smoke evidence.

The captured boot log must not be used to verify Gamma 601 parity rows because the physical board was an Ultra 205. A real Gamma 601 run is still required before Gamma 601 boot/log rows can be marked `verified`.
