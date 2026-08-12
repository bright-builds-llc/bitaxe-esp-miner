# API-009 worklog

## 2026-08-12T17:00:39Z | Plan checkpoint

- Source commit: `f595fa6f97441c1bc44975f90b4b23891292169a`.
- Actions: Selected API-009 first after closing attempt-003, joined its
  `stratum_v1_unsupported` result to the exact generated campaign NVS contract,
  and inspected the pinned ESP-IDF NVS ownership implementation and every
  firmware caller without a new device effect.
- Verification: Attempt-003 proves both writes, trusted runtime identity, safe
  stop, cleanup, and protected modes. Source inspection proves the campaign
  seeds primary SV1 and defaults missing fallback to SV1, while multiple
  concurrent firmware adapters independently acquire one exclusive default-NVS
  partition and collapse acquisition/selector failures into one boolean.
- Evidence: No new parity evidence or hardware action is claimed. Protected
  attempt-003 values remain ignored; only closed categories inform this plan.
- Outcome: The next material correction is a boot-lifetime shared partition
  owner plus a typed protocol-gate decision, not another seed or flash retry.
- Blocker or next safe action: Run plan-only gates, commit and push this
  immutable checkpoint, then implement and verify the ownership fix before any
  fresh ordinal.

## 2026-08-12T17:05:00Z | Plan gate complete

- Source commit: `f595fa6f97441c1bc44975f90b4b23891292169a`.
- Actions: Bound the shared NVS owner, closed protocol decision, focused
  regressions, privacy contract, and single attempt-004 to the unique active
  API-009 task.
- Verification: Ordered Cargo format, clippy, all-target build, all-feature
  tests, Bright Builds, all 39 Bazel tests, parity, progress, redaction,
  reference, selector, unique-task, reference-cleanliness, and diff checks
  pass. PLAN.md SHA-256 is
  `df06a5daa83d9dbc86dbc7fb6161eddecb56af3e2a7efc28fb8908ffd06c454f`.
- Evidence: No hardware action or parity evidence occurred after attempt-003.
  The plan contains only closed categories and source contracts.
- Outcome: The immutable continuation is eligible to commit and push.
- Blocker or next safe action: Push the checkpoint, then implement the
  boot-lifetime owner and typed gate before considering hardware.

## 2026-08-12T17:20:29Z | Source gate complete

- Source commit: `2c603f34b391a0c14c8539724fb28444961798d7`;
  immutable plan commit `2dfdb5eb99d5da73037d8bbcf434cdcaf913f05d`
  was already pushed.
- Actions: Replaced every independent default-NVS acquisition with one
  boot-lifetime owner initialized during ordered startup, routed settings,
  production, protocol-gate, and scoreboard access through shared clones, and
  carried a value-free closed protocol-gate decision through marker v10 and
  result v7. The public command-effects validator now requires `ready`.
- Verification: Focused owner, protocol, marker, flash, automation, and real
  firmware targets pass. The complete ordered Cargo format, clippy, all-target
  build, all-feature test, Bright Builds, all 40 Bazel tests, parity, progress,
  redaction, reference, schema, generated-NVS, ownership, sensitive-output,
  immutable-plan, and diff gates pass. Parity remains 59 verified of 94 active
  rows, with API-009 unchanged at implemented.
- Evidence: This is source-only verification. No device effect, protected
  attempt, public projection, or parity promotion occurred.
- Outcome: The material NVS-owner race is fixed and the exact source is ready
  to commit and push before attempt-004 admission.
- Blocker or next safe action: Commit and push the clean implementation, build
  its exact package, run a fresh detector, then consume at most the single
  bounded attempt-004 defined by the immutable plan.

## 2026-08-12T17:29:47Z | Attempt-004 terminal closure

- Source commit: `2c603f34b391a0c14c8539724fb28444961798d7`.
- Actions: Built and admitted the exact pushed package, freshly detected one
  board-205 ESP32-S3 device, and consumed the sole authorized attempt-004. No
  second attempt was started.
- Verification: Both supervised no-stub writes completed on attempt one. The
  runtime identity was trusted, protocol gate was `ready`, genuine block
  notification and positive block count were observed, pause was requested and
  confirmed, eight candidates qualified, safe stop was confirmed, USB cleanup
  was ready, and every protected directory/file retained mode `0700`/`0600`.
  After the single resume request, the firmware reported
  `safety_prerequisites_stale`; resume was not confirmed and IDENTIFY, dismiss,
  and restart were not requested. Result v7 failed as
  `network_correlation_failed`, and the public projection is absent.
- Evidence: No API-009 parity evidence is claimed. Protected USB, serial,
  network, credential, endpoint, and raw values remain ignored and were not
  copied into repository artifacts.
- Outcome: The attempt-003 protocol blocker is materially resolved. API-009
  remains `implemented` because the later fail-closed readiness transition
  prevented the complete five-command quorum.
- Blocker or next safe action: Close this immutable plan without promotion.
  Any next ordinal requires a fresh plan that first explains and tests the
  transient safety-readiness loss across pause/resume; attempt-004 cannot be
  retried.
