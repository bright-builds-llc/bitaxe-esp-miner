---
name: advance-parity
description: Select or resume exactly one unfinished row from the Bitaxe parity checklist, commit an audit plan before implementation, execute and verify the work, transition the row conservatively, update deterministic progress history and README status, and push the audited commits. Use only when the user explicitly invokes $advance-parity to advance repository parity.
---

# Advance Parity

Advance one parity row per invocation. Treat `verified` as complete,
`deferred` as excluded, and every other status as unfinished.

## Preflight

1. Read `AGENTS.md`, active lessons, `TASKS.md`, and
   `docs/parity/checklist.md`.
2. Require a clean worktree and a checked-out branch with an upstream. Fetch
   `origin`; stop on divergence, unrelated changes, or unresolved conflicts.
3. Run:

   `bazel run //tools/parity:report -- next-item --format json`

4. Resume `maybe_open_plan` when present. Otherwise inspect candidates in
   returned order and choose the first actionable row. Record concrete reasons
   for skipping dependency-blocked, unavailable, unauthorized, or unsafe rows.
5. Hardware work is eligible only when every repo task gate, detector,
   credential, evidence, recovery, retry, and standing-authorization requirement
   is satisfied. Active tasks and fresh progress-backed attempt ordinals do not
   require per-attempt user confirmation. Never infer authority beyond the
   active task contract.

## Persist the Plan

1. Create
   `docs/parity/work-plans/<YYYYMMDDTHHMMSSZ>-<row-id>/PLAN.md` from
   [assets/PLAN.md](assets/PLAN.md). Preserve the template's parity-row
   metadata line exactly.
2. Add or resume one matching active `TASKS.md` block referencing the plan.
3. Include selection inputs, skipped candidates, source and reference commits,
   scope, implementation steps, verification, evidence requirements, safety
   gates, and promotion criteria.
4. Run all repository-required pre-commit checks. Commit the plan and task
   record before editing implementation files. Do not amend this commit.

## Execute and Record

1. Work only on the selected row and append each attempt to `WORKLOG.md`
   using [assets/WORKLOG.md](assets/WORKLOG.md). Never rewrite `PLAN.md`.
2. Keep status conservative:
   - use `in-progress` for incomplete implementation;
   - use `implemented` for implemented behavior lacking accepted parity
     evidence;
   - use `verified` only after the row's exact evidence requirements pass.
3. For verified completion, create `RESULT.md` from
   [assets/RESULT.md](assets/RESULT.md). Record exact commands, evidence,
   conclusion, non-claims, and residual risks.
4. When a plan reaches a terminal non-verified outcome without changing the
   row's checklist fields, create `CLOSURE.md` from
   [assets/CLOSURE.md](assets/CLOSURE.md). Bind it to the immutable `PLAN.md`,
   declare `Verification claimed: no`, and record the blocker, next safe
   action, and non-claims. Never use a closure to represent verified evidence.
5. Run relevant tests and the mandatory Rust checks. Commit implementation and
   evidence without changing the checklist yet. Save this full commit as
   `SOURCE_COMMIT`.

## Finalize Progress

1. Prepare `WORKLOG.md` or `RESULT.md`. When status, implementation target, or
   evidence changes, transition the row:

   `bazel run //tools/parity:report -- transition-item --transition-id <UTC-run-id> --row-id <ID> --to <in-progress|implemented|verified> --evidence '<evidence-cell>' --plan <PLAN.md> [--result <RESULT.md>] [--rust-owned-target '<target-cell>']`

   If none of those checklist fields changed, do not request a no-op
   transition.
2. After a checklist transition, immediately synchronize progress:

   `bazel run //tools/parity:report -- sync-progress --source-commit "$SOURCE_COMMIT" --selected-row <ID> --plan <PLAN.md>`

   When the checklist digest is unchanged, do not append progress history or
   rewrite the README.
3. A valid `CLOSURE.md` closes only the plan lifecycle. Its parity row remains
   unfinished and may appear in later candidate lists. Leave blocked tasks
   active with the blocker and next safe action; do not synchronize progress
   when the checklist digest is unchanged.
4. When verified, add the completion review, append the full task record to
   `TASKS.archive.md`, and remove it from `TASKS.md`. Otherwise leave it
   active with the blocker and next safe action.
5. Run, in order:
   - `cargo fmt --all`
   - `cargo clippy --all-targets --all-features -- -D warnings`
   - `cargo build --all-targets --all-features`
   - `cargo test --all-features`
   - `bun scripts/bright-builds-check.ts all`
   - `just test`
   - `just parity`
   - `just parity-progress`
6. Review `git diff`, commit finalization when it exists, fetch `origin`, rebase only when
   conflict-free and necessary, and push the current branch to its upstream.
   Never force-push.

If no row is actionable, make no repository changes and report the concrete
blockers. If execution stops after the plan commit, push the truthful plan or
checkpoint and resume it on the next invocation.
