# B4 Init State Run G3 (ramp + W9 delay)

Date: 2026-07-06  
Investigation: `frequency_ramp,post_max_baud_delay_2000,initialized_no_mining_gate`  
Evidence: [`b4-init-state-20260706-run-G3/`](b4-init-state-20260706-run-G3/)

## Result

**BLOCKED** — `espflash` connection error during flash-monitor capture after G2 (`Error while connecting to device`). No firmware log beyond connection failure.

## Conclusion

Pending hardware replug / retry. G1 already shows W8+W13 combo silent; G3 unlikely to change outcome without connection recovery.
