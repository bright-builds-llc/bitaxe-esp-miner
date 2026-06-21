---
status: partial
phase: 01-foundation-and-gamma-601-boot-log
source: [01-VERIFICATION.md]
started: 2026-06-21T04:34:21Z
updated: 2026-06-21T04:34:21Z
---

# Phase 01 Human UAT

## Current Test

Gamma 601 flash-monitor hardware smoke is awaiting a connected board.

## Tests

### 1. Gamma 601 flash-monitor hardware smoke

expected: Captured log contains `bitaxe-rust boot: board=Gamma 601 asic=BM1370`, `safe_state: mining=disabled asic_work_submission=disabled hardware_control=disabled`, `reset_reason=`, `partition=` or `image_partition=`, `platform_status=` or `psram_status=`, `firmware_commit=` with an actual commit or `firmware_commit=Unavailable`, and `reference_commit=c1915b0a63bfabebdb95a515cedfee05146c1d50`.

command:

```bash
just flash-monitor board=601 port=<port> evidence-dir=docs/parity/evidence/phase-01-gamma-601-boot-log
```

result: pending

## Summary

total: 1
passed: 0
issues: 0
pending: 1
skipped: 0
blocked: 0

## Gaps

No implementation gaps are recorded. The pending item is live hardware evidence only.
