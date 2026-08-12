# API-009 worklog

## 2026-08-12T17:34:27Z | Plan checkpoint

- Source commit: `aa9ffdb40b4fbf0c42d47dd68d84ad32b75b197e`.
- Actions: Selected API-009 first after closing attempt-004, joined its closed
  stale-resume result to the production readiness, sensor producer,
  notification, recovery, and marker ownership seams without a new device
  effect.
- Verification: Attempt-004 proves the protocol fix, trusted identity, genuine
  block event, confirmed pause, safe stop, cleanup, and protected modes. Source
  inspection proves readiness and marker observations are sampled separately
  around a category-only producer notification, and the exact stale-then-fresh
  pause/resume shell path lacks production-shaped coverage.
- Evidence: No new parity evidence or hardware action is claimed. Protected
  attempt-004 values remain ignored; only closed categories inform this plan.
- Outcome: The next material step is to reproduce the missed fresh transition
  and bind a closed observation epoch to readiness, not to loosen safety or
  repeat the device attempt.
- Blocker or next safe action: Run plan-only gates, commit and push this
  immutable checkpoint, then implement only after the regression identifies
  the exact transition defect.

## 2026-08-12T17:38:25Z | Plan gate complete

- Source commit: `aa9ffdb40b4fbf0c42d47dd68d84ad32b75b197e`.
- Actions: Bound the closed readiness trace, production-shaped red/green
  regression, minimal transition fix, privacy contract, and conditional single
  attempt-005 to the unique active API-009 task.
- Verification: Ordered Cargo format, clippy, all-target build, all-feature
  tests, Bright Builds, all 40 Bazel tests, parity, progress, redaction,
  reference, selector, unique-task, reference-cleanliness, and diff checks
  pass. PLAN.md SHA-256 is
  `2165879f579b01718082943f4df606cd7cbbf0f29205ee333ca16f81143a101b`.
- Evidence: No hardware action or parity evidence occurred after attempt-004.
  The plan contains only closed categories and source contracts.
- Outcome: The immutable continuation is eligible to commit and push.
- Blocker or next safe action: Push the checkpoint, then reproduce the exact
  stale-resume/fresh-notification sequence before implementing a fix.

## 2026-08-12T17:59:42Z | Readiness transition fixed

- Source commit: pending verified implementation commit on top of
  `aeb7e7d2`.
- Actions: Reproduced the production ownership failure with a red source-seam
  regression: `ObservationsChanged` used `try_send`, but a full queue returned
  `Coalesced` without retaining a category-specific wake. Added a one-bit
  release/acquire pending wake owned beside the notification port, consumed it
  in the production owner, and synthesized only the lost category wake. Added
  a value-free transition witness covering wakeup, prior/current blocker,
  pre-event session/campaign/hardware phase, fresh/stale decision, observation
  epoch advancement, and pending-wake recovery. Versioned marker/result schemas
  to v11/v8 and bound their closed vocabularies through the host parser.
- Verification: The ownership regression failed before the latch and passes
  after it. Focused atomic coalescing, stale-resume/fresh-notification recovery,
  retained transition, marker/result parsing, malformed evidence, redaction,
  real child-process, and exact ESP32-S3 firmware build targets pass. The stale
  resume wake remains fail-closed and issues no hardware preparation; the next
  fresh observation wake reprepares the same lease exactly once.
- Evidence: No hardware action or parity promotion occurred. The trace emits no
  values, timestamps, sequences, origins, hostnames, ports, USB/network
  identity, credentials, endpoints, paths, or raw output.
- Outcome: The confirmed missed-transition defect is fixed without changing
  the 1,000-ms freshness bound or synthesizing fresh sensor truth.
- Blocker or next safe action: Run the full ordered mandatory gate, review the
  complete diff and privacy surface, then commit and push the exact source
  before any package or detector action.

## 2026-08-12T18:10:33Z | Source gate complete

- Source commit: pending implementation commit on top of `aeb7e7d2`.
- Actions: Completed a simplification pass by isolating the pending wake,
  owner loop, readiness projection, closed host validator, and large-stream
  regression into focused modules. Updated every affected Bazel source graph
  and its ownership guards.
- Verification: Ordered `cargo fmt --all`, strict all-target/all-feature
  Clippy, Cargo build and tests, Bright Builds, all 41 Bazel tests, parity,
  progress, redaction, reference, exact firmware build, selector, unique task,
  immutable plan digest, reference cleanliness, diff, file-length, and
  sensitive-output checks pass. Selector returns only the open API-009 plan;
  progress remains 59 verified of 94 active rows and PLAN.md retains SHA-256
  `2165879f579b01718082943f4df606cd7cbbf0f29205ee333ca16f81143a101b`.
- Evidence: This remains software evidence only. No origin, hostname, port,
  USB/network identity, credential, endpoint, timestamp, sequence, sensor
  value, raw trace, or private path is emitted by the new closed transition.
- Outcome: The material recovery fix and every pre-hardware source gate are
  complete.
- Blocker or next safe action: Commit and push the exact source, verify remote
  synchronization and cleanliness, then build the exact package. Only a fresh
  successful detector may make the sole attempt-005 eligible.
