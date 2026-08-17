# Parity work plan

- Run ID: `20260817T001716Z-STAT-001`
- Parity row: `STAT-001`
- Initial status: `implemented`
- Source commit: `e979847128fe6617e5ff2593eedb7a3c45c46e3a`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-stat001-hashrate-monitor`

## Selection

The worktree and pinned reference are clean, `main` equals `origin/main`, and
the deterministic selector reports no open plan. It orders `SELF-001`,
`BAP-002`, then `STAT-001`. `SELF-001` remains dependency-blocked because no
production-safe firmware route exists for its hardware self-test submodes or
required hardware-regression evidence. `BAP-002` remains dependency- and
authority-blocked by unfinished `BAP-001` UART ownership/subscription work and
the absence of an authorized live accessory path.

STAT-001 is the first actionable row. Attempts 008 and 009 both reached
trusted exact-package runtime identity and fourteen of twenty required
continuity windows before the campaign consumer sealed
`watchdog_unresponsive` / `watchdog_feed_stale`. Pushed implementation commit
`9e9d6545dbe4881f1cb81ca61da2c152dd791c9b` proves that the device producer
correctly classifies `feed_fresh` through its compiled 5,000-ms timeout while
the host campaign independently and incorrectly rejected every feed age above
2,000 ms. It removes only that duplicate host numeric policy and preserves
reason, participation, presence, sequence, per-window advancement,
earliest-failure, and value-free evidence checks. Current pushed commit
`e979847128fe6617e5ff2593eedb7a3c45c46e3a` adds its truthful closure.
Attempt-010 is therefore a progress-backed test of a targeted correction, not
an unchanged retry.

The active lesson ledgers total 30,773 bytes with a summed conservative
estimate of 10,259 tokens, above both deterministic loading limits. Every
heading was inventoried. All global lessons and complete repository blocks for
authorization, privacy, evidence integrity, retry policy, service ownership,
cross-process boundaries, failure precedence, transport capability,
evaluator identity, operating-state telemetry, and legacy wire units were
loaded within the whole-block budget. Omitted lower-priority blocks are the
GSD frontmatter lesson and historical USB power/session, prearmed native
capture, boot-replay lifetime, silent-transport heartbeat, manual-removal
ownership, physical-identity, and cold-boot-observer lessons. Their active
repo-local equivalents remain controlling. The latest audit baseline has six
later lessons, is under 90 days old, and this plan appends no lesson, so no
distinct audit trigger is due.

## Scope and non-scope

Advance only `STAT-001`. Rebind the existing private-first
`bitaxe-hashrate-monitor-evidence-v1` workflow, independent Rust validator,
generated TypeScript contract, immutable task/plan admission, Bazel runfiles,
protected roots, and real-child fixtures from consumed attempt-009 to fresh
attempt-010. Preserve campaign-result v12, network-continuity v6, the public
projection path, and every closed watchdog, seal, category, precedence,
redaction, and value-free behavior. Add no firmware, mining, hashrate, sensor,
watchdog, or hardware-control behavior.

After all software and package gates pass at a clean pushed commit, one exact
board-205 package may be factory-flashed and normally reset/re-enumerated. The
workflow may seed only ignored local Wi-Fi and pool inputs, derive its target
only from protected current-session evidence, and run the repo-owned
conservative `live-share` profile for exactly 600 accumulated active seconds.
It may join protected serial, HTTP, and reconstructed WebSocket observations,
pause and safe-stop, release USB/process ownership, seal the result, and use at
most one supervisor-owned exact-package recovery flash after a post-flash
failure. No human action is required.

The profile domains remain distinct: ASIC frequency is 400 MHz, the ASIC core
setpoint is 1,100 millivolts (1.1 V), fan duty is 100 percent, and INA260 input
bus voltage is independently measured in volts. Input safety admits only fresh
4.5 through 5.5 V bus truth, at most 15 W, ASIC temperature below 75 C, and
fresh nonzero fan RPM after the 100-percent fan command. Never compare bus
volts with the millivolt core setpoint.

Credentials, owner/worker fields, endpoints, origins, ports, hostnames,
USB/network/process identities, exact hashrates and sensors, bodies, logs,
commands, PIDs, and traces remain private. They may exist only in mode-`0600`
files beneath ignored mode-`0700` roots. Only the plan-named closed projection
may become public after independent validation. No upstream-default or
overclock profile, arbitrary control target, automatic fan mode, unbounded
mining, OTA, erase, raw write, fault injection, physical power action, direct
UART, or pin/pad/header/GPIO/probe/jumper/solder/signal manipulation is
authorized. Electrical calibration, profitability, dynamic retuning, extended
soak, broader share behavior, other ASICs/boards, update/recovery behavior, and
release readiness remain non-claims.

## Implementation

- [ ] Rebind roots, immutable plan/task admission, generated contract, Rust
      validator, Bazel inputs, and fixtures from attempt-009 to attempt-010
      without changing campaign behavior or the public projection path.
- [ ] Preserve v12/v6 and regression-test prior-ordinal rejection, all twenty
      closed watchdog values, success `none`, seal/category gating,
      earliest-failure precedence, redaction, and value-free output across the
      real child-process boundary.
- [ ] Prove current source/reference semantics, exact `live-share` plus
      `conservative` command, protected layout, profile units, and independently
      volt-typed input safety.
- [ ] Run every focused and mandatory software, firmware, privacy, reference,
      package, source-admission, generated-contract, immutable-plan, and diff
      gate; commit and push the exact implementation before detector or
      credential access.
- [ ] Execute only the frozen detector and conditional attempt-010 commands;
      publish and promote only if the independent complete quorum passes.

## Authorized live commands and recovery

After implementation is clean, fully gated, committed, pushed, and its exact
package rebuilt and validated, run only these commands in order:

1. `test ! -e scratch/stat001-hashrate-monitor/wrapper-010 && (umask 077; mkdir -m 700 -p scratch/stat001-hashrate-monitor/wrapper-010 && just detect-ultra205 > scratch/stat001-hashrate-monitor/wrapper-010/detector.stdout 2> scratch/stat001-hashrate-monitor/wrapper-010/detector.stderr)`
2. Only after command 1 exits zero, admits exactly one Ultra 205 through
   `espflash board-info --chip esp32s3 --non-interactive`, cleanup and holder
   checks pass, both ignored credential files are nonempty without being read,
   and the supervisor child, projection, and candidate are absent:
   `test ! -e scratch/stat001-hashrate-monitor/attempt-010 && test ! -e docs/parity/evidence/stat001-hashrate-monitor/hashrate-monitor-projection.json && test -s wifi-credentials.json && test -s pool-credentials.json && (umask 077; just capture-hashrate-monitor-evidence --private-root scratch/stat001-hashrate-monitor/attempt-010 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --pool-credentials pool-credentials.json --detector-output scratch/stat001-hashrate-monitor/wrapper-010/detector.stdout --projection docs/parity/evidence/stat001-hashrate-monitor/hashrate-monitor-projection.json --duration-seconds 600 --capture-timeout-seconds 1500 > scratch/stat001-hashrate-monitor/wrapper-010/capture.stdout 2> scratch/stat001-hashrate-monitor/wrapper-010/capture.stderr)`

The wrapper root must be mode `0700`; detector and capture streams must be
distinct mode-`0600` siblings; and the supervisor-owned attempt child,
projection, and candidate must be absent before launch. Starting command 2
consumes attempt-010. Preserve the earliest typed failure and watchdog label
through bounded safe stop, recovery, sealing, and cleanup. Never retry
unchanged or start attempt-011 under this plan. Stop on detector ambiguity or
failure, non-205 identity, missing credentials, source/reference/package
drift, unsafe state, incomplete quorum, safe-stop/recovery/cleanup/seal/mode/
privacy failure, nonzero command exit, or successful verified projection.
Select one terminal outcome: `complete`, `stop_repeated_boundary`,
`stop_hardware_blocker`, `stop_authority_boundary`, or
`stop_impossible_contract`.

## Verification and promotion

Before hardware, run focused hashrate automation, independent Rust contract,
campaign network/watchdog/profile, generated-contract, real-child,
current-task/plan, source/reference admission, seal, protected-mode,
redaction, and failure-precedence tests. Run `just verify-redaction`,
`just verify-reference`, `just package`, and the mandatory sequence in order:

1. `cargo fmt --all`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo build --all-targets --all-features`
4. `cargo test --all-features`
5. `bun scripts/bright-builds-check.ts all`
6. `just test`
7. `just parity`
8. `just parity-progress`

Commit and push the exact implementation, then rebuild and validate the clean
package before detector access. Promotion requires the independent validator
to prove board 205, attempt 10, exact source/reference/package/plan, one
detector-admitted device, trusted runtime identity, one-second cadence and
pinned register semantics, one ASIC/four domains, active mining and work
renewal in all twenty half-open 30-second windows, at least two changing
coherent positive observations from each transport, positive current and all
four rolling windows after warmup, finite bounded error, watchdog failure
`none`, terminal zero current rate in both transports, safe stop, recovery/
cleanup, protected modes, exact seals, independent validation, and redaction.

On success create `RESULT.md`, commit evidence as `SOURCE_COMMIT`, transition
only STAT-001 to `verified` with
`unit,workflow,api-compare,hardware-smoke,hardware-regression`, synchronize
progress immediately, archive the completed task, run final gates, review,
commit, and push. Any missing fact withholds the projection and verified claim,
leaves STAT-001 `implemented`, records a truthful `CLOSURE.md`, and stops
without an unchanged retry.
