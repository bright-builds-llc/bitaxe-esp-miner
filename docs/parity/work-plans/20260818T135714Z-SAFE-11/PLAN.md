# Parity work plan

- Run ID: `20260818T135714Z-SAFE-11`
- Parity row: `SAFE-11`
- Initial status: `implemented`
- Source commit: `fe40b1ddfb7799efd84671548a08db32ee8a1760`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-safe11-fail-closed-reasons`

## Selection

The clean `main` worktree equals `origin/main`, the pinned reference is clean,
and the selector reports no open plan. Candidate order is `SELF-001`,
`BAP-002`, `STAT-003`, then `SAFE-11`.

`SELF-001` remains unavailable because no production-safe full self-test route
exists for the required hardware regression. `BAP-002` remains dependency- and
safety-blocked by unfinished `BAP-001` firmware/UART lifecycle plus external
electrical UART work that is not authorized. `STAT-003` remains environment-
blocked after attempt-004's distinct `network_unavailable` result; its active
task prohibits an unchanged retry without an objective protected pool/network
recovery signal. `SAFE-11` is therefore first actionable.

SAFE-10 now has accepted detector-gated board-205 evidence at
`docs/parity/evidence/safe10-prerequisite-readiness/safe10-projection.json`.
Current source already represents production-session reasons as the closed
`ProductionSessionBlocker` enum, applies its exact label to fail-closed mining
state in `production_session/runtime.rs`, and exposes that label only through
the blocked/safe-blocked API branch. The remaining SAFE-11 gap is accepted,
current-source evidence: the Phase 22 ledger still cites removed mining-loop
targets and predates the current production-session reason vocabulary.

Loaded guidance materially includes `AGENTS.md`, `AGENTS.bright-builds.md`,
managed architecture/code-shape/verification/testing/Rust standards, the
active task/checklist, and bounded lessons for protected evidence, redaction,
earliest failure, transitive evaluator identity, standing authorization, and
agent-runtime timing. The active lesson inputs total 31,758 bytes, so headings
were inventoried and relevant complete blocks loaded; less-relevant blocks were
omitted under the deterministic budget. The August lesson-audit baseline is
current and no new audit trigger is due.

## Scope and non-scope

Advance only `SAFE-11`. Add a behavior-focused cross-layer regression that
enumerates the current production-session blocker vocabulary, applies each
reason to mining state exactly as the production session does, and proves API
output remains work-blocked, safe-blocked, exact, unique, and redaction-safe.
Reconcile the stale Phase 22 blocker ledger with the real current production
path and create a source-bound SAFE-11 evidence summary that joins those tests
to the independently validated SAFE-10 projection and pinned upstream control
flow.

Do not change production blocker ordering, readiness policy, SAFE-10 production
or evaluator inventory, hardware controls, network/pool behavior, mining
campaign policy, API schema, reference source, or any other parity row. No new
dependency is required. Keep the proof in the functional core and existing
API test surface; do not add a projection workflow when compiled behavior plus
committed source-bound evidence is sufficient.

This plan authorizes local source, tests, documentation, build/package, Git
commit, and push only. It authorizes no protected-input access, detector,
credentials, device/USB/network runtime, flash, monitor, mining, restart,
recovery, new hardware attempt, fault injection, external UART/BAP, pins, or
electrical work.

## Implementation

- [ ] Add one exhaustive production blocker-to-runtime-to-API regression with
      exact stable labels, fail-closed state, uniqueness, and redaction-safe
      label constraints.
- [ ] Replace stale Phase 22 mining-loop claims with the current typed
      production readiness/runtime/API chain and exact non-claims.
- [ ] Produce committed SAFE-11 evidence that binds current source/reference,
      the validated SAFE-10 projection, focused/full tests, and privacy review.
- [ ] Commit evidence and `RESULT.md` without checklist changes, save that
      commit as `SOURCE_COMMIT`, then promote only SAFE-11 if every criterion
      below passes.

## Verification and promotion

Focused verification is:

- `cargo test -p bitaxe-api mining::tests`
- `cargo test -p bitaxe-stratum recovery_policy`
- `bazel test //crates/bitaxe-api:bitaxe_api_test //crates/bitaxe-stratum:bitaxe_stratum_test`
- `just validate-safe10-evidence docs/parity/evidence/safe10-prerequisite-readiness/safe10-projection.json`
- `just verify-reference`

The mandatory ordered gates are `cargo fmt --all`, `cargo clippy --all-targets
--all-features -- -D warnings`, `cargo build --all-targets --all-features`,
`cargo test --all-features`, `bun scripts/bright-builds-check.ts all`, `just
test`, `just parity`, and `just parity-progress`. Evidence verification also
includes exact source/reference identities, SAFE-10 projection digest and mode,
direct sensitive-key/value review, file-size review, selector review, and final
diff review.

Promotion requires all current `ProductionSessionBlocker` variants to have one
unique lowercase-ASCII category label; applying every variant through the same
state operation used by production must yield work submission `blocked`, mining
activity `safe_blocked`, and exact API `blockedReason`. Ready/non-safe shapes
must not expose stale reason text. The current runtime source must consume the
typed enum rather than caller-provided protected data, and reference inspection
must still show upstream fail-closed pool pause/power behavior without claiming
wire-identical reason strings.

The accepted SAFE-10 projection must independently validate and continue to
prove detector-admitted board-205 live prerequisite success, source/reference
lineage, protected modes, safe stop, cleanup, and redaction. SAFE-11 evidence
must contain only commits, digests, paths, closed labels, counts, booleans, and
command outcomes—never credentials, owner/pool/worker data, endpoints, ports,
USB/network identity, telemetry values, raw logs/payloads, or protected IDs.

On success create `RESULT.md`, commit evidence without checklist change and
save that full commit as `SOURCE_COMMIT`; transition only `SAFE-11` to
`verified` with `unit,workflow,hardware-smoke,hardware-regression`, update its
Rust-owned target to the real production readiness/runtime/API chain, sync
progress, archive only this task, run final gates, and push. The final notes
must satisfy the legacy Phase 28 evidence guard while preserving exact
non-claims. On failure create `CLOSURE.md`, leave SAFE-11 `implemented`, and do
not synchronize unchanged progress.

## Non-claims

This plan does not inject live faults or verify individual active voltage, fan,
thermal, or power-control effects; self-test; BAP/UART; arbitrary telemetry;
other boards/ASICs; arbitrary profiles/pools; unbounded mining; OTA/recovery;
or release readiness. Upstream and Rust reason labels need not be wire-identical;
parity is the observable fail-closed behavior and stable Rust operator category.
