# Parity work result

- Parity row: `ASIC-11`
- Final status: `verified`
- Implementation commit: `bbbf390d80326e8aaa46f02ce520efe2aefcc3e3`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Hardware attempts: none; committed sealed ASIC-002, ASIC-003, and ASIC-004
  evidence was joined without a hardware rerun

## Evidence and verification

The source-bound summary at
`docs/parity/evidence/asic11-result-correlation/summary.md` joins three already
accepted public projections after independent Rust validation:

- ASIC-002 `eee750561a7c1dcec1a5698b1e5827d3f1508d43655c3c4aa237097338dcf8d4`
- ASIC-003 `447af65ae9e6cd5cc2199ef639ff8e0fa7f63d4c9708570bd66781c5a162e80c`
- ASIC-004 `e99c054c4d660155d5c2b1ee38d3f17aed5ae7101e7e4a5fd1c6451d1b48b7c7`

The same-attempt chain from hardware commit
`3e0966a140edbff1a14d2a48ca63d140649762c0` proves mining-ready initialization,
retained production UART, live production work, a qualified parsed and
correlated result before submit intent, an accepted response, fresh safety,
confirmed safe stop, cleanup, trusted identity, and passed redaction. ASIC-004
records `job_lookup_validation`, `correlation_semantics_compatible`, and
`live_qualified_result_observed`. Each projection is mode `0644`. No new
projector or protected artifact was used.

Current tests prove the registry maps a nonce observation to submit intent
only for the current generation and active job, and fail-closes uncorrelated,
stale, duplicate, generation-mismatched, and drifted-target results.
Production-session tests prove ASIC effects bind to generation and valid-job
context and that an accepted first submit consumes the current generation.

The following focused gates passed on source `bbbf390d`:

- independent validation of ASIC-002, ASIC-003, and ASIC-004
- `cargo test -p bitaxe-stratum production_work`
- `cargo test -p bitaxe-stratum production_session`
- `just verify-reference`
- `just package`

## Conclusion

`ASIC-11` has a closed proof that a live Ultra 205 BM1366 result was parsed
and correlated to active pool work before submit intent, then reached an
accepted response. This supports `ASIC-11` at `verified` with
`unit,golden,workflow,hardware-smoke,hardware-regression` evidence.

## Non-claims and residual risks

This result does not verify submit-response classification ownership, rejected
share hardware, frequency transitions, voltage/fan/thermal behavior, nonzero
version-mask or multi-midstate breadth, share-hash or network-target policy
beyond the accepted qualified result, live clean-jobs or reconnect, other
ASICs or boards, arbitrary pools or profiles, unbounded mining, OTA/recovery,
or release readiness. It does not promote ASIC-12, STR-08, STR-09, SAFE-12, or
SAFE-13. No credential or protected-attempt access, detector,
device/USB/network runtime, flash, monitor, mining, restart, recovery,
hardware attempt, fault injection, external UART/BAP, pins, or electrical work
occurred during this plan.
