# Parity work plan

- Run ID: `20260818T064430Z-STAT-003`
- Parity row: `STAT-003`
- Initial status: `implemented`
- Source commit: `135db342058fa82933ccb7aabfad9a19c59ded5b`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-stat003-scoreboard`

## Selection

The worktree and pinned reference are clean, `main` equals `origin/main`, and
the deterministic selector reports no open plan. Candidate order begins
`SELF-001`, `BAP-002`, then `STAT-003`.

`SELF-001` is dependency- and safety-blocked because no production-safe route
exercises its fan, voltage, power, ASIC, pass/fail/cancel, reboot, and recovery
effects as one hardware regression. `BAP-002` remains blocked by not-started
`BAP-001` UART/request/subscription ownership; its remaining live-accessory
path requires separately authorized external UART and electrical attachment.
`STAT-003` is first actionable.

The scoreboard core, current-generation nonce receipt, transactional indexed-
NVS owner, boot load, and read-only API are implemented. `STAT-001` attempt-019
now proves the conservative production mining and safe-stop path that was
previously unavailable, so a purpose-bound scoreboard verifier can reuse that
supervisor without reopening hashrate parity. The remaining gap is a fresh
scoreboard-specific causal chain from an NVS seed without scoreboard keys,
through a real nonce insertion, live API/UI visibility, and post-restart boot
durability.

Loaded guidance materially includes `AGENTS.md`, `AGENTS.bright-builds.md`,
`standards-overrides.md`, the managed verification/testing/Rust standards, the
active task/checklist, and bounded whole lesson blocks for safety,
authorization, privacy, retry discipline, units, real process boundaries, and
transitive evaluator identity. The lesson ledger is above budget; all headings
were inventoried, nine omitted blocks were disclosed, and the August 3 audit
baseline means no new audit trigger is due.

## Scope and non-scope

Advance only `STAT-003`. Add the missing independent Rust/TypeScript evidence
contract and CLI for one private-first `bitaxe-scoreboard-evidence-v1`
attempt. Bind the immutable plan/task, exact current/pushed package and pinned
reference, every production/API/UI/evaluator source in a versioned inventory,
the detector, fresh protected roots, the sealed mining-campaign result/network/
serial digests, and one independently validated aggregate projection.

Complete the operator surface with a `/scoreboard` SPA route that fetches only
`/api/system/scoreboard`, renders the exact six server-owned fields with
text-only DOM writes, supports manual refresh, and remains responsive. Add
pure/static regressions for route admission, bounded entry validation,
descending display order, error handling, and no secret/storage/HTML injection
surface. Live evidence must confirm the production SPA route is served and the
same-origin API contains a nonempty stable bounded scoreboard.

The verifier may execute exactly one fresh board-205 `live-share` mining
campaign at 400 MHz, 1,100 mV ASIC core, 100% fan, and 600 accumulated active
seconds, using only ignored local Wi-Fi/pool inputs. The campaign's normal NVS
seed replaces prior device settings and scoreboard records with owner Wi-Fi,
owner pool, package defaults, and no scoreboard keys before mining; this
causal reset is intentional and the prior scoreboard is not restored. The
final device state retains the new bounded scoreboard plus those owner inputs
and safe stopped defaults.

After the campaign confirms terminal safe stop and cleanup, the verifier may
open one passive receive-only USB capture, derive the same-session origin from
recurring exact-package attestation, read the private scoreboard twice, issue
one normal `/api/system/restart`, and poll only that same origin until the next
software-CPU boot. It must require exact ordinal increment, changed boot
session, unchanged package, disabled boot mining, safe public mining state,
and the identical scoreboard twice after restart.

Only mode-0600 private values beneath fresh ignored mode-0700 roots may be
written. Scoreboard entries, difficulties, jobs, extranonce values, times,
nonces, version bits, credentials, endpoints, owner/worker strings, origins,
ports, USB/network identity, exact sensors/hashrates, bodies, serial, commands,
PIDs, and traces remain private. The public projection may contain only schema
and cryptographic identity, booleans, bounded counts, closed categories, and
digests of admitted private documents.

No overclock, arbitrary control, unbounded mining, OTA, erase, raw write, fault
injection, physical power action, external UART/BAP, or pin/pad/header/probe/
jumper/solder/signal manipulation is authorized. The mining supervisor owns at
most one exact-package recovery flash after a post-flash campaign failure; no
later restart/API failure may trigger another flash or retry.

## Implementation

- [ ] Add the scoreboard SPA page, exact API client method, pure/static tests,
      and deterministic packaged assets without weakening existing UI routes.
- [ ] Add a typed Rust evidence contract/validator, TypeScript source inventory
      and private-first orchestrator, CLI/Just/Bazel wiring, generated contracts,
      and real-child privacy/failure/identity regressions.
- [ ] Run focused and mandatory software/firmware/package gates; commit, push,
      and rebuild the exact package before device access.
- [ ] Run only the frozen detector and sole conditional attempt-001 capture;
      promote only on the complete independently validated quorum.

## Authorized live commands and recovery

After the implementation source is clean, fully gated, committed, pushed,
packaged, and identity-checked, run only:

1. `test ! -e scratch/stat003-scoreboard/wrapper-001 && (umask 077; mkdir -m 700 -p scratch/stat003-scoreboard/wrapper-001 && just detect-ultra205 > scratch/stat003-scoreboard/wrapper-001/detector.stdout 2> scratch/stat003-scoreboard/wrapper-001/detector.stderr)`
2. After command 1 exits zero, admits exactly one Ultra 205, inputs are
   nonempty without being read, and child/projection remain absent:
   `test ! -e scratch/stat003-scoreboard/attempt-001 && test ! -e docs/parity/evidence/stat003-scoreboard/scoreboard-projection.json && test -s wifi-credentials.json && test -s pool-credentials.json && (umask 077; just capture-scoreboard-evidence --private-root scratch/stat003-scoreboard/attempt-001 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --pool-credentials pool-credentials.json --detector-output scratch/stat003-scoreboard/wrapper-001/detector.stdout --projection docs/parity/evidence/stat003-scoreboard/scoreboard-projection.json --duration-seconds 600 --capture-timeout-seconds 1800 > scratch/stat003-scoreboard/wrapper-001/capture.stdout 2> scratch/stat003-scoreboard/wrapper-001/capture.stderr)`

The caller owns only wrapper-001 streams; the verifier exclusively creates the
absent attempt child. Starting command 2 consumes attempt-001. Never rerun it
or create attempt-002 under this plan. Preserve the earliest typed failure
through campaign recovery, passive reacquisition, restart, validation, sealing,
and cleanup. Any nonzero command, identity/safety/privacy/cleanup/recovery
failure, or missing fact withholds the projection and stops the plan.

## Verification and promotion

Focused verification covers the scoreboard owner/wire/API/UI, mining receipt,
indexed NVS, boot load, route shell, package assets, exact task/plan binding,
source/reference inventory, generated Rust/TypeScript contract, invocation and
CLI shape, real fresh child processes, NVS-seed causality, sealed campaign
digests, passive origin reacquisition, restart transition, protected modes,
earliest failure, and redaction. Then build the real firmware/package and run
`just verify-redaction` plus `just verify-reference`.

Mandatory ordered gates are `cargo fmt --all`, `cargo clippy --all-targets
--all-features -- -D warnings`, `cargo build --all-targets --all-features`,
`cargo test --all-features`, `bun scripts/bright-builds-check.ts all`, `just
test`, `just parity`, and `just parity-progress`.

Promotion requires exact board/attempt/source/reference/package/plan/detector;
fresh NVS seed without scoreboard keys; accepted conservative 600-second
campaign; qualified nonce and accepted-or-rejected submit evidence; 20/20
renewed windows; stable watchdog; no panic or mixed session; terminal joins;
safe stop and USB cleanup; a nonempty 1-20 entry scoreboard with exact wire
shape, finite positive difficulty, bounded fields, uppercase fixed-width hex,
stable descending order, and immediate repeat stability; live `/scoreboard`
SPA serving; one software-CPU restart with exact next ordinal and new session;
identical post-restart scoreboard and repeat; disabled boot mining; protected
modes; source semantics; independent validator success; and redaction.

On success create `RESULT.md`, commit evidence without changing the checklist,
transition only `STAT-003` to `verified` with
`unit,workflow,api-compare,static-route,hardware-smoke,hardware-regression`, sync
progress, archive the completed task, run every final gate, and push. On
failure create `CLOSURE.md`, leave `STAT-003` implemented, preserve the exact
closed blocker, and do not synchronize unchanged progress.

## Non-claims

No pre-claim of live scoreboard or promotion. This plan does not verify every
top-20 eviction order on hardware, wall-clock age/rank presentation, absolute
nonce difficulty calibration, profitability, arbitrary profiles/pools, other
ASICs/boards, unbounded mining, update recovery, or release readiness.
