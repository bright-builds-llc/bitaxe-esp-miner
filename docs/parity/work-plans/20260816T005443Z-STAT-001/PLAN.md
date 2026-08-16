# Parity work plan

- Run ID: `20260816T005443Z-STAT-001`
- Parity row: `STAT-001`
- Initial status: `implemented`
- Source commit: `edf01789abbf47654281b1b27635e823cf33dcae`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-stat001-hashrate-monitor`

## Selection

The clean synchronized selector reported no open plan and ranked `UI-001`,
`UI-002`, `UI-003`, `SELF-001`, `BAP-002`, then `STAT-001`. UI-001 and UI-002
need trusted physical-panel observation, UI-003 needs a trusted physical-button
observation, SELF-001 has no production-safe hardware execution route, and
BAP-002 needs an authorized compatible accessory and electrical UART setup.
None can be advanced safely or conclusively in this invocation. STAT-001 is
the first actionable row because the existing detector-gated conservative soak
already correlates the production ASIC worker with same-boot HTTP and WebSocket
sampling; it needs only a typed hashrate-specific observation quorum and one
fresh bounded run.

The active lesson inputs exceed the deterministic startup limits. All global
blocks and complete repository safety, authorization, evidence, privacy,
hardware-retry, redaction, protected-root, failure-ordering, direct-electrical-
interface, evaluator-identity, task-authorization, checkpoint, preflight, and
telemetry-range blocks informed this plan. The disclosed unchanged GSD,
USB/cold-boot observer-history, ESP-IDF task-capacity, and HTTP-readiness blocks
were not loaded. The existing audit baseline remains valid and no lesson-audit
trigger is active. Repo-local guidance, the Bright Builds sidecar and
overrides, and the architecture, code-shape, verification, testing, and Rust
standards were reviewed.

## Scope and non-scope

Advance only STAT-001. Extend the existing private mining-campaign network
evidence with a closed hashrate quorum derived from the already parsed
`SystemInfoWire` samples. The pure accumulator must require coherent finite
nonnegative current, 1-minute, 10-minute, and 1-hour rates; exactly one BM1366
ASIC with four domains; finite bounded error percentage; consistency between
the aggregate and per-ASIC total; positive changing active observations through
both HTTP and WebSocket; all four positive rolling windows after warmup; and a
zero current rate in both terminal paused observations. Raw values stay private.

Add a typed private-first `bitaxe-hashrate-monitor-evidence-v1` workflow and an
independent Rust validator. Bind the immutable plan and active task, exact clean
source/reference/package identity, detector output, private result seal and
file modes, current production/reference semantics, the accepted conservative
soak contract, safe stop, cleanup, and redaction. Publish atomically only the
closed schema, identities, digests, fixed topology/cadence/range constants,
counts, booleans, categories, and redaction status.

The sole hardware attempt may build and factory-flash one exact clean board-205
package, perform the normal USB reset and re-enumeration, seed only the ignored
local Wi-Fi and pool credential inputs, derive one same-origin target only from
the protected current-session serial stream, command the repository's
conservative 400 MHz / 1100 mV / 100% fan mining profile for exactly 600 active
seconds, read HTTP/WebSocket/serial observations, pause and safe-stop, and use
at most one exact-package recovery flash after a post-flash failure. The
credentials remain opaque runtime inputs.

No upstream-default or overclock profile, arbitrary voltage/frequency/fan
target, unbounded mining, OTA, erase, interrupted-update test, fault injection,
physical power action, direct UART, or pin/pad/header/probe/jumper/solder/signal
manipulation is authorized. Hostnames, origins, ports, USB/network identity,
pool fields, owner or worker strings, credentials, exact hashrates, sensor
values, boot sessions, HTTP/WebSocket bodies, logs, commands, PIDs, and traces
remain mode `0600` beneath ignored mode `0700` roots. Analog accuracy,
electrical measurement, profitability, accepted/rejected share outcomes,
dynamic retuning, extended soak, other ASICs or boards, updates, recovery, and
release readiness remain explicit non-claims.

## Implementation

- [ ] Add the minimum closed hashrate observation accumulator to the existing
      soak network evidence without retaining or publishing raw values.
- [ ] Add the typed private-first projection, independent Rust validator,
      immutable task/plan and source/reference/package admission, atomic
      publication, protected-mode checks, and generated command wiring.
- [ ] Add focused pure tests for valid active/terminal samples plus zero,
      nonfinite, incoherent topology, aggregate mismatch, unchanged-reading,
      missing rolling-window, identity, seal, mode, source-drift, malformed,
      and sensitive-output failures.
- [ ] Build and admit one exact clean pushed package, then execute only the
      detector and attempt-001 commands below.
- [ ] Publish and independently validate one redacted projection only on the
      full quorum; otherwise preserve `implemented`, record the earliest typed
      blocker, withhold public evidence, and stop without retry.

## Verification and promotion

Before hardware, run focused hashrate core, campaign-network, automation,
independent-validator, real-child, task/plan, source/reference, generated-
contract, privacy, failure-precedence, and redaction tests. Then run, in order,
`cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`,
`cargo build --all-targets --all-features`, and `cargo test --all-features`;
`bun scripts/bright-builds-check.ts all`; the real ESP32-S3 package build;
`just test`; `just parity`; `just parity-progress`; redaction, pinned-reference,
immutable-plan, unique-task, generated-contract, exact-package, sensitive-
output, and diff checks. Commit, fetch/rebase if needed, and push the
implementation before detector or device access.

After those gates pass on a clean pushed implementation, the only authorized
hardware sequence is:

1. `test ! -e scratch/stat001-hashrate-monitor/wrapper-001 && (umask 077; mkdir -m 700 -p scratch/stat001-hashrate-monitor/wrapper-001 && just detect-ultra205 > scratch/stat001-hashrate-monitor/wrapper-001/detector.stdout 2> scratch/stat001-hashrate-monitor/wrapper-001/detector.stderr)`
2. Only after command 1 exits zero, admits exactly one Ultra 205 through
   `espflash board-info --chip esp32s3 --non-interactive`, cleanup and holder
   checks pass, and both ignored credential files are nonempty:
   `test ! -e scratch/stat001-hashrate-monitor/attempt-001 && test ! -e docs/parity/evidence/stat001-hashrate-monitor/hashrate-monitor-projection.json && (umask 077; just capture-hashrate-monitor-evidence --private-root scratch/stat001-hashrate-monitor/attempt-001 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --pool-credentials pool-credentials.json --detector-output scratch/stat001-hashrate-monitor/wrapper-001/detector.stdout --projection docs/parity/evidence/stat001-hashrate-monitor/hashrate-monitor-projection.json --duration-seconds 600 --capture-timeout-seconds 1500 > scratch/stat001-hashrate-monitor/wrapper-001/capture.stdout 2> scratch/stat001-hashrate-monitor/wrapper-001/capture.stderr)`

The wrapper root must be mode `0700`; detector and capture streams must be
distinct mode-`0600` regular files; the supervisor-owned attempt child and
public projection must be absent immediately before launch. Starting command 2
consumes attempt-001. Preserve the earliest typed failure and always attempt the
base transaction's bounded safe stop, recovery, and cleanup after a post-flash
failure. No unchanged retry or attempt-002 is authorized. Non-ready hardware
maps to `hardware_blocked`, malformed or incomplete proof to
`evidence_invalid`, child timeout to `timeout`, and launch failure to
`process_failed`; recovery remains secondary.

Promote STAT-001 only if the independent validator proves board 205, attempt 1,
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
