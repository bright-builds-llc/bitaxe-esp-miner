# Parity work plan

- Run ID: `20260815T225042Z-IO-002`
- Parity row: `IO-002`
- Initial status: `implemented`
- Source commit: `2718da775eee61ba1bdcf174f5afee67e9df71f6`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-io002-adc-observation-attempt-003`
- Corrects closed plan: `docs/parity/work-plans/20260815T222316Z-IO-002/PLAN.md`

## Selection

The clean synchronized selector reported no open plan and ranked `IO-002`
first; no higher-ranked candidate was skipped. Attempt-002 supplied a new,
authoritative boundary signature after the millivolt-domain fix: exact-package
safe boot and cleanup succeeded, but ADC post-processing rejected the real
active task because it omitted the literal
`bitaxe-adc-observation-evidence-v1` binding. The synthetic task fixture had
that binding, so the complete test suite did not exercise the repository
artifact consumed by production.

This continuation is actionable because it changes the failing boundary rather
than merely rotating an ordinal. The minimum fix is to make the task/plan check
a pre-effect admission, exercise it against real Bazel/runfiles `TASKS.md` and
this immutable plan, and add both artifacts to the declared build graph. A
passing real-artifact regression plus the exact schema binding below supplies
the verified progress required to authorize fresh attempt-003.

The active lesson inputs exceed the deterministic startup limits. All global
blocks and complete repository safety, authorization, evidence, privacy,
hardware-retry, redaction, ESP-IDF, protected-root, failure-ordering, physical-
interface, evaluator-identity, flash/monitor, task-authorization, checkpoint,
preflight, and telemetry-range blocks informed this plan. Seven disclosed
lower-priority repository blocks were not loaded. The existing audit baseline
remains valid and no lesson-audit trigger is active. Repo-local guidance, the
Bright Builds sidecar and overrides, and the architecture, code-shape,
verification, testing, Rust, and TypeScript standards were reviewed.

## Scope and non-scope

Advance only IO-002. Rebind the existing closed
`bitaxe-adc-observation-evidence-v1` workflow from consumed attempt-002 to this
immutable attempt-003 task, plan, SHA, paths, and ordinal. Move task/plan
validation before the base system-info transaction so an invalid repository
contract fails before package admission, credential access, detector
consumption, flash/reset, network access, or protected child creation. Add a
real Bazel/runfiles regression that reads the checked-in task and plan rather
than a synthetic substitute. Preserve the corrected disabled-state `u16`
millivolt validator, independent evidence validation, source semantics,
exact-package identity, privacy, and atomic publication.

Do not change production firmware ADC calibration, acquisition, producer
cadence, API projection, safety, mining, or hardware-control behavior. The sole
hardware attempt may factory-flash one exact clean board-205 package, perform
the transaction's normal USB reset/re-enumeration, seed only the local Wi-Fi
credential input, derive one same-origin target only from protected current-
session serial evidence, and issue read-only HTTP/WebSocket/retained-log
observations. If the initial flash occurred and safe recovery cannot otherwise
be proved, the transaction may perform at most one exact-package recovery
flash. No settings or restart request, mining, pool input, ASIC work, voltage/
frequency/fan/power control, raw ADC/GPIO/I2C, OTA, erase, fault injection,
physical power action, direct UART, or pin/pad/header/probe/jumper/solder/signal
manipulation is authorized.

NeverPersistRaw values remain memory-only. Hostnames, origins, ports, USB and
network identities, settings, HTTP bodies, exact ADC values, acquisition
stamps, boot sessions, logs, commands, PIDs, and traces remain mode `0600`
beneath ignored mode `0700` roots. Public evidence may contain only the closed
schema, source/reference/package identities, opaque input digests, fixed public
configuration constants, bounded categories, counts, booleans, and redaction
status. Energized-rail voltage or accuracy, external calibration, induced ADC
failure, voltage actuation, load behavior, long-duration drift, non-205 boards,
and release readiness remain explicit non-claims.

## Implementation

- [ ] Add one production-owned task/plan preflight that reads the exact real
      repository artifacts and runs before all sensitive inputs and hardware
      effects; reuse the same validator after capture only if independently
      necessary.
- [ ] Declare this immutable plan in the Bazel runfiles graph and add a focused
      real-artifact regression that would reproduce attempt-002's missing-schema
      failure while keeping existing synthetic malformed-contract coverage.
- [ ] Replace only the consumed attempt-002 task, plan, path, digest, and ordinal
      bindings with attempt-003 across the evidence core and generated/public
      contract, preserving all corrected millivolt-domain checks.
- [ ] Build and admit an exact clean pushed package, then run exactly one
      detector-gated read-only attempt-003 capture using the commands below.
- [ ] Publish and independently validate one redacted projection only on the
      full safe-state ADC/API quorum; otherwise preserve `implemented`, record
      the earliest typed blocker, withhold evidence, and stop without retry.

## Verification and promotion

Before hardware, run focused task/plan preflight, ADC contract/input,
orchestration, invocation, redaction, source-semantic, protected-layout,
validator-boundary, generated-contract, and real-child tests. Prove the actual
`TASKS.md` block and this exact plan pass from the deployed Bazel/runfiles
layout, and prove a missing literal schema binding fails before the injected
system-info capture port can run. Then run, in order, `cargo fmt --all`, `cargo
clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets
--all-features`, and `cargo test --all-features`; `bun
scripts/bright-builds-check.ts all`; the real ESP32-S3 firmware/package build;
`just test`; `just parity`; `just parity-progress`; redaction, pinned-reference,
immutable-plan, unique-task, generated-contract, exact-package, source/
reference, sensitive-output, and diff checks. Commit, fetch/rebase if needed,
and push the implementation before detector or device access.

After those gates are green on a clean pushed implementation, the only
authorized hardware sequence is:

1. `test ! -e scratch/io002-adc/wrapper-003 && (umask 077; mkdir -m 700 -p scratch/io002-adc/wrapper-003 && just detect-ultra205 > scratch/io002-adc/wrapper-003/detector.stdout 2> scratch/io002-adc/wrapper-003/detector.stderr)`
2. Only after command 1 exits zero, admits exactly one Ultra 205 through
   `espflash board-info --chip esp32s3 --non-interactive`, cleanup/holder checks
   pass, and the ignored local Wi-Fi credential file exists:
   `test ! -e scratch/io002-adc/attempt-003 && test ! -e docs/parity/evidence/io002-adc/adc-observation-projection.json && (umask 077; just capture-adc-observation-evidence --private-root scratch/io002-adc/attempt-003 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --detector-output scratch/io002-adc/wrapper-003/detector.stdout --projection docs/parity/evidence/io002-adc/adc-observation-projection.json --capture-timeout-seconds 360 > scratch/io002-adc/wrapper-003/capture.stdout 2> scratch/io002-adc/wrapper-003/capture.stderr)`

The wrapper root must be mode `0700`; detector and capture streams must be
distinct mode-`0600` regular files; the supervisor-owned attempt child and
public projection must be absent immediately before launch. Starting command 2
consumes attempt-003. Preserve the earliest typed failure and always attempt
the base transaction's bounded recovery and cleanup after a post-flash failure.
No unchanged retry or attempt-004 is authorized. Non-ready hardware maps to
`hardware_blocked`, malformed or incomplete evidence to `evidence_invalid`,
child timeout to `timeout`, and launch failure to `process_failed`; recovery
remains secondary. Success selects `complete`, a hardware boundary selects
`stop_hardware_blocker`, and an authority or impossible-proof boundary selects
its matching closed outcome.

Promote IO-002 only if the independently validated projection binds board 205,
attempt 3, exact clean source/reference/package and workflow identity, detector
admission, stable boot, current pinned ADC semantics, exactly one production
ADC owner, fresh finite integral `u16`-domain millivolt observations in HTTP
and WebSocket, independently validated disabled mining and hardware control,
equal boot session, monotonic sequence/acquisition stamps consistent with the
500 ms producer cadence, exact public numeric/status correlation, complete
cleanup, no recovery flash, protected modes, atomic publication, and passed
redaction. Any missing, malformed, unsafe, incoherent, drifted, or privacy-
invalid member withholds the projection, keeps IO-002 at `implemented`, writes
a truthful `CLOSURE.md`, and stops without retry. A passing projection verifies
only passive safe-state acquisition and public projection; energized,
externally calibrated, failure-injected, and load-dependent behavior remains a
non-claim.
