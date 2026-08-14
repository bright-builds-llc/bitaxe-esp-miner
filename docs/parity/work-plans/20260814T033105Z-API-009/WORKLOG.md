# Parity work log

## 2026-08-14T03:31:05Z | deterministic diagnosis checkpoint

- Source commit: `bb1b66b4cf5104290a67f424f2f5abb00c05e779`.
- Actions: Closed and pushed attempt-014, restored a clean synchronized
  selector, joined its sealed category/boolean result to the production
  campaign-status owner, and ran the focused host test at the matching seam.
- Verification: The hardware result retained two milliseconds active before
  `operator_paused`; the source forces `Run` only until `active_seen`, and the
  passing host test explicitly changes the unchanged paused boot request back
  to `Paused` immediately after the first active snapshot. HTTP command routes
  start only after the production owner, so no explicit pause command can have
  produced this transition.
- Evidence: Public source and tests plus redaction-safe sealed categories. No
  credential, protected raw trace, USB identity, network identity, device,
  display, or hardware interface was accessed during this diagnosis.
- Outcome: Root cause is the command-effects initial requested-intent ownership
  edge, not flashing, package identity, protocol admission, or the fixed
  resumable epoch.
- Blocker or next safe action: Verify, commit, and push this immutable
  software-only plan before adding the red regression. Attempt-015 remains
  unauthorized.

## 2026-08-14T03:36:35Z | immutable-plan verification

- Plan SHA-256:
  `06538c8bf54f6b91474b3b24facb8127b2c5de16e60af34615f6f25f214e53a8`.
- Actions: Ran the complete plan-only gate sequence and queried the selector
  against the unique API-009 task binding.
- Verification: Formatting, strict Clippy, all-target build, all-feature tests,
  Bright Builds, all 44 canonical Bazel test targets, parity,
  parity-progress, focused campaign-status/requested-intent/source-ownership
  tests, redaction, reference cleanliness, the real ESP firmware build, task
  uniqueness, selector ownership, immutable digest, and diff checks pass.
- Evidence: Public plan/task/source and category-only gate outcomes. No
  credential, protected attempt artifact, detector, USB, device, network,
  display, mining, or hardware interface was accessed.
- Outcome: The software-only startup-intent contract is ready to commit and
  push without changing API-009 from `implemented`.
- Blocker or next safe action: Push this checkpoint, confirm clean synchronized
  HEAD and the same open plan, then add the failing regression before the
  production fix.
