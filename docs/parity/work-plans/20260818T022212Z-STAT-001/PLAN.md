# Parity work plan

- Run ID: `20260818T022212Z-STAT-001`
- Parity row: `STAT-001`
- Initial status: `implemented`
- Source commit: `221430b1c6ac91993f9bc4e425600455746732e2`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-stat001-hashrate-monitor`

## Selection

The worktree and pinned reference are clean, `main` equals `origin/main`, and
the deterministic selector reports no open plan. Candidate order begins
`SELF-001`, `BAP-002`, then `STAT-001`.

`SELF-001` is dependency- and safety-blocked: its required hardware regression
still has no production-safe route covering fan, voltage, power, ASIC
diagnostic work, pass/fail/cancel, reboot, and recovery. `BAP-002` is blocked by
not-started `BAP-001` UART/request/subscription ownership; the only remaining
live-accessory path would require separately authorized external UART and
electrical attachment. `STAT-001` is first actionable.

Attempt-017's repeated seqlock retry-exhaustion boundary is materially
superseded. Pushed changes now serialize the watchdog snapshot, feed every
typed ASIC shutdown action, keep live-share active for the full evidence
horizon, and admit the exact terminal transition without an active-state race.
Ignored exact-package diagnostic run-010 at source `e70cefa7` completed 600,216
active milliseconds, all 20 windows, accepted submit evidence, stable watchdog,
trusted identity, fresh safety, terminal joins, safe stop, cleanup, and
redaction. That private run proves the source-level correction but is not
public parity evidence and cannot itself promote the row.

Loaded guidance materially includes `AGENTS.md`, `AGENTS.bright-builds.md`,
`standards/core/architecture.md`, `standards/core/code-shape.md`,
`standards/core/verification.md`, `standards/core/testing.md`,
`standards/languages/rust.md`, the active task/checklist, and bounded whole
lesson blocks for safety, authorization, evidence integrity, privacy, and
hardware retries. The lesson ledger is over budget; all headings were
inventoried, omitted blocks were disclosed, and the August 3 above-limit audit
baseline means no new audit trigger is due.

## Scope and non-scope

Advance only `STAT-001`. Rebind the existing independently validated
`bitaxe-hashrate-monitor-evidence-v1` workflow from consumed attempt-017 to one
fresh attempt-018: immutable plan digest, task binding, protected roots,
attempt ordinal, fixtures, generated/build inputs, result v16, network v11,
closed watchdog/correlation diagnostics, source inventory, seals, modes, units,
and redaction. Do not change production mining or hashrate behavior in this
plan unless a focused software gate exposes drift.

After every software and firmware gate passes on a clean pushed commit, one
exact board-205 package may be factory-flashed and normally reset, seeded only
from ignored local Wi-Fi and pool inputs, and run at the repo-owned conservative
profile: 400 MHz, 1,100 mV ASIC core, 100% fan, and exactly 600 accumulated
active seconds. Require fresh independent input bus voltage 4.5-5.5 V, power at
or below 15 W, ASIC temperature below 75 C, and nonzero fan RPM. Join protected
serial/HTTP/WebSocket observations, then safe-stop, clean up, seal, and allow at
most one supervisor-owned exact-package recovery flash after a post-flash
failure. ASIC core millivolts and input-bus volts remain distinct domains.

Only mode-0600 private values beneath fresh ignored mode-0700 roots may be
written. Credentials, pool/owner/worker strings, origins, ports, USB/network
identity, exact sensor/hashrate values, bodies, serial, commands, PIDs, and
traces remain private. Only the independently validated aggregate projection
may publish. No overclock, arbitrary control target, unbounded mining, OTA,
erase, raw write, fault injection, physical power action, external UART/BAP,
or pin/pad/header/probe/jumper/solder/signal manipulation is authorized.

## Implementation

- [ ] Rebind plan/task/root/ordinal/fixture surfaces from attempt-017 to
      attempt-018 and rotate the immutable plan digest.
- [ ] Preserve result v16, network v11, all closed watchdog and correlation
      diagnostics, exact source/reference inventory, prior-ordinal rejection,
      seals, protected modes, legacy wire units, and redaction.
- [ ] Run focused and mandatory software/firmware gates; commit, push, and build
      the exact package before device access.
- [ ] Run only the frozen detector and sole conditional attempt-018 capture;
      promote only on the complete independently validated quorum.

## Authorized live commands and recovery

After the exact implementation source is clean, gated, committed, pushed,
packaged, and identity-checked, run only:

1. `test ! -e scratch/stat001-hashrate-monitor/wrapper-018 && (umask 077; mkdir -m 700 -p scratch/stat001-hashrate-monitor/wrapper-018 && just detect-ultra205 > scratch/stat001-hashrate-monitor/wrapper-018/detector.stdout 2> scratch/stat001-hashrate-monitor/wrapper-018/detector.stderr)`
2. After command 1 exits zero, admits exactly one Ultra 205, inputs are
   nonempty without being read, and child/projection remain absent:
   `test ! -e scratch/stat001-hashrate-monitor/attempt-018 && test ! -e docs/parity/evidence/stat001-hashrate-monitor/hashrate-monitor-projection.json && test -s wifi-credentials.json && test -s pool-credentials.json && (umask 077; just capture-hashrate-monitor-evidence --private-root scratch/stat001-hashrate-monitor/attempt-018 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --pool-credentials pool-credentials.json --detector-output scratch/stat001-hashrate-monitor/wrapper-018/detector.stdout --projection docs/parity/evidence/stat001-hashrate-monitor/hashrate-monitor-projection.json --duration-seconds 600 --capture-timeout-seconds 1500 > scratch/stat001-hashrate-monitor/wrapper-018/capture.stdout 2> scratch/stat001-hashrate-monitor/wrapper-018/capture.stderr)`

The caller owns only wrapper-018 streams; the supervisor exclusively creates
the absent attempt child. Starting command 2 consumes attempt-018. Never retry
it, reuse attempt-017, or start attempt-019 under this plan. Preserve the
earliest typed failure through safe stop, recovery, sealing, and cleanup. Any
nonzero command, identity/safety/privacy/cleanup/recovery failure, or missing
fact withholds the projection and stops this plan.

## Verification and promotion

Focused verification covers hashrate automation, Rust contract validation,
campaign result/network schemas, watchdog/correlation diagnostics, generated
contracts, real-child behavior, task/plan binding, source/reference identity,
seal, mode, legacy units, redaction, and earliest precedence; then real
firmware/package builds, `just verify-redaction`, and `just verify-reference`.

Mandatory ordered gates are `cargo fmt --all`, `cargo clippy --all-targets
--all-features -- -D warnings`, `cargo build --all-targets --all-features`,
`cargo test --all-features`, `bun scripts/bright-builds-check.ts all`, `just
test`, `just parity`, and `just parity-progress`.

Promotion requires exact board/attempt/source/reference/package/plan/detector;
one BM1366 and four domains at one-second cadence; 20 windows with work renewal;
changing coherent positive HTTP/WebSocket current rates and warm rolling
windows; bounded error; watchdog none/stable; correlation failure none;
terminal zero; accepted or rejected submit evidence; safe stop; cleanup;
protected modes; seals; independent validator success; and redaction. On
success create `RESULT.md`, commit evidence as `SOURCE_COMMIT`, transition only
`STAT-001` to `verified` with `unit,workflow,api-compare,hardware-smoke,hardware-regression`,
sync progress, archive the completed task, run every final gate, and push. On
failure create `CLOSURE.md`, leave the row `implemented`, preserve the blocker,
and do not synchronize unchanged progress.

## Non-claims

No pre-claim of live accuracy or promotion. This plan does not verify
profitability, arbitrary profiles or pools, other ASICs/boards, unbounded
mining, update/recovery behavior beyond the supervisor's exact-package cleanup,
or release readiness.
