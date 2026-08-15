# Parity work log

## 2026-08-15T04:28:22Z | immutable-plan draft

- Source commit: `8f21102fd9ea71ec37dd1d09e457e933c60264df`.
- Actions: Selected API-009 after attempt-026 and drafted the software-only
  count-baseline repair contract.
- Verification: Clean synchronized HEAD, no open plan, API-009 first, and the
  sealed categorical attempt boundary were confirmed. Plan gates are pending.
- Evidence: Public source and approved categorical closure facts only; no
  credentials, protected traces, detector, USB, device/network, or controls
  were accessed.
- Outcome: Source editing remains ineligible until this plan is gated, pushed,
  and immutable.
- Blocker or next safe action: Gate and push this plan before adding the red
  regression.

## 2026-08-15T04:41:03Z | plan-only gates passed

- Plan SHA-256:
  `9f0b46f459be5385961aa489525c42361f328effe7338dd18f3a8cd473d66cbb`.
- Actions: Kept the repair software-only and bound it to the single active
  API-009 task; no implementation source or device-facing state changed.
- Verification: Ordered Cargo format, strict lint, all-target build, and full
  tests; Bright Builds; all Bazel tests; parity and progress; redaction;
  reference cleanliness; firmware build; selector; unique-task; immutable-plan
  digest; sensitive-output; and diff checks pass.
- Evidence: Public source, task, plan, and categorical attempt-026 closure
  facts only. No credentials, protected traces, detector, USB, device/network,
  display, mining, hardware-control, UART, or pin interface was accessed.
- Outcome: The plan is eligible to commit and push as the immutable software-
  only implementation boundary. API-009 remains `implemented`.
- Blocker or next safe action: Commit and push this checkpoint, then add the
  red regression before editing production state-machine behavior.

## 2026-08-15T04:42:00Z | paused-count baseline repair complete

- Source commit: `697688f0cda007ed43da380af06e32fa512a4fa2`.
- Actions: Added the red live-shaped regression, captured the positive count
  from the fully joined paused/safe-stopped sample immediately before the sole
  dismissal request, and added a fail-closed zero-count boundary test.
- Verification: The new pause-convergence regression failed against the prior
  source because the state machine retained the earlier notification count,
  then passed after the repair. Twenty-five command-effects tests, all 292
  flash tests, ordered Cargo format/lint/build/test, Bright Builds, all 44
  Bazel tests, parity/progress, redaction, reference cleanliness, firmware
  build, immutable-plan digest, unique-task, selector, sensitive-output, and
  diff checks pass.
- Evidence: Public source and deterministic loopback tests only. No credential
  contents, protected traces, detector, USB, device/network, display, mining,
  hardware-control, UART, or pin interface was accessed.
- Outcome: The software fix is pushed. API-009 remains `implemented`, no
  checklist transition applies, and no live hardware claim is made.
- Blocker or next safe action: Close this software-only plan, then create a
  fresh immutable attempt-027 plan before any device access.
