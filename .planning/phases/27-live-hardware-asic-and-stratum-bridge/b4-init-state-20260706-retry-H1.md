# B4 Init State Retry H1 (skip boot diagnostic)

Date: 2026-07-06  
Investigation: `skip_boot_diagnostic_work,initialized_no_mining_gate`  
Evidence: [`b4-init-state-20260706-retry-H1/`](b4-init-state-20260706-retry-H1/)

## Observations

| Marker | Timestamp (ms) | Result |
| --- | --- | --- |
| `skip_boot_diagnostic_work bootstrap=initialized_no_mining` | 10030 | **PASS** — pool-only path |
| No boot diagnostic TX | confirmed | **PASS** |
| `work_dispatched` | 17330 | PASS |
| `result_read_attempt` | 17450–27160 | ~9720ms window |
| `production_result_timeout` | 27380 | **FAIL** — still silent |

## Conclusion

Skipping synthetic boot diagnostic does **not** unlock post-pool UART. H3 hypothesis (boot work pollutes state) **rejected** as sole cause.
