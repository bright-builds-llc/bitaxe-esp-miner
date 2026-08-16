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
