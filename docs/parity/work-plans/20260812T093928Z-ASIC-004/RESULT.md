# Parity work result

- Parity row: `ASIC-004`
- Final status: `verified`
- Implementation commit: `2861bfb1d425d3c5d13b3a820c082eb24e1f1a77`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Hardware attempts: none; committed sealed evidence was derived without a
  hardware rerun

## Evidence and verification

The typed `bitaxe-asic-result-parsing-evidence-v1` projection validates the
committed ASIC-003 work-send projection before publishing only closed
categories, constants, digests, commits, and booleans. It binds the source
artifact SHA-256 and proves that the exact admitted Ultra 205 package produced
a live qualified BM1366 result that reached an accepted submit response with
fresh safety, safe stop, and cleanup.

The projector independently validated the source projection, required its
source commit to be an ancestor of current source, and proved the result
transport module unchanged from accepted hardware commit
`3e0966a140edbff1a14d2a48ca63d140649762c0` through implementation commit
`2861bfb1d425d3c5d13b3a820c082eb24e1f1a77`. Unique bounded comparisons bind
strict frame admission, nonce decoding, the adapter job-nonce arm, and worker
nonce emission. Closed semantic fragments bind current-generation job lookup,
stored-context validation, share-submission construction, and submit-intent
production across the compatible correlation refactor.

Current behavior tests prove exact 11-byte frames, preamble and CRC checks,
job-ID lookup, little-endian submit nonce recovery, core and address-interval
validation, version-bit recovery, known-register parsing, eight typed closed
discard categories, and soft-discard continuation. The independent Rust
validator accepted the public artifact, whose SHA-256 is
`e99c054c4d660155d5c2b1ee38d3f17aed5ae7101e7e4a5fd1c6451d1b48b7c7`.
Repository semantic redaction passed with 13 public evidence artifacts.

The following gates passed on the implementation content:

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
- focused BM1366 parser and production-correlation tests
- production-shaped projector and real-child validator tests
- immutable-plan, source compatibility, source digest, task uniqueness,
  reference cleanliness, public-sensitive-value, and `git diff --check` gates

## Conclusion

`ASIC-004` has a closed end-to-end proof that a live result from production
BM1366 work passed the strict Rust result parser and compatible correlation
path before the accepted response observed on the exact admitted package.
Malformed inputs remain typed soft discards, and the proof required no new
device effect or protected campaign access.

## Non-claims and residual risks

This result does not claim work encoding, arbitrary-load serial transport,
frequency transitions, broader Stratum socket behavior, target-validation or
submit policy, default-profile soak, voltage/fan/power/thermal behavior, other
ASICs or boards, updates, recovery, profitability, or release readiness. Those
remain owned by their dedicated rows. No detector, flash, reset, USB session,
credential read, serial or network request, mining, hardware control, direct
UART, pin manipulation, or protected artifact access occurred during this
plan.
