# Parity work closure

- Parity row: `STAT-001`
- Final status: `implemented`
- Outcome: `blocked`
- Verification claimed: `no`
- Plan SHA-256: `d99a5cd8a40107098edf85949e025ac790a74bc3c59edb56f6ddf986d480444c`
- Active task: `task-parity-stat001-hashrate-monitor`

## Closure reason

Exact clean pushed source/package `223d10bc`, the pinned reference, focused and
full gates, and the sole protected detector passed. The single attempt-016
capture stopped after 364,314 active ms and 4/20 credited windows with sealed
v16/v10 category `hardware_blocked` / `watchdog_unresponsive` /
`watchdog_snapshot_retry_exhausted`, coherent read outcome `retry_exhausted`,
owner phase/subphase `unavailable/unavailable`, and wait state `not_waiting`.

Runtime identity and attestation were trusted; safety was fresh; terminal HTTP,
WebSocket, and pool persistence passed; safe stop was confirmed; USB cleanup
was ready; protected modes, result/network digests, independent wrapper
classification, and redaction passed. The public projection is absent.
Attempt-016 is consumed, no retry or out-of-band probe ran, and STAT-001,
checklist, progress history, and README remain unchanged.

The final aggregate `just test` encountered one unrelated macOS child-launch
timeout in the EMC2101 automation case after 382/383 automation cases and all
other Bazel targets passed. One bounded isolated rerun of the exact automation
target passed all 383 cases in 69.7 seconds; no repository change or repeated
hardware action was used.

The precise read outcome rules out history poison and an uninitialized store.
Source tracing shows that owner entry instrumentation publishes a subphase and
then a feed as adjacent sequence-bracketed writes, while the coherent reader
allows only eight immediate spin retries. Under active inbox/effect traffic,
the reader can observe a changing or odd sequence on every attempt and report
retry exhaustion even though the writer continues making progress. This is a
distinct, newly discriminated boundary rather than recurrence of attempt-015's
older tuple, but this hardware-only plan does not authorize a firmware fix.

## Next safe action

Do not start attempt-017. A fresh software-only immutable STAT-001 plan must
reproduce continuous owner-publication contention against the eight-attempt
reader, then apply the smallest targeted correction. The correction should
prefer one coherent subphase-plus-feed publication boundary and/or a bounded
scheduler-aware read retry that cannot hide a stuck writer, with exact
contention, stuck-publication, retry-exhaustion, poison, and value-free evidence
regressions. It must pass real firmware/package and every mandatory gate before
a separate hardware plan may consider attempt-017.

## Non-claims

This closure does not prove which owner effect was active, authorize another
hardware attempt, prove the scheduler or transport healthy, verify STAT-001 or
live hashrate accuracy, complete twenty windows or 600 active seconds,
establish work renewal or terminal zero, or claim arbitrary profiles/pools,
other boards/ASICs, update/recovery, profitability, or release readiness.
