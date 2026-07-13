# Work-Result Deep Dive Run B (W7 + W8 frequency ramp)

Date: 2026-07-05  
Investigation: `frequency_ramp`  
Evidence: `flash-monitor.log`

## Pass criterion

Post-diagnostic `asic_uart_trace=rx_chunk` or `bm1366_diagnostic_result=parsed`.

## Result: **FAIL**

- Frequency ramp observed (~8s init before `use_max_baud baud=1000000`; many `SetFrequency` + delay steps).
- Chip detect RX @ 115200: **PASS**
- REG28 + host 1M: **PASS**
- Post-work diagnostic RX @ 1M: **FAIL** — `bm1366_diagnostic_result=timeout`, no post-work `rx_chunk`.

## Conclusion

W8 ramp alone does not restore diagnostic UART RX.
