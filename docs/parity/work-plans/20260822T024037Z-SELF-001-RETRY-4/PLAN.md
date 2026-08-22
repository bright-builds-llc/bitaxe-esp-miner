# SELF-001 Attempt-005 Domain-Counter Plan

## Identity

- Parity row: `SELF-001`
- Active task: `task-parity-self001-full-lifecycle`
- Original plan: `docs/parity/work-plans/20260821T180800Z-SELF-001/PLAN.md`
- Attempt-004 plan: `docs/parity/work-plans/20260821T211712Z-SELF-001-RETRY-3/PLAN.md`
- Authorized attempt: `attempt-005`

## Hardware finding and root cause

Attempt-004 proved the complete controlled-failure, safe-stop, physical
BOOT-cancellation, receipt replay, restart, and pass-phase admission paths. The
pass run reached warming, 30-second measurement, evaluation, and complete
safe-stop, then failed closed as `domain_failed`; it did not publish a pass,
restart, or projection. Automatic recovery restored the exact package,
settings, theme, and `mineonboot=false`.

The failure is an implementation defect, not evidence of a failed BM1366 hash
domain. Rust assigned accepted nonces to domains with `small_core_id % 4` and
derived domain rates from those buckets. Upstream does not infer domains from
nonce metadata. It polls BM1366 registers `0x88–0x8B` once per second, computes
wrapping counter deltas with the `2^32` hash unit, averages fresh per-domain
rates, rejects non-finite or greater-than-three-times-expected samples, and
retains domains with all or at least 25% rejected samples as unreliable while
keeping external nonce hashrate authoritative.

Attempt-005 is eligible only after Rust uses the existing typed register-read
burst, parser, and pure `HashrateMonitor` for domain measurements; the
synthetic small-core mapping is absent; focused tests prove baseline, wrapping,
one-second cadence, implausible rejection, averages, and unreliable handling;
and aggregate private metrics are logged before evaluation for diagnosis.

## Scope

All original lifecycle, safety, settings, privacy, workload, measurement,
safe-stop, receipt/restart, cleanup, promotion, and non-claim rules remain
unchanged. No threshold is weakened and no failed domain is relabeled. No
public mutation/receipt API, external UART, pin manipulation, actual fault
injection, pool traffic, share submission, OTA, or erase is authorized.

Retry-specific changes are limited to:

- send the existing upstream register burst during diagnostic work;
- admit parsed `Domain0Count` through `Domain3Count` observations into the
  existing pure hashrate counter model with monotonic timestamps;
- average only fresh updated domain samples and record upstream-compatible
  implausible rejections;
- keep total validation based on accepted difficulty-16 nonces;
- remove `small_core_id % 4` domain attribution;
- ordinal-5 intent, state, workflow, and evidence bindings;
- protected `wrapper-005` and `attempt-005` paths.

## Exact hardware contract

After focused tests and every full repository gate pass, commit/push/package
the exact clean source, then run:

1. `just package`
2. `test ! -e scratch/self001-full-lifecycle/wrapper-005 && (umask 077; mkdir -m 700 -p scratch/self001-full-lifecycle/wrapper-005 && just detect-ultra205 > scratch/self001-full-lifecycle/wrapper-005/detector.stdout 2> scratch/self001-full-lifecycle/wrapper-005/detector.stderr)`
3. `test ! -e scratch/self001-full-lifecycle/attempt-005 && test ! -e docs/parity/evidence/self001-full-lifecycle/self-test-projection.json && (umask 077; just self-test-campaign start --private-root scratch/self001-full-lifecycle/attempt-005 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --pool-credentials pool-credentials.json --detector-output scratch/self001-full-lifecycle/wrapper-005/detector.stdout --plan docs/parity/work-plans/20260822T024037Z-SELF-001-RETRY-4/PLAN.md --projection docs/parity/evidence/self001-full-lifecycle/self-test-projection.json > scratch/self001-full-lifecycle/wrapper-005/start.stdout 2> scratch/self001-full-lifecycle/wrapper-005/start.stderr)`
4. Only after start publishes `cancel_ready` with `safe_state=true`, the user
   holds built-in BOOT for two seconds and acknowledges completion.
5. `just self-test-campaign resume --private-root scratch/self001-full-lifecycle/attempt-005 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --detector-output scratch/self001-full-lifecycle/wrapper-005/detector.stdout --plan docs/parity/work-plans/20260822T024037Z-SELF-001-RETRY-4/PLAN.md --projection docs/parity/evidence/self001-full-lifecycle/self-test-projection.json`

Start/resume dry-run admission, serial receipt replay, backup, recovery, and all
original active/pass/safe-stop limits remain mandatory.

## Stop, recovery, and promotion

Preserve the earliest category. After any possible effect, attempt every
independent safe-stop and ordinary exact-package/settings restoration step,
clean USB/process ownership, withhold `RESULT.md`, and do not rerun attempt-005.

Promote only after both phases share the ordinal-5 lease and exact identity and
every total/domain/electrical/fan/thermal/watchdog/stage/restart/restoration/
cleanup/validation/redaction fact passes. On success, create `RESULT.md`,
transition only `SELF-001` to `verified` with
`unit,workflow,hardware-regression`, synchronize progress, archive the task,
final-verify, commit, and push.
