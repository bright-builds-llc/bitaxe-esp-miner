# Parity work result

- Parity row: `ASIC-003`
- Final status: `verified`
- Implementation commit: `32017ba8bb9b99212cad3c2c9ecaed7edf603d19`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Hardware attempts: none; committed sealed evidence was derived without a
  hardware rerun

## Evidence and verification

The typed `bitaxe-asic-work-send-evidence-v1` projection validates the
committed ASIC initialization projection before publishing only closed
categories, constants, digests, commits, and booleans. It proves that the
exact package was admitted on board 205, mining-ready initialization completed,
the production UART was retained, live work was observed, a qualified result
was correlated, and the campaign reached an accepted submit response.

The projector independently validated the source projection, bound its
SHA-256, required its commit to be an ancestor of current source, and proved
three byte-level BM1366 work modules unchanged from hardware commit
`3e0966a140edbff1a14d2a48ca63d140649762c0` through implementation commit
`32017ba8bb9b99212cad3c2c9ecaed7edf603d19`. It also compared unique bounded
source spans for worker dispatch, production send handling, and UART frame
write, so unrelated later changes in the same files cannot hide drift in the
claimed path.

Current behavior tests prove the fixed 82-byte payload, 88-byte job frame,
eight-step job-ID advance within a 128-ID modulus, typed `WriteFrame` action,
production-ready gate, bounded worker dispatch, and fail-closed UART errors.
The independent Rust validator accepted the public artifact, whose SHA-256 is
`447af65ae9e6cd5cc2199ef639ff8e0fa7f63d4c9708570bd66781c5a162e80c`.
Repository redaction validation passed with 12 public evidence artifacts.

The following gates passed on the implementation commit:

- `cargo fmt --all`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo build --all-targets --all-features`
- `cargo test --all-features`
- `bun scripts/bright-builds-check.ts all`
- `just test` (all 37 Bazel test targets passed)
- `just parity`
- `just parity-progress`
- `just verify-redaction`
- `just verify-reference`
- real ESP32-S3 firmware package build
- generated automation contract validation
- independent source and work-send projection validation
- immutable-plan, source-compatibility, reference-cleanliness, and
  `git diff --check` gates

## Conclusion

`ASIC-003` has a closed end-to-end proof that production Rust firmware encoded
and dispatched BM1366 work through the retained UART on the exact admitted
Ultra 205 package, after which a qualified correlated result led to an
accepted response. The source-bound proof closes the former diagnostic-only
gap without another device effect.

## Non-claims and residual risks

This result does not claim detailed result parsing or correlation semantics,
frequency transitions, serial transport under arbitrary load, Stratum socket
behavior, target-validation policy, default-profile soak stability,
voltage/fan/power/thermal behavior, other ASICs or boards, updates, recovery,
profitability, or release readiness. Those remain owned by their dedicated
rows. No protected campaign read, detector, flash, reset, USB session,
credential read, serial or network request, mining, hardware control, direct
UART, or pin effect occurred during this plan.
