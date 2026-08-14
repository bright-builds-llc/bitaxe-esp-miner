# Parity work log

## 2026-08-14T12:33:36Z | immutable-plan draft

- Source commit: `0e9ade815f2f0eb15415fb4d7cf4047503f6d3a1`.
- Actions: Selected API-009 first and drafted one bounded attempt-017 contract
  for the newly verified paused asynchronous signal window.
- Verification: Plan-only mandatory, privacy, reference, firmware, selector,
  task, immutable-digest, and diff gates are pending before commit.
- Evidence: Public source and prior closure facts only. No credential,
  protected attempt content, detector, USB, device, network, display, mining,
  or hardware interface was accessed.
- Outcome: Attempt-017 remains ineligible until this immutable contract and all
  named plan gates are committed and pushed at clean synchronized HEAD.
- Blocker or next safe action: Run the complete plan gate sequence, review the
  diff, commit and push, then perform exact-package admission before the sole
  detector run.

## 2026-08-14T12:37:14Z | immutable-plan verification

- Plan SHA-256:
  `38312cf2bea1621626ece65447c5b8545aac7ded740e52893a2a7b02dc18e716`.
- Actions: Ran the complete plan-only gate sequence and confirmed the selector
  resumes this unique API-009 attempt-017 plan.
- Verification: Formatting, strict Clippy, all-target build, all-feature tests,
  Bright Builds, canonical Bazel tests, parity, parity-progress, redaction,
  reference cleanliness, real ESP firmware build, task uniqueness, open-plan
  selection, immutable digest, and diff checks pass.
- Evidence: Public plan/task/source and category-only gate outcomes. No
  credential, protected attempt content, detector, USB, device, network,
  display, mining, or hardware interface was accessed.
- Outcome: The exact-package detector-gated attempt-017 contract is ready to
  commit and push without changing API-009 from `implemented`.
- Blocker or next safe action: Push this checkpoint, confirm clean synchronized
  HEAD and the same open plan, then perform exact-package admission before the
  sole detector run.

## 2026-08-14T12:41:01Z | exact-package and detector admission

- Source commit: `f0bdb19290f3edcdda1868c5c5059561c543a564`.
- Actions: Confirmed clean synchronized HEAD, built the exact Ultra 205
  package, validated its source/reference/application identity, required the
  ignored Wi-Fi credential file to be non-empty without reading it, and ran the
  fresh detector exactly once.
- Verification: Package admission passed. The detector returned one board-205
  configuration, one ready USB session, one admitted port, exit zero, and the
  required private directory/file modes.
- Evidence: Only package identity and categorical detector facts were used.
  Credentials, port, USB/network identity, origin, hostname, and raw detector
  output remain private.
- Outcome: The exact attempt-017 campaign became eligible and was invoked once.
- Blocker or next safe action: Keep the attached campaign private and wait only
  for a live typed checkpoint or terminal result.

## 2026-08-14T12:48:56Z | single-attempt terminal closure

- Actions: Kept the sole campaign attached without sending any pre-confirmed
  signal. The run completed its factory and NVS flashes, entered serial
  observation, and terminated before publishing `identify-ready.required.json`.
- Verification: The private typed result is `network_target_unavailable`.
  Flash diagnostics classify both transfers `ready`; serial diagnostics show
  observation started, one accepted paused terminal marker, clean framing, and
  zero runtime-attestation candidates. The public wrapper reports
  `hardware_blocked`, safe stop confirmed, USB cleanup complete, recovery not
  attempted, and no secondary recovery failure. The public projection is
  absent.
- Evidence: Only categorical and safe-boolean fields were inspected. No
  credential, origin, hostname, port, USB/network identity, sensor value, or raw
  trace is recorded publicly.
- Outcome: Attempt-017 is consumed and blocked before the asynchronous ready
  window. No ready, rendered, or cleared signal was sent; no display observation
  was requested or claimed; API-009 remains `implemented`.
- Blocker or next safe action: Close this immutable plan without attempt-018.
  The next software investigation should determine why post-flash serial
  capture sees the paused terminal marker but no boot-attestation samples before
  trusted-target admission.

## 2026-08-14T13:09:08Z | closure verification

- Actions: Wrote the blocked closure, preserved API-009 at `implemented`, and
  reviewed the complete public diff and protected-artifact state.
- Verification: Cargo format, strict Clippy, all-target build, all-feature
  tests, Bright Builds, canonical Bazel tests, parity, parity-progress,
  redaction, reference cleanliness, real ESP firmware build, immutable plan
  digest, task uniqueness, public-projection absence, private modes, process
  cleanup, and diff checks pass. The first parity-report invocation encountered
  transient host resource error 35 after all Bazel tests passed; one bounded
  retry passed without a parity finding.
- Evidence: The plan digest remains
  `38312cf2bea1621626ece65447c5b8545aac7ded740e52893a2a7b02dc18e716`.
  No protected value or raw trace entered the public diff.
- Outcome: Attempt-017 is closed as blocked with evidence withheld and no
  attempt-018 authorization.
- Blocker or next safe action: Commit and push the closure, then stop hardware
  work at this terminal plan boundary.
