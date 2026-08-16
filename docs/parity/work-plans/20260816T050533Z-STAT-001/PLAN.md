# Parity work plan

- Run ID: `20260816T050533Z-STAT-001`
- Parity row: `STAT-001`
- Initial status: `implemented`
- Source commit: `b37ebf1ab47a2af28882a142e5e06754893b5abd`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-stat001-hashrate-monitor`

## Selection

The clean synchronized selector reported no open plan and ranked `UI-001`,
`UI-002`, `UI-003`, `SELF-001`, `BAP-002`, then `STAT-001`. UI-001 and UI-002
still require trusted physical panel observation; UI-003 requires trusted
physical button observation; SELF-001 lacks a production-safe hardware
execution route; and BAP-002 depends on BAP-001 plus an unavailable authorized
compatible accessory and qualified electrical UART setup. STAT-001 is the
first actionable row. Attempt-004 was accepted with exact package identity,
trusted runtime identity, zero attestation parse failures, safe stop, and USB
cleanup, but the host coordinator returned network status `not_required` for
the required conservative `LiveShare` stage. Pushed source commit
`89e8c34c794e6cfca499e4f392699be39e20e7dd` materially fixes that exact
production boundary by routing `LiveShare` through the same continuity observer
as `Soak`, with one exhaustive five-stage regression. One fresh attempt-005 is
therefore progress-backed rather than an unchanged retry.

The active lesson inputs total 29,963 bytes with a conservative summed estimate
of 9,990 tokens, above both deterministic loading limits. Every lesson heading
was inventoried; all seven global blocks and the repository service ownership,
opaque handoff, real-process boundary, espflash reset, power/USB distinction,
native capture, boot replay, silent transport, direct electrical authority,
protected-root ownership, earliest failure, HTTP readiness, private
classification, progress-backed retry, qualified transport, evaluator
identity, flash/monitor separation, standing authorization, preflight exit, and
telemetry-state blocks informed this plan. Omitted repository blocks were
`lesson-gsd-frontmatter-body-separators`,
`lesson-manual-removal-needs-owner-observation`,
`lesson-physical-usb-identity-excludes-enumeration-fields`,
`lesson-cold-boot-proof-needs-an-independent-observer`,
`lesson-esp-idf-main-task-runtime-capacity`,
`lesson-time-bounded-physical-checkpoints-must-be-prearmed-and-self-describing`,
and `lesson-never-invite-ready-before-live-checkpoint`. The 2026-08-03 audit
baseline consumed the hard-limit crossing; only five new active lessons have
accumulated, fewer than 90 days have elapsed, and this plan appends no lesson,
so no distinct audit trigger exists.

## Scope and non-scope

Advance only STAT-001. Rebind the existing private-first
`bitaxe-hashrate-monitor-evidence-v1` workflow, independent Rust validator,
generated TypeScript contract, immutable task/plan admission, Bazel runfiles,
protected paths, and tests from consumed attempt-004 to fresh attempt-005.
Preserve campaign-result v10, the closed runtime-attestation parse
discriminator, current source/reference semantic admission, the conservative
campaign, the projection schema, the exact hashrate quorum, and fail-closed
publication. Add no new runtime or hardware-control behavior.

After all implementation and package gates pass at a clean pushed source
commit, the sole attempt may factory-flash one exact board-205 package, perform
normal USB reset/re-enumeration, seed only ignored local Wi-Fi and pool inputs,
derive the network target only from the protected current-session serial
stream, run the repository-owned conservative profile for exactly 600
accumulated active seconds, join serial, HTTP, and reconstructed WebSocket
observations, pause and safe-stop, clean up USB/process ownership, and use at
most one exact-package recovery flash after a post-flash failure.

The profile's units are explicit and distinct: the ASIC frequency is 400 MHz,
the ASIC core-voltage command is 1,100 millivolts (1.1 V), and fan duty is 100
percent. Input-power safety consumes INA260 bus voltage in volts and accepts
4.5 through 5.5 V; it must never compare that field with the millivolt
setpoint. Core-voltage telemetry remains millivolts and is not substituted for
input-bus voltage. Require fresh input-voltage, power, chip-temperature, and fan
truth; at most 15 W; ASIC temperature below 75 C; and nonzero fan RPM after the
100-percent command.

Credentials, pool owner/worker fields, endpoints, origins, ports, hostnames,
USB/network/process identity, exact hashrates and sensor values, bodies, logs,
commands, PIDs, and traces remain private. They may exist only in mode-`0600`
files beneath ignored mode-`0700` roots; only the plan-named closed projection
may become public after independent validation. No upstream-default or
overclock profile, arbitrary control target, automatic fan mode, unbounded
mining, OTA, erase, raw write, fault injection, physical power action, external
UART, or pin/pad/header/GPIO/probe/jumper/solder/signal manipulation is
authorized. Electrical calibration, profitability, dynamic retuning, extended
soak, broader share behavior, other ASICs/boards, update/recovery behavior, and
release readiness remain non-claims.

## Implementation

- [ ] Rebind protected roots, immutable plan admission, task text, generated
      contract, Rust validator, Bazel inputs, and fixtures to attempt-005.
- [ ] Regression-test the exact current task/plan, source/reference semantics,
      real `live-share` plus `conservative` child command, protected layout,
      attempt ordinal, campaign-result v10, incomplete quorum, seal, and closed
      failure envelope.
- [ ] Prove the profile unit contract remains 400 MHz / 1,100 mV / 100 percent
      and input-bus safety remains independently typed in volts.
- [ ] Run every focused and mandatory software, firmware, privacy, reference,
      package, source-admission, and generated-contract gate; commit and push
      the exact source before detector or credential access.
- [ ] Execute only the frozen detector and conditional attempt-005 commands,
      then publish and promote only if the independent complete quorum passes.

## Verification and promotion

Before hardware, run the focused hashrate-monitor automation tests, independent
Rust contract tests, campaign network/profile tests, generated-contract check,
real-child boundary, current task/plan and source/reference admission,
redaction, protected-mode, seal, and failure-precedence tests. Run
`just verify-redaction`, `just verify-reference`, `just package`, and the
mandatory final sequence in order:

1. `cargo fmt --all`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo build --all-targets --all-features`
4. `cargo test --all-features`
5. `bun scripts/bright-builds-check.ts all`
6. `just test`
7. `just parity`
8. `just parity-progress`

