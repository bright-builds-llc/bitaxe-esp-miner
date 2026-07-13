# B3 Production Read Run F1 (Bridge + W10 loop)

Date: 2026-07-06  
Board: Ultra 205  
Port: `/dev/cu.usbmodem1101`  
Investigation: `initialized_no_mining_gate`  
Evidence roots:

- First capture (pre partial-read poll fix): [`b3-production-read-20260706/`](b3-production-read-20260706/)
- Retry (partial-read poll fix + 10s bridge loop): [`b3-production-read-20260706-retry/`](b3-production-read-20260706-retry/)

## Code under test

- Wave 1: production read uses single 10s bounded path (no 1s `READ_RESULT_FRAME` prelude)
- Wave 2: bridge polls result reads across pump iterations until 10s budget; `result_read_attempt` traces
- Wave 3: production job frame golden layout (host)

## Pass criterion

Post-dispatch `asic_uart_trace=rx_chunk` or `asic_production_trace=register_read_parsed`, or confirmed ~10s read window (not ~1s) before timeout.

## First capture (superseded)

| Marker | Timestamp (ms) | Result |
| --- | --- | --- |
| `work_dispatched` | 19050 | PASS |
| `result_read_attempt` | 19160 (1 poll) | Partial — loop started |
| `production_result_malformed` | 19270 (~220ms after dispatch) | FAIL — fail-closed before 10s budget |

UART partial reads during short polls were incorrectly surfaced as `ResultMalformed`; fixed before retry.

## Retry capture (canonical F1)

| Marker | Timestamp (ms) | Result |
| --- | --- | --- |
| `work_dispatched` | 18880 | PASS |
| `result_read_attempt` | 18990–28840 (47 polls) | **PASS** — ~9960ms read window |
| Post-dispatch `rx_chunk` | none | FAIL |
| `register_read_parsed` | none | FAIL |
| `production_result_timeout` | not emitted before safe-stop | INCONCLUSIVE — capture ended ~120ms after last poll |

Compare blocker-fix E1: `work_dispatched` @ 19090ms → `production_result_timeout` @ 20200ms (**~1110ms**, P0 bug).

Production TX: 88-byte pool job frame @ 1M (retry log).

## Conclusion

- **P0 fixed:** Production read window is ~10s, not ~1s.
- **W10 implemented:** Bridge polls across pump iterations until budget elapses.
- **B3 UART proof:** Still **silent** after pool work — no post-dispatch nonce or register-read proof.
- **Share tier:** `blocked_safe_prerequisite` (retry evidence root).
