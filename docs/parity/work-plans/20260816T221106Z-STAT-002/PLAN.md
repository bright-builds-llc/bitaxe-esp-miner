# Parity work plan

- Run ID: `20260816T221106Z-STAT-002`
- Parity row: `STAT-002`
- Initial status: `implemented`
- Source commit: `92740de099ba3bec32654741442664de10a55d5b`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-stat002-statistics-history`

## Selection

The clean synchronized selector reported no open plan and again ranked
`SELF-001`, `BAP-002`, `STAT-001`, then `STAT-002`. `SELF-001` has no production-
safe self-test route. `BAP-002` still depends on unfinished firmware UART/live
accessory interoperability and a separately authorized electrical setup that is
not available. `STAT-001` repeated the sealed `watchdog_feed_stale` signature
after its targeted threshold correction and requires a reproduced scheduler/
feed-owner diagnosis before another ordinal.

`STAT-002` is the first actionable row. Attempt-002 proved that the corrected
420-second supervisor outlived the child's 360-second monitor boundary, that the
exact package flash completed, and that the full monitor produced trusted
runtime attestation. It also isolated the next source defect: the supervisor
clock starts before factory flash, Wi-Fi NVS seed, USB stage gates, and monitor
preparation, so a monitor budget plus 60 seconds is not a whole-operation
budget. This plan removes that partial override and lets the already-established
bounded 900-second process adapter own the complete child lifetime.

The active lesson inputs exceed startup limits. All headings were inventoried;
all global lessons and complete task-relevant safety, privacy, authorization,
evidence, retry, process-boundary, telemetry-unit, and hardware blocks were
loaded. Only the unrelated GSD separator and six older cold-boot/manual-
observer blocks were excluded. Five post-baseline lessons remain below the ten-
lesson audit trigger and 90 days have not elapsed. Repo-local guidance, the
Bright Builds sidecar, empty effective overrides, and architecture, code-shape,
verification, testing, Rust, and TypeScript standards informed this plan.

## Scope and non-scope

Advance only `STAT-002`. Remove the statistics workflow's per-child 420-second
override. Keep `--capture-timeout-seconds 360` as the monitor's child-owned
capture bound and use the CLI's existing `createLocalProcessPort` 900,000-
millisecond default as the whole-operation supervisor for initial and recovery
flash-monitor children. Bind that exact default and absence of a local override
through source admission and regressions.

Replace the incomplete real-child fixture with a scaled process that spends
time in distinct pre-monitor, capture, and post-monitor/result phases, exceeds
the old scaled 420 boundary, writes completed flash/monitor/effect artifacts,
and exits within its enclosing adapter default. Preserve the closed timeout
classifier test. Rebind task/plan/path/invocation validation from consumed
attempt-002 to fresh attempt-003.

Preserve exact source/reference/package identity, one-device detector admission,
passive safe state, one current-session origin, one-field `statsFrequency`
mutation, exact restoration, post-mutation-only bounded same-package recovery,
earliest-failure precedence, owner-only evidence, independent validation, and
atomic aggregate-only publication. Statistics `voltage` and `current` remain
legacy millivolt and milliamp wire fields and are not accuracy evidence.

No pool input, mining, ASIC work, arbitrary frequency/voltage/fan/thermal/power
effect, OTA, erase, fault injection, physical power action, browser session,
direct UART, or electrical pin/pad/header/GPIO/probe/jumper/solder/signal work
is authorized. This plan does not claim full-horizon retention, browser charts,
telemetry accuracy, other boards, updates, recovery parity, or release readiness.

## Implementation

- [ ] Remove the partial flash-monitor lifetime override and bind the existing
      bounded 900-second whole-operation process owner.
- [ ] Add pure/source-admission and scaled real-child regressions covering pre-
      monitor, 360-equivalent capture, post-monitor result delivery, the old
      420-equivalent boundary, and unchanged typed timeout classification.
- [ ] Rebind every closed plan/task/path/invocation source to attempt-003 while
      preserving restoration, recovery, privacy, and failure precedence.
- [ ] Build and admit one exact clean pushed package, then execute only the
      detector and attempt-003 commands below.
- [ ] Publish and independently validate one aggregate-only projection only on
      the complete cadence/API/restoration quorum; otherwise preserve
      `implemented`, record the earliest blocker, and stop without retry.

## Verification and promotion

Before hardware, run focused statistics-history, process, invocation,
generated-contract, Rust validator, source/task/plan, restoration, privacy,
failure-precedence and real-child tests. Then run, in order:

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
12. immutable-plan, unique-task, exact timeout-owner, generated-contract,
    exact-package, protected-mode, sensitive-output, absent-public-output, and
    diff checks

Commit, fetch/rebase only when necessary and conflict-free, and push the
implementation before detector or device access. The package manifest must
identify that exact clean pushed source and the pinned reference commit.

After every gate passes, the sole authorized hardware sequence is:

1. `test ! -e scratch/stat002-statistics-history/wrapper-003 && (umask 077; mkdir -m 700 -p scratch/stat002-statistics-history/wrapper-003 && just detect-ultra205 > scratch/stat002-statistics-history/wrapper-003/detector.stdout 2> scratch/stat002-statistics-history/wrapper-003/detector.stderr)`
2. Only after command 1 exits zero, admits exactly one Ultra 205 through
   `espflash board-info --chip esp32s3 --non-interactive`, confirms cleanup and
   holder checks, and the ignored Wi-Fi credential file is nonempty:
   `test ! -e scratch/stat002-statistics-history/attempt-003 && test ! -e docs/parity/evidence/stat002-statistics-history/statistics-history-projection.json && (umask 077; just capture-statistics-history-evidence --private-root scratch/stat002-statistics-history/attempt-003 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --detector-output scratch/stat002-statistics-history/wrapper-003/detector.stdout --projection docs/parity/evidence/stat002-statistics-history/statistics-history-projection.json --capture-timeout-seconds 360 > scratch/stat002-statistics-history/wrapper-003/capture.stdout 2> scratch/stat002-statistics-history/wrapper-003/capture.stderr)`

The wrapper remains mode `0700`; detector/capture redirects and all contained
files remain mode `0600`; the supervisor alone creates the mode-`0700` attempt
child. Starting command 2 consumes attempt-003 whether it succeeds, fails, or
times out. Attempts 001/002 must never be reused. No attempt-004, relaunch,
alternate capture, or ad hoc device request is authorized. The agent/tool wall
clock must exceed the 900-second whole-operation process bound.

Promotion to `verified` requires independent acceptance of exact source/
reference/package/workflow identity; the admitted detector; completed exact-
package flash; trusted passive safe state and current-session origin; exactly
one settings field mutated; enabled readback; at least three strictly increasing
samples; every adjacent interval within 750-1500 milliseconds; exact labels and
row width; finite numeric cells; identical immediate repeat; later producer
growth; exact original-setting restoration; zero-setting clear when applicable;
disabled mining and hardware control; owner-only modes; complete cleanup; and
passed redaction. Unit evidence remains proof of exact 720-sample eviction. Any
incomplete condition withholds public evidence and leaves the row `implemented`.
