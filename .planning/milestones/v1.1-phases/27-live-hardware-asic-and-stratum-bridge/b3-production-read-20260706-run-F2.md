# B3 Production Read Run F2 (`clear_rx_before_production_work`)

Date: 2026-07-06  
Board: Ultra 205  
Port: `/dev/cu.usbmodem1101`  
Investigation: `clear_rx_before_production_work`  
Evidence root: [`b3-production-read-20260706-run-F2/`](b3-production-read-20260706-run-F2/)

## Config

Flash-monitor only (120s), not full bridge evidence wrapper. Package built with `clear_rx_before_production_work` investigation flag.

## Pass criterion

Same or improved post-dispatch UART proof vs F1 when run under bridge evidence.

## Observations

| Marker | Result |
| --- | --- |
| Boot diagnostic read | 10s timeout @ 12520ms |
| `fail_closed reason=work_result_diagnostic_timeout` | **PASS baseline path** — no W5 bootstrap without `initialized_no_mining_gate` |
| Bridge / pool work | **NOT RUN** — capture stopped at boot diagnostic fail-closed |

## Conclusion

F2 does not add bridge-level signal. `clear_rx_before_production_work` remains available for a future bridge matrix row; no RX improvement observed on boot diagnostic path.
