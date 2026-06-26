______________________________________________________________________

## status: complete phase: 01-foundation-and-gamma-601-boot-log source: [01-VERIFICATION.md, ADR-0014, ultra-205-pivot-safe-state-smoke-2026-06-26.md] started: 2026-06-21T04:34:21Z updated: 2026-06-26T14:08:55Z

# Phase 01 Human UAT

## Current Test

[testing complete]

## Tests

### 1. Superseded Gamma 601 flash-monitor hardware smoke

expected: Captured log contains `bitaxe-rust boot: board=Gamma 601 asic=BM1370`, `safe_state: mining=disabled asic_work_submission=disabled hardware_control=disabled`, `reset_reason=`, `partition=` or `image_partition=`, `platform_status=` or `psram_status=`, `firmware_commit=` with an actual commit or `firmware_commit=Unavailable`, and `reference_commit=c1915b0a63bfabebdb95a515cedfee05146c1d50`.
result: skipped
reason: Superseded by ADR-0014. Gamma 601/BM1370 remains deferred and must not inherit Ultra 205 evidence.

### 2. Ultra 205 flash-monitor safe-state smoke

expected: `just flash-monitor board=205 port=/dev/cu.usbmodem1101` flashes the Ultra 205 package, monitor attaches, `CTRL+R` or RESET produces a boot log containing `bitaxe-rust boot: board=Ultra 205 asic=BM1366`, `safe_state: mining=disabled asic_work_submission=disabled hardware_control=disabled`, `reset_reason=`, `partition=`, `psram_status=`, `firmware_commit=`, `reference_commit=c1915b0a63bfabebdb95a515cedfee05146c1d50`, `esp_idf_version=v5.5.4`, and `rust_target=xtensa-esp32s3-espidf`.
result: pass
evidence: `docs/parity/evidence/ultra-205-pivot-safe-state-smoke-2026-06-26.md`
user_confirmation: "Fantastic, this all works!"

## Summary

total: 2
passed: 1
issues: 0
pending: 0
skipped: 1
blocked: 0

## Gaps

No implementation gaps are recorded. The original Gamma 601 hardware-smoke checkpoint was superseded by ADR-0014, and the Ultra 205 safe-state smoke passed from the user's perspective.
