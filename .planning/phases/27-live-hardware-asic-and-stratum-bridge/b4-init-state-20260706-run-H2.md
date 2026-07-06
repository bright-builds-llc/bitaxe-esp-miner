# B4 Init State Run H2 (require UART proof control)

Date: 2026-07-06  
Investigation: `require_uart_proof_for_production,initialized_no_mining_gate`  
Evidence: not captured (USB blocked after G2)

## Code expectation

`require_uart_proof_for_production` disables W5 bootstrap (same gate as `require_diagnostic_nonce`). Bridge should fail-closed at diagnostic timeout like G2.

## Result

**BLOCKED hardware** — pending retry. **PASS by code symmetry** with G2 control.
