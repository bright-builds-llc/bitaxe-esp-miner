# Parity work result

- Parity row: `ASIC-10`
- Final status: `verified`
- Implementation commit: `9a57318a544ef59d1ab5623fc823ae0fb80760d2`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Hardware attempts: none; committed sealed ASIC-002 and ASIC-003 evidence
  was joined without a hardware rerun

## Evidence and verification

The source-bound summary at
`docs/parity/evidence/asic10-work-registry/summary.md` joins two already
accepted public projections after independent Rust validation:

- ASIC-002 `eee750561a7c1dcec1a5698b1e5827d3f1508d43655c3c4aa237097338dcf8d4`
- ASIC-003 `447af65ae9e6cd5cc2199ef639ff8e0fa7f63d4c9708570bd66781c5a162e80c`

The same-attempt chain from hardware commit
`3e0966a140edbff1a14d2a48ca63d140649762c0` proves mining-ready initialization,
retained production UART, a required production-ready gate, typed production
work, a qualified parsed and correlated result, an accepted response, fresh
safety, confirmed safe stop, cleanup, trusted identity, and passed redaction.
Each projection is mode `0644`. No new projector or protected artifact was
used.

Current tests prove the pool-derived registry enqueues valid jobs, preserves
pool context through dispatch, advances generation on invalidation, clears
work on clean-jobs and reconnect, and redacts raw context. Production-session
tests prove ASIC effects bind to generation and valid-job context and that an
accepted first submit consumes the current generation.

The following focused gates passed on source `9a57318a`:

- independent validation of ASIC-002 and ASIC-003
- `cargo test -p bitaxe-stratum production_work`
- `cargo test -p bitaxe-stratum production_session`
- `just verify-reference`
- `just package`

## Conclusion

`ASIC-10` has a closed proof that live Ultra 205 BM1366 production work was
pool-derived through the current registry-backed session, then reached a
qualified result and accepted response. This supports `ASIC-10` at `verified`
with `unit,golden,workflow,hardware-smoke,hardware-regression` evidence.

## Non-claims and residual risks

This result does not verify result-correlation policy beyond the accepted
predecessor, submit-response classification ownership, frequency transitions,
voltage/fan/thermal behavior, nonzero version-mask or multi-midstate breadth,
live clean-jobs or reconnect, other ASICs or boards, arbitrary pools or
profiles, unbounded mining, OTA/recovery, or release readiness. It does not
promote ASIC-11, ASIC-12, STR-08, STR-09, SAFE-12, or SAFE-13. No credential or
protected-attempt access, detector, device/USB/network runtime, flash,
monitor, mining, restart, recovery, hardware attempt, fault injection, external
UART/BAP, pins, or electrical work occurred during this plan.
