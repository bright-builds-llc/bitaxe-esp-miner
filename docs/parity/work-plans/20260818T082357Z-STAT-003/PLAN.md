# Parity work plan

- Run ID: `20260818T082357Z-STAT-003`
- Parity row: `STAT-003`
- Initial status: `implemented`
- Source commit: `ee81cde7a06f79f92d177fb9ae0d30950e983b91`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-stat003-scoreboard`

## Selection

The worktree and pinned reference are clean, `main` equals `origin/main`, and
the deterministic selector reports no open plan. Candidate order begins
`SELF-001`, `BAP-002`, then `STAT-003`.

`SELF-001` remains dependency- and safety-blocked without a production-safe
self-test hardware route. `BAP-002` remains blocked by not-started `BAP-001`
and its separately authorized external-UART/electrical dependency. `STAT-003`
is first actionable because its sole attempt-001 produced a precise software
terminal-settlement boundary with complete safety and mining evidence.

Attempt-001 completed 600,320 active milliseconds, 20/20 renewed windows, 204
scoreboard candidates, accepted submit, trusted identity, fresh safety, stable
watchdog, no panic/mixed reset, terminal HTTP/WebSocket/pool facts, confirmed
safe stop, and USB cleanup. It failed `terminal_state_unconfirmed`. Subsequent
allowlisted inspection refines the closure inference: the serial analyzer's
sealed final marker is authoritative state `consumed`, while the already-
returned network evidence retained the failure. The defect is therefore the
network worker finalizing from an earlier concurrent snapshot before the
coordinator hands it the analyzer's final terminal state, not a missing final
firmware marker.

Loaded guidance materially includes `AGENTS.md`, `AGENTS.bright-builds.md`,
the managed verification/testing/Rust standards, the active task/checklist,
and bounded whole lesson blocks for safety, retry discipline, earliest failure,
real process boundaries, and transitive evaluator identity. The lesson ledger
is above budget; all headings were inventoried, nine omitted blocks were
disclosed, and the August 3 audit baseline means no new audit is due.

## Scope and non-scope

Advance only `STAT-003` through a software-only terminal-settlement correction.
Do not read raw attempt artifacts or protected scoreboard/device values. Use
only the already recorded closed categories, booleans, counts, and bounded
duration plus deterministic source/tests.

Replace the observer's implicit early `break` decisions with one pure closed
terminal-settlement reducer. Before serial input is closed, complete terminal
HTTP/WebSocket quorum or deadline expiry must request capture closure rather
than finalize evidence. Only after the coordinator applies the analyzer's
terminal handoff and sets `serial_finished` may the worker accept authoritative
consumed state or fail `terminal_state_unconfirmed`. Reason-only state remains
invalid, and earlier safety/correlation/watchdog failures retain precedence.

Rotate network evidence to v12 with value-free settlement diagnostics:
settlement label, close-requested boolean, worker terminal-consumed observation,
final analyzer terminal-consumed observation, and serial-finished observation.
Bind these fields through the result, hashrate and scoreboard validators,
generated contracts, fixtures, Bazel/runfiles, and the scoreboard source
inventory. Do not weaken 20-window, safe-stop, watchdog, identity, pool,
terminal transport, privacy, or redaction gates.

This plan authorizes source, tests, deterministic local processes, firmware/
package builds, documentation, and Git only. It does not authorize credentials,
detector or protected-attempt access, USB/device/network runtime, flash, reset,
restart, monitor, mining, hardware controls, public projection, attempt-002,
external UART/BAP, pins, or electrical work. `STAT-003`, checklist, progress,
and README remain unchanged.

## Implementation

- [ ] Add a pure terminal-settlement reducer and a production-shaped red test
      for complete terminal transports followed by delayed final serial handoff.
- [ ] Make the worker request serial closure and finalize only after the
      coordinator supplies the authoritative analyzer handoff; preserve failure
      precedence and bounded termination.
- [ ] Rotate private network diagnostics to v12 and bind every consumer,
      contract, fixture, source inventory, and runfiles path.
- [ ] Run focused and mandatory gates, commit/push the correction, and close
      without hardware or parity transition.

## Verification and promotion

Focused tests must prove: terminal quorum requests close but cannot accept
before serial finish; deadline requests close but cannot fail before final
handoff; final consumed+HTTP+WebSocket accepts; final non-consumed fails; a
prior failure wins; the coordinator applies analyzer terminal before
`serial_finished`; v12 fields are closed/value-free and rotate evaluator
identity; v11 and missing/unknown fields fail consumers; real child and firmware
build paths remain green.

Mandatory ordered gates are `cargo fmt --all`, `cargo clippy --all-targets
--all-features -- -D warnings`, `cargo build --all-targets --all-features`,
`cargo test --all-features`, `bun scripts/bright-builds-check.ts all`, `just
test`, `just parity`, and `just parity-progress`, plus real firmware/package,
redaction, reference, selector, and diff checks.

Success is a clean pushed software correction with deterministic reproduction
and no hardware effects. Create `WORKLOG.md` and `CLOSURE.md`; do not request a
no-op checklist transition or synchronize unchanged progress. A future
immutable hardware plan may consider fresh attempt-002 only from that pushed
source with the full detector, safety, privacy, recovery, retry, stop, and
promotion contract.

## Non-claims

This plan does not verify live scoreboard API/UI or persistence, promote
STAT-003, authorize a retry, prove every scheduler interleaving, or claim
arbitrary profiles/pools, other hardware, unbounded mining, OTA, or release
readiness.
