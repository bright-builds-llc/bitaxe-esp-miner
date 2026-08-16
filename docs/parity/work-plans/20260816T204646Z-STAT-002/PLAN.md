# Parity work plan

- Run ID: `20260816T204646Z-STAT-002`
- Parity row: `STAT-002`
- Initial status: `implemented`
- Source commit: `07164915ec9c54562bcd79611b23eaeebe3bc492`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-stat002-statistics-history`

## Selection

The clean synchronized selector reported no open plan and ranked `SELF-001`,
`BAP-002`, `STAT-001`, then `STAT-002`. `SELF-001` has no production-safe
self-test route, `BAP-002` depends on unfinished firmware UART interoperability
and an unavailable separately authorized electrical setup, and `STAT-001`
repeated the same `watchdog_feed_stale` hardware discriminator after its
targeted five-second feed correction. Progress-gated retry rules prohibit
another `STAT-001` ordinal until a verified source change alters that boundary.

`STAT-002` is the first actionable row. Its pure bounded history, sole
absolute-cadence producer and read-only API projection are implemented and
covered by unit, workflow and API comparison evidence. Its remaining decisive
proof is independent of mining: one detector-gated exact-package run can enable
history through the ordinary settings API, demonstrate that samples accumulate
on the firmware's one-second producer cadence independently of HTTP request
timing, restore the exact original setting, and publish only aggregate facts.

The active lesson inputs exceed the deterministic startup limits. All lesson
headings were inventoried and the complete safety, privacy, authorization,
evidence, retry, unit, transport and hardware-attempt blocks relevant to this
row were loaded. The existing audit baseline remains valid and no new lesson-
audit trigger is active. Repo-local guidance, the Bright Builds sidecar and
overrides, and the architecture, code-shape, verification, testing, Rust and
TypeScript standards informed this plan.

## Scope and non-scope

Advance only `STAT-002`. Add a private-first
`bitaxe-statistics-history-evidence-v1` workflow and an independent Rust
validator. Bind the immutable plan and task, exact clean source/reference/
package identity, admitted detector output, completed flash effect, passive
safe state, one current-session same origin, request digest, owner-only modes,
restoration, cleanup and redaction. The public projection may contain only
identities and digests; the exact 19-label and row-width constants; sample and
interval counts; minimum/maximum interval milliseconds; and closed booleans or
categories. Raw statistics, settings values, timestamps, origins, hostnames,
addresses, ports, USB/network/process identities, device/session identifiers,
HTTP bodies, logs and credentials remain private.

The sole attempt may factory-flash one exact clean board-205 package, perform
the normal USB reset and re-enumeration, seed only the ignored Wi-Fi credential
input, derive exactly one origin from that current protected monitor capture,
read the original `statsFrequency`, PATCH only that field to a different
nonzero value, and poll `GET /api/system/statistics`. It must prove the exact
upstream-compatible labels and row width, finite numeric rows, strictly
increasing sample timestamps, bounded one-second intervals, an unchanged
immediate repeat read, and later growth without a request-time append. It must
then PATCH the exact original `statsFrequency`, confirm readback, and, when the
original was zero, confirm the history clears after the next producer tick.

Both implementations use nonzero `statsFrequency` to enable history and set
the retained horizon while their producer records every 1000 milliseconds.
The evidence must not interpret statistics `voltage` or `current` as physical
accuracy proof: those legacy wire columns remain millivolts and milliamps, and
this run publishes no raw telemetry values.

After mutation, the primary recovery is one same-origin PATCH of the exact
original setting plus readback. If the origin is unavailable, one and only one
exact-package flash-monitor recovery may re-establish a fresh current-session
origin, after which the workflow must PATCH the original setting and confirm
it. An exact-package flash alone is not restoration because NVS persists. No
recovery is permitted before mutation. The earliest primary failure remains
authoritative even if recovery also fails.

No pool credential, mining, ASIC work, frequency, voltage, fan, thermal or
power-control effect, OTA, erase, fault injection, physical power action,
direct UART, pin/pad/header/probe/jumper/solder/signal manipulation, browser
session or other board is authorized. Telemetry accuracy, physical electrical
measurement, a live 720-sample/full-horizon wait, browser chart rendering,
mining behavior, other boards, update/recovery parity and release readiness
remain explicit non-claims.

## Implementation

- [ ] Add the closed statistics-history evidence schema, generated command
      contract and independent Rust validator.
- [ ] Add a private-first TypeScript workflow with exact-package identity,
      one-field mutation, cadence/request-immutability checks, exact restoration,
      bounded recovery, protected modes and atomic public projection.
- [ ] Add behavior-focused Rust and TypeScript regressions for valid cadence,
      labels/shape, nonfinite or malformed rows, timestamp/interval failure,
      request-time append, missing later growth, identity mismatch, source
      drift, restoration/recovery failure, mode failure and sensitive output.
- [ ] Build and admit one exact clean pushed package, then execute only the
      detector and attempt-001 commands below.
- [ ] Publish and independently validate one redacted projection only on the
      complete quorum; otherwise preserve `implemented`, record the earliest
      typed blocker and terminal closure, and stop without retry.

## Verification and promotion

Before hardware, run focused contract and automation tests, generated-contract
verification, real-child, task/plan, reference/source, restoration, privacy,
failure-precedence and redaction checks. Then run, in order:

1. `cargo fmt --all`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo build --all-targets --all-features`
4. `cargo test --all-features`
5. `bun scripts/bright-builds-check.ts all`
6. the real ESP32-S3 package build
7. `just test`
8. `just parity`
9. `just parity-progress`
10. `just verify-redaction`
11. `just verify-reference`
12. immutable-plan, unique-task, generated-contract, exact-package,
    protected-mode, sensitive-output, no-public-output and diff checks

