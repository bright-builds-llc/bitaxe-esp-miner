# Parity work plan

- Run ID: `20260802T181828Z-ASIC-006`
- Parity row: `ASIC-006`
- Initial status: `implemented`
- Source commit: `de49482a35c430a07f15e58035ace1a56db36d74`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-asic-006-crc-verification`

## Selection

`next-item --format json` reported no open plan and ranked `CFG-001` first.
The following earlier candidates are temporarily ineligible:

- `CFG-001`: its remaining frequency/voltage-default claim is explicitly
  safety-critical and lacks a row-specific authorized hardware contract.
- `CFG-005`: the broad runtime-settings row still lacks complete API PATCH and
  firmware NVS-adapter coverage; the existing narrow adapters do not close it.
- `NET-001`: retry, fallback timing, IPv6, and long-running reconnect parity
  require network/hardware evidence not authorized by a row-specific task.
- `ASIC-002` through `ASIC-005`: initialization, work send, result parsing, and
  serial transport retain explicit live-hardware evidence gaps.

`ASIC-006` is the first bounded software-verifiable candidate. The reference
CRC functions are pure, deterministic protocol logic, and the checklist
explicitly keeps live ASIC communication evidence separate from this row.

## Scope and non-scope

Implement and verify the complete CRC function family exposed by pinned
`reference/esp-miner/components/asic/crc.c`: MSB-first CRC5, CRC16-CCITT with a
zero initial value, and CRC16-FALSE with an all-ones initial value. Verify the
BM1366 command and job framing uses the correct variant and byte ordering.

Do not modify the pinned reference tree, copy its lookup table into MIT source,
touch hardware, flash firmware, access credentials, change ASIC transport or
mining behavior, or claim live communication, initialization, other ASIC-family,
or named-board hardware parity.

## Implementation

- [ ] Add the missing zero-initialized CRC16 API through a small shared bitwise
      CCITT core that preserves the existing GPL-safe independent expression.
- [ ] Add behavior-focused reference vectors for all three variants, empty and
      canonical inputs, CRC5 receive residue, and BM1366 frame CRC placement.
- [ ] Record the exact verification commands and non-claims in `WORKLOG.md` and
      a terminal `RESULT.md` if every promotion criterion passes.

## Verification and promotion

Targeted acceptance:

- `cargo test -p bitaxe-asic crc --all-features`
- `bazel test //crates/bitaxe-asic:tests`
- the reference cleanliness and parity validators retain the pinned commit;
- the mandatory Rust, Bright Builds, Bazel, parity, and progress checks pass.

Promotion to `verified` requires deterministic vectors to cover all reference
CRC entrypoints, BM1366 command/job framing to use CRC5/CRC16-FALSE exactly as
the reference does, receive-frame CRC5 residue rejection to remain covered,
and no unsupported hardware or transport claim. Evidence is `unit,golden` and
is bound to this plan and `RESULT.md` by the transition receipt.
