# Work-Result Blocker Fix Run E1 (Bridge + pool work)

Date: 2026-07-06  
Board: Ultra 205  
Port: `/dev/cu.usbmodem1101`  
Investigation: `initialized_no_mining_gate` (W5 bootstrap for bridge path)  
Evidence root: [`bridge-blocker-fix-20260706/`](bridge-blocker-fix-20260706/)

## Pass criterion

Any post-dispatch `asic_uart_trace=rx_chunk` after bridge production work.

## Observations

| Marker | Result |
| --- | --- |
| B1 thermal gate | **PASS** — bounded thermal; bridge not blocked on `thermal_reading_invalid` |
| B2 pool consumed | **PASS** — `pool_settings_consumed_by_runtime=true` |
| Stratum bridge | **PASS** — `connecting` → `subscribed` → `authorized` → `active` |
| Production work TX | **PASS** — 88-byte pool job frame @ 1M |
| Post-dispatch RX | **FAIL** — `production_result_timeout`; no post-work `rx_chunk` |

## Conclusion

Bridge blockers B1/B2 are resolved. UART post-work silence (B3) persists on production pool-derived work.
