# Parity work plan

- Run ID: `20260816T183130Z-STAT-001`
- Parity row: `STAT-001`
- Initial status: `implemented`
- Source commit: `f62935a49609abe685efd03c94b15bf6cd126f7b`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-stat001-hashrate-monitor`

## Selection

The worktree and pinned reference are clean, `main` equals `origin/main`, and
the deterministic selector reports no open plan. It orders `SELF-001`,
`BAP-002`, then `STAT-001`. `SELF-001` is dependency-blocked because firmware
still has no production-safe route for its fan, voltage, power, thermal, ASIC
diagnostic, cancel, pass, and fail hardware submodes, and the required
hardware-regression safety evidence is absent. `BAP-002` is dependency- and
authority-blocked by `BAP-001`: firmware UART/task/subscription ownership is
not implemented, and no compatible accessory or separately authorized
electrical attachment is available under standing USB authority.

`STAT-001` is the first actionable row. Attempt-006 crossed package/runtime
identity, attestation parsing, production serial, terminal transport and pool
state, safe-stop, cleanup, seal, mode, and privacy boundaries, then failed as
`watchdog_unresponsive` with the overly coarse discriminator
`watchdog_not_participating`. Pushed commit
`91ab642b4b3ee2edf8f23190fad41ca2fc5d0620` fixes that exact source-owned
boundary: campaign-result v12 and network-continuity v6 now preserve every
evaluator reason before the generic participation check, plus distinct
missing, unknown, stale, and inconsistent states. Production-shaped Rust and
real-child TypeScript regressions pass. Attempt-007 is therefore a
progress-backed continuation, not an unchanged retry.

The active lessons total 30,773 bytes with a summed conservative estimate of
10,259 tokens, above both deterministic loading limits. Every heading was
inventoried; all global lessons and nineteen complete repository blocks
covering ESP-IDF ownership/redaction, opaque handoff, real process boundaries,
espflash reset effects, power/USB separation, direct electrical authority,
protected roots, earliest failure, main-task capacity, HTTP readiness, private
classification, progress-backed retries, qualified transports, evaluator
identity, flash/monitor separation, standing authorization, preflight exit,
telemetry state, and legacy wire units informed this plan. Unloaded blocks are
GSD frontmatter, native-capture prearming, boot-proof replay, silent-transport
heartbeat, manual-removal observation, physical USB identity, cold-boot
observation, and the two manual-checkpoint lessons. The 2026-08-03 audit
baseline has six later
lessons, fewer than ten and fewer than 90 days old; this plan appends no lesson,
so no distinct audit trigger exists.

## Scope and non-scope

Advance only `STAT-001`. Rebind the existing private-first
`bitaxe-hashrate-monitor-evidence-v1` workflow, independent Rust validator,
generated TypeScript contract, immutable task/plan admission, Bazel runfiles,
protected roots, and real-child fixtures from consumed attempt-006 to fresh
attempt-007. Preserve campaign-result v12, network-continuity v6, and the
complete twenty-value closed watchdog vocabulary including success `none`.
Add no production hashrate, mining, sensor, watchdog, or hardware-control
behavior.

After all software and package gates pass at a clean pushed commit, one exact
board-205 package may be factory-flashed and normally reset/re-enumerated. The
workflow may seed only ignored local Wi-Fi and pool inputs, derive its target
only from protected current-session serial evidence, and run the repo-owned
conservative `live-share` profile for exactly 600 accumulated active seconds.
It may join protected serial, HTTP, and reconstructed WebSocket observations,
pause and safe-stop, release USB/process ownership, seal the result, and use at
most one supervisor-owned exact-package recovery flash after a post-flash
failure. No human action is required.

The profile units remain distinct: ASIC frequency is 400 MHz, core-voltage
command is 1,100 millivolts (1.1 V), and fan duty is 100 percent. Input-power
safety consumes INA260 bus voltage in volts and admits only 4.5 through 5.5 V;
it must never compare input voltage with the millivolt setpoint. Require fresh
input voltage, power, ASIC temperature, and fan truth; at most 15 W; ASIC
temperature below 75 C; and fresh nonzero fan RPM after the 100-percent fan
command.

Credentials, owner/worker fields, endpoints, origins, ports, hostnames,
USB/network/process identity, exact hashrates and sensor values, bodies, logs,
commands, PIDs, and traces remain private. They may exist only in mode-`0600`
files beneath ignored mode-`0700` roots. Only the plan-named closed projection
may become public after independent validation. No upstream-default or
overclock profile, arbitrary control target, automatic fan mode, unbounded
mining, OTA, erase, raw write, fault injection, physical power action, external
UART, or pin/pad/header/GPIO/probe/jumper/solder/signal manipulation is
authorized. Electrical calibration, profitability, dynamic retuning, extended
soak, broader share behavior, other ASICs/boards, update/recovery behavior, and
release readiness remain non-claims.

## Implementation

- [ ] Rebind roots, immutable plan/task admission, generated contract, Rust
      validator, Bazel inputs, and fixtures to attempt-007 without changing the
      public projection path or campaign behavior.
- [ ] Preserve v12/v6 and regression-test all twenty closed watchdog values,
      success `none`, seal/category gating, earliest-failure precedence,
      redaction, and value-free output at the real child boundary.
- [ ] Prove current source/reference semantics, exact `live-share` plus
      `conservative` command, protected layout, profile units, and independently
      volt-typed input safety.
- [ ] Run every focused and mandatory software, firmware, privacy, reference,
      package, source-admission, and generated-contract gate; commit and push
      the exact implementation before detector or credential access.
- [ ] Execute only the frozen detector and conditional attempt-007 commands;
      publish and promote only if the independent complete quorum passes.

## Authorized live commands and recovery

After the implementation is clean, fully gated, committed, pushed, and its
exact package rebuilt and validated, run only these commands in order:

1. `test ! -e scratch/stat001-hashrate-monitor/wrapper-007 && (umask 077; mkdir -m 700 -p scratch/stat001-hashrate-monitor/wrapper-007 && just detect-ultra205 > scratch/stat001-hashrate-monitor/wrapper-007/detector.stdout 2> scratch/stat001-hashrate-monitor/wrapper-007/detector.stderr)`
2. Only after command 1 exits zero, admits exactly one Ultra 205 through
   `espflash board-info --chip esp32s3 --non-interactive`, cleanup and holder
   checks pass, both ignored credential files are nonempty without being read,
   and the supervisor child, projection, and candidate are absent:
   `test ! -e scratch/stat001-hashrate-monitor/attempt-007 && test ! -e docs/parity/evidence/stat001-hashrate-monitor/hashrate-monitor-projection.json && test -s wifi-credentials.json && test -s pool-credentials.json && (umask 077; just capture-hashrate-monitor-evidence --private-root scratch/stat001-hashrate-monitor/attempt-007 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --pool-credentials pool-credentials.json --detector-output scratch/stat001-hashrate-monitor/wrapper-007/detector.stdout --projection docs/parity/evidence/stat001-hashrate-monitor/hashrate-monitor-projection.json --duration-seconds 600 --capture-timeout-seconds 1500 > scratch/stat001-hashrate-monitor/wrapper-007/capture.stdout 2> scratch/stat001-hashrate-monitor/wrapper-007/capture.stderr)`

The wrapper root must be mode `0700`; detector and capture streams must be
distinct mode-`0600` siblings; and the supervisor-owned attempt child,
projection, and candidate must be absent before launch. Starting command 2
consumes attempt-007. Preserve the earliest typed failure and watchdog label
through bounded safe stop, recovery, sealing, and cleanup. Never retry
unchanged or start attempt-008 under this plan. Stop on detector ambiguity or
failure, non-205 identity, missing credentials, source/reference/package drift,
unsafe state, incomplete quorum, safe-stop/recovery/cleanup/seal/mode/privacy
failure, nonzero command exit, or successful verified projection. Select one
terminal outcome: `complete`, `stop_repeated_boundary`,
`stop_hardware_blocker`, `stop_authority_boundary`, or
`stop_impossible_contract`.

## Verification and promotion

Before hardware, run focused hashrate automation, independent Rust contract,
campaign network/watchdog/profile, generated-contract, real-child, current
task/plan, source/reference admission, seal, protected-mode, redaction, and
failure-precedence tests. Run `just verify-redaction`,
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
to prove board 205, attempt 7, exact source/reference/package/plan, one
detector-admitted device, trusted runtime identity, one-second cadence and
pinned register semantics, one ASIC/four domains, active mining and work
renewal in all twenty half-open 30-second windows, at least two changing
coherent positive observations from each transport, positive current and all
four rolling windows after warmup, finite bounded error, watchdog failure
`none`, terminal zero current rate in both transports, safe stop,
recovery/cleanup, protected modes, exact seals, independent validation, and
redaction.

On success create `RESULT.md`, commit evidence as `SOURCE_COMMIT`, transition
only STAT-001 to `verified` with
`unit,workflow,api-compare,hardware-smoke,hardware-regression`, synchronize
progress immediately, archive the completed task, run final gates, review,
commit, and push. Any missing fact withholds the projection and verified claim,
leaves STAT-001 `implemented`, records a truthful `CLOSURE.md`, and stops
without an unchanged retry.
