# Parity work plan

- Run ID: `20260818T050654Z-STAT-001`
- Parity row: `STAT-001`
- Initial status: `implemented`
- Source commit: `e0a5f7b7d4044a058eb97349ef759d2ceaf4a786`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-stat001-hashrate-monitor`

## Selection

The worktree and pinned reference are clean, `main` equals `origin/main`, and
the deterministic selector reports no open plan. Candidate order begins
`SELF-001`, `BAP-002`, then `STAT-001`.

`SELF-001` is dependency- and safety-blocked because its required hardware
regression still has no production-safe route covering the self-test's fan,
voltage, power, ASIC, pass/fail/cancel, reboot, and recovery effects. `BAP-002`
is blocked by not-started `BAP-001` UART/request/subscription ownership; its
remaining live-accessory route would require separately authorized external
UART and electrical attachment. `STAT-001` is first actionable.

Audited attempt-018 failed after 273,286 active milliseconds and 9/20 windows
with a mixed runtime session whose first closed reset category was `panic`.
Pushed source `0abd10add7187f0cb968dc73355c1aa41fed23a9` now distinguishes seven
ESP-IDF/Rust panic signatures, twelve closed task families, recognized-line
count, and an explicit unknown fallback without retaining raw serial values.
Private diagnostics are v4, all 21 reachable evaluator sources are identity-
bound, and focused plus mandatory software gates pass. This is materially new
diagnostic information at the prior ambiguous boundary, not an ordinal-only
retry. One fresh attempt may either satisfy the full quorum or produce the
closed signature/task/count tuple needed for a targeted diagnosis.

Loaded guidance materially includes `AGENTS.md`, `AGENTS.bright-builds.md`,
`standards-overrides.md`, `standards/index.md`,
`standards/core/verification.md`, `standards/core/testing.md`,
`standards/languages/rust.md`, the active task/checklist, and bounded whole
lesson blocks for safety, authorization, evidence integrity, privacy, units,
panic capacity, and hardware retries. The lesson ledger is over budget; all
headings were inventoried, nine omitted blocks were disclosed, and the August
3 above-limit audit baseline means no new audit trigger is due.

## Scope and non-scope

Advance only `STAT-001`. Rebind the independently validated
`bitaxe-hashrate-monitor-evidence-v1` workflow from consumed attempt-018 to one
fresh attempt-019: immutable plan digest, task binding, protected roots,
attempt ordinal, fixtures, generated/build inputs, result v16, network v11,
private diagnostic v4, closed panic/watchdog/correlation diagnostics, all 21
source-inventory paths, seals, modes, units, and redaction. Do not change
production mining or hashrate behavior unless a focused software gate exposes
drift.

After every software and firmware gate passes on a clean pushed commit, one
exact board-205 package may be factory-flashed and normally reset, seeded only
from ignored local Wi-Fi and pool inputs, and run at the repo-owned conservative
profile: 400 MHz, 1,100 mV ASIC core, 100% fan, and exactly 600 accumulated
active seconds. Require fresh independent input-bus voltage 4.5-5.5 V, power at
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

- [ ] Rebind plan/task/root/ordinal/fixture surfaces from attempt-018 to
      attempt-019 and rotate the immutable plan digest.
- [ ] Preserve result v16, network v11, private diagnostic v4, the complete
      panic/watchdog/correlation tuple, exact 21-source/reference inventory,
      prior-ordinal rejection, seals, protected modes, units, and redaction.
- [ ] Run focused and mandatory software/firmware gates; commit, push, and build
      the exact package before device access.
- [ ] Run only the frozen detector and sole conditional attempt-019 capture;
      promote only on the complete independently validated quorum.

## Authorized live commands and recovery

After the exact implementation source is clean, gated, committed, pushed,
packaged, and identity-checked, run only:

1. `test ! -e scratch/stat001-hashrate-monitor/wrapper-019 && (umask 077; mkdir -m 700 -p scratch/stat001-hashrate-monitor/wrapper-019 && just detect-ultra205 > scratch/stat001-hashrate-monitor/wrapper-019/detector.stdout 2> scratch/stat001-hashrate-monitor/wrapper-019/detector.stderr)`
2. After command 1 exits zero, admits exactly one Ultra 205, inputs are
   nonempty without being read, and child/projection remain absent:
   `test ! -e scratch/stat001-hashrate-monitor/attempt-019 && test ! -e docs/parity/evidence/stat001-hashrate-monitor/hashrate-monitor-projection.json && test -s wifi-credentials.json && test -s pool-credentials.json && (umask 077; just capture-hashrate-monitor-evidence --private-root scratch/stat001-hashrate-monitor/attempt-019 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --pool-credentials pool-credentials.json --detector-output scratch/stat001-hashrate-monitor/wrapper-019/detector.stdout --projection docs/parity/evidence/stat001-hashrate-monitor/hashrate-monitor-projection.json --duration-seconds 600 --capture-timeout-seconds 1500 > scratch/stat001-hashrate-monitor/wrapper-019/capture.stdout 2> scratch/stat001-hashrate-monitor/wrapper-019/capture.stderr)`

The caller owns only wrapper-019 streams; the supervisor exclusively creates
the absent attempt child. Starting command 2 consumes attempt-019. Never retry
it, reuse attempt-018, or start attempt-020 under this plan. Preserve the
earliest typed failure and the first closed panic signature/task/count tuple
through safe stop, recovery, sealing, and cleanup. Any nonzero command,
identity/safety/privacy/cleanup/recovery failure, or missing fact withholds the
projection and stops this plan.

## Verification and promotion

Focused verification covers hashrate automation, Rust contract validation,
campaign result/network/private diagnostic schemas, panic/watchdog/correlation
diagnostics, generated contracts, real-child behavior, task/plan binding,
21-source/reference identity, seal, mode, units, redaction, and earliest
precedence; then real firmware/package builds, `just verify-redaction`, and
`just verify-reference`.

Mandatory ordered gates are `cargo fmt --all`, `cargo clippy --all-targets
--all-features -- -D warnings`, `cargo build --all-targets --all-features`,
`cargo test --all-features`, `bun scripts/bright-builds-check.ts all`, `just
test`, `just parity`, and `just parity-progress`.

Promotion requires exact board/attempt/source/reference/package/plan/detector;
one BM1366 and four domains at one-second cadence; 20 windows with work renewal;
changing coherent positive HTTP/WebSocket current rates and warm rolling
windows; bounded error; watchdog none/stable; correlation failure none; no
panic or mixed session; terminal zero; accepted or rejected submit evidence;
safe stop; cleanup; protected modes; seals; independent validator success; and
redaction. On success create `RESULT.md`, commit evidence as `SOURCE_COMMIT`,
transition only `STAT-001` to `verified` with
`unit,workflow,api-compare,hardware-smoke,hardware-regression`, sync progress,
archive the completed task, run every final gate, and push. On failure create
`CLOSURE.md`, leave the row `implemented`, preserve the closed blocker tuple,
and do not synchronize unchanged progress.

## Non-claims

No pre-claim of live accuracy or promotion. This plan does not verify
profitability, arbitrary profiles or pools, other ASICs/boards, unbounded
mining, update/recovery behavior beyond the supervisor's exact-package cleanup,
or release readiness.
