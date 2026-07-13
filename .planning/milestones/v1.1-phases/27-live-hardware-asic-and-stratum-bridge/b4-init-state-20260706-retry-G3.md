# B4 Init State Retry G3 (ramp + W9 delay)

Date: 2026-07-06  
Investigation: `frequency_ramp,post_max_baud_delay_2000,initialized_no_mining_gate`  
Evidence: [`b4-init-state-20260706-retry-G3/`](b4-init-state-20260706-retry-G3/)

## Observations

| Marker | Timestamp (ms) | Result |
| --- | --- | --- |
| Boot diagnostic read | 12090–22090 (~10s) | Timeout → bootstrap |
| `work_dispatched` | 28820 | PASS |
| `result_read_attempt` | 28930–38500 (~47 polls) | ~9960ms window |
| `production_result_timeout` | 38940 | **FAIL** — no post-dispatch UART |

## Conclusion

W9 + W8 + bootstrap combo does not restore post-production UART.
