# Parity work plan

- Run ID: `20260825T123346Z-STR-005-AUTONOMOUS-CONTINUATION`
- Parity row: `STR-005`
- Initial status: `implemented`
- Source commit: `d6244cc669865aabcd5313bd95b98c3c80852ac6`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-str005-autonomous-continuation`
- Continues: `docs/parity/work-plans/20260824T214920Z-STR-005-RESTORE-RECOVERY3/CLOSURE.md`
- Predecessor closure SHA-256: `19bc1e1560546081f7c066b1ff379843c13b5489ec0f5139f0944e90fe462b67`

## Objective

Autonomously diagnose and fix each progress-bearing STR-005 recovery/campaign
boundary, run fresh hardware ordinals without repeated human confirmation, and
continue until STR-005 is independently verified or the progress-gated hardware
policy selects a terminal stop. This single rolling contract replaces a new
formal plan per failure; the active task records each exact fresh ordinal,
command, boundary signature, fix, regression, outcome, and next decision.

## Progress loop

For each boundary:

1. Build a fast red-capable reproduction at the real failing seam and record the
   closed boundary signature.
2. Rank falsifiable hypotheses, instrument only the distinguishing boundary,
   and retain protected closed diagnostics without raw values.
3. Add the regression before the targeted fix, prove red then green, remove
   temporary instrumentation, and run every required software/privacy gate.
4. Commit/push the exact fix, rebuild the exact clean package, append the next
   fresh ordinal and fully expanded command to the active task, and run it once.
5. Continue only after `continue_after_verified_fix` or an already-authorized
   non-invasive `continue_after_manual_remediation` with objective changed-state
   evidence. Never repeat unchanged inputs or reuse a root.
6. Stop on `stop_repeated_boundary`, `stop_hardware_blocker`,
   `stop_authority_boundary`, or `stop_impossible_contract`.

There is no fixed ordinal cap. A recurring coarse category with a new protected
discriminator may return to diagnosis. The same authoritative boundary after
its targeted real-boundary fix is terminal.

## Initial changed boundary and recovery-004

Recovery-003 stopped `hardware_blocked/runtime_monitor_process` before identity
capture. The current monitor caller uses a 15-second capture and 30-second child
bound despite the accepted device-session contract requiring a 60-second
monitor-admission window. It also discards child exit/timeout/output/USB cleanup
facts. The initial work adds a protected closed monitor-child receipt, a tight
real-process reproduction, and the smallest contract-aligned fix before
recovery-004.

Initial exact command after gates, push, package, and detector:

`just stratum-v2-runtime-monitor-diagnostic --board 205 --port <detector-port> --private-root scratch/str005-runtime-monitor-diagnostic/diagnostic-001 --redact-evidence`

This read-only diagnostic must return `runtime_monitor_ready` with a protected
accepted receipt before recovery-004 becomes eligible. It performs no HTTP read,
flash read/write, NVS access, fixture, pool, mining, or hardware control.

Recovery-004 exact command after the diagnostic passes:

`just stratum-v2-restore-recovery --board 205 --port <detector-port> --private-root scratch/str005-installed-package-recovery/recovery-004 --projection docs/parity/evidence/str005-installed-package-recovery/restore-readiness-projection-004.json --redact-evidence`

Every later fresh recovery command must use the next unused three-digit ordinal,
fresh matching private/projection paths, and the same command shape. Before any
effect, the active task must contain the fully expanded command and the recovery
owner/tests must admit only that exact task-recorded path pair under this plan.

## Campaign commands

After one recovery ordinal publishes accepted readiness, bind its exact bundle
path in source/tests/task and run:

1. `just stratum-v2-campaign-preflight --board 205 --port preflight-only --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --private-root scratch/str005-stratum-v2/attempt-004 --projection docs/parity/evidence/str005-stratum-v2/stratum-v2-projection.json --duration-seconds 180 --redact-evidence --restore-bundle <accepted-recovery-root>/restore-bundle.private.json`
2. `just stratum-v2-runtime-admission --board 205 --port <detector-port> --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --private-root scratch/str005-stratum-v2/attempt-004 --projection docs/parity/evidence/str005-stratum-v2/stratum-v2-projection.json --duration-seconds 180 --redact-evidence --restore-bundle <accepted-recovery-root>/restore-bundle.private.json`
3. Only after both succeed and campaign targets remain absent:
   `just stratum-v2-campaign --board 205 --port <detector-port> --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --restore-bundle <accepted-recovery-root>/restore-bundle.private.json --private-root scratch/str005-stratum-v2/attempt-004 --projection docs/parity/evidence/str005-stratum-v2/stratum-v2-projection.json --duration-seconds 180 --redact-evidence`

The task must replace `<accepted-recovery-root>` with the exact accepted private
root before these commands become eligible. Attempt-004 remains single-use and
is never retried unchanged.

## Effects, evidence, and safety

Recovery ordinals may perform read-only runtime HTTP/USB observation, one
bounded package search, one timestamp-pinned historical rebuild, and the exact
eight-range 460800-baud firmware-only snapshot fallback. They never read NVS or
coredump and never write the device. Private roots/directories are `0700`; all
identity, bundle, receipt, log, and binary files are `0600`.

The campaign retains the existing 180-second lease, host-owned local Noise
fixture, 400 MHz/1100 mV/100% fan ceiling, one accepted-share ceiling,
continuous safety/watchdog checks, safe stop, exact original bundle/settings
restoration with `mineonboot=false`, and complete cleanup. Snapshot restoration
writes only the admitted eight ranges once after campaign effects make rollback
necessary, then seeds Wi-Fi and restores settings/theme.

Public projections contain only closed categories, booleans, bounded
counts/durations, bundle kind, digests, current provenance, and redaction
status. They never contain runtime values, paths, ports, logs, credentials,
endpoints, flash bytes, child output, or raw API bodies.

Before every fresh hardware ordinal: ordered Cargo format/clippy/build/test,
Bright Builds, all Bazel tests, boundary-specific real-launch tests, canonical
firmware build/package, parity/progress, redaction, reference cleanliness,
selector/task lineage, sensitive-value review, diff review, clean pushed source,
exact package identity, fresh detector, and absent fresh outputs must pass.

Raw NVS/coredump capture, new-baseline flash, external pools, direct UART/pins,
fault injection, OTA, erase, unlisted writes, unbounded mining, a second
attempt-004 campaign, and attempt-005 campaign remain prohibited.

## Completion

One independently accepted attempt-004 plus exact original runtime/package/
settings restoration creates `RESULT.md`, transitions only `STR-005` to
`verified` with `unit,golden,workflow,hardware-regression`, synchronizes
progress, archives completed STR-005 tasks, final-verifies, commits, and pushes.

Any terminal stop creates `CLOSURE.md`, withholds `RESULT.md`, preserves
`STR-005` at `implemented`, records the authoritative boundary and cleanup, and
commits/pushes the truthful state.
