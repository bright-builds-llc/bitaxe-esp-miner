# Parity work plan

- Run ID: `20260817T114224Z-STAT-001`
- Parity row: `STAT-001`
- Initial status: `implemented`
- Source commit: `e07571ba2cb11c03e058ebc2f6621e0519cec5a8`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-stat001-hashrate-monitor`

## Selection

The worktree/reference are clean, `main` equals `origin/main`, and the selector
has no open plan. SELF-001 is blocked by the absent production-safe self-test
route and hardware regression. BAP-002 is blocked by unfinished BAP-001 UART/
request/subscription ownership and no authorized accessory path. STAT-001 is
first actionable.

Attempt-016 stopped at the sealed
`watchdog_snapshot_retry_exhausted/retry_exhausted/unavailable/unavailable/
not_waiting` tuple. Pushed `c274be943032db291ad7666d583c34ab9c2ff014`
fixes that exact boundary with one fused owner-entry subphase/feed publication
and a scheduler handoff between the unchanged eight read attempts. Finite odd
contention now recovers; stuck and continuously changing writers remain exact
retry exhaustion. All focused, firmware, package, privacy, reference, file-
length, and mandatory gates pass. Attempt-017 is progress-backed, not blind.

The active lessons exceed budget; headings were inventoried and complete
global plus safety, authorization, evidence, retry, cross-process, unit, and
watchdog-relevant blocks were loaded. Lower-priority GSD/older USB blocks were
omitted and disclosed. The audit baseline is current and no audit triggers.

## Scope and non-scope

Advance only STAT-001. Rebind workflow, Rust validator, generated contract,
task/plan admission, Bazel inputs, protected roots, and real-child fixtures
from consumed attempt-016 to fresh attempt-017. Preserve result v16/network
v10, every closed watchdog tuple field, earliest precedence, fused writer,
bounded reader, priority 5, five-second timeout, 18-source identity, projection,
seals, modes, units, and redaction. Add no production behavior change.

After every gate passes at a clean pushed commit, one exact board-205 package
may be factory-flashed/reset, seeded from ignored Wi-Fi/pool inputs, and run at
the conservative `live-share` profile: 400 MHz, 1,100 mV ASIC core, 100% fan,
600 accumulated active seconds. Require fresh independent input bus 4.5-5.5 V,
power <=15 W, ASIC temperature <75 C, and nonzero fan RPM. Join protected
serial/HTTP/WebSocket evidence, safe-stop, clean up, seal, and allow at most one
supervisor-owned exact-package recovery flash. Core mV and bus V are distinct.

Only mode-0600 private values beneath fresh ignored mode-0700 roots are allowed;
only a validated aggregate projection may publish. No overclock, arbitrary
controls, unbounded mining, OTA, erase, fault injection, physical power action,
external UART/BAP, or electrical manipulation is authorized.

## Implementation

- [ ] Rebind all ordinal/plan/root/contract/fixture surfaces to attempt-017.
- [ ] Preserve v16/v10 diagnostics, fused writer, bounded reader, prior-ordinal
      rejection, 18-source identity, seals, modes, units, and redaction.
- [ ] Pass focused/full gates, commit/push, and rebuild the exact package.
- [ ] Run only the frozen detector and sole conditional capture; promote only
      on the complete independently validated quorum.

## Authorized live commands and recovery

After the exact source is clean, gated, pushed, rebuilt, and validated, run:

1. `test ! -e scratch/stat001-hashrate-monitor/wrapper-017 && (umask 077; mkdir -m 700 -p scratch/stat001-hashrate-monitor/wrapper-017 && just detect-ultra205 > scratch/stat001-hashrate-monitor/wrapper-017/detector.stdout 2> scratch/stat001-hashrate-monitor/wrapper-017/detector.stderr)`
2. After command 1 exits zero, admits exactly one Ultra 205, inputs are nonempty
   without being read, and child/projection remain absent:
   `test ! -e scratch/stat001-hashrate-monitor/attempt-017 && test ! -e docs/parity/evidence/stat001-hashrate-monitor/hashrate-monitor-projection.json && test -s wifi-credentials.json && test -s pool-credentials.json && (umask 077; just capture-hashrate-monitor-evidence --private-root scratch/stat001-hashrate-monitor/attempt-017 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --pool-credentials pool-credentials.json --detector-output scratch/stat001-hashrate-monitor/wrapper-017/detector.stdout --projection docs/parity/evidence/stat001-hashrate-monitor/hashrate-monitor-projection.json --duration-seconds 600 --capture-timeout-seconds 1500 > scratch/stat001-hashrate-monitor/wrapper-017/capture.stdout 2> scratch/stat001-hashrate-monitor/wrapper-017/capture.stderr)`

The caller owns only wrapper-017 streams; the supervisor creates the absent
child. Starting command 2 consumes attempt-017. Never retry it, reuse 016, or
start 018. Preserve earliest failure through safe stop/recovery/sealing/cleanup.
Exact recurrence of attempt-016's tuple selects `stop_repeated_boundary`; any
distinct tuple requires separate diagnosis.

## Verification and promotion

Focused: hashrate automation, contract validator, campaign watchdog/schema,
fused-store/bounded-reader, generated contracts, real child, source/reference,
seal, mode, unit, redaction, and precedence; real firmware and package; `just
verify-redaction`; `just verify-reference`. Mandatory ordered: `cargo fmt
--all`; `cargo clippy --all-targets --all-features -- -D warnings`; `cargo
build --all-targets --all-features`; `cargo test --all-features`; `bun scripts/
bright-builds-check.ts all`; `just test`; `just parity`; `just parity-progress`.

Promotion requires exact board/attempt/source/reference/package/plan/detector;
one ASIC/four domains/one-second cadence; 20 windows and work renewal; changing
coherent positive HTTP/WebSocket rates and warm windows; bounded error;
watchdog none/stable; terminal zero; safe stop; cleanup; modes; seals;
independent validation; redaction. On success create RESULT, transition only
STAT-001 to verified with `unit,workflow,api-compare,hardware-smoke,hardware-
regression`, sync progress, archive task, final-gate, push. Otherwise withhold
projection, create CLOSURE, leave implemented, and stop.

## Non-claims

No pre-claim of watchdog success, hashrate accuracy, profitability, arbitrary
profiles/pools, other hardware, updates, recovery, or release readiness.
