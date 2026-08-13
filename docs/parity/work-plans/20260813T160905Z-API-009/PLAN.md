# Parity work plan

- Run ID: `20260813T160905Z-API-009`
- Parity row: `API-009`
- Initial status: `implemented`
- Source commit: `e2d1fabb6d15dac02a7c395d1966d8077124599f`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-api009-command-effect-evidence-audit`
- Continues plan: `docs/parity/work-plans/20260813T154249Z-API-009/PLAN.md`

## Selection and diagnosis

The clean synchronized selector has no open plan and ranks API-009 first, so
no candidate is skipped. Attempt-011 crossed exact-package flash, runtime
identity, active mining, genuine block, and accepted-share boundaries, then
sent one pause request. Its 130-second pause join expired without the firmware
ever publishing the resumable safe-stop confirmation. The sealed terminal
category is `network_correlation_failed`; `safety_prerequisites_stale` was the
next readiness blocker at the deadline, not the cause of the missing pause.

The production ownership chain exposes the root cause. The HTTP command writer
applies requested operator intent directly to `CommandVisibleState.mining`.
The production owner independently replaces that entire `mining` value on each
session publication. Readiness then recovers requested intent by reading the
same replaceable projection. A publication racing the HTTP pause can therefore
restore the prior `Run` value before the owner reads it. No separate
boot-lifetime requested-intent owner exists. This matches the hardware facts:
active HTTP state continued, no resumable safe stop appeared, and the host
timed out at the exact join deadline.

## Scope and non-scope

Introduce one distinct boot-lifetime requested operator-intent owner inside the
runtime snapshot shell. Boot preference and pause/resume commands may write it;
production readiness may read it; production session publication may not
replace it. Continue publishing the session-derived mining state for API and
telemetry behavior. Add a behavioral host regression for the exact interleaving
of pause write, stale owner publication, and authoritative readiness read, plus
source-ownership assertions that prevent collapsing the two owners again.

This is software-only work. Do not access credentials, the detector, USB, the
device or its network, protected attempt contents beyond the already classified
typed metadata, HTTP effects, flash/reset, mining, hardware controls, direct
UART, pins/pads/GPIO, or attempt-012. Do not change safety thresholds, pause
deadlines, evidence quorum, redaction, recovery, or promotion status.

## Implementation

- [ ] Add a private requested-intent state boundary with explicit boot,
      command-write, and readiness-read methods.
- [ ] Keep production publication limited to session-derived mining state so a
      stale publication cannot overwrite a newer pause or resume request.
- [ ] Route authoritative readiness through requested intent and add behavioral
      interleaving plus source-ownership regressions.
- [ ] Run focused API/firmware/production-session and canonical real-process
      tests, then every mandatory software/privacy/reference gate.
- [ ] Close as a software fix with API-009 still `implemented`; no hardware
      evidence or attempt-012 is authorized by this plan.

## Verification

Before implementation, commit and push this immutable plan/task checkpoint
after selector, task binding, plan digest, reference, diff, and plan-only gate
checks pass. After implementation run focused command-intent, resumable-pause,
firmware source-ownership, and campaign orchestration tests, then in order:

1. `cargo fmt --all`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo build --all-targets --all-features`
4. `cargo test --all-features`
5. `bun scripts/bright-builds-check.ts all`
6. `just test`
7. `just parity`
8. `just parity-progress`

Also run `just verify-redaction`, `just verify-reference`, `just build`, the
real selector, immutable-plan digest, unique task binding, reference
cleanliness, `git diff --check`, and full diff/sensitive-output review. Promote
nothing and publish no hardware evidence.
