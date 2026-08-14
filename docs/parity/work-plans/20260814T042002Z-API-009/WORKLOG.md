# Parity work log

## 2026-08-14T04:20:02Z | deterministic diagnosis checkpoint

- Source commit: `f8a0855713d65930ab9575804e93300d84b07678`.
- Actions: Closed and pushed attempt-015, restored a clean synchronized
  selector, joined its bounded serial diagnostics to the receive-only session
  and campaign fan-out source, and drafted one software-only fix contract.
- Verification: The first malformed marker is the first serial event and is
  shorter than every later accepted marker. The USB session directly forwards
  the first post-open bytes and resets no line boundary; the campaign callback
  then gives that same unadmitted chunk to both analyzers.
- Evidence: Public source and sealed category/count/length/boolean diagnostics.
  No credential, raw trace, detector, USB, device, network, display, mining,
  or hardware interface was accessed during this diagnosis.
- Outcome: Root cause is the receive-session line-admission gap, not package,
  flashing, runtime identity, protocol admission, safe stop, or cleanup.
- Blocker or next safe action: Verify, commit, and push this immutable
  software-only plan before adding the red regression. Attempt-016 remains
  unauthorized.

## 2026-08-14T04:24:55Z | immutable-plan verification

- Plan SHA-256:
  `b361b90301e2cf37a1b207aa0d457c43629466d8578b78bbfe9b3420a0ab8602`.
- Actions: Ran the complete plan-only gate sequence and queried the selector
  against the unique API-009 task binding.
- Verification: Formatting, strict Clippy, all-target build, all-feature tests,
  Bright Builds, all 44 canonical Bazel test targets, parity, parity-progress,
  focused device-session/flash/automation/real-process tests, redaction,
  reference cleanliness, the real ESP firmware build, task uniqueness,
  selector ownership, immutable digest, and diff checks pass.
- Evidence: Public plan/task/source and category-only gate outcomes. No
  credential, protected attempt artifact, detector, USB, device, network,
  display, mining, or hardware interface was accessed.
- Outcome: The software-only receive-ingress contract is ready to commit and
  push without changing API-009 from `implemented`.
- Blocker or next safe action: Push this checkpoint, confirm clean synchronized
  HEAD and the same open plan, then add the failing receive-session regression
  before production code.

## 2026-08-14T04:30:32Z | red-green implementation checkpoint

- Source commit: `daba5d8a7a504b20f41b8a5e11c2d9d923940ca2` plus the uncommitted
  software changes described here.
- Actions: Added four line-boundary/reopen regressions before implementation.
  The focused target failed because the receive-line admission owner did not
  exist. Added one allocation-free resettable owner and applied its admitted
  suffix once at the USB session's ephemeral callback boundary.
- Verification: The exact red compile failure changed to green. Focused tests
  prove the initial line is discarded, a boundary split across chunks waits,
  later chunks pass unchanged, reader reopen resets admission, and the real
  receive-session seam resets after open and admits before callback. Existing
  strict flash marker tests, automation, command-effects real-process, and
  device-session targets pass together.
- Evidence: Public source, test outcomes, and the exact compiler failure. No
  credential, protected attempt artifact, detector, USB, device, network,
  display, mining, or hardware interface was accessed.
- Outcome: The attempt-015 first-line failure is fixed without accepting,
  repairing, or special-casing malformed post-admission evidence. Ordinary
  retained monitor capture remains on its unchanged byte path.
- Blocker or next safe action: Run the complete final gate sequence, review the
  diff and simplification, and close this software-only plan with API-009 still
  `implemented` and attempt-016 unauthorized.

## 2026-08-14T04:40:30Z | software closure checkpoint

- Source commit: `daba5d8a7a504b20f41b8a5e11c2d9d923940ca2` plus the final
  software changes described here.
- Actions: Completed the simplification pass and retained one boolean admission
  owner at the USB session boundary. The ordinary capture branch is unchanged;
  no retry, allocation, content repair, length heuristic, or marker exception
  was added.
- Verification: The red regression, focused boundary/reopen and real-seam
  tests, strict post-admission marker tests, device-session, flash, automation,
  real-process, firmware, mandatory, privacy, reference, selector, digest, and
  diff gates pass. The parent USB module remains at its 628-line limit.
- Evidence: Public source, tests, immutable digest, and category-only gate
  outcomes. No credential, protected attempt artifact, detector, USB, device,
  network, display, mining, or hardware interface was accessed.
- Outcome: Receive-only ephemeral analyzers now begin only after a proved line
  boundary and share the same admitted bytes. API-009 remains `implemented`.
- Blocker or next safe action: Commit and push this closure, restore a clean
  synchronized selector, and require a separate immutable contract before any
  later hardware ordinal. Attempt-016 remains unauthorized by this plan.
