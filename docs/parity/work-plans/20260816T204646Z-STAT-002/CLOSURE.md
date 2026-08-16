# Parity work closure

- Parity row: `STAT-002`
- Final status: `implemented`
- Outcome: `blocked`
- Verification claimed: `no`
- Plan SHA-256: `02e8d6c4f2d9e5b14853c880f310a0fd5cd641594bb96653bea3c8c8e9add93a`
- Active task: `task-parity-stat002-statistics-history`

## Closure reason

Detector-gated attempt-001 consumed the plan's sole hardware ordinal and failed
closed with typed category `timeout` at stage `initial_flash_monitor`. The exact
package flash effect result was valid and completed, but the orchestration
supervisor used the same 360-second lifetime as the child flash-monitor capture.
That equal boundary terminated the child before its own capture timeout could
finish cleanup and return control to the statistics observation workflow.

The failure happened before current-session origin admission and before any
`statsFrequency` PATCH. The public failure envelope therefore correctly records
no restoration, recovery flash, or recovery-origin readmission. The public
projection and candidate are absent. The wrapper and attempt roots plus every
contained artifact retain owner-only modes, the source/reference/package remain
clean and synchronized, and post-attempt checks found no device-port, flash-tool
or espflash holder.

The software evidence workflow, independent validator, generated contract,
source/reference semantics, exact-package build, focused tests, all 45 Bazel
tests, ordered Rust gates, Bright Builds checks, parity checks, redaction and
reference checks passed before the attempt. Those results do not replace the
missing live cadence/API observation.

## Next safe action

A fresh immutable `STAT-002` attempt plan may authorize attempt-002 only after a
verified source correction separates the supervisor lifetime from the child
capture boundary. The simplest correction is to let the repository flash-
monitor own its configured timeout, or to add a strictly larger bounded
supervisor cleanup grace, plus a real-child regression proving the child can
complete its timeout/cleanup path before the supervisor fires. The new plan
must bind a new clean pushed package source, preserve the same pre-mutation
failure ordering and restoration contract, and define exactly one fresh
detector/capture ordinal. Attempt-001 must never be relaunched or reused.

## Non-claims

This closure does not verify live one-second statistics cadence, history growth,
request-time immutability, exact device API labels/rows, zero-setting clearing,
live restoration, the 720-sample/full-horizon duration, telemetry accuracy,
browser charts, mining, ASIC behavior, controls, other boards, updates,
recovery parity or release readiness. It does not change the checklist row,
append progress history, or claim any hardware evidence category.
