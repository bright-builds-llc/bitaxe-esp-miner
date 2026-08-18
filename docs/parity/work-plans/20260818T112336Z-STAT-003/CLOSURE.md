# Parity work closure

- Parity row: `STAT-003`
- Final status: `implemented`
- Outcome: `blocked`
- Verification claimed: `no`
- Plan SHA-256: `40348b78cde951fc820ecf3960294b2c1bdac51354290bf55a31b1f8cd7ea54c`
- Active task: `task-parity-stat003-scoreboard`

## Closure reason

The software-only correction completed at
`251205a57b42a1f0a1e4a59a0d90dd5b5837f5af`. Disabled boot mining now has one
closed definition: false `startMiningOnBoot` plus `miningActivity` equal to
`paused` or `safe_blocked`. Active, unknown, or enabled shapes fail.

Post-restart admission and final evidence call the same predicate. Pure tests
cover every admitted/rejected shape, and a full real-child paused restart now
continues through identical scoreboard persistence and public projection. The
existing safe-blocked workflow and restart-drift failure remain unchanged. All
mandatory gates, firmware/package builds, redaction, and reference checks pass.

No hardware or protected input was used. `STAT-003` remains `implemented`
because deterministic software evidence cannot establish the live post-restart
scoreboard persistence that attempt-003 stopped before reading.

## Next safe action

Create a fresh immutable hardware plan that rotates the scoreboard plan/task/
ordinal/paths/contracts/fixtures/runfiles to attempt-004. After full gates,
commit/push, and exact package binding, it may run one detector and one
conditional bounded scoreboard workflow under the unchanged safety, privacy,
recovery, cleanup, and promotion quorum.

If the exact restart again returns session change, ordinal +1, `software_cpu`,
false boot intent, and `paused` but the workflow blocks before post-restart
scoreboard reads, stop further retries because the targeted fix did not change
the boundary. A different failure requires its own closed diagnosis.

## Non-claims

This closure does not verify live post-restart scoreboard persistence or
promote `STAT-003`. It does not admit active, unknown, or enabled boot-mining
states, reinterpret attempt-003 evidence, or verify arbitrary profiles/pools,
other ASICs/boards, unbounded mining, OTA, recovery, or release readiness.
