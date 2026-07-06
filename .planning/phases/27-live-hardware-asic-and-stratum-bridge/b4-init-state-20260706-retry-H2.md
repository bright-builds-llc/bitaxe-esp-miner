# B4 Init State Retry H2 (require UART proof)

Date: 2026-07-06  
Investigation: `require_uart_proof_for_production,initialized_no_mining_gate`  
Evidence: [`b4-init-state-20260706-retry-H2/`](b4-init-state-20260706-retry-H2/)

## Observations

| Marker | Result |
| --- | --- |
| `work_result_diagnostic_timeout` @ ~20110ms | **PASS** |
| `fail_closed reason=work_result_diagnostic_timeout` | **PASS** |
| `work_dispatched` | **Absent** |

## Conclusion

Hardware confirms code symmetry with G2 — `require_uart_proof_for_production` disables bootstrap and fail-closes before bridge.
