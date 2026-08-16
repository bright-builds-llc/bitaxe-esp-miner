# Parity work plan

- Run ID: `20260816T020135Z-STAT-001`
- Parity row: `STAT-001`
- Initial status: `implemented`
- Source commit: `572e154083018ded41ca16afabd7f5a93a0fe34f`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-stat001-hashrate-monitor`

## Selection

The clean synchronized selector reported no open plan and ranked `UI-001`,
`UI-002`, `UI-003`, `SELF-001`, `BAP-002`, then `STAT-001`. UI-001 and UI-002
need trusted physical-panel observation, UI-003 needs trusted physical-button
interaction, SELF-001 still has no production-safe hardware execution route,
and BAP-002 needs an authorized compatible accessory and electrical UART setup.
STAT-001 is the first actionable row because its prior capture stopped during
pre-effect source admission, the exact overbroad reference fragment is now
narrowed and regression-guarded on clean pushed source, and the active task
explicitly permits a new immutable plan to authorize fresh attempt-002.

The active lesson inputs exceed the deterministic startup limits. All global
blocks and complete repository safety, authorization, privacy, evidence,
redaction, protected-root, retry-progress, earliest-failure, process-boundary,
direct-electrical-interface, evaluator-identity, task-authorization, physical-
checkpoint, preflight, and telemetry-state/range blocks informed this plan.
The unchanged repository blocks about GSD frontmatter; native-USB capture,
boot-replay, heartbeat, manual-removal, physical-identity, and cold-boot
observer history; ESP-IDF main-task capacity; and HTTP readiness were not
loaded. The existing lesson-audit baseline remains valid and no audit trigger
is active. Repo-local guidance, Bright Builds rules and overrides, and the
architecture, code-shape, verification, testing, and Rust standards were
reviewed.

## Scope and non-scope

Advance only STAT-001. Rebind the existing closed
`bitaxe-hashrate-monitor-evidence-v1` workflow, independent Rust validator,
generated TypeScript contract, task/plan admission, and tests from consumed
attempt-001 to fresh attempt-002 paths and ordinal. Preserve the corrected
unique pinned-reference semantics, current-source admission, exact-package
identity, private hashrate accumulator, projection schema, and conservative
campaign behavior. Add a checked-in current-task/current-plan regression so
the complete pre-effect admission passes before any sensitive input or device
effect.

The sole hardware attempt may build and factory-flash one exact clean board-205
package, perform normal USB reset and re-enumeration, seed only the ignored
local Wi-Fi and pool credential inputs, derive one same-origin target only from
the protected current-session serial stream, run the repository's conservative
400 MHz / 1100 mV / 100% fan mining profile for exactly 600 active seconds,
read HTTP/WebSocket/serial observations, pause and safe-stop, and use at most
one exact-package recovery flash after a post-flash failure. Credentials remain
opaque runtime inputs.

Raw hashrates, sensor readings, hostnames, origins, ports, USB/network/process
identity, pool fields, owner or worker strings, credentials, HTTP/WebSocket
bodies, logs, commands, PIDs, and traces stay private in mode-`0600` files below
ignored mode-`0700` roots. Only the already closed aggregate projection may be
published after independent validation. No upstream-default or overclock
profile, arbitrary voltage/frequency/fan target, unbounded mining, OTA, erase,
interrupted-update test, fault injection, physical power action, direct UART,
or pin/pad/header/probe/jumper/solder/signal manipulation is authorized.
Analog accuracy, electrical measurement, profitability, general share-outcome
parity, dynamic retuning, extended soak, other ASICs or boards, updates,
recovery behavior, and release readiness remain non-claims.

## Implementation

- [ ] Rebind the private capture, independent validator, generated contract,
      task/plan admission, and exact paths from attempt-001 to attempt-002.
- [ ] Add behavior-focused pure and real-child regressions that run the current
      checked-in task, plan, production, and pinned-reference admission before
      any sensitive input or effect, while retaining all malformed, mismatch,
      mode, seal, source-drift, and sensitive-output failures.
- [ ] Run focused and full software/firmware/privacy gates, commit and push the
      exact implementation, and build/admit a new exact clean package.
- [ ] Execute only the detector and attempt-002 commands below, then publish
      and independently validate the projection only on the complete quorum.
- [ ] Otherwise preserve `implemented`, withhold the projection, record the
      earliest typed blocker and accepted terminal outcome, and do not retry.

## Verification and promotion

Before hardware, run focused hashrate core, campaign-network, automation,
independent-validator, real-child, task/plan, source/reference, generated-
contract, privacy, failure-precedence, and redaction tests. Then run, in order,
`cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`,
`cargo build --all-targets --all-features`, and `cargo test --all-features`;
`bun scripts/bright-builds-check.ts all`; the real ESP32-S3 package build;
`just test`; `just parity`; `just parity-progress`; redaction, pinned-reference,
immutable-plan, unique-task, generated-contract, exact-package, sensitive-
output, and diff checks. Commit, fetch/rebase if necessary, and push the
implementation before detector or device access.

After those gates pass on clean pushed implementation, the only authorized
hardware sequence is:

1. `test ! -e scratch/stat001-hashrate-monitor/wrapper-002 && (umask 077; mkdir -m 700 -p scratch/stat001-hashrate-monitor/wrapper-002 && just detect-ultra205 > scratch/stat001-hashrate-monitor/wrapper-002/detector.stdout 2> scratch/stat001-hashrate-monitor/wrapper-002/detector.stderr)`
2. Only after command 1 exits zero, admits exactly one Ultra 205 through
   `espflash board-info --chip esp32s3 --non-interactive`, cleanup and holder
   checks pass, and both ignored credential files are nonempty:
   `test ! -e scratch/stat001-hashrate-monitor/attempt-002 && test ! -e docs/parity/evidence/stat001-hashrate-monitor/hashrate-monitor-projection.json && (umask 077; just capture-hashrate-monitor-evidence --private-root scratch/stat001-hashrate-monitor/attempt-002 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --pool-credentials pool-credentials.json --detector-output scratch/stat001-hashrate-monitor/wrapper-002/detector.stdout --projection docs/parity/evidence/stat001-hashrate-monitor/hashrate-monitor-projection.json --duration-seconds 600 --capture-timeout-seconds 1500 > scratch/stat001-hashrate-monitor/wrapper-002/capture.stdout 2> scratch/stat001-hashrate-monitor/wrapper-002/capture.stderr)`

The wrapper root must be mode `0700`; detector and capture streams must be
distinct mode-`0600` regular files; the supervisor-owned attempt child and
public projection must be absent immediately before launch. Starting command 2
consumes attempt-002. Preserve the earliest typed failure and always attempt
the base transaction's bounded safe stop, recovery, and cleanup after a post-
flash failure. No unchanged retry or attempt-003 is authorized. Non-ready
hardware maps to `hardware_blocked`, malformed or incomplete proof to
`evidence_invalid`, child timeout to `timeout`, and launch failure to
`process_failed`; recovery remains secondary.

Promote STAT-001 only if the independent validator proves board 205, attempt 2,
the exact clean pushed source/reference/package, one detector-admitted device,
trusted runtime identity, the one-second monitor cadence and pinned register
semantics, exactly one ASIC and four domains, accepted active mining and work
renewal through all twenty 30-second windows, at least two positive changing
coherent hashrate observations in each transport, positive aggregate and all
four rolling windows after warmup, finite bounded error reporting, terminal
zero current rate in both transports, safe stop, USB cleanup, protected modes,
an exact result seal, independent validation, and redaction. Then write
`RESULT.md`, commit evidence before transition, transition only STAT-001 from
`implemented` to `verified` with
`unit,workflow,api-compare,hardware-smoke,hardware-regression`, immediately sync
progress to the evidence commit, archive the completed task, run all final
gates in the required order, inspect the diff, commit, and push.
