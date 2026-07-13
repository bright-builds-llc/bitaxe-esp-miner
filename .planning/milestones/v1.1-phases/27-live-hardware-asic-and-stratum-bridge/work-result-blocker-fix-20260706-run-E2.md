# Work-Result Blocker Fix Run E2 (W9 post-max-baud delay)

Date: 2026-07-06  
Board: Ultra 205  
Port: `/dev/cu.usbmodem1101`  
Investigation: `post_max_baud_delay_2000`  
Image: `bazel-bin/firmware/bitaxe/bitaxe-ultra205-factory.bin`

## Command

```bash
./scripts/phase27-live-hardware-bridge-package.sh --investigation post_max_baud_delay_2000
just flash-monitor board=205 port=/dev/cu.usbmodem1101 \
  image=bazel-bin/firmware/bitaxe/bitaxe-ultra205-factory.bin \
  evidence-dir=.planning/phases/27-live-hardware-asic-and-stratum-bridge/work-result-blocker-fix-20260706-run-E2 \
  capture-timeout-seconds=120
```

## Pass criterion

Any post-work `asic_uart_trace=rx_chunk` after diagnostic work @ 1M.

## Observations

| Marker | Result |
| --- | --- |
| REG28 + host 1M | **PASS** |
| ~2000ms after `clear_rx` | **PASS** — init gap ~2020ms before next action |
| Chip detect RX @ 115200 | **PASS** |
| Post-diagnostic RX @ 1M | **FAIL** — `bm1366_diagnostic_result=timeout` |

## Conclusion

W9 stabilization delay is implemented and visible in init timing, but does not restore post-work UART RX.

Evidence: `flash-monitor.log` in this directory.
