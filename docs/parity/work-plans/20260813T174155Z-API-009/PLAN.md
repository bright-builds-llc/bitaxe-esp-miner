# Parity work plan

- Run ID: `20260813T174155Z-API-009`
- Parity row: `API-009`
- Initial status: `implemented`
- Source commit: `0a7b239877642adf5f33684402dd92344de88f35`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-api009-command-effect-evidence-audit`
- Continues plan: `docs/parity/work-plans/20260813T163921Z-API-009/PLAN.md`

## Selection and diagnosis

The clean synchronized selector has no open plan and ranks API-009 first, so
no candidate is skipped. Attempt-012 reached its post-flash and monitor USB
admission boundaries, then the TypeScript parent killed the Rust campaign at
its fixed 810-second timeout. The child itself reserves 600 seconds for the
command-effects observation and 180 seconds for terminal grace. Package
admission, factory flash, NVS flash, their recovery windows, monitor admission,
and final cleanup occur outside that 780-second observation window but inside
the same 810-second parent budget. The parent can therefore preempt the child's
typed terminal result, safe stop, and cleanup by construction.

The public timeout category was preserved, but recovery facts defaulted false
because `campaignRecoveryFacts` was not consulted for the timed-out outcome.
The host process adapter did terminate the owned child tree and the admitted
port became holder-free, yet neither fact can replace the missing sealed child
result. This plan fixes the orchestration contract, not the hardware outcome.

## Scope and non-scope

Introduce one typed command-effects transaction budget in the automation
module. It must name the 600-second required observation and child terminal
grace, conservatively include the bounded USB preparation/recovery and process
termination envelope, use checked arithmetic, and derive the parent timeout.
The invariant must be testable without wall-clock hardware runs: the parent is
strictly greater than the complete child lifetime and cannot regress to the
attempt-012 fixed 810-second budget.

When the outer guard still expires, preserve `timeout` as the primary category
while reading any already closed campaign result/network files through the
existing private validator to report safe-stop, cleanup, recovery, and
secondary-recovery booleans. Missing or incomplete recovery artifacts remain
safe false values and never weaken the primary failure.

Add unit tests for budget arithmetic/invariants, typed timeout recovery facts,
missing/malformed recovery artifacts, and primary precedence. Add a real-child
process regression proving the configured parent budget permits a child to
write its closed cleanup artifact before the parent guard. Keep production
campaign duration, terminal grace, device-session timeout, evidence quorum,
safety policy, privacy, and redaction unchanged.

This plan is software-only. Do not access credentials, the detector, USB, the
device or its network, protected attempt contents beyond the public closure,
HTTP effects, flash/reset, mining, display, hardware controls, direct UART,
pins/pads/GPIO, or attempt-013. Do not promote API-009 or publish evidence.

## Verification

- [ ] Add the typed budget and bind the campaign child timeout to it.
- [ ] Preserve typed primary timeout while projecting only validated recovery
      facts already closed by the child.
- [ ] Add focused unit, source-contract, and real-process regressions.
- [ ] Run `cargo fmt --all`, strict all-target/all-feature Clippy, all-target
      build, all-feature tests, Bright Builds, `just test`, `just parity`, and
      `just parity-progress` in order.
- [ ] Run `just verify-redaction`, `just verify-reference`, `just build`, the
      real selector, immutable-plan digest, unique task binding, reference
      cleanliness, sensitive-output review, `git diff --check`, and full diff
      review.
- [ ] Close as a software fix with API-009 still `implemented`; no later
      hardware ordinal is authorized by this plan.

Before implementation, commit and push this immutable plan/task checkpoint
after every plan-only gate passes. After implementation, perform a
simplification pass to keep one source of truth for the outer budget rather
than scattering larger timeout literals.
