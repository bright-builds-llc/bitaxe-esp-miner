# Parity work plan

- Run ID: `20260818T140738Z-SAFE-11`
- Parity row: `SAFE-11`
- Initial status: `implemented`
- Source commit: `2391d7100b7202cb3c62b93169b269eaedd41252`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-safe11-fail-closed-reasons`

## Selection

The clean `main` worktree equals `origin/main`, the pinned reference is clean,
and the selector reports no open plan. Candidate order is `SELF-001`,
`BAP-002`, `STAT-003`, then `SAFE-11`.

`SELF-001` remains unavailable because no production-safe full self-test route
exists for its hardware regression. `BAP-002` remains dependency- and safety-
blocked by unfinished `BAP-001` firmware/UART lifecycle plus unauthorized
external electrical UART work. `STAT-003` remains environment-blocked after
attempt-004's distinct `network_unavailable` result; its active task prohibits
an unchanged retry without an objective protected pool/network recovery signal.
`SAFE-11` is therefore first actionable.

This plan continues the closed
`docs/parity/work-plans/20260818T135714Z-SAFE-11/PLAN.md`. That plan correctly
stopped before implementation because it conflated the operator-controlled
`paused` state with fail-closed failure states. Current production first disables
work for every `ProductionSessionBlocker`, then preserves `OperatorPaused` as
`paused`; the other sixteen variants remain `safe_blocked` and publish their
exact closed reason through the API.

SAFE-10 has accepted detector-gated board-205 evidence at
`docs/parity/evidence/safe10-prerequisite-readiness/safe10-projection.json`.
Current source already represents production-session reasons as a closed enum,
maps them into mining state in `production_session/runtime.rs`, and exposes
failure labels only through the blocked/safe-blocked API branch. The remaining
SAFE-11 gap is accepted current-source evidence: the Phase 22 ledger still
cites removed mining-loop targets and predates this production-session model.

Loaded guidance materially includes `AGENTS.md`, `AGENTS.bright-builds.md`,
managed architecture/code-shape/verification/testing/Rust standards, the
active task/checklist, and bounded lessons for protected evidence, redaction,
earliest failure, transitive evaluator identity, standing authorization, and
agent-runtime timing. Active lesson inputs total 31,758 bytes, so headings were
inventoried and relevant complete blocks loaded; less-relevant blocks were
omitted under the deterministic budget. The August lesson-audit baseline is
current and no new audit trigger is due.

## Scope and non-scope

Advance only `SAFE-11`. Add focused cross-layer regressions that enumerate the
current production-session vocabulary and prove two exact state classes:

- `OperatorPaused` disables work, projects `paused`, and exposes no API failure
  reason.
- Every safety/readiness/transport/ASIC/campaign failure disables work,
  projects `safe_blocked`, and exposes its exact unique redaction-safe API
  reason.

Reconcile the stale Phase 22 ledger with the real current production path and
create a source-bound SAFE-11 evidence summary joining those tests to the
independently validated SAFE-10 projection and pinned upstream control flow.

Do not change production blocker ordering, readiness policy, SAFE-10 production
or evaluator inventory, hardware controls, network/pool behavior, campaign
policy, API schema, reference source, or another parity row. No dependency or
new projection workflow is required; compiled behavior plus committed
source-bound evidence is the minimum sufficient proof.

This plan authorizes local source, tests, documentation, build/package, Git
commit, and push only. It authorizes no protected-input access, detector,
credentials, device/USB/network runtime, flash, monitor, mining, restart,
recovery, hardware attempt, fault injection, external UART/BAP, pins, or
electrical work.

## Implementation

- [ ] Add focused vocabulary and propagation regressions for the one paused
      operator state and all sixteen fail-closed production failure reasons.
- [ ] Replace stale Phase 22 mining-loop claims with the current typed
      production readiness/runtime/API chain and exact non-claims.
- [ ] Produce committed SAFE-11 evidence binding current source/reference, the
      validated SAFE-10 projection, focused/full tests, and privacy review.
- [ ] Commit evidence and `RESULT.md` without checklist changes, save that
      commit as `SOURCE_COMMIT`, then promote only SAFE-11 on the full quorum.

## Verification and promotion

Focused verification is:

- `cargo test -p bitaxe-api mining::tests`
- `cargo test -p bitaxe-stratum every_readiness_blocker_prevents_secret_network_and_asic_effects`
- `bazel test //crates/bitaxe-api:tests //crates/bitaxe-stratum:tests`
- `just validate-safe10-evidence docs/parity/evidence/safe10-prerequisite-readiness/safe10-projection.json`
- `just verify-reference`

The mandatory ordered gates are `cargo fmt --all`, `cargo clippy --all-targets
--all-features -- -D warnings`, `cargo build --all-targets --all-features`,
`cargo test --all-features`, `bun scripts/bright-builds-check.ts all`, `just
test`, `just parity`, and `just parity-progress`. Evidence checks also include
exact source/reference identities, SAFE-10 projection digest/mode, direct
sensitive-key/value review, file-size review, selector review, and final diff.

Promotion requires all seventeen current `ProductionSessionBlocker` variants
to have unique lowercase-ASCII category labels and to disable work submission.
`OperatorPaused` alone must project mining activity `paused` and an empty API
`blockedReason`; every other variant must project activity `safe_blocked` and
its exact enum label as API `blockedReason`. Ready or otherwise non-safe-blocked
shapes must not expose stale reason text. Current runtime source must consume
the typed enum rather than protected caller data, while reference inspection
must show upstream fail-closed pool pause/power behavior without claiming
wire-identical reason strings.

The accepted SAFE-10 projection must independently validate and continue to
prove detector-admitted board-205 live prerequisite success, source/reference
lineage, protected modes, safe stop, cleanup, and redaction. SAFE-11 evidence
may contain only commits, digests, paths, closed labels, counts, booleans, and
command outcomes—never credentials, owner/pool/worker data, endpoints, ports,
USB/network identity, telemetry values, raw logs/payloads, or protected IDs.

On success create `RESULT.md`, commit evidence without checklist change and
save that full commit as `SOURCE_COMMIT`; transition only `SAFE-11` to
`verified` with `unit,workflow,hardware-smoke,hardware-regression`, update its
Rust-owned target to the real production readiness/runtime/API chain, sync
progress, archive only this task, run final gates, and push. Final notes must
satisfy the legacy Phase 28 evidence guard while preserving exact non-claims.
On failure create `CLOSURE.md`, leave SAFE-11 `implemented`, and do not sync
unchanged progress.

## Non-claims

This plan does not inject live faults or verify individual active voltage, fan,
thermal, or power-control effects; self-test; BAP/UART; arbitrary telemetry;
other boards/ASICs; arbitrary profiles/pools; unbounded mining; OTA/recovery;
or release readiness. Upstream and Rust reason labels need not be wire-identical;
parity is observable fail-closed behavior plus stable Rust operator categories.
