# Quick Task 260626-bnt: Pivot V1 Parity Target To Ultra 205

## Goal

Pivot the initial V1 hardware/parity target from Gamma 601/BM1370 to Ultra 205/BM1366 while preserving completed Gamma history and existing Ultra 205 evidence.

## Tasks

1. Update Rust identity, package, and flash workflows so `board=205` is the supported safe-state path and `board=601` is deferred.
1. Add a superseding ADR and update active project, roadmap, state, milestone, and parity docs to describe Ultra 205 first parity.
1. Verify with Rust formatting/lint/build/tests, Just/Bazel checks, dry-run flash commands, parity validation, and hardware smoke if the connected Ultra 205 remains available.

## Constraints

- Do not enable ASIC init, mining, voltage, fan, thermal, or power control.
- Do not modify `reference/esp-miner`.
- Preserve existing uncommitted Ultra 205 evidence files.
