# Parity work plan

- Run ID: `20260816T235908Z-STAT-001`
- Parity row: `STAT-001`
- Initial status: `implemented`
- Source commit: `b19a011dd6c1ea89e797864d1594ef5f05f8c7d0`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-stat001-hashrate-monitor`

## Selection

The worktree and pinned reference are clean, `main` equals `origin/main`, and
the deterministic selector reports no open plan. It orders `SELF-001`,
`BAP-002`, then `STAT-001`. `SELF-001` remains blocked because no
production-safe hardware self-test route exists. `BAP-002` remains blocked by
the unfinished `BAP-001` UART ownership and subscription lifecycle.

STAT-001 is the first actionable row. Its attempt-009 closure requires a
software-only diagnosis after attempts 008 and 009 both reached trusted
runtime identity and 14 of 20 continuity windows, then sealed
`watchdog_unresponsive` / `watchdog_feed_stale`. An earlier plan checkpoint
for this diagnosis used a noncanonical metadata label; its dependent commits
were reverted without editing the immutable file, the full Rust commit gates
passed, and the selector is healthy again at this source commit.

## Root cause and objective

The exact-package firmware supplies compiled
`CONFIG_ESP_TASK_WDT_TIMEOUT_S` to the pure runtime-health evaluator, which
reports `feed_fresh` through 5,000 ms and `feed_stale` only after it. The
production campaign accepts that reason but then independently converts every
feed age above 2,000 ms into `watchdog_feed_stale`. That contradictory host
policy can reject a truthful producer observation between 2,001 and 5,000 ms
and explains the repeated sealed signature.

Remove only the campaign's duplicate numeric freshness policy. Keep the
exact-package runtime-health verdict as the sole freshness classification and
retain independent checks for supervisor health, reason vocabulary,
participation consistency, feed sequence and age presence, checkpoint
presence, HTTP/WebSocket checkpoint and feed advancement, earliest-failure
precedence, and value-free evidence.

## Implementation

- [ ] Add a red production-campaign regression proving `feed_fresh` at 2,001
      ms is currently misclassified.
- [ ] Remove the duplicate 2,000-ms campaign threshold and prove producer-
      classified `feed_fresh` at 2,001 and 5,000 ms is accepted while
      `feed_stale` at 5,001 ms remains rejected.
- [ ] Preserve missing/inconsistent/unknown reason checks, both transport-
      window sequence checks, earliest-failure precedence, and value-free
      serialization; guard against reintroducing a numeric consumer policy.
- [ ] Run every focused and mandatory gate, commit and push the exact
      correction, and close without checklist transition or hardware access.

## Authorization and evidence policy

This is a software-only plan. It authorizes local source, fixtures, tests,
builds, documentation, and ordinary git operations. It does not authorize
protected attempt or credential access, detector/device/network runtime,
flash, reset, monitor, mining, controls, OTA, erase, raw write, fault injection,
power manipulation, direct UART, or electrical pin/pad/header/GPIO/probe/
jumper/solder/signal work. Attempt 009 remains consumed; this plan does not
authorize attempt 010 or a public projection.

Tests use synthetic closed values only. No private value is required. The
checklist and progress history remain unchanged because software correction
cannot establish the live twenty-window quorum.

## Acceptance and verification

Acceptance requires a red failure at the real campaign sample boundary,
correct green behavior at 2,001/5,000/5,001 ms, unchanged closed reason and
participation failures, unchanged per-window HTTP/WebSocket advancement,
unchanged precedence and redaction, no firmware/schema/hardware behavior
change, and an immutable plan hash.

Run focused campaign-network and runtime-health tests, source ownership,
`just verify-redaction`, `just verify-reference`, `just package`, and this
mandatory ordered sequence:

1. `cargo fmt --all`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo build --all-targets --all-features`
4. `cargo test --all-features`
5. `bun scripts/bright-builds-check.ts all`
6. `just test`
7. `just parity`
8. `just parity-progress`

After diff review, commit and push the implementation. Create `CLOSURE.md`,
not `RESULT.md`: final status remains `implemented`, outcome is `blocked`, and
verification claimed is `no`. Do not change STAT-001 checklist fields,
deterministic progress history, or README status. A future attempt 010 needs a
separate complete immutable hardware plan after this correction is pushed.

## Lessons and non-claims

The global lesson file and task-relevant whole repository lesson blocks were
loaded under the deterministic size policy; unrelated GSD and USB-observer
history blocks were omitted, and no audit trigger is due. This plan applies
real-process boundary, earliest-failure, compiled-runtime, retry-new-
information, evaluator-identity, preflight-exit, and legacy-unit guardrails.

This plan does not verify STAT-001, hardware watchdog responsiveness, live
BM1366 accuracy, twenty-window continuity, the full 600-second quorum,
HTTP/WebSocket hashrate coherence, terminal zero, electrical accuracy,
profitability, arbitrary profiles or pools, other boards or ASICs, updates,
recovery, or release readiness.
