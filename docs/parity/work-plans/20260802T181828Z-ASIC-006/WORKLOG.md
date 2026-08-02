# Parity work log

## 2026-08-02T18:34:25Z | implementation and focused verification

- Source commit: `b255e7fcfae2b29898595512b0c7aa12cc4cc3c6`
- Actions: Audited all three CRC functions in the pinned reference; added the
  missing zero-initialized CRC16 API through a shared bitwise CCITT core;
  expanded the pinned fixture with boundary, canonical, and BM1366 job-frame
  vectors; and replaced a self-derived job-frame assertion with exact bytes.
- Verification: `cargo fmt --all`; `jq empty
  crates/bitaxe-asic/fixtures/bm1366/protocol-cases.json`; `cargo test -p
  bitaxe-asic crc --all-features` (6 passed); `bazel test
  //crates/bitaxe-asic:tests` (passed).
- Evidence: `crates/bitaxe-asic/fixtures/bm1366/protocol-cases.json` and the
  focused Rust/Bazel tests that consume it.
- Outcome: Focused verification passed for the complete software CRC contract.
- Blocker or next safe action: Run every repository verification gate, commit
  the implementation evidence without changing the checklist, then create the
  terminal result and transition receipt.

## 2026-08-02T18:34:25Z | corrected invalid fixture assumption

- Source commit: `b255e7fcfae2b29898595512b0c7aa12cc4cc3c6`
- Actions: Removed a proposed CRC5 receive-residue vector after its focused test
  disproved the assumption that appending a command checksum byte produces a
  valid response residue. Kept receive-residue behavior under its existing
  parser tests, which construct bytes satisfying the upstream zero-residue
  receive contract.
- Verification: The first focused run failed only the invalid fixture vector;
  the corrected focused Cargo and Bazel runs both passed.
- Evidence: `crates/bitaxe-asic/src/bm1366/result.rs` and
  `crates/bitaxe-asic/src/bm1366/chip_detect.rs` retain receive-side residue
  tests; the CRC fixture now contains only independently justified values.
- Outcome: The evidence makes no unsupported equivalence between transmit and
  receive CRC framing.
- Blocker or next safe action: None beyond the required full verification gate.

## 2026-08-02T18:39:00Z | full implementation gate

- Source commit: `b255e7fcfae2b29898595512b0c7aa12cc4cc3c6`
- Actions: Ran the complete pre-commit and repository verification surfaces
  against the implementation worktree.
- Verification: `cargo fmt --all`; isolated-target `cargo clippy
  --all-targets --all-features -- -D warnings`; isolated-target `cargo build
  --all-targets --all-features`; isolated-target `cargo test --all-features`;
  `bun scripts/bright-builds-check.ts all` (zero findings); `just test` (82 of
  82 Bazel tests passed, including the firmware build and package targets);
  `just parity` (no validation errors); `just parity-progress` (27/94, 28.7%);
  and `git diff --check`.
- Evidence: This worklog, the checked-in fixture, and the source/tests named in
  the plan.
- Outcome: Every required implementation gate passed. The checklist remains
  unchanged until a source commit can be bound into the result and receipt.
- Blocker or next safe action: Commit implementation evidence, capture the full
  commit as `SOURCE_COMMIT`, then perform the one-row transition.
