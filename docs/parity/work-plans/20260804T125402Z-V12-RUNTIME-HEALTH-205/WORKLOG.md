# Parity work log

## 2026-08-04T12:54:02Z | selection and plan

- Source commit: `74ef3921352381356e13ec08f9249f3a32f976b9`.
- Actions: Continued the deterministic advance-parity loop after closing the
  operator-snapshot row, confirmed no open plan, evaluated remaining candidate
  constraints, and selected the narrow passive runtime-health correction.
- Evidence: Phase 36 typed projection records `runtime_health: null` and the
  decision matrix records exact reason `runtime_health_insufficient`; current
  firmware produces coherent HTTP/WebSocket/retained runtime-health values.
- Outcome: Immutable plan and task-gated one-attempt contract ready.
- Blocker or next safe action: Run the mandatory planning-commit gate, commit
  the task and plan, then implement without changing `PLAN.md`.

## 2026-08-04 | typed passive capture implementation

- Source commit: implementation based on planning commit `9788a681`.
- Actions: Added `capture-runtime-health-evidence`, a Rust-owned v1 evidence
  contract and validator, strict detector handoff, private exact-package
  flash-monitor/API/WebSocket/retained-log capture, full retained tuple binding,
  healthy checkpoint and task-watchdog admission, prepublication validation,
  and bounded exact-package recovery with primary-failure precedence.
- Privacy and safety: The workflow performs no restart, self-test, setting,
  mining, or hardware-control request. Public evidence is closed to digests,
  bounded revisions/sequences, booleans, and state labels; private artifacts
  remain mode `0600` beneath a mode-`0700` root.
- Verification: Focused Rust validator and automation builds passed. Focused
  contract and automation tests passed, including healthy tuple admission,
  stale health rejection, missing retained tuple, failed recovery precedence,
  sensitive-value denial, and a real child-process boundary.
- Outcome: Implementation ready for the mandatory full software gate.
- Blocker or next safe action: Run all repository gates, commit and push only
  if clean, then execute the single task-authorized passive hardware capture.

## 2026-08-04 | mandatory implementation gate

- Source commit: implementation based on planning commit `9788a681`.
- Actions: Closed the remaining behavior-test gaps for WebSocket boot-session
  mismatch, health-sequence regression, and final-validator rejection, then
  ran the full repository verification sequence in the required order.
- Verification: `cargo fmt --all`, strict Clippy, all-target/all-feature Cargo
  build and tests, Bright Builds checks, 28 Bazel test targets through
  `just test`, parity validation and progress, redaction, reference cleanliness,
  and `git diff --check` all passed. Parity remains 36 of 94 active rows before
  this hardware attempt.
- Outcome: Software implementation is ready to commit and push; no hardware
  interaction has occurred under this task yet.
- Blocker or next safe action: Re-run the mandatory gate after recording this
  checkpoint, commit and push the exact implementation, then run the one
  detector-gated `attempt-001` capture.

## 2026-08-04 | detector-gated attempt-001 complete

- Source commit: `56f4eb1ce50f81fe2a1dd80ac344e551b16771c9`.
- Actions: Built the exact clean package, admitted exactly one Ultra 205 with
  the private detector, and ran the sole authorized passive runtime-health
  capture. No retry or recovery flash was used.
- Evidence: The closed v1 projection joins same-boot HTTP revision 614 and
  WebSocket revision 615 with both exact retained tuples. Checkpoint sequences
  advanced from 3347 to 3348; watchdog feed sequence remained non-regressing at
  734. Supervisor and watchdog facts were healthy and fresh, mining and hardware
  control remained disabled, cleanup completed, and redaction passed.
- Verification: Independent Rust validation, exact-source assertions, private
  root/file mode checks, a sensitive-token scan, `just verify-redaction`, and
  `git diff --check` passed.
- Outcome: `complete`; the evidence satisfies the exact promotion contract for
  `V12-RUNTIME-HEALTH-205`.
- Blocker or next safe action: Create the bound result and transition receipt,
  synchronize progress, archive this completed task, then run all final gates.
