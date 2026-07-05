# Work-Result Deep Dive Run A (W7 default)

Date: 2026-07-05  
Board: Ultra 205  
Port: `/dev/cu.usbmodem1101`  
Investigation: (default — W7 reg28 prelude + host 1M)  
Source commit: `255d495d7ec3` (approx at flash time)  
Image: `bazel-bin/firmware/bitaxe/bitaxe-ultra205-factory.bin`

## Command

```bash
just flash-monitor board=205 port=/dev/cu.usbmodem1101 \
  image=bazel-bin/firmware/bitaxe/bitaxe-ultra205-factory.bin \
  evidence-dir=.planning/phases/27-live-hardware-asic-and-stratum-bridge/work-result-deep-dive-20260705-run-A \
  capture-timeout-seconds=120
```

## Pass criterion

Any `asic_uart_trace=rx_chunk` after diagnostic work TX at 1M, or `bm1366_diagnostic_result=parsed`.

## Observations

| Marker | Result |
| --- | --- |
| Chip detect RX @ 115200 | **PASS** — `rx_chunk hex=aa 55 13 66...` |
| REG28 max-baud frame @ 115200 | **PASS** — `tx ... 00 28 11 30 02 00 03` before `use_max_baud baud=1000000` |
| Mining-ready init (23 actions) | **PASS** |
| Diagnostic work TX @ 1M (88 bytes) | **PASS** |
| Post-work RX during 10s read | **FAIL** — zero `rx_chunk`; `bm1366_diagnostic_result=timeout` |
| W5 bootstrap | **NOT triggered** (default off after Wave 4) → `fail_closed reason=work_result_diagnostic_timeout` |

## Conclusion

W7 reg28 + host 1M ordering is correctly implemented and visible on the wire, but **does not restore UART RX** during the bounded diagnostic read. W8 frequency ramp and 115200 control (run C) remain warranted.

Evidence: `flash-monitor.log`, `flash-command-evidence.json` in this directory.
