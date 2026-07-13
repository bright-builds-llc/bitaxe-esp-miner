# Work-Result Blocker Fix Run E3 (Fail-closed baseline)

Date: 2026-07-06  
Board: Ultra 205  
Port: `/dev/cu.usbmodem1101`  
Investigation: (default — no W5 bootstrap on diagnostic timeout)  
Image: `bazel-bin/firmware/bitaxe/bitaxe-ultra205-factory.bin`

## Command

```bash
./scripts/phase27-live-hardware-bridge-package.sh
just flash-monitor board=205 port=/dev/cu.usbmodem1101 \
  image=bazel-bin/firmware/bitaxe/bitaxe-ultra205-factory.bin \
  evidence-dir=.planning/phases/27-live-hardware-asic-and-stratum-bridge/work-result-blocker-fix-20260706-run-E3 \
  capture-timeout-seconds=120
```

## Pass criterion

Fail-closed on diagnostic timeout without retaining production UART (`work_result_diagnostic_timeout`).

## Observations

| Marker | Result |
| --- | --- |
| Mining-ready init + diagnostic TX | **PASS** |
| Diagnostic read | **FAIL** (expected) — timeout |
| Final status | **PASS baseline** — `fail_closed reason=work_result_diagnostic_timeout`, `initialized=false` |

## Conclusion

Default fail-closed gate remains intact without `initialized_no_mining_gate`.

Evidence: `flash-monitor.log` in this directory.
