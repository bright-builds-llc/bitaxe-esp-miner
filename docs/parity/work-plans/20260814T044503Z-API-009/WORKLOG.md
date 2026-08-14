# Parity work log

## 2026-08-14T04:45:03Z | immutable-plan draft

- Source commit: `0fdff982c7c2914e1c455c6afd49fa95502f99e4`.
- Actions: Selected API-009 first from the clean synchronized selector and
  drafted one bounded attempt-016 contract for the newly verified serial line-
  admission fix.
- Verification: Plan-only software, privacy, reference, firmware, selector,
  task, immutable-digest, and diff gates are pending before commit.
- Evidence: Public source and task/plan metadata only. No credential,
  detector, protected attempt artifact, USB, device, network, display, mining,
  or hardware interface was accessed.
- Outcome: Attempt-016 remains ineligible until this immutable contract and all
  named plan gates are committed and pushed at clean synchronized HEAD.
- Blocker or next safe action: Run the complete plan gate sequence, review the
  diff, commit and push, then perform exact-package admission before the sole
  detector run.

## 2026-08-14T04:49:02Z | immutable-plan verification

- Plan SHA-256:
  `41dc4d7740f9fcc1ba1ef470fe5274ad5c84bc3bbdd7b7d09289be064f9b3aa1`.
- Actions: Ran the complete plan-only gate sequence and queried the selector
  against the unique API-009 task binding.
- Verification: Formatting, strict Clippy, all-target build, all-feature tests,
  Bright Builds, all 44 canonical Bazel test targets, parity, parity-progress,
  focused line-admission/device-session/flash/automation/real-process tests,
  redaction, reference cleanliness, the real ESP firmware build, task
  uniqueness, selector ownership, immutable digest, and diff checks pass.
- Evidence: Public plan/task/source and category-only gate outcomes. No
  credential, protected attempt artifact, detector, USB, device, network,
  display, mining, or hardware interface was accessed.
- Outcome: The exact-package, detector-gated attempt-016 contract is ready to
  commit and push without changing API-009 from `implemented`.
- Blocker or next safe action: Push this checkpoint, confirm clean synchronized
  HEAD and the same open plan, then perform exact-package admission before the
  sole detector run.

## 2026-08-14T11:56:52Z | attempt-016 terminal closure

- Source commit: `a3732dcb3bb209f4d9fd3993997a80f87815b4b0`.
- Actions: Built and admitted the exact package, ran the protected detector
  exactly once, and started the sole attempt-016 campaign. The campaign reached
  the operator-ready checkpoint but received no timely physical-ready report.
- Verification: The private result records trusted runtime identity, 286 valid
  markers, confirmed safe stop, ready USB cleanup, and the typed terminal facts
  `network_correlation_failed` / `safety_prerequisites_stale`. Private roots and
  files retain modes `0700` and `0600`; no campaign process remains and the
  public projection is absent.
- Evidence: Only redacted category, count, digest, lifecycle, and mode facts are
  recorded publicly. The later ready confirmation was written after campaign
  closure and was never consumed; no display observation is inferred from it.
- Outcome: Attempt-016 is consumed. API-009 remains `implemented`; no physical
  IDENTIFY, dismissal, restart, or complete device-user quorum is claimed.
- Blocker or next safe action: Close this immutable plan without attempt-017 or
  an unchanged retry. Any future attempt requires new objectively verified
  progress and a separate clean immutable plan.
