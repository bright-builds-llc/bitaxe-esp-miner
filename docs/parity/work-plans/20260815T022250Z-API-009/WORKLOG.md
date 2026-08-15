# Parity work log

## 2026-08-15T02:22:50Z | immutable-plan draft

- Source commit: `f6e215b7db0c19fcc17afb83417afe7520e633c9`.
- Actions: Confirmed clean synchronized HEAD, no open plan, API-009 first, and
  the closed attempt-024 reactivation-safety boundary; drafted one
  software-only state-transition plan.
- Verification: Plan-only focused, mandatory, privacy, reference, firmware,
  selector, task, digest, and diff gates are pending before commit.
- Evidence: Public source, selector, task, plan, and categorical attempt-024
  closure facts only. No credential, protected attempt artifact, detector,
  USB, device/network, display, mining, hardware-control, UART, or pin
  interface was accessed.
- Outcome: Implementation remains ineligible until this immutable plan and its
  complete plan-only gate sequence are committed and pushed.
- Blocker or next safe action: Verify, commit, and push the plan before changing
  implementation source.

## 2026-08-15T02:29:17Z | plan-only gates passed

- Plan SHA-256:
  `468995b183a45fd3f639b3ad2004727c8e30cca06bce785768306d614b251331`.
- Actions: Kept the immutable plan and active task within the software-only
  reactivation-safety scope; no implementation source was changed.
- Verification: Focused readiness-recovery and resumable-budget tests passed.
  The complete ordered Cargo format, strict lint, all-target build, and
  all-feature test sequence; Bright Builds; all 44 Bazel tests; parity and
  progress; redaction; reference cleanliness; firmware build; selector;
  unique-task; immutable-plan digest; and diff checks passed.
- Evidence: Public source, task, plan, tests, and categorical attempt-024
  closure facts only. No credential, protected attempt artifact, detector,
  USB, device/network, display, mining, hardware-control, UART, or pin
  interface was accessed.
- Outcome: The plan is eligible to be committed and pushed as the immutable
  implementation boundary. API-009 remains `implemented`.
- Blocker or next safe action: Commit and push this checkpoint, revalidate the
  plan digest and selector, then implement the pure state-transition repair.

## 2026-08-15T02:40:12Z | reactivation-safety repair complete

- Actions: Added one private predicate that distinguishes a stale safety sample
  during post-pause reactivation from initial activation and current active
  mining. The resumable path now safe-stops prepared hardware while preserving
  the lease and accumulated active budget, then accepts a later fresh
  observation for reprepare, reconnect, and active recovery.
- Failure signal: The production-shaped regression first failed because the
  post-preparation stale transition emitted a terminal safe stop instead of
  `ResumablePause`, matching the categorical attempt-024 boundary.
- Verification: The repaired live-shaped test and both terminal negative
  controls pass, along with campaign timing, recovery, focused engine,
  firmware-owner, and production-session verification. The complete ordered
  Cargo sequence, Bright Builds, all 44 Bazel tests, parity, progress,
  redaction, reference cleanliness, and real firmware build pass. A combined
  parity invocation encountered transient host `os error 35`; the exact
  isolated `just parity` retry passed.
- Evidence: Public source, tests, plan, task, and categorical attempt-024 facts
  only. No credential, protected attempt artifact, detector, USB,
  device/network, display, mining, hardware-control, UART, or pin interface was
  accessed.
- Outcome: The software-only plan is complete and API-009 remains
  `implemented`; no live device-user evidence was created.
- Blocker or next safe action: Close, commit, and push this plan. A fresh
  immutable exact-package hardware contract is required before attempt-025.
