# B5 H4 Orchestration J2c (continuous + job pump)

Date: 2026-07-06  
Investigation: `continuous_result_task,job_redispatch_pump,initialized_no_mining_gate,frequency_ramp`  
Evidence: [`b5-h4-orchestration-20260706-J2c/`](b5-h4-orchestration-20260706-J2c/)  
Port: `/dev/cu.usbmodem1101` (detector-gated)

## Observations

| Marker | Timestamp (ms) | Result |
| --- | --- | --- |
| `h4_continuous_result=listener_armed` | 26451 | **PASS** — upstream-style listener before dispatch |
| `phase25_live_stratum_status=active` | 27341 | PASS |
| `result_read_attempt` (continuous) | 26471–50821+ | Many polls; **no fail-closed** |
| `h4_continuous_result=timeout_continue` | 36461, 46591 | **PASS** — non-fatal timeout (upstream NULL-continue semantics) |
| `work_dispatched` | — | **NOT OBSERVED** — no pool work TX this session |
| Post-dispatch `rx_chunk` / `register_read_parsed` | — | **NOT MET** (no dispatch) |
| `production_result_timeout` | — | **NOT OBSERVED** (continuous mode avoids fail-closed) |

## Conclusion

H4 investigation flags behave as designed: continuous listener arms after pool settings consumed, polls UART across 10s windows, and continues on timeout without `production_result_timeout` fail-closed. Stratum reached `active` but **no `work_dispatched`** occurred in this capture, so post-dispatch UART proof tier remains unmet. Share outcome stays `blocked_safe_prerequisite`.

## Capture notes

- Foreground `just flash-monitor` + pool-input-bridge watcher (evidence wrapper background subprocess failed in J2/J2b).
- Duration: 360s capture-timeout.
