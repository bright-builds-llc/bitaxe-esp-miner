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

## 2026-08-14T03:40:50Z | red-green implementation checkpoint

- Source commit: `bdbc2d5b866db612b2ac1b95b3f485c2cf69ca60` plus the uncommitted
  changes described here.
- Actions: Added the requested-intent, campaign-status, and production-owner
  regressions before implementation. The focused target failed because no
  command-effects lease bootstrap existed. Added one narrow requested-intent
  bootstrap, a pure tracker eligibility predicate, and one adapter call after
  admitted tracker construction and before owner readiness.
- Verification: The exact red compile failure changed to green. Focused tests
  prove the disabled boot preference is replaced only for admitted command
  effects, the first active snapshot remains runnable, explicit pause and
  resume overwrite the request, consumed leases force pause, ordinary
  campaigns do not opt in, and the bootstrap cannot mutate projected mining,
  settings, or NVS. Campaign-status, requested-intent, source-ownership,
  Stratum, flash, automation, real-process fixture, and real ESP firmware
  targets pass.
- Evidence: Public source, test outcomes, and the exact compiler failure. No
  credential, detector, protected attempt artifact, USB, device, network,
  display, mining, or hardware interface was accessed.
- Outcome: The attempt-014 startup-intent race is fixed at the current-boot
  requested-intent owner without a second state machine or timing workaround.
- Blocker or next safe action: Run the complete final gate sequence, review the
  full diff, and close this software-only plan with API-009 still
  `implemented` and attempt-015 unauthorized.

## 2026-08-14T03:46:49Z | software closure checkpoint

- Source commit: `bdbc2d5b866db612b2ac1b95b3f485c2cf69ca60` plus the final
  software changes described here.
- Actions: Kept the bootstrap at the requested-intent owner, removed an empty
  test shell, and completed the explicit simplification pass. No timer, delay,
  persistent write, second state machine, or compatibility path was added.
- Verification: The red regression and focused campaign-status,
  requested-intent, source-ownership, Stratum, flash, automation,
  real-process, and firmware targets pass. The final closure gate sequence and
  complete diff review are recorded in `CLOSURE.md`.
- Evidence: Public source, tests, plan digest, and category-only gate outcomes.
  No credential, detector, protected attempt artifact, USB, device, network,
  display, mining, or hardware interface was accessed.
- Outcome: The command-effects current-boot startup-intent defect is fixed in
  software while persisted mine-on-boot and explicit operator command
  ownership remain unchanged.
- Blocker or next safe action: Commit and push this closure, restore a clean
  synchronized selector, and require a separate immutable contract before any
  later hardware ordinal. API-009 remains `implemented`.
