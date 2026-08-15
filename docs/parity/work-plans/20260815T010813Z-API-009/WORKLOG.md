# Parity work log

## 2026-08-15T01:08:13Z | immutable-plan draft

- Source commit: `0fbcf9c041b38d28a68b3343806912a91b979f6b`.
- Actions: Confirmed clean synchronized HEAD, no open plan, API-009 first, and
  the attempt-023 resume-correlation closure; drafted one software-only
  resume, active-budget, and recovery-safe-stop plan.
- Verification: Plan-only focused, mandatory, privacy, reference, firmware,
  selector, task, digest, and diff gates are pending before commit.
- Evidence: Public source, selector, task, plan, and categorical closure facts
  only. No credential, protected attempt artifact, detector, USB,
  device/network, display, mining, hardware-control, UART, or pin interface was
  accessed.
- Outcome: Source implementation remains ineligible until this immutable plan
  and its complete plan-only gate sequence are committed and pushed.
- Blocker or next safe action: Verify, commit, and push the plan before changing
  implementation source.

## 2026-08-15T01:22:00Z | plan-only gates passed

- Plan SHA-256:
  `fa18d647af689334a2a15a521bc6c5738fdd8e2bf8e9ed0220706b1420ad6228`.
- Actions: Kept the immutable plan and active task within software-only scope;
  no implementation source was changed before verification.
- Verification: Focused resumable-campaign, command-effects, and automation
  tests passed. The mandatory Cargo format, lint, build, and test sequence;
  Bright Builds checks; repository Bazel tests; parity and progress reports;
  redaction and reference checks; firmware build; and diff check all passed.
- Evidence: Public source, task, plan, test, and categorical prior-closure facts
  only. No credential, protected attempt artifact, detector, USB,
  device/network, display, mining, hardware-control, UART, or pin interface was
  accessed.
- Outcome: The plan is eligible to be committed and pushed as the immutable
  implementation boundary. API-009 remains `implemented`.
- Blocker or next safe action: Commit and push this checkpoint, revalidate the
  plan digest and selector, then implement the software repair.

## 2026-08-15T01:50:32Z | implementation and closure gates passed

- Plan SHA-256:
  `fa18d647af689334a2a15a521bc6c5738fdd8e2bf8e9ed0220706b1420ad6228`.
- Actions: Added active-segment lease accounting, separate resume-intent and
  bounded reactivation states, one-shot correlated recovery pause, closed
  evidence v6, and behavior-focused engine, firmware, host, automation, and
  real-child regressions. Added the new Rust module to Bazel's explicit source
  graph after the first complete run exposed that missing edge. Final review
  moved the recovery serial baseline before the blocking pause request so a
  safe-stop marker emitted while that request is in flight remains admissible.
- Verification: Focused targets passed. The complete ordered Cargo format,
  strict lint, all-target build, and all-feature test sequence; Bright Builds;
  all 44 Bazel tests; parity and progress; redaction; reference cleanliness;
  firmware build; plan digest; and diff checks passed.
- Evidence: Public source, tests, task, plan, closure, and categorical prior
  attempt facts only. No credential, protected attempt artifact, detector,
  USB, device/network, display, mining, hardware-control, UART, or pin
  interface was accessed.
- Outcome: The software repair is complete and API-009 remains `implemented`.
  This plan creates no hardware evidence and does not authorize attempt-024.
- Blocker or next safe action: Close this plan, commit and push the verified
  source, then create a fresh immutable exact-package hardware plan before any
  detector or device access.