Commit, fetch/rebase only when conflict-free and necessary, push the exact
implementation, then rebuild and validate the exact clean package. Only then
may these two commands run in order:

1. `test ! -e scratch/stat001-hashrate-monitor/wrapper-005 && (umask 077; mkdir -m 700 -p scratch/stat001-hashrate-monitor/wrapper-005 && just detect-ultra205 > scratch/stat001-hashrate-monitor/wrapper-005/detector.stdout 2> scratch/stat001-hashrate-monitor/wrapper-005/detector.stderr)`
2. Only after command 1 exits zero, admits exactly one Ultra 205 through
   `espflash board-info --chip esp32s3 --non-interactive`, cleanup and holder
   checks pass, both ignored credential files are nonempty without being read,
   and the supervisor child and public projection are absent:
   `test ! -e scratch/stat001-hashrate-monitor/attempt-005 && test ! -e docs/parity/evidence/stat001-hashrate-monitor/hashrate-monitor-projection.json && test -s wifi-credentials.json && test -s pool-credentials.json && (umask 077; just capture-hashrate-monitor-evidence --private-root scratch/stat001-hashrate-monitor/attempt-005 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --pool-credentials pool-credentials.json --detector-output scratch/stat001-hashrate-monitor/wrapper-005/detector.stdout --projection docs/parity/evidence/stat001-hashrate-monitor/hashrate-monitor-projection.json --duration-seconds 600 --capture-timeout-seconds 1500 > scratch/stat001-hashrate-monitor/wrapper-005/capture.stdout 2> scratch/stat001-hashrate-monitor/wrapper-005/capture.stderr)`

The wrapper root must be mode `0700`; detector and capture streams must be
distinct mode-`0600` files; and the supervisor-owned attempt child, projection,
and projection candidate must be absent before launch. Starting command 2
consumes attempt-005. Preserve the earliest typed failure through bounded safe
stop, recovery, seal, and cleanup. Never retry unchanged or start attempt-006
under this plan. Classify one terminal outcome: `complete`,
`stop_repeated_boundary`, `stop_hardware_blocker`, `stop_authority_boundary`, or
`stop_impossible_contract`.

Promote STAT-001 only if the independent validator proves board 205, attempt 5,
the exact clean pushed source/reference/package, one detector-admitted device,
trusted runtime identity, one-second monitor cadence and pinned register
semantics, one ASIC and four domains, active mining and work renewal in all
twenty half-open 30-second windows, at least two positive changing coherent
hashrate observations from each network transport, positive current and all
four rolling windows after warmup, finite bounded error, terminal zero current
rate in both transports, safe stop, USB/process cleanup, protected modes, exact
seals, independent validation, and redaction. On success create `RESULT.md`,
commit evidence, transition only STAT-001 to `verified` with
`unit,workflow,api-compare,hardware-smoke,hardware-regression`, synchronize
progress to that evidence commit, archive the completed task, run the terminal
gates, review, commit, and push. Any missing fact withholds promotion, leaves
STAT-001 `implemented`, and requires a truthful `CLOSURE.md`.
