# Parity work plan

- Run ID: `20260812T173427Z-API-009`
- Parity row: `API-009`
- Initial status: `implemented`
- Source commit: `aa9ffdb40b4fbf0c42d47dd68d84ad32b75b197e`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-api009-command-effect-evidence-audit`
- Continues plan: `docs/parity/work-plans/20260812T170039Z-API-009/PLAN.md`

## Selection

The clean synchronized selector again ranks API-009 first, so no row is
skipped. Attempt-004 materially resolved the prior NVS-owner blocker: both
writes completed on attempt one, exact runtime identity was trusted, the typed
protocol gate was `ready`, genuine block notification and eight qualified
candidates were observed, and pause was requested and confirmed. The sole
attempt then stopped after the one resume request because the production
readiness decision reported `safety_prerequisites_stale`; resume was not
confirmed and IDENTIFY, dismissal, and restart were not attempted.

The terminal campaign marker simultaneously retained all five required Ultra
205 observations as fresh and safe, while the earlier readiness blocker stayed
`safety_prerequisites_stale`. Source inspection shows the production owner
samples readiness on both a one-second deadline and category-only observation
notifications, while the independent sensor owner runs a 500-ms deadline with
a 1,000-ms age bound. The marker independently resamples observations after
the readiness event. This permits a wakeup to decide stale immediately before
a producer replacement and then publish fresh marker facts without recording
which observation epoch the decision used. The pure recovery policy is
intended to reprepare from stopped hardware after a later fresh observation,
but no production-shaped test currently proves that exact stale-then-fresh
pause/resume path or the notification/coalescing boundary.

The active lessons remain above the deterministic loading budget with the
unchanged 2026-08-03 audit baseline and no new trigger. The same complete
safety, authorization, evidence, retry, redaction, USB-identity,
earliest-failure, real-process, ESP-IDF, and host-stall lesson blocks remain
loaded; the previously disclosed unrelated omitted set remains unchanged.

## Scope and non-scope

Add a bounded, value-free readiness-transition record owned by the production
session. It must bind the wakeup category, previous and current closed blocker,
campaign/hardware phase, whether the exact sampled observation epoch was safe,
and whether a later producer notification advanced beyond it. Carry only this
closed record through campaign marker v11 and protected result v8; do not emit
sensor values, timestamps, sequence numbers, origins, hostnames, ports, USB or
network identities, credentials, endpoints, paths, or raw logs.

Build a production-shaped deterministic shell test for the exact sequence:
active campaign, pause, confirmed hardware safe stop, resume wake at a stale
observation boundary, a later fresh producer replacement/notification, and
repreparation to active mining under the same lease. Use the test to locate the
real missed transition. Fix the smallest confirmed ownership or scheduling
defect so a fresh observation epoch is processed after the stale resume wake;
do not weaken freshness limits, bypass fail-closed safety, synthesize fresh
truth, or accept stale observations.

Only if the regression fails before the fix, passes after it, and every source,
privacy, recovery, package, and detector gate passes may one fresh
`attempt-005` use the existing `just api-command-effects-campaign` interface,
an exact pushed package, a fresh mode-`0700` ignored private root, a fresh
public projection, and fresh detection of exactly one Ultra 205. Effects and
the 600-second local-fixture lease remain identical to attempt-004.

No external pool, owner pool credential, diagnostic setter, erase, OTA,
rollback, power cycle, direct UART, pin/header/test-point interaction, fault
injection, voltage/frequency/fan override, control override, or second retry is
allowed. Instrumentation without a reproduced and fixed transition is not
hardware eligibility.

## Implementation

- [ ] Add one closed readiness-transition model and bind the exact sampled
      observation epoch to production state and campaign marker/result schemas.
- [ ] Reproduce stale-resume then fresh-notification recovery at the real
      production shell seam and fix only the confirmed missed transition.
- [ ] Add ordering, coalescing, stale-preservation, recovery, schema,
      redaction, and sensitive-output regressions without weakening safety.
- [ ] Run every focused and mandatory gate, review the ownership and privacy
      surfaces, then commit and push the exact source before hardware.
- [ ] Conditionally run the sole detector-gated attempt-005 and publish only a
      complete API-009 quorum.

## Verification and promotion

Focused tests must prove stale input remains fail-closed, the stale resume wake
cannot itself prepare hardware, a later genuinely fresh producer epoch is not
lost to notification coalescing or deadline scheduling, the same lease
reprepares and returns active exactly once, closed transition evidence has
bounded vocabulary, and no raw or identifying values enter protected results
or public output. The real firmware target must compile against pinned
ESP-IDF. Then run, in order:

1. `cargo fmt --all`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo build --all-targets --all-features`
4. `cargo test --all-features`
5. `bun scripts/bright-builds-check.ts all`
6. `just test`
7. `just parity`
8. `just parity-progress`

Also require `just verify-redaction`, `just verify-reference`, generated
contracts, selector and unique-task binding, immutable-plan digest, reference
cleanliness, sensitive-output review, fresh attempt/projection paths,
`git diff --check`, and final diff review. Commit and push this plan/task
checkpoint before implementation, and commit and push verified source before
hardware.

Promotion still requires the complete five-command device-user quorum: genuine
network-target notification dismissal, both physical IDENTIFY observations,
pause/resume, exactly one software restart, same physical device, exact build,
changed boot session, ordinal `N+1`, safe stop, cleanup, recovery, and
redaction. Otherwise retain API-009 at `implemented`, preserve the earliest
typed category, withhold public evidence, close truthfully, and do not retry.
