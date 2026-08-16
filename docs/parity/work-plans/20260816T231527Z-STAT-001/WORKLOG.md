# Parity work log

## 2026-08-16T23:15:27Z | plan checkpoint

- Source commit: `5c3d9f180e4a00a1799d1755fa7e9e9378462da3`
- Actions: Loaded the required policies, active task and checklist, inventoried
  active lessons, ran clean/sync/reference preflight, applied the deterministic
  selector, and froze the progress-backed attempt-009 contract.
- Verification: The worktree and pinned reference are clean; `main` equals
  `origin/main`; no open plan exists; STAT-001 is the first actionable row.
- Evidence: Immutable `PLAN.md` and the matching STAT-001 task block.
- Outcome: Plan checkpoint ready for mandatory pre-commit verification.
- Blocker or next safe action: Commit and push this exact plan/task checkpoint,
  then rebind the workflow to attempt-009 without editing `PLAN.md`.

## 2026-08-16T23:22:00Z | plan verification

- Source commit: `5c3d9f180e4a00a1799d1755fa7e9e9378462da3`
- Actions: Ran the mandatory ordered repository gates plus privacy, reference,
  and exact firmware-package verification.
- Verification: Cargo format, clippy, build, and tests; Bright Builds; all 46
  Bazel tests; redaction; reference cleanliness; firmware package; parity; and
  progress passed. The first parity report launch hit transient host resource
  exhaustion after its report; one bounded rerun passed with
  `validation_errors: none`, and progress remained 75 verified of 94 active.
- Evidence: Plan SHA-256
  `9d29c5afd51083d26ada6653bfbf731900eef1b44591becad985d2f76a5df600`.
- Outcome: The immutable plan/task checkpoint is verified for commit and push.
- Blocker or next safe action: Rebind only the plan-authorized attempt-009
  workflow after the checkpoint is present on `origin/main`.

## 2026-08-16T23:30:00Z | attempt-009 software rebind

- Source commit: pending exact implementation commit
- Actions: Rebound the protected roots, immutable plan/task admission, Bazel
  runfiles, Rust validator, TypeScript contract, public evidence ordinal, and
  real-child fixtures from consumed attempt-008 to fresh attempt-009. Production
  hashrate, watchdog, mining, sensor, and control behavior was not changed.
- Verification: Focused Rust and real-child automation targets passed. Cargo
  format, clippy, build, and tests; Bright Builds; all 46 Bazel tests; privacy;
  reference cleanliness; exact firmware packaging; parity; and progress passed.
  Each first parity invocation encountered transient host resource exhaustion
  after rendering its report; each single bounded rerun passed with
  `validation_errors: none`. The immutable plan hash remains unchanged.
- Evidence: `bitaxe-hashrate-monitor-evidence-v1` now admits only board 205,
  attempt 9, the pushed plan hash, fresh wrapper/attempt roots, and the complete
  unchanged campaign acceptance boundary.
- Outcome: Exact attempt-009 implementation is ready for commit and push.
- Blocker or next safe action: Push the exact source, rebuild and validate its
  package, then run only the frozen detector and conditional capture commands.

## 2026-08-16T23:43:00Z | attempt-009 terminal closure

- Source commit: `6fd586dab96a3eca15b7dd68d92de60c275bc5de`
- Actions: Rebuilt the exact clean pushed package, admitted one Ultra 205 with
  the frozen detector, and ran the sole protected attempt-009 capture. No retry
  or attempt-010 was started.
- Verification: The closed result reported `hardware_blocked`, terminal
  `watchdog_unresponsive`, discriminator `watchdog_feed_stale`, trusted runtime
  identity, no attestation parse failure, 14 of 20 windows, confirmed safe stop,
  and ready USB cleanup. Campaign-result and network seals, exact file set,
  mode-`0700` roots, mode-`0600` files, and projection absence passed.
- Evidence: Protected attempt-009 artifacts remain ignored and private;
  `CLOSURE.md` contains only the allowlisted closed outcome.
- Outcome: `stop_repeated_boundary`; STAT-001 remains `implemented` and no
  checklist field changes.
- Blocker or next safe action: Diagnose the remaining feed-staleness boundary
  in a separate software-only immutable plan before any new hardware ordinal.
