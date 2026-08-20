# Parity work result

- Parity row: `STR-08`
- Final status: `verified`
- Implementation commit: `8f86924a34e3988da15b0bc6b274ecd1c3806c21`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Hardware attempts: none; committed sealed STR-001 and STR-006 evidence was
  joined without a hardware rerun

## Evidence and verification

The source-bound summary at
`docs/parity/evidence/str08-live-socket-lifecycle/summary.md` joins two already
accepted public projections after independent Rust validation:

- STR-001 `dcb3eed396a268114b017d7ef4fbca9c427a390d7acf405fc52fbef6472122b8`
- STR-006 `f008171f26b7a8ae6b08859e3cfef4f0c5bf88937c049dd66b6f868c9bbfd6f7`

The same-attempt chain from hardware commit
`3e0966a140edbff1a14d2a48ca63d140649762c0` proves exact-package admission,
trusted runtime identity, typed production TCP connect/write/close and event
ownership, hardware preparation before pool access, authorization before ASIC
dispatch, live work/result correlation, a qualified result before submit, an
accepted response, fresh safety, ordered safe stop, lease cleanup, USB cleanup,
and redaction. Both projections are mode `0644`. No new projector or protected
artifact was used.

Current live-runtime tests cover the complete configure/subscribe/authorize/
notify/work/submit state machine, reconnect and session invalidation, clean-jobs,
and redacted context. Production-session tests cover typed transport epochs,
primary/fallback policy, retry budgets, response identity, safe stop, and
redaction. The firmware production-transport loopback target proves the actual
worker connects, writes, preserves partial input, reports typed failure, and
redacts diagnostics.

The following focused and required gates passed on source `8f86924a`:

- independent validation of STR-001 and STR-006
- `cargo test -p bitaxe-stratum live_runtime`
- `cargo test -p bitaxe-stratum production_session`
- `bazel test //firmware/bitaxe:production_transport_tests`
- the ordered format, strict Clippy, all-target build, and all-feature test gates
- `bun scripts/bright-builds-check.ts all`
- `just verify-reference`
- `just package`

## Conclusion

`STR-08` has a closed proof that the current production lifecycle established
and owned a real Ultra 205 Stratum v1 TCP session through authorized live work,
accepted response, and ordered safe stop. This supports `STR-08` at `verified`
with `unit,workflow,hardware-smoke,hardware-regression` evidence.

## Non-claims and residual risks

This result does not verify fallback or reconnect on hardware, exact upstream
timeout or keepalive option equivalence, DNS/IP-family preference parity,
arbitrary pools, TLS, Stratum v2, rejected-share hardware, unbounded socket
stability, other boards, updates, recovery, profitability, or release readiness.
It does not promote STR-09, SAFE-12, or SAFE-13. No credential or protected-
attempt access, detector, device/USB/network runtime, flash, monitor, mining,
restart, recovery, hardware attempt, fault injection, external UART/BAP, pins,
or electrical work occurred during this plan.
