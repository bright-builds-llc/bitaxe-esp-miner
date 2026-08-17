# Parity work plan

- Run ID: `20260817T054416Z-STAT-001`
- Parity row: `STAT-001`
- Initial status: `implemented`
- Source commit: `ba5b27c2d5072071aa3f1ec0985f2a9ca72c83f6`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-stat001-hashrate-monitor`

## Selection

The worktree/reference are clean, `main` equals `origin/main`, and the
deterministic selector reports no open plan. It orders `SELF-001`, `BAP-002`,
then `STAT-001`. SELF-001 remains blocked by the absence of a production-safe
self-test route/hardware regression. BAP-002 remains blocked by unfinished
BAP-001 UART/subscription ownership and no authorized accessory path.

STAT-001 is first actionable. Attempt-012 stopped after 13/20 windows with
sealed `watchdog_feed_stale` at `waiting_inbox`; identity, safety, terminal
state, cleanup, modes, and seals passed. Exact pushed implementation
`9604d145f92f6d1b93fa446ce24154e2ccb04e5f` now atomically arms a wrap-aware
receive deadline before the waiting phase and derives a closed wait state after
copying observations. It projects through v14/v8 seals, pins priority 5, binds
18 evaluator sources, passes the real Xtensa build and every mandatory gate.
Attempt-013 is therefore a progress-backed diagnostic attempt, not an
unchanged retry.

Active lessons exceed loading limits; all headings were inventoried. Global
lessons plus complete authorization, direct-UART, protected-evidence,
earliest-failure, runtime-capacity, private-classification, retry, transport,
evaluator-identity, telemetry-state, and legacy-unit blocks were loaded.
Lower-priority historical USB/GSD blocks were omitted within the disclosed
whole-block budget; repo-local equivalents remain controlling. Six lessons
postdate the current audit baseline, it is under 90 days old, and no append is
proposed, so no audit trigger is due.

## Scope and non-scope

Advance only STAT-001. Rebind the private-first hashrate workflow, independent
Rust validator, generated TypeScript contract, task/plan admission, Bazel
runfiles, protected roots, and real-child fixtures from consumed attempt-012
to fresh attempt-013. Preserve result v14, network v8, all watchdog labels,
owner phase, closed wait state, priority 5, 18-source identity, public
projection path, seals, precedence, redaction, and value-free reporting. Add
no firmware, scheduling, timeout, mining, sensor, hashrate, or control change.

After all software/package gates pass at a clean pushed commit, one exact
board-205 package may be factory-flashed/reset and privately seeded with
ignored Wi-Fi/pool inputs. Run only the conservative `live-share` 400 MHz /
1,100 mV / 100% fan profile for exactly 600 accumulated active seconds, join
protected current-session serial/HTTP/WebSocket evidence, pause, safe-stop,
release ownership, seal results, and permit at most one supervisor-owned exact-
package recovery flash after a post-flash failure. No human action is required.

ASIC core 1,100 mV and independently measured INA260 bus volts remain distinct.
Safety requires fresh 4.5-5.5 V input, <=15 W, ASIC temperature <75 C, and
fresh nonzero fan RPM. Credentials, endpoints, identities, exact rates/sensors,
bodies, logs, commands, PIDs, and traces remain private mode-0600 beneath fresh
ignored mode-0700 roots. Only independently validated aggregate projection may
be public. No overclock, arbitrary controls, unbounded mining, OTA, erase, raw
write, fault injection, power action, direct UART, or electrical manipulation
is authorized.

## Implementation

- [ ] Rebind ordinal/roots/plan/task/generated contracts/validator/Bazel/
      fixtures from attempt-012 to attempt-013; preserve v14/v8 and behavior.
- [ ] Preserve every watchdog/phase/wait label, 18-source identity, prior-
      ordinal rejection, seal/category/precedence/redaction/value-free tests.
- [ ] Run focused and mandatory software, firmware, privacy, reference,
      package, exact-source, generated-contract, immutable-plan, and diff gates;
      commit/push before detector or credential access.
- [ ] Execute only the frozen detector and conditional attempt-013 commands;
      promote only on the complete independently validated quorum.

## Authorized live commands and recovery

After the exact implementation is clean, gated, committed, pushed, rebuilt,
and package-validated, run only:

1. `test ! -e scratch/stat001-hashrate-monitor/wrapper-013 && (umask 077; mkdir -m 700 -p scratch/stat001-hashrate-monitor/wrapper-013 && just detect-ultra205 > scratch/stat001-hashrate-monitor/wrapper-013/detector.stdout 2> scratch/stat001-hashrate-monitor/wrapper-013/detector.stderr)`
2. Only after command 1 exits zero, admits exactly one Ultra 205 with successful
   board-info/cleanup/holder checks, both ignored credential files are nonempty
   without being read, and child/projection/candidate are absent:
   `test ! -e scratch/stat001-hashrate-monitor/attempt-013 && test ! -e docs/parity/evidence/stat001-hashrate-monitor/hashrate-monitor-projection.json && test -s wifi-credentials.json && test -s pool-credentials.json && (umask 077; just capture-hashrate-monitor-evidence --private-root scratch/stat001-hashrate-monitor/attempt-013 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --pool-credentials pool-credentials.json --detector-output scratch/stat001-hashrate-monitor/wrapper-013/detector.stdout --projection docs/parity/evidence/stat001-hashrate-monitor/hashrate-monitor-projection.json --duration-seconds 600 --capture-timeout-seconds 1500 > scratch/stat001-hashrate-monitor/wrapper-013/capture.stdout 2> scratch/stat001-hashrate-monitor/wrapper-013/capture.stderr)`

Wrapper/attempt modes must be 0700/0600 and the supervisor child absent before
launch. Starting command 2 consumes attempt-013. Preserve earliest failure,
owner phase, and wait state through safe stop/recovery/sealing/cleanup. Never
retry unchanged or start attempt-014. Stop on detector/source/package/safety/
credential/quorum/cleanup/recovery/seal/privacy failure or verified projection.
Terminal outcome must be one of complete, stop_repeated_boundary,
stop_hardware_blocker, stop_authority_boundary, stop_impossible_contract.

## Verification and promotion

Run focused hashrate automation, Rust validator, campaign watchdog/phase/wait/
schema, generated-contract, real-child, source/reference, seal, mode,
redaction, and precedence tests. Run `just verify-redaction`, `just
verify-reference`, `just package`, then in order: `cargo fmt --all`; `cargo
clippy --all-targets --all-features -- -D warnings`; `cargo build --all-targets
--all-features`; `cargo test --all-features`; `bun
scripts/bright-builds-check.ts all`; `just test`; `just parity`; `just
parity-progress`.

Promotion requires board205/attempt13/exact identities, one ASIC/four domains,
one-second cadence, all twenty windows/work renewal, changing coherent positive
HTTP/WebSocket current plus warm rolling rates, bounded error, watchdog
failure `none`, terminal zero, safe stop, cleanup, modes, seals, validator, and
redaction. On success create RESULT, commit evidence as SOURCE_COMMIT,
transition only STAT-001 to verified with unit,workflow,api-compare,hardware-
smoke,hardware-regression, sync progress, archive the task, final-gate and push.
Any missing fact withholds projection, creates CLOSURE, leaves implemented, and
stops without retry.

## Non-claims

This plan does not pre-claim wait-state outcome, verification, profitability,
arbitrary scheduling/pools/profiles, other boards/ASICs, update/recovery, or
release readiness.
