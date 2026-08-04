# Parity work log

## 2026-08-04T22:25:59Z | attempt-006 software-remediation contract

- Source commit: `dc2ea737236a8f0bff3b225218a7cd7cc6d29bc9`.
- Actions: Resumed only `API-010` after the pushed attempt-005 stop and reduced
  the protected panic evidence to a source-scoped remediation hypothesis.
- Verification: Attempt-005 proves the exact flash effect completed, every
  observed reset was `panic`, the allowlisted panic category was stack
  overflow, and the failure cadence aligns with the first 10-second replay
  owned by the 8 KiB boot-evidence observer.
- Evidence: Closed categories, booleans, bounded counts, public source
  provenance, and the pushed prior checkpoint only. Protected trace material
  and local hardware/network values remain private.
- Outcome: A minimal 16 KiB observer-stack change plus an ownership regression
  is actionable, but the hypothesis remains unverified until software gates
  and one new exact-package hardware attempt pass.
- Blocker or next safe action: Verify, commit, and push this immutable
  plan/task checkpoint before editing firmware or build files.

## 2026-08-04T22:30:38Z | pre-implementation plan gate complete

- Source commit: `dc2ea737236a8f0bff3b225218a7cd7cc6d29bc9`.
- Actions: Ran the complete repository gate against the new immutable
  attempt-006 plan and active task without editing implementation files.
- Verification: Formatting, strict Clippy, all-target/all-feature build, all
  Cargo tests, Bright Builds, all Bazel tests, parity validation/progress,
  semantic redaction, pinned-reference cleanliness, and diff checks passed.
- Evidence: Public software outcomes only; no new package, detector, device,
  credential, or hardware action occurred.
- Outcome: The attempt-006 remediation contract is ready to commit and push
  without amendment.
- Blocker or next safe action: Commit and push this checkpoint, then implement
  only the scoped observer stack budget and ownership regression.

## 2026-08-04T22:33:54Z | observer stack remediation implemented

- Source commit: `84ab62c47ed6f4a4e010ddf970863fd7be1bc499`.
- Actions: Added the focused boot-evidence source-ownership target, observed it
  fail against the 8 KiB stack contract, raised only the observer budget to
  16 KiB, and reran the same target successfully.
- Verification: The regression proves one explicit budget declaration, one
  `.stack_size` use, one boot-lifetime observer spawn, and unchanged
  10-second identity replay. The canonical ESP32-S3 firmware image builds.
  Formatting, strict Clippy, all-target/all-feature build, all Cargo tests,
  Bright Builds, all 35 Bazel tests, parity validation/progress, semantic
  redaction, and pinned-reference cleanliness pass.
- Evidence: Red/green host regression, public source diff, and successful
  software build/gate outcomes only. No device or credential action occurred.
- Outcome: The minimal source-level fix is software-clean; live evidence is
  still required before the stack-overflow hypothesis or API-010 is verified.
- Blocker or next safe action: Commit and push the clean implementation, then
  run only the plan-recorded package, detector, and conditional attempt-006
  capture.
