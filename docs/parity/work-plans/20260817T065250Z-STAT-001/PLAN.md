# Parity work plan

- Run ID: `20260817T065250Z-STAT-001`
- Parity row: `STAT-001`
- Initial status: `implemented`
- Source commit: `aca4bdbea3c6c55a7045cf69b880be8ac8ebfc57`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-stat001-hashrate-monitor`

## Selection

The worktree and pinned reference are clean, `main` equals `origin/main`, and
the deterministic selector reports no open plan. It orders `SELF-001`,
`BAP-002`, then `STAT-001`. SELF-001 remains blocked because no production-safe
self-test route or required hardware regression exists. BAP-002 remains
blocked by unfinished BAP-001 firmware UART/request/subscription ownership and
the absence of an authorized live accessory path.

STAT-001 is first actionable. Attempt-013 stopped after 12 of 20 windows with
the sealed authoritative signature `watchdog_feed_stale` / `waiting_inbox` /
`within_deadline`. Source commit
`f5a8fd144ada04503cd5fa49c7dcc175a112aaf6` fixes that exact mixed-snapshot
boundary: feed history, owner phase, and wait deadline now share a sequence-
bracketed firmware snapshot with eight bounded fail-closed retries. The exact
old-feed/new-wait regression fails the first read and returns the new coherent
instant; stable, retry-exhaustion, poison, ownership, evaluator, real Xtensa,
package, privacy, reference, and every mandatory gate pass. Attempt-014 is
therefore progress-backed, not an unchanged retry.

Active lessons exceed their bounded loading budget. All headings were
inventoried; the complete global file and the authorization, direct-UART,
protected-evidence, earliest-failure, runtime-capacity, readiness,
classification, retry, transport, evaluator-identity, telemetry-state, and
unit-boundary blocks were loaded. Lower-priority historical GSD/USB blocks
were disclosed as omitted. The latest audit baseline is under 90 days old,
fewer than ten lessons are new, and no lesson append is proposed, so no audit
trigger is due. Repo-local hardware, privacy, timeout, and progress policy plus
Bright Builds verification/testing rules govern this plan.

## Scope and non-scope

Advance only STAT-001. Rebind the private-first hashrate workflow, independent
Rust validator, generated TypeScript contract, task/plan admission, Bazel
runfiles, protected roots, and real-child fixtures from consumed attempt-013
to fresh attempt-014. Preserve campaign-result v14, network-continuity v8,
every watchdog/phase/wait label, priority 5, compiled five-second timeout,
coherent snapshot behavior, 18-source evaluator identity, public projection
path, seals, earliest-failure precedence, redaction, and value-free reporting.
Add no firmware, scheduling, timeout, mining, sensor, hashrate, or control
behavior change.

After the rebind and every software/package gate pass at a clean pushed commit,
one exact board-205 package may be factory-flashed/reset and privately seeded
from the ignored Wi-Fi and sole pool inputs. Run only the repo-owned
conservative `live-share` profile at 400 MHz, 1,100 mV ASIC core setpoint, and
100% fan for exactly 600 accumulated active seconds. Join protected current-
session serial, HTTP, and reconstructed WebSocket evidence; then pause,
safe-stop, release USB/process ownership, seal results, and permit at most one
supervisor-owned exact-package recovery flash after a post-flash failure. No
human checkpoint or action is required.

ASIC core 1,100 mV and independently measured INA260 input-bus volts are
distinct domains and must never be compared. Admission requires fresh
4.5-5.5 V input, power at most 15 W, ASIC temperature below 75 C, and fresh
nonzero fan RPM after the 100% command. Credentials, endpoints, identities,
exact hashrates/sensors, bodies, logs, commands, PIDs, and traces remain
ProtectedOperational in mode-0600 files under fresh ignored mode-0700 roots;
NeverPersistRaw values reach neither disk nor terminal. Only an independently
validated aggregate projection may become public.

No upstream-default/overclock profile, arbitrary control, automatic fan mode,
unbounded mining, OTA, erase, raw write, fault injection, physical power
action, external UART, BAP transport, or pin/pad/header/GPIO/probe/jumper/
solder/signal manipulation is authorized.

## Implementation

- [ ] Rebind ordinal, roots, immutable plan/task admission, Rust validator,
      generated contract, Bazel inputs, and fixtures from attempt-013 to
      attempt-014 without changing production behavior or schemas.
- [ ] Preserve the complete watchdog/phase/wait vocabulary, coherent snapshot,
      18-source identity, attempt-013 rejection, seal/category/precedence,
      private-mode, unit, and redaction regressions.
- [ ] Run focused and mandatory software, firmware, privacy, reference,
      package, exact-source, generated-contract, immutable-plan, and diff
      gates; commit and push before detector, credential handoff, or hardware.
- [ ] Execute only the frozen detector and conditional attempt-014 commands;
      promote only on the complete independently validated quorum.

## Authorized live commands and recovery

After the exact implementation is clean, fully gated, committed, pushed,
rebuilt, and package-validated, run only:

1. `test ! -e scratch/stat001-hashrate-monitor/wrapper-014 && (umask 077; mkdir -m 700 -p scratch/stat001-hashrate-monitor/wrapper-014 && just detect-ultra205 > scratch/stat001-hashrate-monitor/wrapper-014/detector.stdout 2> scratch/stat001-hashrate-monitor/wrapper-014/detector.stderr)`
2. Only after command 1 exits zero, repo-owned output admits exactly one Ultra
   205 with successful board-info/cleanup/holder checks, both ignored inputs
   are nonempty without being read, and child/projection paths remain absent:
   `test ! -e scratch/stat001-hashrate-monitor/attempt-014 && test ! -e docs/parity/evidence/stat001-hashrate-monitor/hashrate-monitor-projection.json && test -s wifi-credentials.json && test -s pool-credentials.json && (umask 077; just capture-hashrate-monitor-evidence --private-root scratch/stat001-hashrate-monitor/attempt-014 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --pool-credentials pool-credentials.json --detector-output scratch/stat001-hashrate-monitor/wrapper-014/detector.stdout --projection docs/parity/evidence/stat001-hashrate-monitor/hashrate-monitor-projection.json --duration-seconds 600 --capture-timeout-seconds 1500 > scratch/stat001-hashrate-monitor/wrapper-014/capture.stdout 2> scratch/stat001-hashrate-monitor/wrapper-014/capture.stderr)`

The caller owns only wrapper-014 and its distinct mode-0600 streams; the
supervisor exclusively creates the absent attempt-014 child. Starting command
2 consumes attempt-014. Preserve the earliest category plus watchdog phase and
wait state through bounded safe stop, optional exact-package recovery, sealing,
and cleanup. Never retry attempt-014, reuse attempt-013, or start attempt-015.
Stop on nonzero preflight/detector/capture exit, ambiguous/non-205 identity,
source/reference/package drift, missing input, unsafe state, incomplete quorum,
failed safe stop/recovery/cleanup/seal/mode/privacy, or a complete projection.
Record exactly one closed outcome: `complete`, `stop_repeated_boundary`,
`stop_hardware_blocker`, `stop_authority_boundary`, or
`stop_impossible_contract`. Recurrence of the exact attempt-013 signature after
this targeted fix selects `stop_repeated_boundary`.

## Verification and promotion

Run focused hashrate automation, Rust validator, campaign watchdog/phase/wait/
schema, coherent-store, generated-contract, real-child, source/reference,
seal, mode, unit, redaction, and precedence tests. Run `just verify-redaction`,
`just verify-reference`, `just package`, then in order: `cargo fmt --all`;
`cargo clippy --all-targets --all-features -- -D warnings`; `cargo build
--all-targets --all-features`; `cargo test --all-features`; `bun
scripts/bright-builds-check.ts all`; `just test`; `just parity`; `just
parity-progress`.

Promotion requires board205/attempt14/exact source/reference/package/plan and
detector identity; one ASIC and four domains; one-second cadence; all twenty
active windows with work renewal; changing coherent positive HTTP/WebSocket
current hashrate and positive warm rolling rates; bounded error; watchdog
failure `none`; terminal zero current/ASIC/domain rates; safe stop; cleanup;
protected modes; seals; independent validation; and redaction. On success,
create RESULT, commit evidence as SOURCE_COMMIT, transition only STAT-001 to
`verified` with `unit,workflow,api-compare,hardware-smoke,hardware-regression`,
sync progress, archive the completed task, run final gates, and push. Any
missing fact withholds the projection, creates CLOSURE, leaves STAT-001
`implemented`, and stops without retry.

## Non-claims

This plan does not pre-claim watchdog success, hashrate accuracy, profitability,
arbitrary scheduling/pools/profiles, other boards/ASICs, update/recovery, or
release readiness.
