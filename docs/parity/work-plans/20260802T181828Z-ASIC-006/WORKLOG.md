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

## 2026-08-02T18:41:29Z | transition integration guard repair

- Source commit: `268a118b565579674695bba523b7c970c7db734a`
- Actions: Transitioned only `ASIC-006`, synchronized progress, and ran the
  final repository gate. The gate exposed a Phase 35 shell contract that still
  compared the immutable comprehensive-revision digest directly to the live
  checklist. Updated that historical contract to compare against the
  transition ledger's immutable baseline and added the baseline to its Bazel
  runfiles.
- Verification: The first final `just test` run passed 81 of 82 tests and failed
  only `//scripts:phase35_promotion_contract_test` with
  `category=comprehensive-revision-root-drift`; every other test and firmware
  build/package target passed.
- Evidence: `docs/parity/checklist-transitions/baseline.md` preserves the exact
  comprehensive revision; the transition receipt chains that baseline to the
  selected-row result.
- Outcome: The historical comprehensive inventory remains immutable while
  ordinary one-row transitions can advance the live checklist through the
  guarded append-only ledger.
- Blocker or next safe action: Re-run the focused contract and every required
  final gate before committing.

## 2026-08-02T18:44:09Z | finalization verified

- Source commit: `268a118b565579674695bba523b7c970c7db734a`
- Actions: Re-ran the corrected Phase 35 contract, the complete Rust gate,
  Bright Builds, the full Bazel graph, and both parity commands against the
  transitioned and archived worktree.
- Verification: `//scripts:phase35_promotion_contract_test` passed; the Cargo
  format, warnings-denied Clippy, all-target build, and all-feature tests
  passed; Bright Builds reported zero findings; `just test` passed 82 of 82;
  `just parity` passed; `just parity-progress` reported 28/94 and 29.8%; and
  `git diff --check` passed.
- Evidence: The hash-bound transition receipt, progress record, README marker,
  terminal result, and archived task record in this finalization tree.
- Outcome: `ASIC-006` is verified and the first real `advance-parity`
  transition completes without weakening historical revision validation.
- Blocker or next safe action: Commit finalization, re-check upstream sync, and
  push the current branch without force.
