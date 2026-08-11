# Parity work log

## 2026-08-11T19:08:28Z | selection and plan checkpoint

- Source commit: `1714c70fdc5dd94315b17b743d2385ab1ddbeccf`
- Actions: Loaded the active task and checklist state, ran the deterministic
  selector, audited every earlier candidate, and designed a passive
  exact-package retained-download/raw-stream correlation contract.
- Verification: Clean synchronized `main`, clean pinned reference, no open
  parity plan, the ordered Rust format/Clippy/build/test sequence, Bright
  Builds, all 36 Bazel tests, parity/progress, redaction, and reference checks
  passed.
- Evidence: Immutable plan and matching active task contract.
- Outcome: `LOG-001` selected as the first actionable row.
- Blocker or next safe action: Commit and push this checkpoint, then implement
  the typed capture without editing `PLAN.md`.

## 2026-08-11T19:24:48Z | typed capture implementation

- Source commit: pending implementation commit.
- Actions: Added the typed `bitaxe-log-buffer-evidence-v1` contract and
  independent Rust validator, a private-first exact-package capture command,
  exact response-header and retained-body correlation, raw text WebSocket
  capture, no-clobber/mode/privacy enforcement, CLI and Just surfaces, and
  focused behavior and real-child-process regressions.
- Verification: Focused Cargo and Bazel tests passed. The complete ordered
  `cargo fmt --all`, strict Clippy, all-target build, all-feature test, Bright
  Builds, all 36 Bazel tests, parity/progress, redaction, and pinned-reference
  checks passed before the implementation commit.
- Evidence: Software-only typed capture and regression suite; no public live
  evidence has been published.
- Outcome: Implementation is ready for the exact clean pushed-package gate.
- Blocker or next safe action: Review the complete diff, commit and push the
  implementation, then run the immutable plan's one detector and conditional
  one capture without retry.

## 2026-08-11T19:36:30Z | exact-package hardware evidence

- Source commit: `f1aca309239d38c1764992794cab2aa80832d037`.
- Actions: Built the exact clean package, ran the sole detector, and ran the
  sole conditional log-buffer capture. The detector admitted one Ultra 205;
  the capture performed one exact-package flash, two retained-log downloads,
  and one receive-only raw WebSocket connection.
- Verification: Exact package and same-boot identity, passive safe state,
  exact download headers, plain-text frame type and marker, baseline-prefix
  preservation, a marker-count increase of exactly one, private modes,
  cleanup, and the Rust validator all pass. The aggregate projection records
  190647 baseline bytes, 192602 final bytes, a 31-byte frame, and marker counts
  zero then one.
- Evidence: Public aggregate-only projection at
  `docs/parity/evidence/log001-retained-stream/log-buffer-projection.json`;
  all detector, credential, port, origin, network, serial, HTTP, WebSocket,
  retained-log, and process material remains in ignored protected roots.
- Outcome: The single attempt passed and supports promoting only `LOG-001` to
  `verified`.
- Blocker or next safe action: Close the semantic-redaction admission check,
  run the final evidence gate, and commit the evidence before transition.

## 2026-08-11T19:37:04Z | semantic redaction admission correction

- Source commit: `f1aca309239d38c1764992794cab2aa80832d037` plus the reviewed
  evidence-admission correction.
- Actions: Added the new log-buffer schema to the generic semantic-redaction
  scanner, required the capture to scan its private candidate before future
  publication, and added a regression rejecting operational device fields.
- Verification: The focused automation suite passes, the independent Rust
  validator accepts the absolute public projection path, and `just
  verify-redaction` now checks seven schemas and accepts the captured
  projection. No hardware action was repeated.
- Evidence: The unchanged public projection and the scanner regression.
- Outcome: All promotion evidence gates are closed.
- Blocker or next safe action: Run the complete final evidence gate, commit and
  push the evidence source, then transition only `LOG-001` and synchronize
  deterministic progress.

## 2026-08-11T19:40:14Z | verified transition checkpoint

- Source commit: `0389eebc51b0a9d77596e10963bd8e386350e098`.
- Actions: Committed and pushed the closed evidence source, transitioned only
  `LOG-001` to `verified`, and synchronized deterministic progress from that
  exact commit.
- Verification: The first uncommitted transition rendering omitted required
  Markdown code spans from the Rust-owned target cell, and `just parity`
  rejected it. The generated checklist, receipt, progress, and README changes
  were restored before re-running the same transition ID with valid code
  spans. Transition `20260811T193954Z-LOG-001` is now recorded, the checklist
  validates, and exactly one progress record reports 44 of 94 active rows
  (46.8%).
- Evidence: The committed result and aggregate projection are authoritative;
  no private artifact was added and no hardware action was repeated.
- Outcome: `LOG-001` is verified.
- Blocker or next safe action: Archive only the completed LOG-001 task, run the
  mandatory finalization gate and diff checks, commit, fetch, and push.

## 2026-08-11T19:46:45Z | finalization gate

- Source commit: `0389eebc51b0a9d77596e10963bd8e386350e098`.
- Actions: Archived only the completed LOG-001 task and ran the complete
  finalization gate after the corrected transition and progress sync.
- Verification: Cargo format, strict Clippy, all-target build, all-feature
  tests, Bright Builds, all 36 Bazel tests, parity/progress, semantic
  redaction, and pinned-reference checks pass. Progress is 44 of 94 active
  rows (46.8%).
- Evidence: The committed result, projection, transition receipt, checklist,
  and progress record form the closed public chain.
- Outcome: LOG-001 finalization is ready to commit and push.
- Blocker or next safe action: None for this parity row.
