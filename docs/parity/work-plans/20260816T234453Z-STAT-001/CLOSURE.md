# Parity work closure

- Parity row: `STAT-001`
- Final status: `implemented`
- Outcome: `corrected`
- Terminal outcome: `software_correction_complete`
- Verification claimed: `no`
- Plan SHA-256: `3a82785aede895c12890cb3ee363ae97e06a05c369f53b0819c9c5b9b8664ff1`
- Plan commit: `d75d67a3c559921002a28005591c0c4e0b0a3206`
- Implementation commit: `812bcd45b284e44d7a1d5ec3bc35f2148b14b7ff`
- Active task: `task-parity-stat001-hashrate-monitor`

## Closure reason

The repeated attempt-008 and attempt-009
`watchdog_unresponsive` / `watchdog_feed_stale` signature was caused by a
contradictory host evaluator policy, not by volts-versus-millivolts, the
compiled ESP-IDF watchdog setting, or proved owner-feed staleness.

The exact-package firmware already supplies
`CONFIG_ESP_TASK_WDT_TIMEOUT_S` to the pure runtime-health evaluator. That
producer classifies a feed as `feed_fresh` through the configured 5,000-ms
boundary and `feed_stale` only after it. The campaign consumer first accepted
that typed reason, then independently rejected every numeric feed age above
2,000 ms. A normal 2,001-to-5,000-ms observation could therefore be sealed as
`watchdog_feed_stale` even while the device truthfully reported a fresh,
participating feed.

Pushed source `812bcd45b284e44d7a1d5ec3bc35f2148b14b7ff` removes only the
duplicate numeric classification. The campaign continues to require a healthy
supervisor, present and closed watchdog reason, consistent participation,
present feed sequence and age, present checkpoint sequence, per-window HTTP
and WebSocket checkpoint/feed advancement, earliest-failure preservation, and
value-free evidence. Producer-classified `feed_stale` remains the closed
failure.

The new regression first failed red at 2,001 ms with
`WatchdogFeedStale`, then passed after the correction. Focused coverage proves
2,001 ms and the exact 5,000-ms configured boundary remain fresh; 5,001 ms
remains stale; every missing, inconsistent, unknown, stagnant, precedence, and
redaction boundary remains fail-closed. A source guard prevents a new numeric
feed-age policy from being added to the campaign consumer.

The 107 campaign-network tests, 16 runtime-health tests, focused Bazel tests,
exact firmware package, Cargo format/clippy/build/tests, Bright Builds, all 46
Bazel tests, privacy, reference cleanliness, parity validation, plan
immutability, and diff review pass. The first full parity report encountered
transient host resource exhaustion after rendering; one bounded rerun passed
with `validation_errors: none`. No detector, credentials, device, network
runtime, protected evidence, public projection, or hardware effect was
accessed.

STAT-001 remains `implemented`; its checklist status, evidence field, notes,
progress history, and README status are unchanged because this software proof
does not supply the required live hardware quorum.

## Next safe action

Run the clean synchronized selector in a new `$advance-parity` invocation. A
fresh immutable STAT-001 plan may consider exactly one attempt-010 only if it
binds pushed source `812bcd45b284e44d7a1d5ec3bc35f2148b14b7ff` to a new exact
board-205 package and defines the complete detector, safety, privacy, recovery,
cleanup, retry, stop, and promotion contract. Attempts 008 and 009 remain
consumed and must not be reused. Any future attempt must still stop on the
device producer's typed stale verdict or stagnant per-window sequences; this
correction does not weaken those boundaries.

## Non-claims

This closure does not verify STAT-001, hardware task-watchdog responsiveness,
live BM1366 counter accuracy, twenty-window continuity, the complete
600-second hashrate quorum, work renewal, HTTP/WebSocket hashrate coherence,
terminal zero behavior, mining outcomes, electrical accuracy, profitability,
extended soak, arbitrary profiles or pools, other boards or ASICs, updates,
recovery, or release readiness. It does not claim attempt 010 will pass; it
proves and corrects the false campaign classification that invalidated the two
prior attempts.
