# Parity work result

- Parity row: `STR-002`
- Final status: `verified`
- Implementation commit: `f7b750843d9e6cf094713391b432b2224f895354`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`

## Evidence and verification

The pinned reference defines the deterministic Stratum v1 JSON-RPC shapes for
subscribe, authorize, version-rolling configure, difficulty suggestion and
notification, extranonce subscription and assignment, mining notify, share
submit, response/error variants, ping/pong, reconnect, pool messages, and
version queries/responses. It also caps mining-notify Merkle branches at 32.

Implementation commit `f7b750843d9e6cf094713391b432b2224f895354`
provides fixture-driven parser and serializer evidence for all 23 named shapes
and enforces the exact 32-branch boundary. Committed evidence is in:

- `crates/bitaxe-stratum/fixtures/v1/protocol-cases.json`
- `crates/bitaxe-stratum/src/v1/messages/tests/golden.rs`
- `crates/bitaxe-stratum/src/v1/messages/server.rs`
- `crates/bitaxe-stratum/src/v1/messages/tests/server/failures.rs`
- `crates/bitaxe-stratum/src/v1/messages/tests/server/success.rs`
- `docs/parity/work-plans/20260802T184857Z-STR-002/WORKLOG.md`

The following commands passed on the implementation tree:

- `jq empty crates/bitaxe-stratum/fixtures/v1/protocol-cases.json`
- `cargo test -p bitaxe-stratum messages --all-features` (55 focused tests passed)
- `bazel test //crates/bitaxe-stratum:tests`
- `bazel run //scripts:verify_reference_clean`
- `cargo fmt --all`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo build --all-targets --all-features`
- `cargo test --all-features`
- `bun scripts/bright-builds-check.ts all` (zero findings)
- `just verify-redaction`
- `just test` (82 of 82 Bazel tests passed)
- `just parity` (no validation errors)
- `just parity-progress` (pre-transition baseline 28/94, 29.8%)

## Conclusion

Every deterministic message family named by `STR-002` is represented by the
pinned synthetic fixture and exercised through the owned Rust parser or
serializer. The parser accepts the reference maximum of 32 Merkle branches and
rejects 33. This is sufficient unit and golden evidence for the software-only
Stratum v1 message contract.

## Non-claims and residual risks

This result does not claim live sockets, networking, TLS, credential handling,
pool timing, retry or reconnect lifecycle, ASIC behavior, share acceptance or
rejection, mining, or hardware behavior. Those effects remain owned by other
checklist rows and require their own workflow or hardware evidence. No hardware,
credentials, network discovery, direct UART, pin manipulation, or reference
tree edits were used for this result.
