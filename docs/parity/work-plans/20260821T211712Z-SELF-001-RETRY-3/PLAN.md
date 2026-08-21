# SELF-001 Attempt-004 Recovery Plan

## Identity

- Parity row: `SELF-001`
- Active task: `task-parity-self001-full-lifecycle`
- Original plan: `docs/parity/work-plans/20260821T180800Z-SELF-001/PLAN.md`
- Attempt-003 plan: `docs/parity/work-plans/20260821T200723Z-SELF-001-RETRY-2/PLAN.md`
- Authorized attempt: `attempt-004`

## Prior evidence and remaining defect

Attempts 001 and 002 stopped before USB on corrected host preflight defects.
Attempt-003 passed exact dry-run admission, installed the exact package and
private intent, completed controlled diagnostic load and safe-stop, published
`cancel_ready` with `safe_state=true`, and accepted the user's built-in
two-second BOOT hold. A later bounded monitor proved ordinary advancing runtime,
so the physical cancellation and restart occurred. Resume nevertheless missed
the lease-bound receipt because firmware logged it once during early startup,
before the post-action monitor attached. The retained HTTP buffer had already
overwritten that marker. Phase B never started, no projection was published,
and the exact settings/theme backup was restored with `mineonboot=false`.

Attempt-004 is eligible only after the existing private serial runtime observer
replays the persisted lease-bound terminal receipt every 10 seconds. The marker
must remain serial-only and absent from HTTP/WebSocket models and public APIs.
Resume must still require the exact lease/outcome and must automatically run
ordinary exact-package/settings recovery if cancellation evidence is missing.

## Scope

All original lifecycle, safety, settings, privacy, workload, measurement,
safe-stop, restart, cleanup, promotion, and non-claim rules remain unchanged.
No public self-test mutation or receipt API, external UART, pin manipulation,
probe, jumper, actual sensor/thermal/electrical fault, communication fault,
pool transport, share submission, OTA, or erase is authorized.

Retry-specific changes are limited to:

- register a validated persisted receipt with the existing serial observer;
- replay only `self_test_receipt outcome=<closed> lease=<16-lower-hex>` every
  `BOOT_EVIDENCE_INTERVAL_MS` (10 seconds);
- keep replay out of retained HTTP/WebSocket projections;
- recover exact package/settings if resume cannot prove the cancellation;
- ordinal-4 intent, state, workflow, and evidence bindings;
- protected `wrapper-004` and `attempt-004` paths.

## Exact hardware contract

After focused replay/source-ownership/recovery tests and every full repository
gate pass, commit/push/package the exact clean source, then run:

1. `just package`
2. `test ! -e scratch/self001-full-lifecycle/wrapper-004 && (umask 077; mkdir -m 700 -p scratch/self001-full-lifecycle/wrapper-004 && just detect-ultra205 > scratch/self001-full-lifecycle/wrapper-004/detector.stdout 2> scratch/self001-full-lifecycle/wrapper-004/detector.stderr)`
3. `test ! -e scratch/self001-full-lifecycle/attempt-004 && test ! -e docs/parity/evidence/self001-full-lifecycle/self-test-projection.json && (umask 077; just self-test-campaign start --private-root scratch/self001-full-lifecycle/attempt-004 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --pool-credentials pool-credentials.json --detector-output scratch/self001-full-lifecycle/wrapper-004/detector.stdout --plan docs/parity/work-plans/20260821T211712Z-SELF-001-RETRY-3/PLAN.md --projection docs/parity/evidence/self001-full-lifecycle/self-test-projection.json > scratch/self001-full-lifecycle/wrapper-004/start.stdout 2> scratch/self001-full-lifecycle/wrapper-004/start.stderr)`
4. Only after start publishes `cancel_ready` with `safe_state=true`, the user
   holds the built-in BOOT button for two seconds and acknowledges completion.
5. `just self-test-campaign resume --private-root scratch/self001-full-lifecycle/attempt-004 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --detector-output scratch/self001-full-lifecycle/wrapper-004/detector.stdout --plan docs/parity/work-plans/20260821T211712Z-SELF-001-RETRY-3/PLAN.md --projection docs/parity/evidence/self001-full-lifecycle/self-test-projection.json`

Start and resume must pass their exact no-USB dry-run before corresponding real
children. The start settings backup, durable pre-effect state, controlled
failure, pass envelope, safe-stop, and restoration requirements remain those
of the original plan and attempt-003 hardening plan.

## Stop, recovery, and promotion

Preserve the earliest category. After any possible effect, attempt every
independent safe-stop and ordinary exact-package/settings restoration step,
clean USB/process ownership, withhold `RESULT.md`, and do not rerun attempt-004.
Before effects, retain the typed failure and stop.

Promote only after both phases share the ordinal-4 lease and exact source,
reference, plan, package, detector, cancellation receipt/restart, pass
receipt/restart, restoration, cleanup, independent validation, and redaction
quorum. On success, create `RESULT.md`, transition only `SELF-001` to
`verified` with `unit,workflow,hardware-regression`, synchronize progress,
archive the completed task, final-verify, commit, and push.
