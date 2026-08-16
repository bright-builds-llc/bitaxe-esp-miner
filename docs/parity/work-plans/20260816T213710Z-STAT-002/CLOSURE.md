# Parity work closure

- Parity row: `STAT-002`
- Final status: `implemented`
- Outcome: `blocked`
- Verification claimed: `no`
- Plan SHA-256: `f4e95d79dd91526596713ee5ac90a535214bc5ce709912d5f712313975a7b509`
- Active task: `task-parity-stat002-statistics-history`

## Closure reason

The immutable plan's sole detector-gated attempt-002 consumed its ordinal and
failed closed with typed category `timeout` at `initial_flash_monitor`. The
source correction worked at the boundary it targeted: the supervisor no longer
terminated the child at the child's 360-second monitor boundary and instead
used the verified 420,000-millisecond child-plus-cleanup lifetime.

Attempt-002 exposed the next distinct boundary. The protected child-owned
record proves the exact package flash completed and the full 360-second monitor
ended with trusted runtime attestation. The enclosing supervisor lifetime,
however, begins before factory flash, optional Wi-Fi NVS seeding, USB stage
gates, monitor preparation, and monitor capture. Those pre-monitor phases plus
the 360-second capture exhausted the 420-second whole-process lifetime before
the child could return its phase-36 effect result. The public envelope therefore
correctly reports a supervisor timeout, missing effect result, and no published
projection even though the private child record separately proves completed
flash and trusted monitor capture.

The supervisor stopped before the statistics workflow admitted a current-
session origin or made any HTTP request. No `statsFrequency` PATCH occurred, so
restoration and recovery were not required. The projection and candidate are
absent. The wrapper/attempt trees and every contained file retain owner-only
modes, the exact source/reference/package remain clean and synchronized, and
post-attempt checks found no serial or flash-tool holder.

## Next safe action

A fresh immutable `STAT-002` plan may authorize attempt-003 only after a
verified source correction models the entire flash-monitor operation instead of
adding cleanup grace only to the post-flash monitor budget. The simplest robust
correction is to let the child own its 360-second monitor timeout while the
process adapter's existing bounded 900-second whole-operation lifetime governs
factory flash, NVS seed, stage gates, capture, evidence finalization, effect-
result delivery, and cleanup. An explicit derived whole-operation budget is
also acceptable only if it independently bounds every named pre-monitor,
monitor, and post-monitor phase.

Regression coverage must use a real child whose pre-monitor phase plus capture
boundary exceeds 420 seconds in scaled time, then prove it completes evidence
and effect-result delivery under the selected whole-operation owner. The new
plan must bind a new clean pushed package and exactly one fresh detector/capture
ordinal. Attempts 001 and 002 must never be relaunched or reused.

## Non-claims

This closure does not verify the device statistics API, one-second history
cadence, request-time immutability, exact live labels/rows, later producer
growth, zero-setting clearing, settings restoration, 720-sample/full-horizon
retention, telemetry accuracy, browser charts, mining, ASIC behavior, controls,
other boards, updates, recovery parity, or release readiness. Statistics
`voltage` and `current` remain millivolt and milliamp legacy wire columns, but
no live raw value or accuracy claim is made. This closure does not change the
checklist row or progress history.