Commit, fetch/rebase if needed and push the implementation before detector or
device access. The package manifest must identify that exact clean pushed
source commit and the pinned reference commit.

After those gates pass, the sole authorized hardware sequence is:

1. `test ! -e scratch/stat002-statistics-history/wrapper-001 && (umask 077; mkdir -m 700 -p scratch/stat002-statistics-history/wrapper-001 && just detect-ultra205 > scratch/stat002-statistics-history/wrapper-001/detector.stdout 2> scratch/stat002-statistics-history/wrapper-001/detector.stderr)`
2. Only after command 1 exits zero, admits exactly one Ultra 205 through
   `espflash board-info --chip esp32s3 --non-interactive`, confirms cleanup and
   holder checks, and the ignored Wi-Fi credential file is nonempty:
   `test ! -e scratch/stat002-statistics-history/attempt-001 && test ! -e docs/parity/evidence/stat002-statistics-history/statistics-history-projection.json && (umask 077; just capture-statistics-history-evidence --private-root scratch/stat002-statistics-history/attempt-001 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --detector-output scratch/stat002-statistics-history/wrapper-001/detector.stdout --projection docs/parity/evidence/stat002-statistics-history/statistics-history-projection.json --capture-timeout-seconds 360 > scratch/stat002-statistics-history/wrapper-001/capture.stdout 2> scratch/stat002-statistics-history/wrapper-001/capture.stderr)`

The wrapper must remain mode `0700`; detector/capture redirects and every
supervisor-owned artifact must remain mode `0600`; the attempt child must be
created by the workflow at mode `0700`. Starting command 2 consumes attempt-001
whether it succeeds, fails or times out. No attempt-002, relaunch, alternate
capture or ad hoc device request is authorized by this plan.

Promotion to `verified` requires the independent validator to accept exact
source/reference/package/workflow identity; the admitted single-board detector;
completed exact-package flash; passive safe state; one current-session origin;
only one settings field mutated; immediate enabled readback; at least three
strictly increasing samples; every adjacent interval within 750–1500 ms; exact
labels and row width; finite numeric cells; an identical immediate repeat;
later producer growth; exact original-setting restoration; zero-setting clear
when applicable; disabled mining and hardware control; owner-only modes;
complete USB/process cleanup; and passed redaction. Unit evidence remains the
proof of the exact 720-sample eviction boundary, while this live evidence proves
the production cadence and device API ownership. Any incomplete condition
withholds public evidence and leaves the row `implemented`.
