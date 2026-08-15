# Parity work plan

- Run ID: `20260815T222316Z-IO-002`
- Parity row: `IO-002`
- Initial status: `implemented`
- Source commit: `ce2e876c5fd0027a6d8bcff29c49114c14e4e766`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-io002-adc-observation-attempt-002`
- Corrects closed plan: `docs/parity/work-plans/20260815T210711Z-IO-002/PLAN.md`

## Selection

The clean synchronized selector reported no open plan and ranked `IO-002`
first, so no higher-ranked candidate was skipped. Attempt-001 reached a safe,
disabled device and produced fresh HTTP and WebSocket ADC observations, but its
immutable validator incorrectly imposed a 400–2,000 mV energized-rail range on
the disabled state. The resulting closure consumed attempt-001 and prohibited
an unchanged retry.

The corrective software commit `ce2e876c5fd0027a6d8bcff29c49114c14e4e766`
establishes retry eligibility without learning an acceptance value from the
failed attempt: the reference and Rust producer both expose calibrated integer
millivolts, the Rust adapter's source type is `u16`, and the independently
validated system-info evidence binds the observation to disabled mining and
hardware control. Focused Rust and real child-process regressions prove that
zero is valid in that closed wire domain while fractional, negative, above-
`u16`, stale, incoherent, and non-disabled observations remain rejected.

The active lesson inputs exceed the deterministic startup limits. All global
blocks and complete repository safety, authorization, evidence, privacy,
hardware-retry, redaction, ESP-IDF, protected-root, failure-ordering, physical-
interface, evaluator-identity, flash/monitor, task-authorization, checkpoint,
preflight, and telemetry-range blocks informed this plan. Seven disclosed
lower-priority repository blocks were not loaded. The existing audit baseline
remains valid and no lesson-audit trigger is active.

## Scope and non-scope

Advance only IO-002. Rebind the existing closed ADC evidence workflow from the
consumed attempt-001 plan, task, paths, and ordinal to this immutable
attempt-002 contract. Preserve its independent validators, source-semantic
checks, exact-package identity, atomic publication, protected artifacts, and
read-only system-info transaction. Do not change the production ADC adapter,
calibration, producer cadence, public API projection, safety policy, mining, or
hardware-control behavior.

The sole hardware attempt may factory-flash one exact clean board-205 package,
perform the transaction's normal USB reset/re-enumeration, seed only the local
Wi-Fi credential input, derive one same-origin target only from protected
current-session serial evidence, and issue read-only HTTP/WebSocket/retained-
log observations. If the initial flash occurred and safe recovery cannot
otherwise be proved, the transaction may perform at most one exact-package
recovery flash. No settings mutation, restart request, mining, pool input, ASIC
work, voltage/frequency/fan/power control, raw ADC/GPIO/I2C, OTA, erase, fault
injection, physical power action, direct UART, or pin/pad/header/probe/jumper/
solder/signal manipulation is authorized.

NeverPersistRaw values remain memory-only. Hostnames, origins, ports, USB and
network identities, settings, HTTP bodies, exact ADC values, acquisition
stamps, boot sessions, logs, commands, PIDs, and traces remain mode `0600`
beneath ignored mode `0700` roots. Public evidence may contain only the closed
schema, source/reference/package identities, opaque input digests, fixed public
configuration constants, bounded categories, counts, booleans, and redaction
status. Energized-rail voltage or accuracy, externally measured calibration,
induced ADC failure, voltage actuation, load behavior, long-duration drift,
non-205 boards, and release readiness remain explicit non-claims.

## Implementation

- [ ] Replace the consumed attempt-001 task, plan, path, SHA, and ordinal
      bindings with the immutable attempt-002 contract in the ADC evidence
      functional core and its generated/public schema boundary.
- [ ] Update behavior-focused Rust, TypeScript, invocation, protected-path,
      immutable-plan, and real child-process regressions for attempt-002 while
      retaining rejection coverage for all corrected millivolt boundaries.
- [ ] Build and admit an exact clean pushed package, then run exactly one
      detector-gated read-only attempt-002 capture using the commands below.
- [ ] Publish and independently validate one redacted IO-002 projection only if
      every source, package, ADC, cadence, correlation, disabled-state, cleanup,
      and privacy member passes; otherwise preserve `implemented`, record the
      earliest typed blocker, withhold evidence, and stop without retry.
- [ ] Transition only IO-002, synchronize progress, write RESULT or CLOSURE,
      and archive the task atomically only after the complete quorum passes.

## Verification and promotion

Before hardware, run focused ADC contract/input, orchestration, invocation,
redaction, source-semantic, protected-layout, validator-boundary, and real-
child tests. Then run, in order, `cargo fmt --all`, `cargo clippy --all-targets
--all-features -- -D warnings`, `cargo build --all-targets --all-features`, and
`cargo test --all-features`; `bun scripts/bright-builds-check.ts all`; the real
ESP32-S3 firmware/package build; `just test`; `just parity`; `just
parity-progress`; redaction, pinned-reference, immutable-plan, unique-task,
generated-contract, exact-package, source/reference, sensitive-output, and
diff checks. Commit, fetch/rebase if needed, and push the implementation before
detector or device access.

After those gates are green on a clean pushed implementation, the only
authorized hardware sequence is:

1. `test ! -e scratch/io002-adc/wrapper-002 && (umask 077; mkdir -m 700 -p scratch/io002-adc/wrapper-002 && just detect-ultra205 > scratch/io002-adc/wrapper-002/detector.stdout 2> scratch/io002-adc/wrapper-002/detector.stderr)`
2. Only after command 1 exits zero, admits exactly one Ultra 205 through
   `espflash board-info --chip esp32s3 --non-interactive`, cleanup/holder checks
   pass, and the ignored local Wi-Fi credential file exists:
   `test ! -e scratch/io002-adc/attempt-002 && test ! -e docs/parity/evidence/io002-adc/adc-observation-projection.json && (umask 077; just capture-adc-observation-evidence --private-root scratch/io002-adc/attempt-002 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --detector-output scratch/io002-adc/wrapper-002/detector.stdout --projection docs/parity/evidence/io002-adc/adc-observation-projection.json --capture-timeout-seconds 360 > scratch/io002-adc/wrapper-002/capture.stdout 2> scratch/io002-adc/wrapper-002/capture.stderr)`

The wrapper root must be mode `0700`; detector and capture streams must be
distinct mode-`0600` regular files; the supervisor-owned attempt child and
public projection must be absent immediately before launch. Starting command 2
consumes attempt-002. Preserve the earliest typed failure and always attempt
the base transaction's bounded recovery and cleanup after a post-flash failure.
No unchanged retry or attempt-003 is authorized. Non-ready hardware maps to
`hardware_blocked`, malformed or incomplete evidence to `evidence_invalid`,
child timeout to `timeout`, and launch failure to `process_failed`; recovery
remains secondary. Success selects `complete`, a hardware boundary selects
`stop_hardware_blocker`, and an authority or impossible-proof boundary selects
its matching closed outcome.

Promote IO-002 only if the independently validated projection binds board 205,
attempt 2, exact clean source/reference/package and workflow identity, detector
admission, stable boot, current pinned ADC semantics, exactly one production
ADC owner, fresh finite integral `u16`-domain millivolt observations in both
HTTP and WebSocket, independently validated disabled mining and hardware
control, equal boot session, monotonic sequence/acquisition stamps consistent
with the 500 ms producer cadence, exact public numeric/status correlation,
complete cleanup, no recovery flash, protected modes, atomic publication, and
passed redaction. Any missing, malformed, unsafe, incoherent, drifted, or
privacy-invalid member withholds the projection, keeps IO-002 at `implemented`,
records a truthful `CLOSURE.md`, and stops without retry. A passing projection
verifies only passive safe-state acquisition and public projection; every
energized, externally calibrated, failure-injected, or load-dependent behavior
listed above remains a non-claim.
