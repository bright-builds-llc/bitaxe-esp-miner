# Parity work result

- Parity row: `STR-004`
- Final status: `verified`
- Implementation commit: `b55228706d28f9b34d71a092656ef3ca6f3f649a`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`

## Evidence and verification

The pinned reference defines four Bitcoin CompactSize widths, BIP-34 block
height extraction, printable ScriptSig projection, bounded coinbase outputs and
total value, compact-target network difficulty, and BIP-54/BIP-110 signal
decisions. Its standard output scripts are classified separately from the
Base58 and SegWit address codecs owned by `STR-012`.

Implementation commit `b55228706d28f9b34d71a092656ef3ca6f3f649a`
adds a typed, bounds-checked decoder and executable fixture evidence in:

- `crates/bitaxe-stratum/src/v1/coinbase/decoder.rs`
- `crates/bitaxe-stratum/src/v1/coinbase/decoder_tests.rs`
- `crates/bitaxe-stratum/fixtures/v1/coinbase-decoder-cases.json`
- `docs/parity/work-plans/20260802T201136Z-STR-004/WORKLOG.md`

The following commands passed on the implementation tree:

- `jq empty crates/bitaxe-stratum/fixtures/v1/coinbase-decoder-cases.json`
- `cargo test -p bitaxe-stratum coinbase_decoder --all-features` (11 focused
  tests passed)
- `cargo clippy -p bitaxe-stratum --all-targets --all-features -- -D warnings`
- `bazel test //crates/bitaxe-stratum:tests`
- `bazel run //scripts:verify_reference_clean`
- `cargo fmt --all`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo build --all-targets --all-features`
- `cargo test --all-features`
- `bun scripts/bright-builds-check.ts all`
- `just verify-redaction`
- `just test`
- `just parity` (`validation_errors: none`)
- `just parity-progress` (pre-transition baseline 30/94, 31.9%)

## Conclusion

The Rust-owned decoder reassembles split Stratum coinbase data, validates every
read and checked arithmetic boundary, derives the reference network difficulty,
extracts the BIP-34 height and printable pool tag without extranonces, sums all
outputs while retaining at most six, classifies every reference script shape,
and applies the exact BIP-54 and BIP-110 decision boundaries. Pinned golden
vectors cover all CompactSize widths and every in-scope decoded field; focused
regressions cover truncation, invalid split placement, disabled projection,
output retention, the BIP-110 expiry height, and the BIP-54 sequence guard.
This satisfies the row's deterministic `unit,golden` evidence requirement.

## Non-claims and residual risks

This result does not encode Base58Check, Bech32, or Bech32m addresses and does
not match outputs to a configured payout address; those remain explicitly
owned by unverified row `STR-012`. It does not claim live pool compatibility,
TLS, credentials, networking, ASIC dispatch, nonce parsing, share outcomes,
production mining, timing, or hardware behavior. The result uses independently
constructed fixture data rather than copied GPL source expression. No hardware,
credentials, network discovery, direct UART, pin manipulation, or reference
tree edits were used.
