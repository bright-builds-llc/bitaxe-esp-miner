# Parity work result

- Parity row: `STR-09`
- Final status: `verified`
- Implementation commit: `532ab568228312157b3164820d9ad9f9ae221dbf`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Hardware attempts: none; committed sealed STR-001, STR-006, and ASIC-004
  evidence was joined without a hardware rerun

## Evidence and verification

The source-bound summary at
`docs/parity/evidence/str09-submit-response-classification/summary.md` joins
three already accepted public projections after independent Rust validation:

- STR-001 `dcb3eed396a268114b017d7ef4fbca9c427a390d7acf405fc52fbef6472122b8`
- STR-006 `f008171f26b7a8ae6b08859e3cfef4f0c5bf88937c049dd66b6f868c9bbfd6f7`
- ASIC-004 `e99c054c4d660155d5c2b1ee38d3f17aed5ae7101e7e4a5fd1c6451d1b48b7c7`

The same-attempt chain from hardware commit
`3e0966a140edbff1a14d2a48ca63d140649762c0` proves exact-package admission,
trusted runtime identity, an authorized production socket, ASIC-derived work,
a qualified correlated result before submit intent, a matching accepted
response, fresh safety, ordered safe stop, lease cleanup, USB cleanup, and
redaction. All projections are mode `0644`. No new projector or protected
artifact was used.

Current classifier tests prove acceptance requires matching current-generation
submit intent and response identity, while missing, mismatched, stale, unrelated,
and rejection shapes fail closed or preserve only redacted reason labels.
Live-runtime and production-session tests bind the correlated ASIC result to
submit action, classified response, lease consumption, and ordered safe stop.

The canonical Phase 30 conclusion now records
`STR-09.live_submit_response_classified: true`,
`STR-09.asic_correlation: passed`, and
`STR-09.safe_stop_status: complete`. Its checked-in current-artifact tests
require all three promoted Phase 30 rows and the full report to pass together.

The following focused and required gates passed on implementation source
`532ab568`:

- independent validation of STR-001, STR-006, and ASIC-004
- `cargo test -p bitaxe-stratum submit_response`
- `cargo test -p bitaxe-stratum live_runtime`
- `cargo test -p bitaxe-stratum production_session`
- `bazel test //tools/parity:tests`
- the ordered format, strict Clippy, all-target build, and all-feature test gates
- `bun scripts/bright-builds-check.ts all`
- `just verify-reference`
- `just package`

## Conclusion

`STR-09` has a closed proof that a live Ultra 205 ASIC-derived submit intent
received a matching response classified as accepted before ordered safe stop.
This supports `STR-09` at `verified` with
`unit,workflow,hardware-smoke,hardware-regression` evidence.

## Non-claims and residual risks

This result does not verify rejected-share hardware, mismatched or stale
response paths on hardware, fallback or reconnect on hardware, exact upstream
timeout or keepalive equivalence, arbitrary pools, TLS, Stratum v2, unbounded
mining, other boards or ASICs, updates, recovery, profitability, or release
readiness. It does not promote SAFE-12 or SAFE-13. No credential or protected-
attempt access, detector, device/USB/network runtime, flash, monitor, mining,
restart, recovery, hardware attempt, fault injection, external UART/BAP, pins,
or electrical work occurred during this plan.
