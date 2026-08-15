# Parity work plan

- Run ID: `20260815T210711Z-IO-002`
- Parity row: `IO-002`
- Initial status: `implemented`
- Source commit: `14b7d0c2886aca76f0ba14733c8135cdcd114b25`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-io002-adc-observation`
- Continues plan: `docs/parity/work-plans/20260804T140933Z-IO-002/PLAN.md`

## Selection

The clean synchronized selector reported no open plan and ranked `IO-002`
first; no higher-ranked candidate was skipped. The earlier immutable IO-002
plan and result implemented the exact ADC1/channel-1 curve-calibrated owner,
typed unavailable/fault/stale reduction, 500 ms producer cadence, and fresh-
only API projection. Its completion review deliberately withheld `verified`
because no admitted Ultra 205 proved the production ADC sample and public
projection on the same exact package.

This continuation is actionable because the repository already owns the
read-only system-info evidence transaction needed for exact-package flash,
detector admission, private serial-origin discovery, HTTP/WebSocket capture,
recovery, cleanup, and base validation. The missing work is one narrow typed
ADC projection and composite repo-owned command. No candidate is skipped and
no existing hardware artifact contains a current nonzero, stamped IO-002
sample eligible for reuse.

The active lesson inputs exceed the deterministic startup limits. Complete
safety, authorization, evidence, privacy, hardware-retry, redaction,
ESP-IDF, protected-root, exact-failure, physical-interface, and newest
preflight blocks plus all global blocks informed this plan. The six disclosed
lower-priority repository blocks were not loaded, and no lesson-audit trigger
is active. Repo-local guidance, the Bright Builds sidecar, standards override
file, and the architecture, code-shape, verification, testing, and Rust
standards were reviewed.

## Scope and non-scope

Add one closed `bitaxe-adc-observation-evidence-v1` functional-core contract,
independent Rust validator, generated TypeScript binding, and repo-owned
`capture-adc-observation-evidence` orchestration command. Reuse the existing
system-info capture transaction for one exact clean board-205 package and
validate the private HTTP/WebSocket snapshots without publishing their raw
values. Bind current production and pinned-reference semantics for ADC unit 1,
channel 1/GPIO2, 12 dB attenuation, default resolution, curve calibration,
the sole 500 ms producer, typed observation state, and the public
`coreVoltageActual`/`coreVoltageActualStatus` projection.

The sole hardware attempt may factory-flash one exact clean board-205 package,
perform the transaction's normal USB reset/re-enumeration, seed only the local
Wi-Fi credential input, derive one same-origin target only from protected
current-session serial evidence, and issue read-only HTTP/WebSocket/retained-
log observations. If the initial flash occurred and safe recovery cannot
otherwise be proved, the existing transaction may perform at most one exact-
package recovery flash. No settings mutation, restart request, mining, pool
credential, ASIC work, voltage/frequency/fan/power control, raw ADC/GPIO/I2C,
OTA, erase, fault injection, barrel-power action, direct UART, pin, pad,
header, probe, jumper, solder, or injected signal is authorized.

NeverPersistRaw values remain memory-only. Hostnames, origins, ports, USB and
network identities, settings, HTTP bodies, exact ADC values, acquisition
stamps, boot sessions, logs, commands, PIDs, and traces remain mode `0600`
beneath ignored mode `0700` roots. Public evidence may contain only the closed
schema, source/reference/package identities, opaque input digests, fixed public
configuration constants, bounded categories, counts, booleans, and redaction
status. Absolute electrical accuracy, externally measured voltage, induced ADC
failure, voltage actuation, long-duration drift, non-205 boards, and release
readiness remain explicit non-claims.

## Implementation

- [ ] Add and unit-test the closed Rust ADC evidence contract and independent
      input validator for private HTTP/WebSocket core-voltage observations.
- [ ] Add the generated binding, composite capture command, human command
      surface, source-semantic admission, redaction admission, protected-file
      checks, and behavior-focused real-child regressions.
- [ ] Commit and push the complete software path, build and admit an exact clean
      package, then run exactly one detector-gated read-only `attempt-001`
      capture through the protected command below.
- [ ] Publish and independently validate one redacted IO-002 projection only if
      every source, package, ADC, cadence, correlation, safety, cleanup, and
      privacy member passes; otherwise preserve `implemented`, the earliest
      typed failure, recovery facts, evidence withholding, and the accepted
      terminal stop outcome.
- [ ] Transition only IO-002, synchronize progress, complete the result/task
      review, and archive the task atomically only after the full quorum passes.

## Verification and promotion

Before hardware, run focused evidence-contract, private-input, automation,
invocation, redaction, source-semantic, protected-layout, validator-boundary,
and real-child tests; build the real ESP32-S3 firmware and exact package; then
run the mandatory ordered Cargo checks, Bright Builds, `just test`, `just
parity`, `just parity-progress`, redaction, pinned-reference, immutable-plan,
unique-task, generated-contract, exact-package, source/reference, sensitive-
output, and diff checks.

After the immutable plan/task checkpoint and complete implementation are
clean, committed, and pushed, the only authorized hardware sequence is:

1. `test ! -e scratch/io002-adc/wrapper-001 && (umask 077; mkdir -m 700 -p scratch/io002-adc/wrapper-001 && just detect-ultra205 > scratch/io002-adc/wrapper-001/detector.stdout 2> scratch/io002-adc/wrapper-001/detector.stderr)`
2. Only after command 1 exits zero, admits exactly one Ultra 205 through
   `espflash board-info --chip esp32s3 --non-interactive`, cleanup/holder checks
   pass, and the ignored local Wi-Fi credential file exists:
   `test ! -e scratch/io002-adc/attempt-001 && test ! -e docs/parity/evidence/io002-adc/adc-observation-projection.json && (umask 077; just capture-adc-observation-evidence --private-root scratch/io002-adc/attempt-001 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --detector-output scratch/io002-adc/wrapper-001/detector.stdout --projection docs/parity/evidence/io002-adc/adc-observation-projection.json --capture-timeout-seconds 360 > scratch/io002-adc/wrapper-001/capture.stdout 2> scratch/io002-adc/wrapper-001/capture.stderr)`

The wrapper root must be mode `0700`; detector and capture streams must be
distinct mode-`0600` regular files; the supervisor-owned attempt child and
public projection must be absent immediately before launch. Starting command 2
consumes attempt-001. Preserve the earliest typed failure and always attempt
the base transaction's bounded recovery and cleanup after a post-flash
failure. No unchanged retry or attempt-002 is authorized. Non-ready hardware
maps to `hardware_blocked`, malformed or incomplete evidence to
`evidence_invalid`, child timeout to `timeout`, and launch failure to
`process_failed`; recovery remains secondary. Success selects `complete`, a
hardware boundary selects `stop_hardware_blocker`, and an authority or
impossible-proof boundary selects its matching closed outcome.

Promote IO-002 only if the independently validated projection binds board 205,
attempt 1, exact clean source/reference/package and workflow identity,
detector admission, stable boot, current pinned ADC semantics, exactly one
production ADC owner, fresh finite plausible positive millivolt observations
in both HTTP and WebSocket, equal boot session, monotonic non-regressing
sequence/acquisition stamps consistent with the 500 ms producer cadence,
exact public numeric/status correlation, disabled mining and hardware control,
complete cleanup, no recovery flash, protected modes, atomic publication, and
passed redaction. Any missing, malformed, unsafe, incoherent, drifted, or
privacy-invalid member withholds the projection, keeps IO-002 at
`implemented`, records a truthful `CLOSURE.md`, and stops without retry.
