# Parity work result

- Parity row: `ASIC-002`
- Final status: `verified`
- Implementation commit: `f9df1412abbc05a4852022f3fb6741f67ab43272`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Hardware attempts: none; sealed attempt `attempt-007` was projected without a
  hardware rerun

## Evidence and verification

The typed `bitaxe-asic-initialization-evidence-v1` projection validates the
sealed accepted-share campaign and its protected private inputs before
publishing only closed categories, counts, digests, commits, and booleans. It
proves that the exact package was admitted on board 205, all nine ordered
initialization steps completed, exactly one BM1366 was detected, the mining-
ready boundary completed, production UART was retained, and initialized live
work reached an accepted submit response.

The projector independently verified the campaign result seal, both bound
private digests, mode `0700` on the protected attempt root, mode `0600` on all
four protected files, 18 accepted and zero invalid preparation events, and a
terminal `retain_production_uart/completed` event. It also proved that seven
initialization-owning source paths are byte-identical between accepted-attempt
commit `3e0966a140edbff1a14d2a48ca63d140649762c0` and implementation commit
`f9df1412abbc05a4852022f3fb6741f67ab43272`.

The same closed projection records trusted runtime identity and attestation,
clean serial outcome, fresh safety, `mine_on_boot` disabled, confirmed safe
stop, confirmed lease cleanup, USB cleanup readiness, and no hardware rerun.
The independent Rust validator accepted the public artifact, whose SHA-256 is
`eee750561a7c1dcec1a5698b1e5827d3f1508d43655c3c4aa237097338dcf8d4`.
Repository redaction validation passed with 11 public evidence artifacts.

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
- independent Rust projection validation
- immutable-plan, protected-mode, source-compatibility, reference-cleanliness,
  and `git diff --check` gates

## Conclusion

`ASIC-002` has a closed end-to-end proof that the production Rust firmware
completed its BM1366 initialization transaction on the exact admitted Ultra
205 package and progressed into live accepted work while preserving safety and
cleanup. The sealed campaign evidence and unchanged initialization paths close
the former gap without another device effect.

## Non-claims and residual risks

This result does not claim frequency-transition parity, voltage/fan/power
parity, thermal response, detailed work-send or result-parsing parity,
pool/Stratum parity, default-profile soak stability, other ASICs or boards,
updates, recovery, profitability, or release readiness. Those remain owned by
their dedicated rows. No detector, flash, reset, USB session, credential read,
serial or network request, mining, hardware control, direct UART, or pin effect
occurred during this plan.
