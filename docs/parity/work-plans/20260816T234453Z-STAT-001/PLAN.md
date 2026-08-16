# STAT-001 campaign watchdog-policy diagnosis plan

- Parity row: `STAT-001`
- Starting status: `implemented`
- Source commit: `a63e0243e68f7cc904fc4cad889a29925cf2ec5d`
- Pinned reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-stat001-hashrate-monitor`
- Plan timestamp: `2026-08-16T23:44:53Z`

## Selection and prior evidence

The canonical selector returned no open plan. `SELF-001` remains blocked below
verified because the repository has no production-safe hardware self-test
route. `BAP-002` remains blocked on the `BAP-001` UART ownership, request, and
subscription lifecycle gap. `STAT-001` is therefore the first actionable row:
its attempt-009 closure explicitly requires a software-only diagnosis before
any new hardware ordinal.

Attempts 008 and 009 both reached trusted exact-package runtime identity and
14 of 20 continuity windows, then sealed the same
`watchdog_unresponsive` / `watchdog_feed_stale` boundary with safe stop,
cleanup, modes, seals, and projection withholding intact. Attempt 009 included
the production-owner cooperative-feed correction, so another unchanged
hardware attempt is prohibited.

Source inspection isolates a contradictory policy boundary:

- `firmware/bitaxe/src/runtime_health_adapter.rs` supplies the compiled
  `CONFIG_ESP_TASK_WDT_TIMEOUT_S` to the pure runtime-health evaluator.
- `crates/bitaxe-core/src/runtime_health.rs` reports `feed_fresh` through the
  exact configured timeout and `feed_stale` only after it.
- `tools/flash/src/campaign/network/watchdog.rs` accepts the typed
  `feed_fresh` reason but then independently reclassifies any reported feed age
  above 2,000 ms as `watchdog_feed_stale`.

The host-only 2,000-ms rule is not the device watchdog contract. It can reject
a truthful, participating, exact-package feed between 2,001 ms and the
compiled 5,000-ms boundary, explaining the repeated closed discriminator
without requiring a subscription, owner-feed, or clock failure.

## Objective

Make the campaign consume the exact-package runtime-health watchdog verdict as
the sole feed-freshness classification while retaining independent checks for
supervisor health, reason presence and vocabulary, participation consistency,
feed-sequence presence and per-window advancement, feed-age presence,
checkpoint presence and advancement, earliest-failure precedence, and
value-free evidence.

Prove at the production campaign boundary that a `feed_fresh` observation
after the obsolete 2,000-ms boundary is accepted, the exact compiled 5,000-ms
boundary remains accepted by the producer, the first stale millisecond is
still rejected through the producer's `feed_stale` reason, stagnant sequences
remain rejected per transport window, and closed evidence does not serialize
the numeric age.

## Implementation

- [ ] Remove the campaign's duplicate 2,000-ms feed-age classification while
      retaining presence, typed-reason, participation, sequence, advancement,
      checkpoint, precedence, and redaction checks.
- [ ] Replace regressions that encode the false host threshold with
      production-shaped producer/consumer contract tests covering 2,001 ms,
      5,000 ms, 5,001 ms, all closed reasons, and both transport-window
      sequence requirements.
- [ ] Add a source-ownership guard against reintroducing an independent
      campaign feed-age threshold when the exact-package producer already owns
      freshness classification.
- [ ] Run focused and mandatory verification, commit and push the exact
      correction, and record a non-promotion closure without hardware access.

## Authorization, privacy, and effects

This is a software-only plan. It authorizes local source, fixtures, tests,
builds, documentation, and ordinary git operations. It does not authorize
reading protected attempt roots, ignored credentials, USB/device detection,
device or network access, flash, reset, monitor, mining, controls, OTA, erase,
raw writes, fault injection, power manipulation, direct UART, or any electrical
pin/pad/header/GPIO/probe/jumper/solder/signal action. Attempt 009 remains
consumed, and this plan does not authorize attempt 010 or a public projection.

No private value is needed. Tests use synthetic closed values only. The
checklist and deterministic progress history must remain unchanged because
software correction cannot establish the required live twenty-window quorum.

## Evidence and acceptance

Acceptance requires:

1. A red test at the real campaign sample boundary proves that `feed_fresh`
   with age 2,001 ms is currently misclassified by the duplicate host rule.
2. The corrected campaign accepts producer-classified `feed_fresh` at 2,001
   and 5,000 ms while requiring the age and feed sequence to be present.
3. Producer-classified `feed_stale` at 5,001 ms remains the earliest closed
   `watchdog_feed_stale` result, and missing/inconsistent/unknown values retain
   their exact closed categories.
4. HTTP and WebSocket checkpoint/feed sequences must still advance inside
   every credited window.
5. Failed public evidence remains value-free, no production firmware behavior
   changes, the plan hash remains immutable, and every verification gate
   passes on a clean pushed source.

Run focused campaign-network, runtime-health, source-ownership, generated
contract, privacy, and real-boundary tests as affected. Then run
`just verify-redaction`, `just verify-reference`, `just package`, and the
mandatory ordered sequence:

1. `cargo fmt --all`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo build --all-targets --all-features`
4. `cargo test --all-features`
5. `bun scripts/bright-builds-check.ts all`
6. `just test`
7. `just parity`
8. `just parity-progress`

After diff review, commit and push the implementation as the source commit.
Create `CLOSURE.md`, not `RESULT.md`: STAT-001 stays `implemented`; its
checklist status, evidence field, notes, progress history, and README status do
not change. A future attempt 010 requires a separate immutable plan with a
complete hardware, privacy, recovery, cleanup, retry, stop, and promotion
contract and may be considered only after this correction is verified and
pushed.

## Lessons and non-claims

The complete active global lesson file and the relevant complete repository
lesson blocks were loaded under the deterministic size policy. Unloaded
repository blocks were the unrelated GSD-frontmatter and USB-observer history
blocks; the current audit baseline does not trigger a new lesson audit. This
plan applies the durable rules for real process boundaries, earliest typed
failure, compiled runtime capacity, retry-only-after-new-information,
transitive evaluator identity, preflight exit checks, and legacy unit tracing.

This plan does not verify STAT-001, hardware watchdog responsiveness, live
BM1366 counter accuracy, twenty-window continuity, the full 600-second quorum,
HTTP/WebSocket hashrate coherence, terminal zero behavior, electrical
accuracy, profitability, arbitrary profiles or pools, other boards or ASICs,
updates, recovery, or release readiness.
