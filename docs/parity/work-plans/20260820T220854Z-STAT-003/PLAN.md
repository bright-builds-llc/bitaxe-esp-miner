# Parity work plan

- Run ID: `20260820T220854Z-STAT-003`
- Parity row: `STAT-003`
- Initial status: `implemented`
- Source commit: `373f3335cc41c6b74fbabe9a6bb6ff6668ef29db`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-stat003-scoreboard`

## Selection

The worktree and pinned reference are clean, `main` equals `origin/main`, and
the selector reports no open plan. The user explicitly authorizes the evidence
plan and the work necessary to test and verify the corrected `STAT-003`
scoreboard behavior.

Attempt-005 is the lowest-risk eligible evidence source. Its immutable public
closure records exact source `a31af2873e6b2d41fe47aa18a57626f33aaf099b`,
pinned reference, a sealed/mode-correct accepted 600-second campaign, 20/20
windows, 19 accepted shares, trusted identity, fresh safety, healthy watchdog,
safe stop, cleanup, stable pre/post scoreboard reads, live SPA, and one valid
restart. It failed only because the old verifier required full runtime
difficulty equality after upstream-compatible one-decimal NVS persistence.
Source `4594760b08e606959d952a1fc7803095967e5bf2` corrected that exact comparison
with source-bound Rust/C semantics and positive/negative regressions.

Re-evaluating the existing sealed attempt avoids another flash, mining run,
share submission, restart, or device effect. A fresh hardware ordinal is not
authorized by this plan and is unnecessary unless this re-evaluation proves
the retained evidence incomplete.

## Scope and non-scope

Add a repo-owned `recheck-scoreboard-evidence` command that reads only the
existing protected roots `scratch/stat003-scoreboard/attempt-005` and
`scratch/stat003-scoreboard/wrapper-005`. It must:

- require a clean pushed evaluator source, clean pinned reference, the exact
  capture plan and closure, exact capture source, absent public projection, and
  mode-0700 directories/mode-0600 regular files with no symlinks;
- allowlist the sealed campaign result/network/diagnostics/flash files, pre- and
  post-restart system/scoreboard reads, live SPA response, restart response,
  detector output, and capture terminal output needed to prove the original
  invocation and earliest failure;
- parse protected values only in memory, never print or copy scoreboard rows,
  device/network identity, USB port, sensor values, pool values, owner/worker,
  credentials, raw logs, or NVS secrets;
- recompute campaign seals/quorum, detector admission, exact capture package
  identity, pre/post system/restart identity, live SPA markers, exact same-boot
  repeats, and corrected durable restart persistence from retained bytes;
- require capture output to identify only the expected old
  `hardware_blocked` restart-persistence boundary and reject any earlier or
  different failure;
- bind the old capture plan/source and the new evaluation plan/current source
  into the workflow digest, retain `hardware_rerun_used=false`, independently
  validate a mode-0600 candidate, then atomically publish only the existing
  redacted `bitaxe-scoreboard-evidence-v1` projection at mode 0644; and
- fail closed with a redaction-safe category, remove only its candidate, leave
  the protected attempt immutable, and withhold promotion on any ambiguity.

The command may append only absent mode-0600 `recheck.stdout` and
`recheck.stderr` wrapper streams owned by the caller. It may not modify the
attempt root, capture streams, detector streams, readiness roots, credentials,
device, network, package artifacts, checklist, or prior plans/closures.

This plan authorizes protected attempt-005 read-only re-evaluation, one public
redacted projection, independent validation/redaction review, source/evidence
commits, and—only on the full quorum—promotion of `STAT-003`. It does not
authorize credentials, external network access, detector execution, USB/device
access, flash, monitor, mining, share submission, restart, recovery, attempt-006,
external UART/BAP, pins, or electrical work.

## Implementation

- [ ] Add the closed recheck command, invocation/Just/Bazel wiring, protected
      allowlist/mode/identity checks, and deterministic real-child fixtures.
- [ ] Prove valid durable-only restart change publishes while missing/tampered
      seals, wrong failure, raw repeat drift, non-difficulty drift, bad modes,
      symlinks, source drift, and secret-bearing output withhold projection.
- [ ] Run all focused/full gates, commit and push the exact evaluator, and
      require clean source/reference before protected access.
- [ ] Run the sole protected re-evaluation command, independently validate the
      projection, review redaction, and create source-bound result evidence.
- [ ] Promote only `STAT-003` on the complete quorum; otherwise preserve
      `implemented`, projection withholding, immutable attempt-005, and no
      attempt-006.

## Authorized command, recovery, and stop conditions

After implementation is fully verified, committed, pushed, and clean, run only:

`test -d scratch/stat003-scoreboard/attempt-005 && test -d scratch/stat003-scoreboard/wrapper-005 && test ! -e scratch/stat003-scoreboard/wrapper-005/recheck.stdout && test ! -e scratch/stat003-scoreboard/wrapper-005/recheck.stderr && test ! -e docs/parity/evidence/stat003-scoreboard/scoreboard-projection.json && (umask 077; just recheck-scoreboard-evidence --private-root scratch/stat003-scoreboard/attempt-005 --wrapper-root scratch/stat003-scoreboard/wrapper-005 --capture-plan docs/parity/work-plans/20260820T150151Z-STAT-003/PLAN.md --capture-closure docs/parity/work-plans/20260820T150151Z-STAT-003/CLOSURE.md --evaluation-plan docs/parity/work-plans/20260820T220854Z-STAT-003/PLAN.md --projection docs/parity/evidence/stat003-scoreboard/scoreboard-projection.json > scratch/stat003-scoreboard/wrapper-005/recheck.stdout 2> scratch/stat003-scoreboard/wrapper-005/recheck.stderr)`

The command must open no network sockets and launch only bounded local Git and
the independent Rust validator. Starting it consumes the sole re-evaluation.
Any missing/unexpected file, path/mode/symlink violation, malformed or
inconsistent protected value, seal/source/reference/plan drift, non-expected
capture failure, incomplete quorum, validator failure, privacy finding, or
nonzero exit stops without retry, hardware, or promotion. Remove only an
unpublished candidate created by this command; never delete or rewrite the
protected attempt.

## Verification and promotion

Focused tests must cover exact IEEE-754 durable projection, full real-child
success, every protected-input integrity class above, no secret/raw output,
candidate cleanup, atomic publication, source-plan dual binding, and unchanged
31-path evaluator identity. Run ordered workspace Cargo gates, Bright Builds,
focused automation/validator tests, all Bazel tests, firmware build/package,
redaction, reference, parity, progress, selector, sensitive-value, file-size,
and diff checks before protected access.

Promotion requires the resulting projection to pass the independent Rust
validator and repo redaction review; bind exact capture/evaluator sources,
reference, old/new plans, protected input digests, corrected durable
persistence, no hardware rerun, and all original campaign/restart quorums.
Create `RESULT.md` and a redacted evidence summary, commit/push evidence without
checklist change, then transition only `STAT-003` to `verified` with
`unit,workflow,api-compare,static-route,hardware-smoke,hardware-regression`,
sync progress, archive the completed task, run final gates, and push.

On failure create `CLOSURE.md`, keep the active task and `STAT-003` implemented,
record only the redaction-safe first blocker, and do not authorize attempt-006.

## Non-claims

This plan does not pre-claim verification, expose protected values, validate
absolute difficulty calibration or profitability, authorize arbitrary pools or
profiles, other ASICs/boards, unbounded mining, OTA/recovery, or release
readiness.
