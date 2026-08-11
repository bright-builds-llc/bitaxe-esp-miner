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
