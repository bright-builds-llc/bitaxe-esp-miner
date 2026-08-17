# Parity work closure

- Parity row: `STAT-001`
- Final status: `implemented`
- Outcome: `blocked`
- Verification claimed: `no`
- Plan SHA-256: `fe68aaa337d0ed759b7dcd68c94b33dfe6a8001a7e9c1fb2b9dc352c2b9db106`
- Active task: `task-parity-stat001-hashrate-monitor`

## Closure reason

Pushed source `f5a8fd144ada04503cd5fa49c7dcc175a112aaf6` fixes the
attempt-013 mixed-snapshot race at its firmware concurrency boundary. One
typed store now owns feed history, owner phase, and wait deadline. Every
producer publication advances a 32-bit single-writer sequence from even to odd
and back; the reader accepts only a matching even sequence around all copied
facts. Eight bounded retries fail closed to unavailable feed/phase and
non-waiting state rather than returning a mixed instant.

The exact old-feed/new-wait regression injects a new successful feed and armed
wait after the reader copies the old feed history. The first read is rejected
and the retry returns the new feed with the new wait. Stable reads preserve
both history entries and wait state; repeated publication races and poisoned
history return the closed default. Runtime health consumes only this combined
snapshot before sampling evaluation time. Task priority 5, one-second cadence,
compiled watchdog timeout, low-32-bit wrap-aware deadline, v14/v8 wire facts,
failure precedence, and the 18-source evaluator inventory remain intact.

Focused runtime-health, watchdog-store, source-ownership, phase-34, and
automation tests pass. The real ESP32-S3 firmware and canonical package build;
semantic redaction and pinned-reference checks pass; and the complete ordered
Cargo, Bright Builds, 47-target Bazel, parity, and progress gates pass. The
only transient was the known macOS `os error 35` at `just parity`, whose single
isolated retry passed with `validation_errors: none`. No hardware, credentials,
protected attempt, device/network runtime, public projection, checklist,
progress-history, or README change occurred, so STAT-001 remains `implemented`.

## Next safe action

A fresh immutable STAT-001 plan may bind exact pushed source `f5a8fd14` to a
newly built board-205 package and authorize at most one detector-gated
attempt-014. It must restate the complete conservative-profile, unit, safety,
privacy, evidence, recovery, cleanup, retry, stop, and promotion contract. Do
not reuse or retry attempt-013. Promotion still requires the independent full
twenty-window hashrate/watchdog/work-renewal/terminal-zero quorum.

## Non-claims

This closure does not verify STAT-001, authorize hardware under this plan,
prove the full live campaign, establish scheduler or timed-wait behavior on the
device, prove hashrate accuracy, complete twenty windows or 600 active seconds,
establish work renewal or terminal zero, or claim profitability, arbitrary
profiles/pools, other boards/ASICs, update/recovery, or release readiness.
