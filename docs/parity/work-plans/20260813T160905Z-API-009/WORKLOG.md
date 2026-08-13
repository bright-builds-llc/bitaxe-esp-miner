# Parity work log

## 2026-08-13T16:09:05Z | lost-intent diagnosis

- Source commit: `e2d1fabb6d15dac02a7c395d1966d8077124599f`.
- Actions: Closed and pushed attempt-011, re-ran the clean selector, and traced
  the failed pause from the typed host join through firmware readiness and
  runtime snapshot ownership.
- Verification: The HTTP pause writer, production session publisher, and
  authoritative readiness reader all share the replaceable
  `CommandVisibleState.mining` value. The owner publisher assigns the entire
  value, so it can erase a newer command intent before readiness consumes it.
  No separate requested-intent state exists. Attempt-011 then expired at the
  exact 130-second pause-join deadline with no resumable safe-stop marker.
- Evidence: Public source and already classified redaction-safe attempt-011
  categories only. No raw trace, sensitive field, credential, detector, USB,
  device, or network access occurred.
- Outcome: Root cause confirmed as a command-versus-projection ownership race;
  the stale-safety terminal detail is secondary.
- Blocker or next safe action: Push this software-only plan, then separate the
  requested intent owner and add the exact interleaving regression.

## 2026-08-13T16:12:04Z | immutable plan verification

- Plan SHA-256:
  `092660fe0eac80ac983b6f139c6968dca46f0e387dc7c39e44bb5adc2ae8d83e`.
- Verification: The complete ordered Cargo format, strict Clippy, all-target
  build, and all-feature test sequence passed, followed by Bright Builds,
  canonical Bazel tests, parity, parity-progress, redaction, reference
  cleanliness, the real firmware build, and diff checks. The selector resumes
  only this API-009 plan and `TASKS.md` has exactly one binding to it.
- Outcome: The software-only plan/task checkpoint is ready to commit and push;
  no hardware-capable input or interface was accessed.
- Blocker or next safe action: Push this immutable checkpoint before editing
  the operator-intent ownership boundary.

## 2026-08-13T16:37:57Z | ownership fix verified

- Actions: Added a typed requested-operator-intent owner, routed boot and
  pause/resume writes through it, routed production readiness reads through
  it, and kept session publication limited to the derived mining projection.
  Added exact interleaving and source-ownership regressions.
- Verification: The red ownership regression failed before the fix. After the
  fix, focused intent, production campaign-status, Stratum session, flash
  campaign, and real automation-process tests pass. Cargo format, strict
  Clippy, all-target build, all-feature tests, Bright Builds, all 44 Bazel
  tests, parity, parity-progress, redaction, reference cleanliness, real ESP
  firmware build, and diff checks pass.
- Evidence: Public source/tests and redaction-safe exit status only. No
  credential, protected evidence, detector, USB, device, network, HTTP, or
  hardware interface was accessed.
- Outcome: `software_fix_complete`. API-009 remains `implemented`; no parity
  transition or hardware evidence is claimed.
- Blocker or next safe action: Push the closure and fix. A future attempt-012
  requires a fresh selector and separate immutable one-attempt contract.
