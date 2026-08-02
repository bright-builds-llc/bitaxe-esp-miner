# Parity work log

## 2026-08-02T18:53:00Z | initial implementation and invalid boundary fixture

- Source commit: `e4bfc80c8a865e1e442f18f500368317498ba0e0`
- Actions: Relabeled and expanded the pinned protocol fixture for `STR-002`,
  added fixture-driven golden parser/serializer coverage, added the reference
  32-Merkle-branch guard, and added exact-boundary tests.
- Verification: `jq empty crates/bitaxe-stratum/fixtures/v1/protocol-cases.json`
  passed. The first `cargo test -p bitaxe-stratum messages --all-features`
  run passed 53 tests and failed the two new branch-boundary tests.
- Evidence: All three golden fixture tests passed. The two boundary tests
  returned `InvalidJson` because their local builder emitted backslash-escaped
  quote tokens outside JSON strings.
- Outcome: The production guard was not disproved; the failed inputs never
  reached it. Correct the test-only JSON construction and rerun the focused
  gate without weakening parser validation.
- Blocker or next safe action: Replace the two malformed quote builders with
  valid JSON string literals, then rerun formatting, JSON validation, and both
  focused Cargo and Bazel tests.

## 2026-08-02T18:54:58Z | corrected boundary fixture and focused verification

- Source commit: `e4bfc80c8a865e1e442f18f500368317498ba0e0`
- Actions: Corrected only the two test JSON builders, formatted the workspace,
  and retained the production Merkle cap plus all existing fail-closed parser
  validation unchanged.
- Verification: `jq empty
  crates/bitaxe-stratum/fixtures/v1/protocol-cases.json`; `cargo test -p
  bitaxe-stratum messages --all-features` (55 passed); `bazel test
  //crates/bitaxe-stratum:tests` (passed); `bazel run
  //scripts:verify_reference_clean` (pinned reference clean); and `git diff
  --check` all passed.
- Evidence: The three fixture-driven golden tests cover all 23 named message
  shapes, and the exact 32-branch acceptance plus 33-branch rejection tests
  now reach and prove the reference boundary.
- Outcome: Focused `STR-002` implementation and evidence pass.
- Blocker or next safe action: Complete the mandatory full repository gate,
  review the diff for scope and simplification, then commit the implementation
  and worklog without changing the checklist.

## 2026-08-02T19:05:25Z | full implementation gate and scope review

- Source commit: `e4bfc80c8a865e1e442f18f500368317498ba0e0`
- Actions: Reviewed the complete implementation diff, confirmed that the
  Merkle limit remains private to the parser, and completed the explicit
  simplification pass without expanding the public API or effectful scope.
- Verification: `cargo fmt --all`; `cargo clippy --all-targets --all-features
  -- -D warnings`; `cargo build --all-targets --all-features`; `cargo test
  --all-features`; `bun scripts/bright-builds-check.ts all`; `just
  verify-redaction`; `just test`; `just parity`; `just parity-progress`; and
  `git diff --check` all passed. The unchanged checklist reported 28 of 94
  active rows verified (29.8%) with no validation errors.
- Evidence: The full Rust and Bazel graphs exercise the expanded 23-case
  fixture, client serialization, pool parsing, and both Merkle boundary tests.
- Outcome: The implementation is ready for a source commit while the parity
  checklist remains unchanged.
- Blocker or next safe action: Commit the implementation and worklog, bind the
  result to that immutable source commit, then request the guarded `STR-002`
  transition.
