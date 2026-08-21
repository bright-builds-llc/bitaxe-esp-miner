# SELF-001 Attempt-003 Recovery Plan

## Identity

- Parity row: `SELF-001`
- Active task: `task-parity-self001-full-lifecycle`
- Original plan: `docs/parity/work-plans/20260821T180800Z-SELF-001/PLAN.md`
- Attempt-002 plan: `docs/parity/work-plans/20260821T192123Z-SELF-001-RETRY/PLAN.md`
- Authorized attempt: `attempt-003`

## Prior pre-effect failures

Attempt-001 stopped before backup or write because the supervisor used the
nonexistent `/api/system/theme` route and its typed error was not registered.
Attempt-002 proved the corrected settings and theme backup, then stopped before
USB because `admit_self_test_intent` incorrectly passed 40-character source
and reference commits to a 64-character digest validator. A subsequent
explicit dry-run reproduced `invalid_source_commit`; a bounded receive-only
monitor proved ordinary advancing runtime with no self-test admission, stage,
checkpoint, terminal, or receipt markers. No package or NVS write, self-test
effect, safe-stop, BOOT checkpoint, restart, restoration write, or public
projection occurred in either attempt.

Attempt-003 is eligible only after the commit validator accepts exactly 40
lowercase hexadecimal characters, rejects invalid lengths and alphabets, and
the supervisor runs the exact flash-monitor invocation once in `--dry-run`
mode before the real child. That admission must validate plan, intent, package,
image, Wi-Fi input, and command construction without USB or evidence output.
The real child is forbidden unless the dry-run exits successfully.

## Scope

The lifecycle, safety envelope, settings preservation, evidence boundary,
built-in BOOT-button checkpoint, pass criteria, cleanup, promotion rules, and
non-claims remain exactly those in the original plan. No public mutation API,
external UART, pin manipulation, probe, jumper, actual sensor/thermal/electrical
fault, communication fault, pool transport, share submission, OTA, or erase is
authorized.

Retry-specific changes are limited to:

- exact 40-character validation for source and reference commits;
- exact 64-character validation for SHA-256 fields;
- a pre-effect `--dry-run` of each failure/pass flash-monitor specification;
- ordinal-3 intent, state, workflow, and evidence bindings;
- protected `wrapper-003` and `attempt-003` paths.

## Exact hardware contract

After implementation, tests, ordered Cargo gates, Bright Builds, all Bazel
tests, package, parity/progress, redaction, reference, sensitive-value, and diff
review pass and the source is committed, pushed, clean, and package-bound, run:

1. `just package`
2. `test ! -e scratch/self001-full-lifecycle/wrapper-003 && (umask 077; mkdir -m 700 -p scratch/self001-full-lifecycle/wrapper-003 && just detect-ultra205 > scratch/self001-full-lifecycle/wrapper-003/detector.stdout 2> scratch/self001-full-lifecycle/wrapper-003/detector.stderr)`
3. `test ! -e scratch/self001-full-lifecycle/attempt-003 && test ! -e docs/parity/evidence/self001-full-lifecycle/self-test-projection.json && (umask 077; just self-test-campaign start --private-root scratch/self001-full-lifecycle/attempt-003 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --pool-credentials pool-credentials.json --detector-output scratch/self001-full-lifecycle/wrapper-003/detector.stdout --plan docs/parity/work-plans/20260821T200723Z-SELF-001-RETRY-2/PLAN.md --projection docs/parity/evidence/self001-full-lifecycle/self-test-projection.json > scratch/self001-full-lifecycle/wrapper-003/start.stdout 2> scratch/self001-full-lifecycle/wrapper-003/start.stderr)`
4. Only after start publishes `cancel_ready` with `safe_state=true`, the user
   holds the built-in BOOT button for two seconds, and the physical action is
   acknowledged: `just self-test-campaign resume --private-root scratch/self001-full-lifecycle/attempt-003 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --detector-output scratch/self001-full-lifecycle/wrapper-003/detector.stdout --plan docs/parity/work-plans/20260821T200723Z-SELF-001-RETRY-2/PLAN.md --projection docs/parity/evidence/self001-full-lifecycle/self-test-projection.json`

Start and resume must each execute and pass their exact no-USB dry-run before
the corresponding real flash-monitor child. The start preflight must capture
`/api/system/info` and `/api/theme`, prove the ignored local inputs reconstruct
all restorable settings, and persist the mode-0600 backup before mutation. All
active, thermal, electrical, workload, watchdog, safe-stop, restart,
restoration, cleanup, evidence, and redaction bounds remain unchanged.

## Stop, recovery, and promotion

Persist campaign state immediately after backup and intent creation, before
the first real child, so recovery remains possible after any later failure.
Preserve the earliest closed category. After any possible effect, attempt every
independent safe-stop and exact settings/package restoration step, clean USB
and process ownership, withhold `RESULT.md`, and do not rerun attempt-003.
Before effects, retain the protected typed failure and stop.

Promote only after both phases share the ordinal-3 lease and exact source,
reference, plan, package, detector, restart, restoration, cleanup, validation,
and redaction quorum. On success, create `RESULT.md`, transition only
`SELF-001` to `verified` with `unit,workflow,hardware-regression`, synchronize
progress, archive the completed task, final-verify, commit, and push.
