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
