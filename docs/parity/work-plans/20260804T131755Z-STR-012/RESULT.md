# Parity work result

- Parity row: `STR-012`
- Final status: `verified`
- Implementation commit: `1729e847fa6afb51788fc637642f4b67d5378d16`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`

## Evidence and verification

The pure Rust implementation is in
`crates/bitaxe-stratum/src/v1/payout_address.rs`; behavior tests are in the
adjacent `payout_address/tests.rs`; and the public golden corpus is
`crates/bitaxe-stratum/fixtures/payout-address-vectors.json`. The fixture binds
seven vectors to the pinned upstream Base58/Bech32 tests and BIP-0173/BIP-0350
rules. It covers all five supported standard scripts across mainnet, testnet,
and regtest. Six tests additionally cover future witness versions, exact
leading-zero preservation, invalid alphabets and checksums, mixed case,
Bech32/Bech32m version mismatch, invalid padding and program bounds,
cross-network rejection, script mismatch, and unsupported scripts.

Commands run:

```text
cargo test -p bitaxe-stratum payout_address --all-features
cargo clippy -p bitaxe-stratum --all-targets --all-features -- -D warnings
cargo test -p bitaxe-stratum --all-features
bazel test //crates/bitaxe-stratum:tests
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo build --all-targets --all-features
cargo test --all-features
bun scripts/bright-builds-check.ts all
just test
just parity
just parity-progress
just verify-redaction
just verify-reference
git diff --check
```

Every command passed. Strict Clippy is clean, all 258 Stratum tests pass, all
28 Bazel test targets pass, Bright Builds reports zero findings, and parity has
no validation errors.

## Conclusion

The implementation independently reproduces the complete row-level codec and
payout validation behavior: canonical Base58Check, Bech32/Bech32m SegWit,
network-specific decoding, standard P2PKH/P2SH/P2WPKH/P2WSH/P2TR script
rendering, and exact address-to-script matching. Invalid or noncanonical input
fails closed through typed categories. This satisfies `STR-012` with pure
`unit,golden` evidence and introduces no new dependency or effectful path.

## Non-claims and residual risks

This result does not read or validate a local owner's configured address,
claim that a payout was received, alter coinbase accounting projections,
connect to a pool, mine, dispatch ASIC work, validate Stratum V2 keys, exercise
hardware, or verify any network beyond Bitcoin mainnet, testnet, and regtest.
Configured-address integration and live payout observation remain separate
from the pure codec parity proved here.
