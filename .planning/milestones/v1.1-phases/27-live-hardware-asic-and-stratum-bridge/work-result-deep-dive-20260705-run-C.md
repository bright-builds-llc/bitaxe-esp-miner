# Work-Result Deep Dive Run C (115200 control)

Date: 2026-07-05  
Investigation: `skip_max_baud` (no reg28 prelude, no host 1M switch)  
Evidence: `flash-monitor.log`

## Pass criterion

RX at 115200 after diagnostic work would implicate 1M baud mismatch.

## Result: **FAIL (baud mismatch ruled out as sole cause)**

- No `use_max_baud` or reg28 TX in log — UART stayed @ 115200.
- Chip detect RX @ 115200: **PASS**
- Diagnostic work dispatched (88 bytes @ 115200).
- Post-work RX: **FAIL** — still `bm1366_diagnostic_result=timeout`, no post-work `rx_chunk`.

## Conclusion

Silence is **not explained by host-only 1M mismatch alone**. Likely W11 synthetic diagnostic job and/or remaining init/work-path gaps.
