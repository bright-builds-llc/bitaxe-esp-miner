# Parity work result

- Parity row: `PWR-002`
- Final status: `verified`
- Implementation commit: `3ed71e721bec145c56839ca886795426eebc9cd5`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Hardware attempts: none; the sealed accepted ASIC-002 evidence was projected
  without a hardware rerun

## Evidence and verification

The typed `bitaxe-asic-power-initialization-evidence-v1` projection
independently validates the committed ASIC-002 initialization projection and
publishes only closed categories, fixed safe constants, counts, digests,
commits, and booleans. It proves that the exact admitted Ultra 205 package
completed the conservative power-initialization transaction, detected exactly
one BM1366, advanced through mining-ready initialization to accepted work, and
then completed safe stop and cleanup.

The projector binds the validated source projection SHA-256
`eee750561a7c1dcec1a5698b1e5827d3f1508d43655c3c4aa237097338dcf8d4`,
the immutable plan SHA-256
`7ff2ca77e4967f2f823033ef68cfab264863fc20caad841a1ac30c8ecf5d14ff`,
the active task, pinned reference, and accepted-attempt commit
`3e0966a140edbff1a14d2a48ca63d140649762c0`. Six power-owning paths are
byte-identical from that attempt through implementation commit
`3ed71e721bec145c56839ca886795426eebc9cd5`; three later-changed paths pass
unique matching semantic admissions at both commits.

The closed observation binds the conservative 400 MHz, 1100 mV, and 100% fan
profile; all nine preparation steps and eighteen accepted preparation events;
fresh safety before effects; fan-before-voltage ordering with a fresh nonzero
post-command RPM; the complete 500 ms stabilization arm; active-low ASIC
enable; reset and exactly-one-chip detection; mining-ready initialization;
retained production UART; and an accepted submit. It also binds all eight
idempotent rollback attempts, preservation of the initial preparation failure
as primary, ASIC disable during safe stop, cleanup, and no hardware rerun.

The resulting public projection is
`docs/parity/evidence/pwr002-asic-power-initialization/power-initialization-projection.json`;
the independent Rust validator accepted it, its mode is `0644`, and its
SHA-256 is
`0668c274d09b3e39d7d5edfea4b2e66c97248ff77de9192981f3af00e547ddfe`.
The first production-shaped invocation safely withheld evidence because a
short stabilization token was ambiguous with the module import. The corrected
full wait-arm guard and regression passed before this projection was produced.

The following gates passed on the implementation commit:

- `cargo fmt --all`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo build --all-targets --all-features`
- `cargo test --all-features`
- `bun scripts/bright-builds-check.ts all`
- `just test` (all 41 Bazel test targets passed)
- `just parity`
- `just parity-progress`
- `just verify-redaction`
- `just verify-reference`
- focused Rust contract and TypeScript projector regressions
- source and final projection validation by independent Rust binaries
- exact generated automation contract verification
- immutable-plan, source-compatibility, reference-cleanliness, task-binding,
  file-mode, candidate-absence, sensitive-output, and `git diff --check` gates

## Conclusion

`PWR-002` has a closed end-to-end proof that the production Rust firmware
issued the upstream-aligned conservative ASIC power-initialization transaction
on the exact admitted Ultra 205 package, completed its ordered safety and power
boundaries, produced successful initialized work, and completed safe stop and
cleanup. The source-bound projection closes the former hardware-regression gap
without another device effect.

## Non-claims and residual risks

This result proves the commanded production transaction and its successful
downstream behavior. It does not independently measure analog voltage
accuracy, rail rise time, electrical waveform shape, ESP-IDF scheduler timing,
automatic fan behavior, arbitrary power profiles, or recovery from a
physically injected fault. Thermal response, extended soak, other ASIC or board
behavior, updates, recovery, profitability, and release readiness remain owned
by their dedicated rows. No protected campaign read, detector, package build,
flash, reset, USB or network session, credentials, mining rerun, hardware
control, direct UART, or pin effect occurred during this plan.
