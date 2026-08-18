# Parity work plan

- Run ID: `20260818T122819Z-SAFE-10`
- Parity row: `SAFE-10`
- Initial status: `implemented`
- Source commit: `b348b16c60350ab1ecf29b2adb88babf628c668a`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-safe10-prerequisite-readiness`

## Selection

The worktree and reference are clean, `main` equals `origin/main`, and the
selector reports no open plan. Candidate order begins `SELF-001`, `BAP-002`,
`STAT-003`, then `SAFE-10`.

`SELF-001` remains blocked without a production-safe full hardware self-test
route. `BAP-002` remains blocked by not-started `BAP-001` and external UART/
electrical work outside standing USB authorization. `STAT-003` is environment-
blocked after its sole attempt-004 produced distinct `network_unavailable`; no
objective pool/network recovery signal permits another attempt. `SAFE-10` is
first actionable.

The preserved mode-0700 attempt-003 root contains a sealed accepted conservative
campaign from exact source `60a56d4935ced15eeb5ec6950b1ad4ea35fdf223`.
It records 600,746 active milliseconds, 20/20 network windows, live ASIC work
and accepted submit, trusted runtime identity, fresh production safety, exact
required/fresh observation classes for power, bus voltage, current, chip
temperature, and fan RPM, no VR-temperature requirement on Ultra 205, an
advanced ready production transition with no blocker, stable watchdog, terminal
confirmation, safe stop, and cleanup. Its scoreboard plan closed later at the
restart verifier, so these prerequisite facts remain valid protected evidence
but were never promoted for `SAFE-10`.

Loaded guidance materially includes `AGENTS.md`, `AGENTS.bright-builds.md`,
managed architecture/code-shape/verification/testing/Rust standards, the active
task/checklist, and bounded whole lesson blocks for protected roots, private-
first classification, first-failure precedence, retry discipline, transport
qualification, evaluator identity, and agent-runtime timing. The lesson ledger
remains above startup budget; all headings were inventoried, less-relevant whole
blocks were omitted, and the existing August audit baseline means no new audit
trigger is due.

## Scope and non-scope

Advance only `SAFE-10`. Add a typed `bitaxe-safe10-evidence-v1` Rust contract
and independent validator, generated TypeScript type, versioned source inventory,
private-first TypeScript projector, CLI/Just/Bazel wiring, and real-process
tests. The projector must exclusively consume:

- `scratch/stat003-scoreboard/attempt-003/campaign/` and its wrapper-003
  detector transcript as protected local evidence;
- attempt-003's committed immutable plan/closure for source binding;
- the current clean Git source and pinned reference;
- no credentials, endpoints, device, or network runtime.

Verify protected mode/containment, result seal, result-to-network/diagnostics/
observations digests, exact accepted live-share/conservative/600-second shape,
at least 600,000 active milliseconds, trusted identity, fresh safety, exact
five-of-five Ultra 205 required/fresh observation classes, ready advanced
readiness transition, accepted submit/work, 20/20 continuity, work renewal,
active/safety/watchdog validity, no panic/mixed reset/correlation failure,
terminal consumed/transport/pool facts, safe stop, and cleanup.

Build a transitive source inventory for the prerequisite predicate, readiness
state machine, firmware authoritative-readiness shell, campaign marker/parser/
evidence producer, projector, Rust validator, and pinned reference breadcrumbs.
Compare production-source bytes at the attempt source commit with current HEAD;
fail if any production prerequisite semantics drifted. Bind every current
evaluator source through Bazel runfiles and hash relative path plus bytes.

The public projection may contain only schema/workflow identity, board, source/
reference commits, cryptographic digests, path counts, bounded counts/durations,
closed categories, and booleans. It must never contain lease/session IDs,
scoreboard values, credentials, owner/pool/worker data, endpoints, ports, USB/
network identity, sensor readings, hashrates, HTTP, serial, commands, PIDs,
traces, or raw child output.

This plan authorizes protected attempt-003 read-only classification, source,
deterministic local children, tests, build/package, one public projection, docs,
Git commit, and push. It does not authorize detector execution, credentials,
device/USB/network runtime, flash, monitor, mining, restart, recovery, new
attempts, external UART/BAP, pins, or electrical work. Never mutate protected
attempt-003 artifacts.

## Implementation

- [ ] Add the Rust evidence contract/validator and generated TypeScript schema
      with exact SAFE-10 acceptance invariants and rejection tests.
- [ ] Add the transitive source/reference inventory, commit-compatibility check,
      private-first projector, CLI/Just/Bazel wiring, and real-child privacy,
      drift, seal, mode, and validation regressions.
- [ ] Run focused/full gates, commit/push the implementation, then run the sole
      software projection command against preserved attempt-003 evidence.
- [ ] Independently validate the projection and promote only `SAFE-10` if every
      live prerequisite, source, privacy, and cleanup gate passes.

## Authorized projection command

After implementation is clean, fully gated, committed, and pushed, run once:

`test ! -e docs/parity/evidence/safe10-prerequisite-readiness/safe10-projection.json && just project-safe10-evidence --attempt-root scratch/stat003-scoreboard/attempt-003 --detector-output scratch/stat003-scoreboard/wrapper-003/detector.stdout --attempt-plan docs/parity/work-plans/20260818T102038Z-STAT-003/PLAN.md --attempt-closure docs/parity/work-plans/20260818T102038Z-STAT-003/CLOSURE.md --projection docs/parity/evidence/safe10-prerequisite-readiness/safe10-projection.json`

The command is read-only except for exclusive creation of the final candidate/
projection. Any missing/malformed/mode/identity/source/seal/quorum/privacy or
validator failure removes the candidate, withholds projection, preserves the
first typed category, and ends this plan without retry.

## Verification and promotion

Focused tests cover the Rust contract, every prerequisite boolean, attempt/current
source drift, reference drift, task/plan/closure binding, protected modes,
existing projection, detector transcript admission, result seal/digest chain,
malformed/missing/private fields, real Git and validator child boundaries,
projection permissions, and redaction.

Mandatory ordered gates are `cargo fmt --all`, `cargo clippy --all-targets
--all-features -- -D warnings`, `cargo build --all-targets --all-features`,
`cargo test --all-features`, `bun scripts/bright-builds-check.ts all`, `just
test`, `just parity`, and `just parity-progress`, plus firmware/package,
redaction, reference, selector, file-size, sensitive-value, and diff checks.

Promotion requires an independently validated closed projection proving every
fact above. On success create `RESULT.md`, commit evidence without checklist
change and save that commit as `SOURCE_COMMIT`; transition only `SAFE-10` to
`verified` with `unit,workflow,hardware-smoke,hardware-regression`, update the
Rust-owned target only if implementation ownership changed, sync progress,
archive only this completed task, run final gates, and push. On failure create
`CLOSURE.md`, keep `SAFE-10` implemented, and do not sync unchanged progress.

## Non-claims

This plan verifies production prerequisite readiness only. It does not verify
fault injection, individual active voltage/fan/thermal control policy, self-
test, arbitrary telemetry ranges, fail-closed blocker labels (`SAFE-11`), other
ASICs/boards, arbitrary profiles/pools, unbounded mining, OTA, recovery, or
release readiness.
