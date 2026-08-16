# Parity work closure

- Parity row: `STAT-001`
- Final status: `implemented`
- Outcome: `blocked`
- Verification claimed: `no`
- Plan SHA-256: `a9077945201412d58f343b42eead664fdc04cde1e71191a8fabda55ffede044c`
- Active task: `task-parity-stat001-hashrate-monitor`

## Closure reason

The exact pushed package and fresh detector admitted exactly one Ultra 205,
then the sole attempt-002 capture exited with the closed wrapper category
`hardware_blocked`. Its sealed aggregate result reported
`admission_failed`, an untrusted runtime identity, no observed safe stop, and
no proven USB cleanup; no public projection was created.

Source inspection establishes the root cause without opening protected traces:
the wrapper supplied `--stage soak --profile conservative`, while campaign
admission accepts `soak` only with `upstream-default`. The conservative
600-second campaign is the `live-share` stage. Because the protected attempt
root and sealed result exist, evidence-root preflight completed; the
contradictory stage/profile pair then failed campaign admission before package
admission or campaign USB execution. The wrapper now requests and validates
`live-share` plus `conservative`, and the real-child regression rejects the old
inadmissible command shape. Attempt-002 remains consumed and was not retried.

## Next safe action

A future immutable STAT-001 plan may authorize fresh attempt-003 only after
this targeted correction is committed, pushed, fully gated, and bound to a new
exact clean package. It must repeat fresh detector admission and may not reuse
attempt-002 artifacts. The next attempt must still withhold promotion unless
the complete independent hashrate, runtime, safe-stop, cleanup, seal, mode, and
redaction quorum passes.

## Non-claims

This closure does not verify BM1366 counter accuracy on hardware, any exact or
aggregate hashrate, HTTP or WebSocket telemetry, active mining, work renewal,
rolling windows, runtime identity, safe stop, USB cleanup, or STAT-001 hardware
parity. It does not claim that attempt-002 flashed firmware, seeded NVS, or
started mining; source control flow proves the campaign stopped before those
effects.
