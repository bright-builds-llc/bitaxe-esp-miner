# B4 Init State Run G2 (require_diagnostic_nonce control)

Date: 2026-07-06  
Board: Ultra 205  
Investigation: `require_diagnostic_nonce,initialized_no_mining_gate`  
Evidence: [`b4-init-state-20260706-run-G2/`](b4-init-state-20260706-run-G2/)

## Goal

Confirm bridge blocked without W5 bootstrap when diagnostic proof required.

## Observations

| Marker | Result |
| --- | --- |
| `work_result_diagnostic_timeout` @ ~20110ms | **PASS** |
| `fail_closed reason=work_result_diagnostic_timeout` | **PASS** |
| `work_dispatched` | **Absent** — bridge never reached production work |

## Conclusion

Control **PASS** — `require_diagnostic_nonce` disables bootstrap; bridge path correctly fail-closed.
