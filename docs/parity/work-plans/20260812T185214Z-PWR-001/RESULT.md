# Parity work result

- Parity row: `PWR-001`
- Final status: `verified`
- Implementation commit: `9cd2ec3741c09e3a3636c8358c0203de183805bb`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Hardware attempts: none; the sealed accepted-share evidence was projected
  without a hardware rerun

## Evidence and verification

The typed `bitaxe-asic-reset-evidence-v1` projection independently validates
the committed ASIC-002 initialization projection before publishing only
closed categories, fixed durations and counts, digests, commits, and
booleans. It proves that the exact admitted Ultra 205 package completed the
production reset-and-detect boundary, detected exactly one BM1366, advanced
through mining-ready initialization to accepted work, and then completed safe
stop and cleanup.

The projector binds the validated source projection SHA-256
`eee750561a7c1dcec1a5698b1e5827d3f1508d43655c3c4aa237097338dcf8d4`,
the immutable plan SHA-256
`3b3fb9ca3ae38156b006863a8b3ffded8ebfea43995fa3e3ef9cbec8e3911a79`,
the active task, the pinned reference, and accepted-attempt commit
`3e0966a140edbff1a14d2a48ca63d140649762c0`. Six reset-owning paths are
byte-identical from that attempt through implementation commit
`9cd2ec3741c09e3a3636c8358c0203de183805bb`.

Unique current-source admissions bind the literal active-low 100 ms low / 100
ms high action, the low and high GPIO transitions with both delays, production
reset-and-detect dispatch, fail-closed `HoldResetLow`, and ordered safe-stop
`HoldResetLow`. The prior accepted hardware campaign supplies the downstream
exactly-one-chip response and accepted submit observation. The resulting
public projection is
`docs/parity/evidence/pwr001-asic-reset/asic-reset-projection.json`; the
independent Rust validator accepted it, its mode is `0644`, and its SHA-256 is
`11bb816e6f6e2393b796b13c49ae7db5d181f719dc94898ca00e17ce384d469b`.

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
- focused Rust contract and exact selector regressions
- focused TypeScript projection, failure-withholding, redaction, and real-child
  validator regressions
- exact generated automation contract verification
- independent source and final projection validation
- immutable-plan, source-compatibility, reference-cleanliness, task-binding,
  file-mode, candidate-absence, and `git diff --check` gates

## Conclusion

`PWR-001` has a closed end-to-end proof that the production Rust firmware
issued the upstream-aligned active-low reset transaction on the exact admitted
Ultra 205 package, received the expected single BM1366 response, progressed to
accepted work, and restored the chain to reset-low during safe stop. The
source-bound projection closes the former hardware-regression gap without
another device effect.

## Non-claims and residual risks

This result does not independently measure the GPIO waveform or ESP-IDF
scheduler accuracy. It does not claim voltage, fan, power, thermal, sensor,
arbitrary fault-injection, soak, other ASIC or board behavior, updates,
recovery, profitability, or release readiness. Those remain owned by their
dedicated rows. No protected campaign read, detector, package build, flash,
reset, USB or network session, credentials, mining, hardware control, direct
UART, or pin effect occurred during this plan.
