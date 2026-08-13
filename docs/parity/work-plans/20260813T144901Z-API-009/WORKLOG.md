# Parity work log

## 2026-08-13T14:49:01Z | selection and diagnosis checkpoint

- Source commit: `6f99365b9c72a4c0fed42fcc4eb8820dd4afd54f`.
- Actions: Selected API-009 first and traced IDENTIFY from HTTP planning through
  retained state, atomic screen projection, 500-ms screen flow, retained I2C
  display ownership, and SSD1306 frame flush. Audited the host checkpoint order
  against the pinned 30-second effect.
- Verification: The host currently sends IDENTIFY before operator readiness and
  emits only a non-descriptive `rendered` signal. The later normal-screen report
  occurred after the effect window and cannot classify physical rendering.
- Evidence: Current production source, public attempt-010 closure, and timing of
  the live checkpoint conversation only. No protected attempt content,
  credential, package, detector, or hardware interface was accessed.
- Outcome: API-009 remains `implemented`; a software-only v2 checkpoint fix is
  actionable and attempt-011 remains prohibited.
- Blocker or next safe action: Commit and push this immutable plan/task/lesson
  checkpoint before editing implementation or correcting the prior closure.

## 2026-08-13T15:08:00Z | immutable-plan verification checkpoint

- Plan SHA-256:
  `eae200d52f71f0c32ffc53054f076424f1c0b25b2c0fd6cd84f56057b5cc950d`.
- Verification: `cargo fmt --all`, strict Clippy, all-target build, all-feature
  tests, Bright Builds checks, `just test`, parity validation and progress,
  redaction validation, reference cleanliness, firmware build, and
  `git diff --check` all exited successfully.
- Outcome: The software-only plan is ready for its immutable commit. No device,
  network, credential, package, detector, or hardware interface was accessed.

## 2026-08-13T16:02:00Z | implementation and focused verification checkpoint

- Actions: Added the private v2 `ready` / `rendered` / `cleared` transaction,
  gated the first IDENTIFY request on consumed readiness, added the readiness
  quorum to v2 command evidence, renamed the confirmation flag to
  `--checkpoint`, and made every public v2 signal self-describing.
- Regression evidence: The production-boundary Rust test proves readiness is
  armed with zero IDENTIFY requests and no rendered checkpoint. Request-once,
  malformed, missing, mode, order, projection, redaction, and real-child
  supervisor tests pass through the canonical flash and automation targets.
- Historical correction: Attempt-010 now closes as
  `stop_authority_boundary`; its late report does not classify earlier display
  rendering, and all sealed hardware and cleanup facts remain intact.
- Outcome: The root mismatch is fixed without hardware access or a row-status
  transition.

## 2026-08-13T16:18:00Z | full verification checkpoint

- Verification: `cargo fmt --all`, strict Clippy, all-target/all-feature build,
  all-feature tests, Bright Builds checks, all 42 Bazel tests, parity report and
  progress, redaction, pinned-reference cleanliness, real firmware build, plan
  digest, and diff checks all pass.
- Result: `software_fix_complete`; API-009 remains `implemented`, public parity
  evidence remains withheld, and attempt-011 remains outside this plan.
- Device effects: None. No credential, package, detector, USB, network/device,
  mining, HTTP, restart, recovery, OTA, UART, or pin interface was accessed.
