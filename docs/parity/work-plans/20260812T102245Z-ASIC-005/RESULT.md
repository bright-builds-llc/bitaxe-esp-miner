# Parity work result

- Parity row: `ASIC-005`
- Final status: `verified`
- Implementation commit: `bec4af3d6f105f4e58cfdd6f51e995eaa60eb9d9`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Hardware attempts: none; committed sealed evidence was derived without a
  hardware rerun

## Evidence and verification

The typed `bitaxe-asic-serial-transport-evidence-v1` projection independently
validates both committed prerequisite artifacts before publishing only closed
categories, constants, digests, commits, and booleans. The ASIC-003 projection
proves that the exact admitted Ultra 205 package sent production work through
the retained UART, then observed a qualified result and accepted submit
response. The ASIC-004 projection proves the same attempt's live result passed
strict parsing and compatible production correlation.

The projector binds the prerequisite SHA-256 values
`447af65ae9e6cd5cc2199ef639ff8e0fa7f63d4c9708570bd66781c5a162e80c`
and
`e99c054c4d660155d5c2b1ee38d3f17aed5ae7101e7e4a5fd1c6451d1b48b7c7`.
It proves the complete UART module and adapter surface are byte-identical from
accepted hardware commit `3e0966a140edbff1a14d2a48ca63d140649762c0`
through implementation commit `bec4af3d6f105f4e58cfdd6f51e995eaa60eb9d9`,
and uniquely bounds compatible production TX and RX spans.

Current-source admission proves Ultra 205 TX17/RX18, initial 115200-baud 8N1
configuration without flow control, a 1,000-ms TX-completion bound, exact full-
frame writes, a 2,048-byte RX buffer, 64-byte read chunks, one absolute read
deadline, partial-read accumulation, idle timeout behavior, and partial-frame
rejection with RX cleanup. Tests cover malformed and incomplete prerequisite
evidence, validator rejection, module/span/dirty-path drift, typed process-
launch failure, atomic no-clobber publication, sensitive-output rejection, and
real child-process/file behavior.

The independent Rust validator accepted
`docs/parity/evidence/asic005-serial-transport/asic-serial-transport-projection.json`,
whose SHA-256 is
`bad828db694ee59c4ef3d77b2e58ef89e0195ef382526b97912d0a71e882ad69`.
Repository semantic redaction passed with 13 existing public artifact roots;
the new projection's explicit denylist scan found no sensitive values.

The following gates passed on the implementation content:

- `cargo fmt --all`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo build --all-targets --all-features`
- `cargo test --all-features`
- `bun scripts/bright-builds-check.ts all`
- `just test` (all 37 Bazel test targets passed)
- `just parity`
- `just parity-progress`
- `just package`
- `just verify-redaction`
- `just verify-reference`
- generated automation contract and independent evidence validation
- canonical TypeScript compilation and real-child automation tests
- immutable-plan, source compatibility, source digest, task uniqueness,
  reference cleanliness, public-sensitive-value, and `git diff --check` gates

## Conclusion

`ASIC-005` has a closed end-to-end proof that the exact admitted Ultra 205
package transmitted live production work and received its qualified BM1366
result through the same unchanged, bounded Rust UART transport before the
accepted response. Current partial writes and partial frames fail closed, and
the proof required no new device effect or protected campaign access.

## Non-claims and residual risks

This result does not claim arbitrary baud-rate or board support, direct
external UART use, frequency transitions, broader Stratum socket behavior,
target-validation or submit policy, default-profile soak, voltage/fan/power/
thermal behavior, other ASICs or boards, updates, recovery, profitability, or
release readiness. Those remain owned by their dedicated rows. No detector,
flash, reset, USB session, credential read, serial or network request, mining,
hardware control, direct UART, pin manipulation, or protected artifact access
occurred during this plan.
