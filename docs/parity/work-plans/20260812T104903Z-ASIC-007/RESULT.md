# Parity work result

- Parity row: `ASIC-007`
- Final status: `verified`
- Implementation commit: `be2bbca0f16d4fc48510e7ff8fc2089773e6a55d`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Hardware attempts: none; committed sealed evidence was derived without a
  hardware rerun

## Evidence and verification

The typed `bitaxe-asic-frequency-transition-evidence-v1` projection
independently validates the exact committed ASIC-002 initialization artifact at
SHA-256 `eee750561a7c1dcec1a5698b1e5827d3f1508d43655c3c4aa237097338dcf8d4`
before publishing only closed categories, constants, digests, commits, and
booleans. That prerequisite proves one admitted Ultra 205 completed all nine
preparation boundaries, initialized exactly one BM1366, retained its production
UART, produced live initialized work and an accepted response, and completed
safe stop and cleanup.

At accepted hardware source commit
`3e0966a140edbff1a14d2a48ca63d140649762c0`, the conservative preparation path
selects the explicit production frequency ramp. The pure planner begins at
50 MHz, advances in 6.25-MHz steps with 100-ms delays, emits 56 typed frequency
commands and 56 delays, and terminates at the conservative 400-MHz target. The
production executor returns success only after every typed action completes;
only then can the already-proved preparation boundary complete and admit the
subsequent live work.

The projector proves the complete ramp-planning, actuation, adapter, and UART
modules are unchanged from the accepted source through implementation commit
`be2bbca0f16d4fc48510e7ff8fc2089773e6a55d`, and that the two unique production
executor spans remain byte-compatible despite unrelated changes elsewhere in
their module. Tests cover malformed, incomplete, and digest-drifted prerequisite
evidence, validator rejection, commit/module/span/dirty-path drift, typed child
launch failure, atomic publication, sensitive-output exclusion, and real child-
process/file behavior.

The independent Rust validator accepted
`docs/parity/evidence/asic007-frequency-transition/asic-frequency-transition-projection.json`,
whose SHA-256 is
`34ac6bc0df593bd75b6026eedcecda5f4b34e00cde0f3541a156794f2c7512ae`.
The artifact is mode 0644. Repository semantic redaction passed with 13 existing
public artifact roots, and the explicit sensitive-value scan found no matches.

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
- canonical generated-contract and independent evidence validation
- canonical TypeScript compilation and real-child automation tests
- immutable-plan, prerequisite-digest, source-compatibility, task-uniqueness,
  reference-cleanliness, public-sensitive-value, and `git diff --check` gates

## Conclusion

`ASIC-007` has a closed bounded hardware-regression proof that the accepted
conservative Ultra 205 session completed the upstream-aligned 50-to-400-MHz
BM1366 frequency ramp before live initialized work and an accepted response,
then achieved confirmed safe stop and cleanup. The proof required no new device
effect and did not reopen protected campaign evidence.

## Non-claims and residual risks

This result does not claim arbitrary frequency targets, dynamic runtime
retuning, default-profile or overclock behavior, direct external UART use,
voltage/fan/power/thermal parity, other ASICs or boards, soak stability, updates,
recovery, profitability, or release readiness. No detector, package rebuild,
flash, reset, USB/network session, credential read, mining, hardware control,
direct UART, pin manipulation, or protected artifact access occurred during
this plan.
