# Parity work plan

- Run ID: `20260818T132739Z-SAFE-10`
- Parity row: `SAFE-10`
- Initial status: `implemented`
- Source commit: `c07521e7bed2b49fad7c8e9b275cdc39e91fc116`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-safe10-prerequisite-readiness`

## Selection

The worktree/reference are clean, `main` equals `origin/main`, and the selector
reports no open plan. `SELF-001` remains blocked without a production-safe full
self-test route; `BAP-002` remains blocked by `BAP-001` and external electrical
UART work; `STAT-003` remains environment-blocked without an objective pool/
network recovery signal. `SAFE-10` is first actionable.

The prior SAFE-10 plan implemented and fully gated the Rust contract,
independent validator, 19-path transitive inventory, attempt-source compatibility
check, private-first projector, CLI/Just/Bazel wiring, and real-validator tests.
Its sole command failed before candidate creation because the specialized binary
omitted the validator from runfiles. The pushed fix `1c0ad96d` adds the validator
to generic, specialized, and test runfiles; the built projector path is
executable and all gates pass. The public projection and candidate remain absent,
and protected attempt-003 remains immutable.

Loaded guidance materially includes `AGENTS.md`, `AGENTS.bright-builds.md`,
managed architecture/code-shape/verification/testing/Rust standards, active
task/checklist, and bounded lessons for protected roots, private-first
classification, first-failure precedence, evaluator identity, and agent-runtime
timing. The lesson ledger remains above startup budget; headings were inventoried,
less-relevant whole blocks omitted, and the August audit baseline means no new
audit trigger is due.

## Scope and non-scope

Advance only `SAFE-10`. Rotate the projector/test/runfiles binding from the
closed plan `20260818T122819Z-SAFE-10` to this immutable plan and digest. Do not
change evidence schema, source membership, prerequisite semantics, attempt
source, protected paths, or promotion quorum.

After focused/full gates, commit/push the binding change and execute once:

`test ! -e docs/parity/evidence/safe10-prerequisite-readiness/safe10-projection.json && just project-safe10-evidence --attempt-root scratch/stat003-scoreboard/attempt-003 --detector-output scratch/stat003-scoreboard/wrapper-003/detector.stdout --attempt-plan docs/parity/work-plans/20260818T102038Z-STAT-003/PLAN.md --attempt-closure docs/parity/work-plans/20260818T102038Z-STAT-003/CLOSURE.md --projection docs/parity/evidence/safe10-prerequisite-readiness/safe10-projection.json`

The command may read only the fixed protected attempt/detector inputs and
repository source/history, then exclusively create one candidate/projection.
Any mode/path/task/plan/seal/digest/prerequisite/readiness/source/privacy/
validator failure removes the candidate, withholds projection, preserves the
first typed category, and ends this plan without retry.

The public projection may contain only schema/workflow identity, commits,
digests, path counts, bounded counts/duration, closed categories, and booleans.
It must contain no lease/session IDs, scoreboard fields, credentials, owner/
pool/worker data, endpoints, ports, USB/network identity, readings, hashrates,
HTTP, serial, commands, PIDs, traces, or raw child output.

This plan authorizes source binding, tests/builds, read-only protected
classification, one projection, docs, Git commit, and push. It authorizes no
detector execution, credentials, device/USB/network runtime, flash, monitor,
mining, restart, recovery, new attempt, external UART/BAP, pins, or electrical
work.

## Implementation

- [ ] Rotate only the expected plan path/digest plus test/runfiles input to this
      immutable plan, preserving every evidence and privacy invariant.
- [ ] Run focused/full gates, commit/push, and run the sole corrected projection
      command with absent-output admission.
- [ ] Independently validate projection schema, modes, redaction, attempt/current
      source compatibility, and complete live prerequisite quorum.
- [ ] Commit projection/`RESULT.md` as `SOURCE_COMMIT`, transition only SAFE-10
      to verified, sync progress, archive this task, final-gate, and push.

## Verification and promotion

Focused tests cover current inventory, plan/task binding, executable validator
runfiles, complete real-validator fixture, prerequisite/source drift withholding,
protected modes, atomic publication, and redaction.

Mandatory ordered gates are `cargo fmt --all`, `cargo clippy --all-targets
--all-features -- -D warnings`, `cargo build --all-targets --all-features`,
`cargo test --all-features`, `bun scripts/bright-builds-check.ts all`, `just
test`, `just parity`, and `just parity-progress`, plus firmware/package,
redaction, reference, selector, file-size, sensitive-value, and diff checks.

Promotion requires independently validated `bitaxe-safe10-evidence-v1` proving
board 205, attempt-003 detector/source/reference/plan lineage, seal/digest chain,
five required/fresh observation classes with VR temperature explicitly not
required, unblocked running-primary ready transition, at least 600,000 active
ms, accepted submit/work, 20/20 continuity, work renewal, active/safety/watchdog
validity, terminal transport/pool/final-consumed facts, safe stop, cleanup,
protected modes, attempt/current production compatibility, current evaluator/
reference semantics, and redaction.

On success create `RESULT.md`, commit evidence without checklist change and save
that commit as `SOURCE_COMMIT`; transition only `SAFE-10` to `verified` with
`unit,workflow,hardware-smoke,hardware-regression`, sync progress, archive only
this completed task, run final gates, and push. On failure create `CLOSURE.md`,
leave implemented, and do not sync unchanged progress.

## Non-claims

This plan verifies prerequisite readiness only, not SAFE-11 blocker labels,
fault injection, individual active controls, self-test, arbitrary telemetry,
other ASICs/boards, arbitrary profiles/pools, unbounded mining, OTA, recovery,
or release readiness.
