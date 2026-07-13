# B4 Init State Run G1 (frequency_ramp + bootstrap)

Date: 2026-07-06  
Board: Ultra 205  
Port: `/dev/cu.usbmodem1101`  
Investigation: `frequency_ramp,initialized_no_mining_gate`  
Evidence: [`b4-init-state-20260706-run-G1/`](b4-init-state-20260706-run-G1/)

## Goal

W8 + W13 combo — post-dispatch UART proof after stepped frequency ramp.

## Observations

| Marker | Timestamp (ms) | Result |
| --- | --- | --- |
| Chip-detect `rx_chunk` | 1970 | PASS (pre-work only) |
| Boot diagnostic read | 10100–20100 (~10s) | Timeout → bootstrap |
| `work_dispatched` | 27240 | PASS |
| `result_read_attempt` | 27350–37230 (~47 polls) | ~9990ms window |
| Post-dispatch `rx_chunk` / `register_read_parsed` | none | **FAIL** |

## Conclusion

Frequency ramp + W13 bootstrap does **not** restore post-production UART. Escalate to H3 (skip diagnostic) and structural default ramp (Wave 3).
