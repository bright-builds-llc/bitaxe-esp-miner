# Parity work plan

- Run ID: `20260820T052841Z-SAFE-12`
- Parity row: `SAFE-12`
- Initial status: `implemented`
- Source commit: `41c9ebaae34a535e96f36803d8a6df7fee59fe79`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-safe12-production-safe-stop`

## Selection

The clean `main` worktree equals `origin/main`, the pinned reference is clean,
and the selector reports no open plan. Candidate order begins `SELF-001`,
`BAP-002`, `STAT-003`, then `SAFE-12`.

`SELF-001` remains unavailable because no production-safe full self-test route
exists for its hardware regression. `BAP-002` remains dependency- and safety-
blocked by unfinished `BAP-001` firmware/UART lifecycle plus unauthorized
external electrical UART work. `STAT-003` remains environment-blocked after
attempt-004's distinct `network_unavailable` result; its active task prohibits
an unchanged retry without an objective protected pool/network recovery signal.
`SAFE-12` is therefore first actionable.

Later accepted evidence supersedes the Phase 25/28 prerequisite artifacts for
this exact row. SAFE-10 proves a detector-admitted 600-second accepted campaign
reached final consumed state, confirmed device-local safe stop, and cleanup.
STR-006 proves ordered terminal safe stop after an accepted response. PWR-002
proves the rollback plan attempted all eight steps and commanded ASIC disable;
PWR-003 proves the active-low core-rail disable path. Current pure/session and
firmware host tests cover effect ordering, physical step ordering, status
publication, watchdog progress, and idempotent confirmation.

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

Advance only `SAFE-12`. Produce a source-bound evidence summary joining the
accepted SAFE-10, STR-006, PWR-002, and PWR-003 projections to current
production-session and firmware safe-stop tests. No new runtime implementation
or hardware projector is needed because the complete ordered stop path and its
accepted hardware confirmations already exist.

Bind these accepted projections exactly:

- SAFE-10 prerequisite readiness:
  `4e9b91bd29629aec098b9967b9bb27b9c1358f64c11819a77f8c8da4c212a20e`
- STR-006 protocol coordinator:
  `f008171f26b7a8ae6b08859e3cfef4f0c5bf88937c049dd66b6f868c9bbfd6f7`
- PWR-002 ASIC power initialization:
  `0668c274d09b3e39d7d5edfea4b2e66c97248ff77de9192981f3af00e547ddfe`
- PWR-003 core-voltage control:
  `11dd1abbf6fda86d203fdcff49b420ab5139e1d29c35f4d17000c61c3112ae68`

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

- [ ] Independently validate the accepted SAFE-10, STR-006, PWR-002, and
      PWR-003 projections.
- [ ] Run current production-session safe-stop and firmware actuation/status/
      owner-progress tests.
- [ ] Produce `summary.md`, `WORKLOG.md`, and `RESULT.md` with exact digests,
      stop sequence, conclusions, and non-claims.
- [ ] Commit evidence as `SOURCE_COMMIT`, transition only SAFE-12, sync
      progress, archive this task, final-gate, and push.

## Verification and promotion

Focused verification is:

- `cargo test -p bitaxe-stratum safe_stop`
- `cargo test -p bitaxe-stratum production_session`
- `bazel test //firmware/bitaxe:mining_actuation_tests //firmware/bitaxe:production_campaign_status_tests //firmware/bitaxe:production_owner_progress_tests`
- the existing SAFE-10, STR-006, PWR-002, and PWR-003 Rust evidence validators
  over absolute projection paths
- `just verify-reference`
- `just package`

The mandatory ordered gates are `cargo fmt --all`, `cargo clippy --all-targets
--all-features -- -D warnings`, `cargo build --all-targets --all-features`,
`cargo test --all-features`, `bun scripts/bright-builds-check.ts all`, `just
test`, `just parity`, and `just parity-progress`, plus projection digests/modes,
redaction, selector, sensitive-value, source-diff, and final diff checks.

Promotion requires detector-gated live safety hardware proof that one accepted
Ultra 205 production campaign stopped submissions, invalidated work, closed its
pool transport, stopped ASIC interaction, performed the complete safe hardware
shutdown sequence, published disabled mining/control/work-submission state,
consumed the lease, confirmed terminal state and cleanup, and retained trusted
identity, fresh safety, current source/tests, independent validation, and
redaction. Checklist notes must include the Phase 28 summary path,
`detector-gated live safety hardware proof`, `redaction_status: passed`, and
`exact_non_claims`, and must omit blocker terms.

On success create `RESULT.md`, commit evidence without a checklist change and
save that full commit as `SOURCE_COMMIT`; transition only `SAFE-12` to
`verified` with `unit,workflow,hardware-smoke,hardware-regression`, sync
progress, archive only this task, run final gates, and push. On failure create
`CLOSURE.md`, leave SAFE-12 `implemented`, and do not sync unchanged progress.

## Non-claims

This plan does not verify fault-injected safe stop on hardware, per-step
electrical timing or waveform measurement, power-loss interruption, automatic
thermal/fan fault recovery, arbitrary profiles or pools, other boards/ASICs,
unbounded mining, OTA/recovery, or release readiness. It does not promote
SAFE-13.
