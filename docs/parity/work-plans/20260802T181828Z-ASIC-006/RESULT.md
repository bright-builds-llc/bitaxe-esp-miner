# Parity work result

- Parity row: `ASIC-006`
- Final status: `verified`
- Implementation commit: `268a118b565579674695bba523b7c970c7db734a`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`

## Evidence and verification

The pinned reference exposes CRC5 with initial value `0x1f`, zero-initialized
CRC16-CCITT, and CRC16-CCITT-FALSE with initial value `0xffff`. It uses CRC5
for command framing and appends CRC16-FALSE big-endian to job frames.

Implementation commit `268a118b565579674695bba523b7c970c7db734a` provides
all three algorithms without copying the upstream lookup table, consumes the
pinned reference fixture from a Rust test, and checks exact job-frame bytes.
Committed evidence is in:

- `crates/bitaxe-asic/fixtures/bm1366/protocol-cases.json`
- `crates/bitaxe-asic/src/bm1366/crc.rs`
- `crates/bitaxe-asic/src/bm1366/crc/tests.rs`
- `crates/bitaxe-asic/src/lib.rs`
- `docs/parity/work-plans/20260802T181828Z-ASIC-006/WORKLOG.md`

The following commands passed on the implementation tree:

- `cargo test -p bitaxe-asic crc --all-features` (6 focused tests passed)
- `bazel test //crates/bitaxe-asic:tests`
- `cargo fmt --all`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo build --all-targets --all-features`
- `cargo test --all-features`
- `bun scripts/bright-builds-check.ts all` (zero findings)
- `just test` (82 of 82 Bazel tests passed)
- `just parity` (no validation errors)
- `just parity-progress` (pre-transition baseline 27/94, 28.7%)

## Conclusion

The deterministic Rust behavior now covers every CRC function in the pinned
reference, the algorithm boundaries and canonical checks agree with fixed
fixtures, and an independently fixed BM1366 job-frame vector proves coverage
and byte order. Existing receive parsers retain zero-residue rejection tests.
This is sufficient unit and golden evidence for the software-only `ASIC-006`
contract.

## Non-claims and residual risks

This result does not claim live UART communication, ASIC initialization,
command acceptance, work submission, nonce/result handling, mining, or any
hardware-control behavior. Those behaviors remain owned by separate checklist
rows with their own hardware and safety evidence requirements. No hardware,
credentials, network discovery, direct UART, pin manipulation, or reference
tree edits were used for this result.
