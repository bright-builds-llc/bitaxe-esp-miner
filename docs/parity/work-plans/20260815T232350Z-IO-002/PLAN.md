# Parity work plan

- Run ID: `20260815T232350Z-IO-002`
- Parity row: `IO-002`
- Initial status: `implemented`
- Source commit: `51d0ee9050ccf63762de7a44ec92fa11dd617a4d`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-io002-adc-observation-attempt-004`
- Corrects closed plan: `docs/parity/work-plans/20260815T225042Z-IO-002/PLAN.md`

## Selection

The clean synchronized selector reported no open plan and ranked `IO-002`
first; no higher-ranked candidate was skipped. Attempt-003 supplied a new,
authoritative boundary signature after the real task preflight fix: exact-
package safe boot, same-origin capture, disabled-state millivolt input
validation, redaction, and cleanup passed, but final source-provenance admission
rejected the breadcrumb `.bitwidth = ADC_BITWIDTH_DEFAULT` as non-unique. The
pinned upstream file contains that legitimate initializer field three times.

The pre-hardware suite did not catch the defect because
`adc-observation-evidence.test.ts` was compiled but omitted from the
`all.test.ts` test aggregator. An uncached Bazel run executed 328 other tests
without naming either checked-in ADC regression, while directly invoking the
deployed production semantic validator reproduced `ADC source semantic
fragment is not unique`. This continuation is actionable because it changes
both real failing boundaries: one exact contextual upstream breadcrumb replaces
the ambiguous token, and the ADC suite is registered in the deployed Bazel test
entrypoint before any fresh ordinal is consumed.

The active lesson inputs exceed the deterministic startup limits. All global
blocks and complete repository safety, authorization, evidence, privacy,
hardware-retry, redaction, ESP-IDF, protected-root, failure-ordering, physical-
interface, evaluator-identity, flash/monitor, task-authorization, checkpoint,
preflight, and telemetry-range blocks informed this plan. Seven disclosed
unchanged lower-priority repository transport-history blocks were not loaded.
The existing audit baseline remains valid and no lesson-audit trigger is
active. Repo-local guidance, the Bright Builds sidecar and overrides, and the
architecture, code-shape, verification, testing, Rust, and TypeScript standards
were reviewed.

## Scope and non-scope

Advance only IO-002. Register the existing ADC evidence tests in the deployed
automation test aggregator and first prove that the real checked-in source-
semantic regression fails on the three-occurrence breadcrumb. Replace only that
broad upstream token with one whitespace-normalized contextual fragment that
uniquely identifies the production ADC channel configuration block. Keep the
pinned reference file read-only and explicitly declared in Bazel runfiles.
Rebind the closed `bitaxe-adc-observation-evidence-v1` workflow from consumed
attempt-003 to this immutable attempt-004 task, plan, SHA, paths, and ordinal.

Preserve the pre-effect real task/plan admission, corrected disabled-state
`u16` millivolt validator, production firmware ADC calibration and acquisition,
500 ms producer cadence, API projection, safety, mining, hardware-control,
independent evidence validation, exact-package identity, privacy, and atomic
publication behavior. The sole hardware attempt may factory-flash one exact
clean board-205 package, perform the transaction's normal USB reset and re-
enumeration, seed only the local Wi-Fi credential input, derive one same-origin
target only from protected current-session serial evidence, and issue read-only
HTTP, WebSocket, and retained-log observations. If the initial flash occurred
and safe recovery cannot otherwise be proved, the transaction may perform at
most one exact-package recovery flash.

No settings or restart request, mining, pool input, ASIC work, voltage,
frequency, fan or power control, raw ADC/GPIO/I2C, OTA, erase, fault injection,
physical power action, direct UART, or pin/pad/header/probe/jumper/solder/signal
manipulation is authorized. NeverPersistRaw values remain memory-only.
Hostnames, origins, ports, USB and network identities, settings, HTTP bodies,
exact ADC values, acquisition stamps, boot sessions, logs, commands, PIDs, and
traces remain mode `0600` beneath ignored mode `0700` roots. Public evidence may
contain only the closed schema, source/reference/package identities, opaque
input digests, fixed public configuration constants, bounded categories,
counts, booleans, and redaction status. Energized-rail voltage or accuracy,
external calibration, induced ADC failure, voltage actuation, load behavior,
long-duration drift, non-205 boards, and release readiness remain explicit
non-claims.

## Implementation

- [ ] Register `adc-observation-evidence.test.ts` in the production Bazel test
      entrypoint and prove the previously omitted checked-in semantic regression
      fails before changing the breadcrumb.
- [ ] Replace the ambiguous upstream bit-width token with one unique contextual
      ADC channel-configuration fragment and prove missing, duplicated, and
      drifted context fail closed through the real runfiles boundary.
- [ ] Replace only the consumed attempt-003 task, plan, path, digest, and ordinal
      bindings with attempt-004 across the evidence core and generated/public
      contract, preserving every corrected millivolt-domain check.
- [ ] Build and admit an exact clean pushed package, then run exactly one
      detector-gated passive attempt-004 capture using the commands below.
- [ ] Publish and independently validate one redacted projection only on the
      full safe-state ADC/API quorum; otherwise preserve `implemented`, record
      the earliest typed blocker, withhold evidence, and stop without retry.

## Verification and promotion

Before hardware, run the ADC suite from the real `all.test.ts` Bazel entrypoint
with cache disabled and require its test names in the emitted log. Add focused
pure checks for unique context plus missing, duplicated, and drifted context;
then run task/plan preflight, ADC contract/input, orchestration, invocation,
redaction, protected-layout, validator-boundary, generated-contract, and real-
child tests. Prove the actual `TASKS.md`, this exact plan, and pinned upstream
ADC file are present in the deployed runfiles layout. Then run, in order,
`cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`,
`cargo build --all-targets --all-features`, and `cargo test --all-features`;
`bun scripts/bright-builds-check.ts all`; the real ESP32-S3 package build;
`just test`; `just parity`; `just parity-progress`; redaction, pinned-reference,
immutable-plan, unique-task, generated-contract, exact-package, source/reference,
sensitive-output, and diff checks. Commit, fetch/rebase if needed, and push the
implementation before detector or device access.

After those gates are green on a clean pushed implementation, the only
authorized hardware sequence is:

1. `test ! -e scratch/io002-adc/wrapper-004 && (umask 077; mkdir -m 700 -p scratch/io002-adc/wrapper-004 && just detect-ultra205 > scratch/io002-adc/wrapper-004/detector.stdout 2> scratch/io002-adc/wrapper-004/detector.stderr)`
2. Only after command 1 exits zero, admits exactly one Ultra 205 through
   `espflash board-info --chip esp32s3 --non-interactive`, cleanup and holder
   checks pass, and the ignored local Wi-Fi credential file exists:
   `test ! -e scratch/io002-adc/attempt-004 && test ! -e docs/parity/evidence/io002-adc/adc-observation-projection.json && (umask 077; just capture-adc-observation-evidence --private-root scratch/io002-adc/attempt-004 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --detector-output scratch/io002-adc/wrapper-004/detector.stdout --projection docs/parity/evidence/io002-adc/adc-observation-projection.json --capture-timeout-seconds 360 > scratch/io002-adc/wrapper-004/capture.stdout 2> scratch/io002-adc/wrapper-004/capture.stderr)`

The wrapper root must be mode `0700`; detector and capture streams must be
distinct mode-`0600` regular files; the supervisor-owned attempt child and
public projection must be absent immediately before launch. Starting command 2
consumes attempt-004. Preserve the earliest typed failure and always attempt the
base transaction's bounded recovery and cleanup after a post-flash failure. No
unchanged retry or attempt-005 is authorized. Non-ready hardware maps to
`hardware_blocked`, malformed or incomplete evidence to `evidence_invalid`,
child timeout to `timeout`, and launch failure to `process_failed`; recovery
remains secondary. Success selects `complete`, a repeated post-fix provenance
signature selects `stop_repeated_boundary`, a distinct hardware boundary
selects `stop_hardware_blocker`, and an authority or impossible-proof boundary
selects its matching closed outcome.

Promote IO-002 only if the independently validated projection binds board 205,
attempt 4, exact clean source/reference/package and workflow identity, detector
admission, stable boot, current pinned ADC semantics, exactly one production ADC
owner, fresh finite integral `u16`-domain millivolt observations in HTTP and
WebSocket, independently validated disabled mining and hardware control, equal
boot session, monotonic sequence and acquisition stamps consistent with the
500 ms producer cadence, exact public numeric/status correlation, complete
cleanup, no recovery flash, protected modes, atomic publication, and passed
redaction. Any missing, malformed, unsafe, incoherent, drifted, or privacy-
invalid member withholds the projection, keeps IO-002 at `implemented`, writes
a truthful `CLOSURE.md`, and stops without retry. A passing projection verifies
only passive safe-state acquisition and public projection; energized,
externally calibrated, failure-injected, and load-dependent behavior remains a
non-claim.
