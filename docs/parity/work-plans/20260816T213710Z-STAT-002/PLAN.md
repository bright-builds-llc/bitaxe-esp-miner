# Parity work plan

- Run ID: `20260816T213710Z-STAT-002`
- Parity row: `STAT-002`
- Initial status: `implemented`
- Source commit: `0294d1963489a9f5a86e640c88587cc0aeba5236`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-stat002-statistics-history`

## Selection

The clean synchronized selector reported no open plan and ranked `SELF-001`,
`BAP-002`, `STAT-001`, then `STAT-002`. `SELF-001` still has no production-safe
self-test route. `BAP-002` still depends on unfinished firmware UART/live
accessory interoperability and an unavailable separately authorized electrical
setup. `STAT-001` repeated the same sealed `watchdog_feed_stale` boundary after
its targeted five-second threshold correction; progress-gated retry rules
require a new reproduced diagnosis and source fix before another ordinal.

`STAT-002` is the first actionable row. Attempt-001 established a precise
software defect before origin admission or settings mutation: the evidence
supervisor assigned the flash-monitor child the same 360-second lifetime that
the child owned for capture, so the supervisor terminated the child at its own
timeout boundary before cleanup and typed result delivery. This plan applies
that root-cause correction, proves it through a real spawned child, and permits
one fresh attempt-002 only after the corrected source is clean, pushed,
packaged, and admitted.

The active lesson inputs exceed the deterministic startup limits. Every lesson
heading was inventoried; all global lessons and complete task-relevant safety,
privacy, authorization, evidence, retry, transport, process-boundary, telemetry
unit, and hardware-attempt blocks were loaded. The unrelated GSD separator and
six older cold-boot/manual-observer blocks were not loaded. The August 3 audit
baseline remains current: five later lessons are below the ten-lesson trigger
and 90 days have not elapsed. Repo-local guidance, the Bright Builds sidecar,
the empty effective overrides, and the architecture, code-shape, verification,
testing, Rust, and TypeScript standards informed this plan.

## Scope and non-scope

Advance only `STAT-002`. Replace the equal child/supervisor timeout with one
explicit bounded supervisor lifetime equal to the configured child capture
budget plus exactly 60 seconds for child cleanup and result delivery. Keep the
child's `--capture-timeout-seconds 360` unchanged. The same helper and boundary
must govern initial and recovery flash-monitor children. Add a real-process
regression that models a child reaching its own smaller capture boundary,
finishing cleanup, writing its completed effect and monitor artifacts, and
exiting before the strictly later supervisor boundary. Preserve pure tests for
the exact production 420,000-millisecond lifetime and equal-boundary rejection.

Rebind the closed workflow, task/source validator, generated invocation
fixtures, private roots, wrapper roots, immutable plan digest, and tests from
consumed attempt-001 to fresh attempt-002. Preserve exact source/reference/
package identity, detector admission, passive safe state, same-origin API
ownership, one-field `statsFrequency` mutation, exact restoration, bounded
same-package recovery after mutation only, earliest-failure precedence,
owner-only evidence modes, independent validation, and atomic publication.

Statistics `voltage` and `current` remain legacy millivolt and milliamp wire
columns. This plan does not use their raw values or claim telemetry accuracy.
It does not authorize pool credentials, mining, ASIC work, frequency, voltage,
fan, thermal or power-control effects, OTA, erase, fault injection, physical
power action, browser work, direct UART, or any pin/pad/header/GPIO/probe/
jumper/solder/signal manipulation. It does not claim live full-horizon 720-
sample retention, browser charts, other boards, updates, recovery parity, or
release readiness.

## Implementation

- [ ] Add the bounded child-timeout-plus-cleanup-grace supervisor policy and
      use it for both initial and recovery flash-monitor processes.
- [ ] Add behavior-focused pure and real-process regressions proving exact
      production arithmetic, strict boundary separation, child-owned timeout,
      cleanup/result delivery, and unchanged failure classification.
- [ ] Rebind the closed attempt paths, task/plan digest, invocation fixture,
      source validation, and protected-mode checks to attempt-002.
- [ ] Build and admit one exact clean pushed package, then execute only the
      detector and attempt-002 commands below.
- [ ] Publish and independently validate one aggregate-only projection only on
      the complete live cadence/API/restoration quorum; otherwise preserve
      `implemented`, record the earliest blocker, and stop without retry.

## Verification and promotion

Before hardware, run focused statistics-history, process-boundary, invocation,
generated-contract, Rust validator, task/plan, source/reference, restoration,
privacy, failure-precedence and redaction tests. Then run, in order:

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
    protected-mode, sensitive-output, absent-public-output, and diff checks

Commit, fetch/rebase only when necessary and conflict-free, and push the
implementation before detector or device access. The package manifest must
identify that exact clean pushed source and the pinned reference commit.

After every gate passes, the sole authorized hardware sequence is:

1. `test ! -e scratch/stat002-statistics-history/wrapper-002 && (umask 077; mkdir -m 700 -p scratch/stat002-statistics-history/wrapper-002 && just detect-ultra205 > scratch/stat002-statistics-history/wrapper-002/detector.stdout 2> scratch/stat002-statistics-history/wrapper-002/detector.stderr)`
2. Only after command 1 exits zero, admits exactly one Ultra 205 through
   `espflash board-info --chip esp32s3 --non-interactive`, confirms cleanup and
   holder checks, and the ignored Wi-Fi credential file is nonempty:
   `test ! -e scratch/stat002-statistics-history/attempt-002 && test ! -e docs/parity/evidence/stat002-statistics-history/statistics-history-projection.json && (umask 077; just capture-statistics-history-evidence --private-root scratch/stat002-statistics-history/attempt-002 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --detector-output scratch/stat002-statistics-history/wrapper-002/detector.stdout --projection docs/parity/evidence/stat002-statistics-history/statistics-history-projection.json --capture-timeout-seconds 360 > scratch/stat002-statistics-history/wrapper-002/capture.stdout 2> scratch/stat002-statistics-history/wrapper-002/capture.stderr)`

The wrapper must remain mode `0700`; detector/capture redirects and every
supervisor-owned artifact must remain mode `0600`; the attempt child must be
created by the workflow at mode `0700`. Starting command 2 consumes attempt-002
whether it succeeds, fails, or times out. Attempt-001 must never be reused. No
attempt-003, relaunch, alternate capture, or ad hoc device request is authorized
by this plan. The agent/tool wall clock must exceed the 420-second child-plus-
cleanup supervisor bound.

Promotion to `verified` requires the independent validator to accept exact
source/reference/package/workflow identity; the admitted single-board detector;
completed exact-package flash; passive safe state; one current-session origin;
only one settings field mutated; enabled readback; at least three strictly
increasing samples; every adjacent interval within 750-1500 milliseconds; exact
labels and row width; finite numeric cells; an identical immediate repeat;
later producer growth; exact original-setting restoration; zero-setting clear
when applicable; disabled mining and hardware control; owner-only modes;
complete cleanup; and passed redaction. Unit evidence remains proof of exact
720-sample eviction. Any incomplete condition withholds public evidence and
leaves the row `implemented`.
