# Hardware Run — Work-Result Fix (2026-07-05)

## Target

- Board: Ultra 205
- Port: `/dev/cu.usbmodem1101`
- Image: Phase 27 factory `bitaxe-ultra205-factory.bin` (explicit `image=`, not `manifest=`)

## Commands

```bash
just detect-ultra205
./scripts/phase27-live-hardware-bridge-package.sh
just flash-monitor board=205 port=/dev/cu.usbmodem1101 \
  image=bazel-out/.../bitaxe-ultra205-factory.bin \
  evidence-dir=.planning/phases/27-live-hardware-asic-and-stratum-bridge/work-result-verify-20260705 \
  capture-timeout-seconds=360
```

## Trace run (pre-W5 bootstrap)

Evidence: `work-result-trace-20260705/flash-monitor.log`

- Chip detect: `asic_status=chip_detected chips=1`
- Mining-ready init: 21 actions, max baud 1M, `asic_status=initialized_no_mining`
- Diagnostic work TX at 1M baud, zero RX during 10s → `work_result_diagnostic_timeout`

## Verify run (post-W5 bootstrap)

Evidence: `work-result-verify-20260705/flash-monitor.log`

| Marker | Observed |
| --- | --- |
| `asic_status=chip_detected chips=1` | yes |
| `asic_status=initialized_no_mining` | yes |
| `bm1366_diagnostic_result=bounded_no_result bootstrap=initialized_no_mining` | yes |
| `asic_production_status=initialized` | yes |
| `asic_initialized_gate_missing` | **not observed** |
| Bridge | progresses to `phase27_pool_wait_timeout` (pool credentials not supplied) |

## Conclusion

W1+W2 fixes (mining-ready init + max baud) eliminate silent pre-init work dispatch but do not produce a diagnostic nonce within 10s on this hardware. W5 bootstrap gate unblocks `production_ready()` via `InitializedNoMining` + chip detected, allowing Phase 27 bridge to pass the ASIC init gate. End-to-end pool bridge remains blocked on pool wait (expected without local pool credentials).
