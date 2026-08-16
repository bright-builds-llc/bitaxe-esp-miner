# Parity work closure

- Parity row: `STAT-001`
- Final status: `implemented`
- Outcome: `blocked`
- Verification claimed: `no`
- Plan SHA-256: `e59517bafbaaaa6bb8417abe97ac8f94e3551b147d53b9187196d810e31bac08`
- Active task: `task-parity-stat001-hashrate-monitor`

## Closure reason

The software-only plan found and corrected the attempt-003 producer/parser
boundary defect. Firmware intentionally emits
`runtime_boot_attestation=unavailable` diagnostics, while the campaign analyzer
previously admitted any line containing the raw marker substring. Those
lookalikes therefore reached a parser that accepts only the standalone marker
token and collapsed to the coarse `malformed` status.

Pushed implementation commit `f26fff55c1513f342946f16999d8564cc761ba01`
requires a complete whitespace-delimited marker token in both the pure parser
and production serial analyzer, counts lookalikes separately, and carries a
closed value-free parse-failure discriminator with saturating per-category
counts through serial diagnostics v2 and sealed campaign result v10. Focused
regressions and every required software, build, privacy, reference, parity, and
repository gate pass. This plan intentionally authorized no hardware access and
cannot supply the remaining live BM1366 parity evidence, so STAT-001 stays
`implemented` and the plan ends blocked from verification.

## Next safe action

A fresh immutable STAT-001 plan may authorize attempt-004 only after it binds
the pushed correction to a newly built exact package and records the complete
detector admission, private evidence policy, recovery and cleanup behavior,
bounded retry policy, hardware safety limits, and exact promotion quorum.
Attempt-003 remains consumed and this closure does not authorize attempt-004.

## Non-claims

This closure does not verify runtime package identity on hardware, live BM1366
counter accuracy or topology, one-second sampling cadence, HTTP or WebSocket
hashrate coherence, rolling-window behavior, terminal-zero behavior, twenty
active observation windows, mining or pool behavior, safe-stop behavior, or
STAT-001 parity. It does not claim any behavior for non-Ultra-205 hardware.
