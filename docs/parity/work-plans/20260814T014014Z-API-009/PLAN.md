# Parity work plan

- Run ID: `20260814T014014Z-API-009`
- Parity row: `API-009`
- Initial status: `implemented`
- Source commit: `1c3f8d5bb180cbe3e1fff41010cc30e8233fb4df`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-api009-command-effect-evidence-audit`
- Continues plan: `docs/parity/work-plans/20260813T180631Z-API-009/PLAN.md`

## Selection

The clean synchronized selector has no open plan and ranks API-009 first, so
no candidate is skipped. Attempt-013 proved exact-package factory and NVS
flashing, trusted runtime identity, fresh safety, and ready USB cleanup, but
closed `terminal_state_unconfirmed` after only two milliseconds of active
mining and before any command checkpoint.

A fast ignored production-session harness now deterministically reproduces the
failure: a resumable lease with a 100-millisecond duration enters
`SafeStopping` at the exact 100-millisecond boundary while hardware remains
`Preparing` and before the session has ever been active. Boundary probes at
100 and 200 milliseconds prove linear clock units. Production admission maps
command effects to `ResumableWallClockDuration`, submit consumption is guarded
to `FirstSubmitResponse`, and the firmware's sealed marker—not the host—records
`campaign_lease_consumed`. The confirmed cause is the single lease clock being
anchored when preparation is requested instead of when the resumable active
epoch begins.

This is objectively new, locally reproducible production-boundary information.
The software fix is actionable without credentials, USB, device, network, or
another attempt ordinal.

## Scope and non-scope

Replace the ambiguous one-clock resumable lease with an explicit bounded
two-phase contract. The activation phase starts when hardware preparation is
requested and has its own closed timeout. The resumable observation epoch
starts exactly once on the first active state, continues across operator pause
and resume, and expires only after its own duration. Preserve one-shot lease
identity, ordered safe stop, first failure, pool recovery, and all other lease
variants.

Carry the two-phase timing through production settings admission, firmware
status projection, host campaign capture, and the TypeScript parent/fixture
budget so no supervising process can preempt the new child maximum. Add a
typed activation-timeout terminal reason/category rather than misreporting a
pre-active failure as successful command-effects completion. Keep all clocks
integer, checked, and derived from one source contract where they cross
language boundaries.

This plan is software-only. Do not access credentials, detector or protected
attempt traces; USB, device or network interfaces; issue HTTP commands; flash,
reset, restart, mine, manipulate the display or hardware controls; use direct
UART or pins/pads/GPIO; publish evidence; or create/run attempt-014. The
ignored red harness may be retained only until its regression is moved to the
real tracked seam, then must be removed.

ADR-0016/0017 require the Production Mining Session to remain the sole deep
owner. The functional-core, code-shape, Rust, testing, and verification
standards require typed illegal-state prevention, focused Arrange/Act/Assert
tests, thin adapters, and repo-native gates.

## Implementation

- [ ] Turn the exact pre-active expiry into a fast failing regression at the
      public Production Mining Session seam before changing implementation.
- [ ] Model activation and resumable-epoch clocks separately; prove activation
      is bounded, the epoch cannot expire before first active, begins exactly
      once, persists across pause/resume, and cannot be replayed after terminal
      consumption.
- [ ] Preserve `FirstSubmitResponse` and `ActiveDuration` semantics and add a
      closed activation-timeout blocker/marker/host terminal category.
- [ ] Derive Rust capture and TypeScript child/parent/fixture bounds from the
      activation timeout, 600-second resumable epoch, terminal grace, USB
      envelope, and cleanup rather than adding another unrelated literal.
- [ ] Add focused production-session, settings mapping, marker assessment,
      campaign timeout, cross-language source-contract, and real-process tests.
- [ ] Remove the ignored repro and all temporary instrumentation; perform a
      simplification pass before closure.

## Verification and promotion

First run the new regression alone and record its red verdict, apply the fix,
then require the same regression and original ignored harness to turn green.
Run focused `bitaxe-stratum`, firmware host, flash campaign, automation budget,
CLI, source-contract, and real-process targets.

Before plan commit and final source commit, run in order:

1. `cargo fmt --all`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo build --all-targets --all-features`
4. `cargo test --all-features`
5. `bun scripts/bright-builds-check.ts all`
6. `just test`
7. `just parity`
8. `just parity-progress`

Also run `just verify-redaction`, `just verify-reference`, a real ESP firmware
build, immutable-plan digest and unique task-binding checks, selector closure,
reference cleanliness, `git diff --check`, sensitive-output review, and full
diff review. Close and push this plan with API-009 still `implemented`; no
checklist transition or parity evidence is allowed. Only a later clean
synchronized selector may decide whether the regression-backed fix justifies a
separate immutable attempt-014 contract.
