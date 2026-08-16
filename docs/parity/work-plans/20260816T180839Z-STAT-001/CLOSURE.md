# Parity work closure

- Parity row: `STAT-001`
- Final status: `implemented`
- Outcome: `blocked`
- Verification claimed: `no`
- Plan SHA-256: `7c9130657a23fb7fbc5993be885652864c7cf50d79d8253b239ddfbaa3045fc0`
- Plan commit: `a8e5597628e1f5748629c476a4e5c060dc48c264`
- Implementation commit: `91ab642b4b3ee2edf8f23190fad41ca2fc5d0620`
- Active task: `task-parity-stat001-hashrate-monitor`

## Closure reason

The consumed attempt-006 failure identified a source-owned diagnostic collapse:
the campaign classifier checked coarse watchdog participation before the
runtime evaluator's closed reason. As a result, every production-shaped
non-participating cause became `watchdog_not_participating`, and later reason
branches could not be reached. Its regression constructed the inconsistent
pair `not_participating` plus `feed_fresh`, so the real boundary was absent.

Implementation commit `91ab642b4b3ee2edf8f23190fad41ca2fc5d0620`
corrects the order and preserves distinct value-free failures for every
evaluator reason, missing or unknown reason, and inconsistent fresh-feed
participation. Campaign-result v12 and network-continuity v6 seal the changed
vocabulary, and all Rust and TypeScript producers, consumers, fixtures, and
gates require the new schemas. Focused tests, ordered Cargo gates, Bright
Builds, canonical tests, the firmware package, parity/progress invariance,
redaction, reference integrity, selector admission, immutable-plan validation,
and diff review passed before the implementation was committed and pushed.

This completes the authorized software correction, but it cannot promote
STAT-001. No hardware was accessed, no attempt-007 was allocated, no public
projection was created, and the checklist and progress history remain exact.
The row remains `implemented` with `unit,workflow` evidence until a future
authorized hardware attempt satisfies the complete campaign quorum.

## Next safe action

Run the clean synchronized selector in a new invocation. If STAT-001 is again
the first actionable row, create a separate immutable hardware plan for a
single attempt-007 that admits only the pushed v12/v6 implementation and the
new reason-specific discriminator boundary. That plan must independently
define detection, privacy, retry, recovery, cleanup, stop, and promotion gates;
never reuse attempt-006 or retry without this verified source change.

## Non-claims

This closure does not verify STAT-001, task-watchdog responsiveness on hardware,
twenty-window continuity, full 600-second hashrate accuracy, work renewal,
electrical accuracy, profitability, extended soak, arbitrary pools or
profiles, other boards or ASICs, update/recovery behavior, or release
readiness. Deterministic source and schema tests prove diagnostic fidelity, not
live mining or hardware parity.
