# B4 Init State Retry H3b (ramp + skip diagnostic combo)

Date: 2026-07-06  
Investigation: `frequency_ramp,skip_boot_diagnostic_work,initialized_no_mining_gate`  
Evidence: [`b4-init-state-20260706-retry-H3b/`](b4-init-state-20260706-retry-H3b/)

## Observations

| Marker | Timestamp (ms) | Result |
| --- | --- | --- |
| `skip_boot_diagnostic_work bootstrap=initialized_no_mining` | 10030 | **PASS** |
| `work_dispatched` | 16950 | PASS |
| `result_read_attempt` | 17070–26810 (~47 polls) | ~9760ms window |
| `production_result_timeout` | 27030 | **FAIL** — zero post-dispatch UART |

## Conclusion

Best combo candidate (W8 + pool-only path) **still silent**. B4 init-state matrix complete; next work is upstream task orchestration (H4), not more init/timing tuning.
