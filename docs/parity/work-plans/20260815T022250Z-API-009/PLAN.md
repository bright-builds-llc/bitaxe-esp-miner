# Parity work plan

- Run ID: `20260815T022250Z-API-009`
- Parity row: `API-009`
- Initial status: `implemented`
- Source commit: `f6e215b7db0c19fcc17afb83417afe7520e633c9`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-api009-command-effect-evidence-audit`
- Continues plan: `docs/parity/work-plans/20260815T015842Z-API-009/PLAN.md`

## Selection and diagnosis

Clean synchronized HEAD has no open plan and the deterministic selector ranks
API-009 first. Attempt-024 proved the repaired host boundary by confirming one
resume request and API-visible resume intent after the complete physical
IDENTIFY transaction. The first failure then moved into firmware: an unchanged
stale safety observation arrived while the prior active epoch was paused, the
operator had requested resume, the campaign was not yet active, hardware was
ready, and the primary pool was reconnecting. The production session treated
that transient reactivation state as a terminal safety blocker, consumed the
lease, and safe-stopped before the host's bounded reactivation phase could
succeed.

The pure session already preserves a resumable lease when the stale sample is
present on the initial resume wake while hardware is stopped. It does not
preserve the lease if the same lapse arrives after hardware preparation but
before active mining. The missing distinction is whether the campaign has a
prior resumable active epoch and no current active segment. Active-mining
safety loss must remain terminal.

## Scope and non-scope

This is a software-only state-transition repair. During a resumable campaign
that has previously been active, treat `SafetyPrerequisitesStale` as a
resumable safe-stop purpose only while no active segment is currently running.
Stop any prepared hardware, keep the same lease and accumulated active budget,
remain armed, and allow a later fresh observation to reprepare and reconnect.
Once active mining resumes, clear this narrow eligibility naturally through
the current-active-segment state; any later stale safety observation remains a
terminal safe stop.

Do not broaden resumability to missing leases, operator-independent network or
protocol failures, actuation blockers, preparation failures, active mining, or
non-resumable campaigns. Preserve all ordering, pool teardown, work
invalidation, hardware safe-stop, terminal publication, duration, activation,
overflow, clock-regression, and higher-lease behavior.

Update only pure production-session logic and behavior-focused tests, plus any
firmware owner/marker regression required to prove the live attempt-024 shape.
Preserve host resume/recovery orchestration and closed evidence v6 unchanged.
This plan authorizes source, tests, deterministic fixtures, documentation,
tracker, worklog, closure, and ordinary firmware builds only. It does not
authorize credentials, protected attempt artifacts, detector, USB,
device/network access, device HTTP, display claims, mining, restart, hardware
control, direct UART, pins/pads/GPIO, public parity evidence, checklist
promotion, attempt-025, or any hardware attempt.

## Implementation and verification

- [ ] Add one explicit pure predicate for resumable pre-active safety
      reactivation: resumable lease, prior active epoch, no current active
      segment, and stale safety blocker.
- [ ] Use that predicate to request `ResumablePause` hardware safe stop and
      retain the lease/budget across a stale sample after hardware preparation.
- [ ] Add the live-shaped regression: active -> operator pause -> stopped ->
      resume -> hardware ready/primary connecting -> stale safety -> resumable
      stopped -> fresh observation -> reprepare/reconnect -> active.
- [ ] Add negative controls proving initial activation and already-active
      safety staleness remain terminal, and preserve existing lease timing and
      readiness-recovery behavior.
- [ ] Run focused pure-session and firmware-owner targets, then the complete
      ordered Cargo, Bright Builds, Bazel, parity, privacy, reference,
      real-firmware, selector, unique-task, immutable-plan, sensitive-output,
      and diff gates.

Run in order: `cargo fmt --all`;
`cargo clippy --all-targets --all-features -- -D warnings`;
`cargo build --all-targets --all-features`; `cargo test --all-features`;
`bun scripts/bright-builds-check.ts all`; `just test`; `just parity`; and
`just parity-progress`. Also run `just verify-redaction`,
`just verify-reference`, `just build`, selector, unique-task, immutable-plan
digest, reference cleanliness, sensitive-output, `git diff --check`, and full
diff review.

Success closes this software-only plan with API-009 still `implemented`. It
proves deterministic reactivation safety handling only. A fresh immutable
hardware plan is required before detector admission or attempt-025.
