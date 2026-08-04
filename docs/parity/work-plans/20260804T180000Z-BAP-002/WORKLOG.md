# BAP-002 worklog

## 2026-08-04T18:00:00Z | plan created

- Source commit: `ff2d1114c7e08868ffd58ce4cb523dfcca81364d`.
- Actions: Selected `BAP-002` through the deterministic parity selector and
  inspected the pinned protocol vocabulary, checksum, parser compatibility,
  request handlers, subscription admission, setting validation, and current
  `bitaxe-core` ownership boundary.
- Verification: Confirmed the ten command tokens, eighteen parameter tokens,
  256-byte bound, XOR framing, checksum-free subscription compatibility,
  one-second duplicate window, AP-mode error values, request projections, and
  safe pure/imperative split.
- Evidence: `PLAN.md`, pinned reference breadcrumbs, current Rust/Bazel source,
  and the active parity checklist.
- Outcome: The pure protocol core is actionable without an accessory, UART,
  credentials, network activity, hardware control, or device interaction.
- Blocker or next safe action: Commit the immutable plan/task checkpoint, then
  implement the pure wire contract before handler decisions.

## 2026-08-04T18:20:00Z | pure protocol implementation complete

- Source commit: implementation tree based on `05bc0b81`.
- Actions: Added the exact BAP command and request-parameter vocabularies,
  bounded canonical wire encoder/parser, asymmetric subscription checksum
  compatibility, redacted one-second duplicate admission, connected/AP command
  planning, request projections, subscription decisions, and setting intents.
- Verification: Twelve focused Cargo tests and the `bitaxe-core` Bazel target
  pass. The first Bazel attempt rejected a misplaced `compile_data` attribute;
  moving BAP sources and fixtures from the identity filegroup to the Rust
  library fixed the ownership boundary, and the same target then passed.
- Evidence: Synthetic golden frames with independent provenance, fixed checksum
  vectors, malformed-frame categories, exact response tokens, AP-mode errors,
  setting boundaries, and diagnostic redaction assertions.
- Outcome: The pure protocol is complete without a firmware adapter, UART,
  accessory, credential use, persistence, restart, or hardware effect. The
  simplification pass keeps wire and semantics in separate sub-500-line modules.
- Blocker or next safe action: Run the ordered mandatory Rust sequence and all
  repository-wide parity/privacy/reference gates, then commit only if clean.

## 2026-08-04T18:40:00Z | mandatory implementation gate passed

- Source commit: implementation tree based on `05bc0b81`.
- Actions: Completed the simplification pass, renamed the no-handler category
  to avoid misclassifying the host-origin `CMD` token, and added an exact
  nine-value public error-vocabulary regression.
- Verification: Twelve focused BAP tests, focused Bazel and strict Clippy, the
  ordered Rust sequence, Bright Builds with zero findings, all 30 Bazel tests,
  the ESP32-S3 artifact build, parity/progress, redaction, reference cleanliness,
  sensitive-value review, file-size review, and diff checks pass.
- Evidence: Fixed synthetic wire vectors, closed malformed categories, exact
  custom response parameter tokens, pure setting intents, and raw-value Debug
  denial assertions.
- Outcome: The implementation is ready for an exact source commit and an
  `implemented` transition only. No live accessory or firmware UART evidence
  was inferred.
- Blocker or next safe action: Commit the implementation, bind `RESULT.md` to
  that exact commit, apply the typed `BAP-002` transition, and publish metadata.

## 2026-08-04T18:50:00Z | implementation committed and transitioned

- Source commit: `c88ae0470e41716f011be2162d0a084bd1ac02c4`.
- Actions: Bound `RESULT.md` to the exact implementation and reference commits,
  applied typed transition `20260804T185000Z-BAP-002`, and synchronized the
  progress ledger from the same source commit.
- Verification: The transition tool accepted the conservative `implemented`
  status, `unit,golden` evidence, exact pure-core ownership targets, immutable
  plan, and commit-bound result. The progress ledger remains at 39 of 94 active
  rows verified (41.5%).
- Evidence: `RESULT.md`, the typed transition receipt, updated checklist, and
  synchronized progress ledger.
- Outcome: `BAP-002` is implemented without a live accessory or firmware UART
  claim. No hardware or sensitive runtime value was used or published.
- Blocker or next safe action: Re-run the required commit gates for the metadata
  tree, publish it, and resume deterministic parity selection.
