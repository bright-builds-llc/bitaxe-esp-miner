# Parity work plan

- Run ID: `20260820T150151Z-STAT-003`
- Parity row: `STAT-003`
- Initial status: `implemented`
- Source commit: `e263fd8562bce19be6bf2b6875e8677b01fa8175`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-stat003-scoreboard`

## Selection

The worktree and pinned reference are clean, `main` equals `origin/main`, and
the deterministic selector reports no open plan. Candidate order is
`ASIC-009`, `ASIC-010`, `SELF-001`, `BAP-002`, `STAT-003`, then `BAP-001`.

`ASIC-009` and `ASIC-010` have complete pure protocol cores but remain blocked
without supported BM1368/BM1397 boards, firmware adapters, and their own safe
hardware evidence. `SELF-001` still lacks a production-safe complete hardware
self-test route and compatible pass/fail/cancel environment. `BAP-002` remains
blocked by not-started `BAP-001`, external accessory UART hardware, and the
fresh direct-UART authority gate. `STAT-003` is now first actionable because
the user explicitly authorizes one protected owner-pool readiness transaction
and, only after objective recovery, one fresh mining/share-submitting scoreboard
attempt using the existing ignored pool credentials. `BAP-001` remains outside
that authority and dependency boundary.

Attempt-004 stopped at distinct `network_unavailable` after 229,579 active ms,
8/20 windows, no qualified candidate, and no submit response. Its closure
forbids an unchanged retry and requires an objective repo-owned signal that the
protected owner pool/network path is available again. This plan supplies that
signal through three consecutive bounded Stratum V1 configure/subscribe/
authorize sessions before any detector, device, flash, mining, or attempt-005
effect. A failed probe closes this plan without hardware. A ready probe is the
only condition that authorizes the conditional fresh ordinal.

Material guidance includes `AGENTS.md`, `AGENTS.bright-builds.md`, the empty
effective overrides, architecture, code-shape, verification, testing, and Rust
standards, the active task/checklist, and priority lessons for credential
privacy, exclusive evidence-root ownership, earliest failure, diagnostic
completeness, retry discipline, qualified transports, and agent-runtime timing.
Active lesson inputs total 31,758 bytes and exceed both deterministic startup
limits; all headings, the complete global file, and complete priority repository
blocks were loaded. Non-priority repository blocks were omitted under policy.
The existing August audit baseline remains current and no new audit trigger is
due.

## Scope and non-scope

Advance only `STAT-003`. Add a repo-owned host tool at
`tools/pool-readiness` that:

- exclusively creates one absent mode-0700 private root and mode-0600 result;
- reads the existing ignored four-field pool credential file only in memory;
- never renders, persists outside the private root, hashes into public output,
  or summarizes the endpoint, port, owner/worker, password, or raw traffic;
- runs exactly three sequential, independently bounded Stratum V1 sessions,
  each resolving and connecting to the configured target, sending configure and
  subscribe, requiring their valid responses, sending authorize, requiring a
  successful matching response, and closing without `mining.submit`;
- admits at most 64 KiB and 256 newline-delimited server messages per sample;
  classifies only closed credential, resolution, connection, transport,
  timeout, protocol, subscribe, or authorize outcomes; and
- reports `ready` only when all three consecutive samples pass, binding the
  clean full source commit, pinned reference, attempt ordinal 5, bounded
  timeouts, sample counts, `shares_submitted=false`, and redaction booleans.

Use the existing typed Stratum serializer/parser rather than hand-maintaining a
second protocol shape. Add real loopback-process tests for success, rejection,
malformed and oversized input, timeout, no-secret diagnostics, mode ownership,
and projection withholding. The readiness command itself may not mine, submit,
touch the device, discover unrelated network targets, retry a failed sample set,
or create public evidence.

Rotate the existing scoreboard evidence contract from consumed attempt 4 to
fresh attempt 5 only. Preserve its full 31-path source identity, package,
detector, campaign, API, SPA, restart, persistence, privacy, safe-stop,
recovery, and independent-validation quorum. The user authorizes the existing
conservative 400 MHz, 1,100 mV core, 100% fan, 600-active-second campaign to
send work and submit qualifying shares using the owner-supplied protected pool
configuration after readiness passes.

No arbitrary or overclocked profile, unbounded mining, external or alternate
pool, TLS, Stratum V2, OTA, erase, raw write, fault injection, physical power
action, external UART/BAP, pin/pad/header/probe/jumper/solder/signal work, or
second attempt is authorized.

## Implementation

- [ ] Add the closed, secret-safe three-sample pool-readiness tool, real
      loopback tests, Bazel/Cargo/Just wiring, and source/mode/privacy guards.
- [ ] Rotate only the scoreboard attempt ordinal, private paths, plan/task
      binding, fixtures, generated contracts, runfiles, and validators from 4
      to 5 without weakening any existing evidence criterion.
- [ ] Pass all focused/full gates, commit and push the exact implementation,
      and require a clean source/reference before the readiness effect.
- [ ] Run the sole readiness command. Only on `ready`, package exact HEAD, run
      one detector, then run the sole attempt-005 mining/share capture.
- [ ] Promote only on the complete independently validated scoreboard quorum;
      otherwise preserve `implemented`, earliest failure, safe stop, cleanup,
      evidence withholding, and no attempt-006.

## Authorized commands, recovery, and stop conditions

After the plan/task checkpoint and implementation are verified, committed,
pushed, and clean, run only this sequence:

1. `test ! -e scratch/stat003-scoreboard/readiness-wrapper-005 && test ! -e scratch/stat003-scoreboard/readiness-005 && (umask 077; mkdir -m 700 -p scratch/stat003-scoreboard/readiness-wrapper-005 && just check-pool-readiness --private-root scratch/stat003-scoreboard/readiness-005 --pool-credentials pool-credentials.json --attempt-ordinal 5 --samples 3 --sample-timeout-seconds 15 --sample-delay-seconds 2 > scratch/stat003-scoreboard/readiness-wrapper-005/readiness.stdout 2> scratch/stat003-scoreboard/readiness-wrapper-005/readiness.stderr)`
2. Only after command 1 exits zero with a validated private `ready` result:
   `just package`
3. `test ! -e scratch/stat003-scoreboard/wrapper-005 && (umask 077; mkdir -m 700 -p scratch/stat003-scoreboard/wrapper-005 && just detect-ultra205 > scratch/stat003-scoreboard/wrapper-005/detector.stdout 2> scratch/stat003-scoreboard/wrapper-005/detector.stderr)`
4. Only after exact detector admission, nonempty ignored inputs, clean package
   identity, and absent attempt/projection:
   `test ! -e scratch/stat003-scoreboard/attempt-005 && test ! -e docs/parity/evidence/stat003-scoreboard/scoreboard-projection.json && test -s wifi-credentials.json && test -s pool-credentials.json && (umask 077; just capture-scoreboard-evidence --private-root scratch/stat003-scoreboard/attempt-005 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --pool-credentials pool-credentials.json --detector-output scratch/stat003-scoreboard/wrapper-005/detector.stdout --projection docs/parity/evidence/stat003-scoreboard/scoreboard-projection.json --duration-seconds 600 --capture-timeout-seconds 1800 > scratch/stat003-scoreboard/wrapper-005/capture.stdout 2> scratch/stat003-scoreboard/wrapper-005/capture.stderr)`

The readiness tool owns only its absent child; callers own separate wrapper
streams. The scoreboard supervisor exclusively creates attempt-005. Any
readiness nonzero exit, non-ready private result, mode/privacy failure, dirty or
mismatched source, detector failure, package mismatch, missing input, or absent
recovery/evidence prerequisite stops before the next effect.

Starting command 4 consumes attempt-005. Preserve earliest failure through
campaign, API/SPA/restart stages, recovery, sealing, validation, and cleanup.
The existing recovery may issue its bounded safe pause and use one exact-package
factory recovery flash only after a post-flash campaign failure; later API or
restart failures may not flash or retry. Any incomplete campaign/evidence/
privacy/cleanup fact withholds the public projection and ends without
attempt-006.

## Verification and promotion

Focused verification covers credential shape without values, exact three-
sample session order, request IDs, configure/subscribe/authorize responses,
bounded DNS/connect/read/write and line admission, early close, malformed/
oversized/timeout/auth rejection, no submit, private modes, safe debug/errors,
clean-source binding, attempt-005 task/plan/path/ordinal/runfile contracts, real
child processes, generated types, all scoreboard stopped-state cases, 31-path
source identity, and redaction.

Run the relevant pool-readiness Cargo/Bazel tests, complete scoreboard
automation and Rust validator tests, `just verify-redaction`, `just
verify-reference`, and the real package before effects. Mandatory ordered gates
are `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D
warnings`, `cargo build --all-targets --all-features`, `cargo test
--all-features`, `bun scripts/bright-builds-check.ts all`, `just test`, `just
parity`, and `just parity-progress`.

Promotion requires the private readiness result to bind three consecutive
successful configure/subscribe/authorize sessions at exact pushed source with
no share submission or secret output, followed by exact board/attempt/source/
reference/package/plan/detector; fresh NVS without scoreboard keys; accepted
conservative 600-second campaign; qualified nonce and accepted-or-rejected
submit evidence; 20/20 windows; stable watchdog; no panic/mixed session;
accepted v12 terminal settlement; safe stop/cleanup; stable nonempty 1-20
exact-shape scoreboard with finite positive difficulty, bounded fields,
uppercase hex, and descending order; live SPA; one exact next-ordinal software
restart with new session; disabled boot mining; identical post-restart
scoreboard/repeat; protected modes; current source semantics; independent
validation; and redaction.

On success create `RESULT.md`, commit source-bound evidence without checklist
change, transition only `STAT-003` to `verified` with
`unit,workflow,api-compare,static-route,hardware-smoke,hardware-regression`,
sync progress, archive the complete task, final-gate, and push. On any failure
create `CLOSURE.md`, leave `STAT-003` implemented, retain this task with the
exact blocker and next safe action, do not sync unchanged progress, and push
the truthful closure.

This plan does not pre-claim scoreboard parity, arbitrary pool availability,
absolute nonce-difficulty calibration, profitability, arbitrary profiles,
other ASICs/boards, unbounded mining, updates/recovery, or release readiness.
