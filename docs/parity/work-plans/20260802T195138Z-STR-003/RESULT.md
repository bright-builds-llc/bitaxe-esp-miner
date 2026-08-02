# Parity work result

- Parity row: `STR-003`
- Final status: `verified`
- Implementation commit: `242a51ebaa61a6451b11f1122ff159b26a274b5e`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`

## Evidence and verification

The pinned reference defines little-endian extranonce2 generation, double-SHA256
coinbase hashing, iterative Merkle-branch folding, and the byte order used to
populate BM1366 work fields. It also supplies complete mining-job inputs and
expected coinbase and Merkle hashes in its Stratum unit vectors.

Implementation commit `242a51ebaa61a6451b11f1122ff159b26a274b5e`
provides executable fixture-driven evidence for those deterministic operations
and for retained job context. Committed evidence is in:

- `crates/bitaxe-stratum/fixtures/v1/mining-job-cases.json`
- `crates/bitaxe-stratum/src/v1/mining/golden_tests.rs`
- `crates/bitaxe-stratum/src/v1/mining.rs`
- `crates/bitaxe-stratum/src/v1/coinbase.rs`
- `docs/parity/work-plans/20260802T195138Z-STR-003/WORKLOG.md`

The following commands passed on the implementation tree:

- `jq empty crates/bitaxe-stratum/fixtures/v1/mining-job-cases.json`
- `cargo test -p bitaxe-stratum mining_job --all-features` (16 focused tests passed)
- `bazel test //crates/bitaxe-stratum:tests`
- `bazel run //scripts:verify_reference_clean`
- `cargo fmt --all`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo build --all-targets --all-features`
- `cargo test --all-features`
- `bun scripts/bright-builds-check.ts all` (zero findings)
- `just verify-redaction`
- `just test` (82 of 82 Bazel tests passed after one bounded retry of a
  recovered ESP-IDF remote-query timeout)
- `just parity` (no validation errors)
- `just parity-progress` (pre-transition baseline 29/94, 30.9%)

## Conclusion

The pinned upstream job vector now executes through the Rust-owned mining work
builder. Exact assertions cover all deterministic BM1366 work fields,
extranonce2, coinbase hash, Merkle root, Stratum job identity, clean-jobs state,
pool difficulty, version mask, and malformed-branch rejection. This satisfies
the row's `unit,golden` evidence requirement for deterministic mining-job
construction and typed BM1366 work preparation.

## Non-claims and residual risks

This result does not claim live sockets, networking, TLS, credentials, pool
timing, reconnect behavior, ASIC dispatch, nonce validation, share outcomes,
production mining, or hardware behavior. It does not extend the deterministic
BM1366 job-vector evidence to other ASIC families. Those effects remain owned
by other checklist rows and require their own workflow or hardware evidence.
No hardware, credentials, network discovery, direct UART, pin manipulation, or
reference tree edits were used for this result.
