# Parity work log

## 2026-08-14T20:05:47Z | immutable-plan draft

- Source commit: `d2e0e03f941436c541a9fff6ff5ed4d4109c7513`.
- Actions: Ran the clean synchronized selector, selected API-009 first, and
  drafted a software-only plan for one explicit replayable IDENTIFY window.
- Verification: Plan-only mandatory, privacy, reference, firmware, selector,
  task, immutable-digest, and diff gates are pending before commit.
- Evidence: Public source and attempt-021's public declined/cleanup result only.
  No credential, protected attempt content, USB, device, network, display,
  mining, or hardware interface was accessed.
- Outcome: Implementation remains ineligible until this immutable contract and
  all named plan gates are committed and pushed at clean synchronized HEAD.
- Blocker or next safe action: Run the plan-only gate sequence, review the diff,
  commit and push, then build the deterministic red feedback loop.

## 2026-08-14T20:10:07Z | immutable-plan verification

- Plan SHA-256:
  `78bbac00fd79966db1f23fc5b6013b13f57f74f385b63bf032da65f85f429daf`.
- Actions: Ran the complete plan-only gate sequence and confirmed the selector
  resumes this unique API-009 replay-protocol plan.
- Verification: Formatting, strict Clippy, all-target build, all-feature tests,
  Bright Builds, all 44 Bazel test targets, parity, parity-progress, redaction,
  reference cleanliness, real ESP firmware build, task uniqueness, open-plan
  selection, immutable digest, sensitive-pattern scan, and diff checks pass.
- Evidence: Public plan/task/source and category-only gate outcomes. No
  credential, protected attempt content, USB, device, network, display, mining,
  or hardware interface was accessed.
- Outcome: The software-only replay implementation is ready to commit and push
  without changing API-009 from `implemented`.
- Blocker or next safe action: Push this checkpoint, confirm clean synchronized
  HEAD and the same open plan, then create the fast red production-shaped
  feedback loop before inspecting implementation details.

## 2026-08-14T20:30:02Z | replay diagnosis and implementation

- Source commit: `a213ee33f64fad84dd88cb60fcd4321ec8d74704`.
- Actions: Added the typed `replay` rendered-checkpoint response, one queued
  replay phase, a distinct replayed observation checkpoint, replay-bound
  evidence counts, and matching real-process supervisor validation. Split the
  checkpoint file protocol into a focused private module.
- Diagnosis: The one-shot state machine had no replay outcome. A safe repair
  also required separate confirmation and replay-not-before boundaries: HTTP
  response latency must be allowed to shorten the admissible observation
  window, but must never allow the second toggle while the first physical
  effect may still be active.
- Verification: The parser regression failed twice before the typed outcome
  existed, then passed. A production-shaped loopback HTTP test now proves a
  missed first window sends no early request, issues exactly one replay at the
  safe boundary, accepts its timely confirmation, and closes with three total
  IDENTIFY requests. All 281 pre-regression flash tests and the canonical
  real-child-process automation target pass; final mandatory gates are pending.
- Evidence: Software tests and public source only. No credential, protected
  attempt content, detector, USB, device, network, display, mining, or hardware
  interface was accessed.
- Outcome: The smallest safe replay protocol is implemented without changing
  API-009's `implemented` status or creating hardware evidence.
- Blocker or next safe action: Run the complete final gate sequence from the
  finished tree, review the final diff, and add a non-verifying closure.

## 2026-08-14T20:48:55Z | final software closure verification

- Source commit: `a213ee33f64fad84dd88cb60fcd4321ec8d74704`.
- Actions: Completed the simplification pass, split checkpoint file ownership
  from the command driver, split real-child automation tests to satisfy the
  enforced file-length boundary, declared the new Rust module in the Bazel
  graph, and added the non-verifying closure and task completion review.
- Verification: Focused parser, timing, loopback HTTP, CLI, evidence mismatch,
  and canonical real-process tests pass. Formatting, strict Clippy, all-target
  build, all-feature tests, Bright Builds, all 44 Bazel tests, parity,
  parity-progress, redaction, reference cleanliness, real ESP firmware build,
  immutable plan digest, task uniqueness, selector closure, sensitive-pattern
  review, and diff checks pass. One combined-run parity invocation encountered
  transient macOS resource exhaustion after all preceding gates; an isolated
  rerun passed with `validation_errors: none`, and parity-progress passed.
- Evidence: Public source, deterministic fixtures, loopback HTTP, and
  category-only gate outcomes. No credential, protected attempt content,
  detector, USB, device, network, display, mining, or hardware interface was
  accessed.
- Outcome: The replay protocol and its closed evidence contract are complete.
  API-009 remains `implemented`; the selector reports no open plan, no hardware
  evidence exists, and attempt-022 did not run.
- Blocker or next safe action: Commit and push this closed software lifecycle.
  A later hardware attempt requires its own immutable exact-package contract.
