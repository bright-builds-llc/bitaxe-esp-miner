# B3 Production Read Run F3 (Default package regression guard)

Date: 2026-07-06  
Board: Ultra 205  
Port: `/dev/cu.usbmodem1101`  
Investigation: (default — no `initialized_no_mining_gate`)  
Evidence root: [`b3-production-read-20260706-run-F3/`](b3-production-read-20260706-run-F3/)

## Pass criterion

Fail-closed baseline unchanged: diagnostic timeout stops bridge path before production work.

## Observations

| Marker | Timestamp (ms) | Result |
| --- | --- | --- |
| `asic_status=initialized_no_mining` | 2420 | Init completes |
| `bm1366_diagnostic_result=timeout` | 12520 | Expected |
| `fail_closed reason=work_result_diagnostic_timeout` | 12520 | **PASS** — W5 bootstrap disabled by default |
| Bridge / `work_dispatched` | absent | **PASS** — fail-closed before bridge |

## Conclusion

Default package regression guard **PASS**. Fail-closed bootstrap behavior unchanged after B3 production-read code changes.
