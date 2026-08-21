# SELF-001 Attempt-002 Recovery Plan

## Identity

- Parity row: `SELF-001`
- Active task: `task-parity-self001-full-lifecycle`
- Prior plan: `docs/parity/work-plans/20260821T180800Z-SELF-001/PLAN.md`
- Prior attempt: `attempt-001`
- Authorized attempt: `attempt-002`

## Reason for a fresh attempt

Attempt-001 stopped before settings backup, NVS mutation, package installation,
or self-test hardware effects. Its earliest failure was the closed settings
preflight category: the host supervisor requested the nonexistent
`/api/system/theme` route instead of the upstream-compatible `/api/theme`
route. Its typed error was also absent from the CLI failure registry, so the
outer result lost the more specific `hardware_blocked` category. The protected
wrapper output and empty mode-0700 attempt root remain preserved.

Attempt-002 is eligible only after both root causes have regression tests, all
software/privacy/package gates pass, the repair is committed and pushed, and a
new canonical package is bound to that exact clean source. This is a changed
attempt after verified progress, not an unchanged retry.

## Scope

The lifecycle, safety envelope, configuration preservation, private/public
evidence boundary, built-in BOOT-button checkpoint, pass criteria, cleanup,
promotion rules, and non-claims remain exactly those in the prior plan. No
public mutation API, external UART, pin manipulation, probe, jumper, sensor
fault, thermal fault, electrical fault, or communication fault is authorized.

The retry-specific implementation changes are limited to:

- use `/api/theme` for theme capture, restoration, and confirmation;
- retain `SelfTestCampaignError` as a typed CLI failure;
- bind every intent, state, evidence record, and workflow digest to ordinal 2;
- use protected `wrapper-002` and `attempt-002` paths.

## Exact hardware contract

After the repair is clean, verified, committed, pushed, and package-bound, run
only:

1. `just package`
2. `test ! -e scratch/self001-full-lifecycle/wrapper-002 && (umask 077; mkdir -m 700 -p scratch/self001-full-lifecycle/wrapper-002 && just detect-ultra205 > scratch/self001-full-lifecycle/wrapper-002/detector.stdout 2> scratch/self001-full-lifecycle/wrapper-002/detector.stderr)`
3. `test ! -e scratch/self001-full-lifecycle/attempt-002 && test ! -e docs/parity/evidence/self001-full-lifecycle/self-test-projection.json && (umask 077; just self-test-campaign start --private-root scratch/self001-full-lifecycle/attempt-002 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --pool-credentials pool-credentials.json --detector-output scratch/self001-full-lifecycle/wrapper-002/detector.stdout --plan docs/parity/work-plans/20260821T192123Z-SELF-001-RETRY/PLAN.md --projection docs/parity/evidence/self001-full-lifecycle/self-test-projection.json > scratch/self001-full-lifecycle/wrapper-002/start.stdout 2> scratch/self001-full-lifecycle/wrapper-002/start.stderr)`
4. Only after start publishes `cancel_ready` with `safe_state=true`, the user
   holds the built-in BOOT button for two seconds, and the physical action is
   acknowledged: `just self-test-campaign resume --private-root scratch/self001-full-lifecycle/attempt-002 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --detector-output scratch/self001-full-lifecycle/wrapper-002/detector.stdout --plan docs/parity/work-plans/20260821T192123Z-SELF-001-RETRY/PLAN.md --projection docs/parity/evidence/self001-full-lifecycle/self-test-projection.json`

The start preflight must capture `/api/system/info` and `/api/theme`, prove
mode-0600 ignored credential inputs can exactly reconstruct all restorable
settings, and write the private backup before any mutation. All active and
safe-stop bounds remain unchanged from the prior plan.

## Stop, recovery, and promotion

Preserve the earliest closed failure. After any effect, attempt every
independent safe-stop and exact settings/package restoration step, clean USB
and process ownership, withhold `RESULT.md`, and do not rerun attempt-002.
Before any effect, retain the protected typed failure and stop. No unchanged
retry is authorized.

Promote only after both phases share the ordinal-2 lease and exact source,
reference, plan, package, detector, restart, restoration, cleanup, validation,
and redaction quorum. On success, create `RESULT.md`, transition only
`SELF-001` to `verified` with `unit,workflow,hardware-regression`, synchronize
progress, archive the completed task, final-verify, commit, and push.
