# Parity work plan

- Run ID: `20260820T054935Z-SAFE-13`
- Parity row: `SAFE-13`
- Initial status: `implemented`
- Source commit: `dd312c358f9de6604178a2a90a9f97b35de0d590`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-safe13-live-watchdog-responsiveness`

## Selection

The clean `main` worktree equals `origin/main`, the pinned reference is clean,
and the selector reports no open plan. Candidate order begins `SELF-001`,
`BAP-002`, `STAT-003`, then `SAFE-13`.

`SELF-001` remains unavailable because no production-safe full self-test route
exists for its hardware regression. `BAP-002` remains dependency- and safety-
blocked by unfinished `BAP-001` firmware/UART lifecycle plus unauthorized
external electrical UART work. `STAT-003` remains environment-blocked after
attempt-004's distinct `network_unavailable` result; its active task prohibits
an unchanged retry without an objective protected pool/network recovery signal.
`SAFE-13` is therefore first actionable.

Later accepted evidence supersedes the Phase 25/28 prerequisite artifacts for
this exact row. SAFE-10 proves `watchdog_valid` throughout all 20 windows of a
600-second detector-admitted accepted campaign. STR-006 proves the production
owner loop feeds its watchdog across the accepted coordinator lifecycle. The
runtime-health projection independently proves participating, fresh,
non-regressing task-watchdog and supervisor-checkpoint observations through
same-boot HTTP/WebSocket views. Current safety/core/session and firmware host
tests cover thresholds, participation failures, progress subphases, freshness,
sequence behavior, and effect heartbeats.

Loaded guidance materially includes `AGENTS.md`, `AGENTS.bright-builds.md`,
`standards-overrides.md`, managed architecture, code-shape, verification,
testing, and Rust standards, the active tracker/checklist, and bounded lessons
for protected evidence, redaction, hardware authorization, retries, units, and
agent-runtime timing. Active lesson inputs total 31,758 bytes, so every heading
was inventoried and priority complete blocks plus the complete global file were
loaded; remaining non-priority repository blocks were omitted under the
deterministic budget. The August lesson-audit baseline remains current and no
new audit trigger is due.

## Scope and non-scope

Advance only `SAFE-13`. Produce a source-bound evidence summary joining the
accepted SAFE-10, STR-006, and runtime-health projections to current watchdog,
runtime-health, production-session, owner-progress, checkpoint, and observation
tests. No new runtime implementation or hardware projector is needed because
the sustained-load watchdog proof and current observation path already exist.

Bind these accepted projections exactly:

- SAFE-10 prerequisite readiness:
  `4e9b91bd29629aec098b9967b9bb27b9c1358f64c11819a77f8c8da4c212a20e`
- STR-006 protocol coordinator:
  `f008171f26b7a8ae6b08859e3cfef4f0c5bf88937c049dd66b6f868c9bbfd6f7`
- V12 runtime health:
  `44f081451d61ecc59dd21f70d72fae7d71e9611441d406f31f441727e5a11e14`

The evidence summary may contain only repository paths, commits, digests,
closed labels, booleans, counts, bounded durations, and command outcomes. It
must contain no pool or credential values, endpoints, ports, IP/MAC/USB
identity, device URLs, raw ASIC/Stratum values, logs/payloads, PIDs, traces, or
protected identifiers.

This plan authorizes local tests, committed-public evidence reads,
documentation, build/package, Git commit, and push only. It authorizes no
credential or protected-attempt access, detector, device/USB/network runtime,
flash, monitor, mining, restart, recovery, hardware attempt, fault injection,
external UART/BAP, pins, or electrical work.

## Implementation

- [ ] Independently validate the accepted SAFE-10, STR-006, and runtime-health
      projections.
- [ ] Run current watchdog/runtime-health/session tests and firmware watchdog/
      owner-progress/checkpoint observation targets.
- [ ] Produce `summary.md`, `WORKLOG.md`, and `RESULT.md` with exact digests,
      sustained-load facts, conclusions, and non-claims.
- [ ] Commit evidence as `SOURCE_COMMIT`, transition only SAFE-13, sync
      progress, archive this task, final-gate, and push.

## Verification and promotion

Focused verification is:

- `cargo test -p bitaxe-safety watchdog`
- `cargo test -p bitaxe-core runtime_health`
- `cargo test -p bitaxe-stratum production_session`
- `bazel test //firmware/bitaxe:production_owner_progress_tests //firmware/bitaxe:supervisor_checkpoint_production_tests //firmware/bitaxe:task_watchdog_observation_tests //firmware/bitaxe:runtime_health_no_effects_test`
- the existing SAFE-10, STR-006, and runtime-health Rust evidence validators
  over absolute projection paths
- `just verify-reference`
- `just package`

The mandatory ordered gates are `cargo fmt --all`, `cargo clippy --all-targets
--all-features -- -D warnings`, `cargo build --all-targets --all-features`,
`cargo test --all-features`, `bun scripts/bright-builds-check.ts all`, `just
test`, `just parity`, and `just parity-progress`, plus projection digests/modes,
redaction, selector, sensitive-value, source-diff, and final diff checks.

Promotion requires detector-gated live safety hardware proof that the
production watchdog remained valid through every window of a bounded accepted
Ultra 205 mining campaign, while current source proves owner-loop subscription,
feed/progress behavior, bounded thresholds, fresh participating observation,
non-regressing sequences, healthy supervisor checkpoints, safe stop, cleanup,
independent validation, and redaction. Checklist notes must include the Phase 28
summary path, `detector-gated live safety hardware proof`,
`redaction_status: passed`, and `exact_non_claims`, and must omit blocker terms.

On success create `RESULT.md`, commit evidence without a checklist change and
save that full commit as `SOURCE_COMMIT`; transition only `SAFE-13` to
`verified` with `unit,workflow,hardware-smoke,hardware-regression`, sync
progress, archive only this task, run final gates, and push. On failure create
`CLOSURE.md`, leave SAFE-13 `implemented`, and do not sync unchanged progress.

## Non-claims

This plan does not verify deliberate watchdog starvation or task stalls on
hardware, actual watchdog-triggered reset/recovery, arbitrary long-running or
unbounded load, every firmware task, other boards/ASICs, fault injection,
OTA/recovery, or release readiness.
