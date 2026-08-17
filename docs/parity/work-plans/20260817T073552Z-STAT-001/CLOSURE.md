# Parity work closure

- Parity row: `STAT-001`
- Final status: `implemented`
- Outcome: `blocked`
- Verification claimed: `no`
- Plan SHA-256: `7086cfa69ba14ce5f439b380eb0e9ade1021234a901b9d0f0967ce56f0ddeb10`
- Active task: `task-parity-stat001-hashrate-monitor`

## Closure reason

Pushed source `c3b0dcb997d503a4b6751dc35288bd53abcea8c5`
corrects attempt-014's real diagnostic boundary without inferring protected
facts. Runtime health now carries one closed coherent-store read outcome:
`stable`, `uninitialized`, `retry_exhausted`, or `history_poisoned`. The
firmware store returns the exact outcome; retry exhaustion and mutex poison map
to distinct fail-closed watchdog reasons instead of generic `unproved`; and the
field reaches HTTP, WebSocket, retained runtime health, campaign aggregation,
sealed evidence, and the private-first wrapper.

Source tracing proved a second defect in the v14/v8 evidence: the accumulator
latched the earliest watchdog failure but later terminal samples overwrote its
owner phase and wait state. The new `WatchdogDiagnostic` tuple binds read
outcome, owner phase, wait state, and failure together. The attempt-014-shaped
regression first reproduces an uninitialized/unproved failure, then supplies a
later waiting/within-deadline terminal sample, and proves the original tuple is
retained. Unknown read outcomes fail closed without republishing free text.
Private campaign result/network schemas rotate to v15/v9; the public hashrate
evidence schema, checklist, progress history, and README remain unchanged.

Focused store/core/wire/campaign/wrapper/parity regressions, generated-contract
verification, the 18-source evaluator inventory, real ESP32-S3 firmware and
canonical package, semantic redaction, pinned-reference checks, managed
file-length checks, and the complete ordered repository gate pass. The known
macOS parity `os error 35` passed on one isolated tail retry. No credentials,
protected attempt, detector/device/network runtime, hardware effect, public
projection, or checklist transition occurred, so STAT-001 remains
`implemented`.

## Next safe action

Attempt-014's actual store-read outcome is not recoverable from its older
v14/v8 evidence and must not be guessed. A fresh immutable STAT-001 plan may
consider one detector-gated attempt-015 only as a progress-backed observation
of the corrected v15/v9 diagnostic boundary. It must bind the pushed source to
a newly built exact package and restate the complete conservative-profile,
unit, safety, privacy, evidence, recovery, cleanup, retry, stop, and promotion
contract. Never retry or splice attempt-014. Any observed precise outcome must
receive its own real-boundary diagnosis and regression-backed correction before
another continuation.

## Non-claims

This closure does not identify attempt-014's hidden store-read outcome, prove
reader contention, mutex poison, scheduler behavior, subscription loss, or a
transport defect; authorize hardware under this plan; verify STAT-001 or live
hashrate accuracy; complete twenty windows or 600 active seconds; establish
work renewal or terminal zero; or claim arbitrary profiles/pools, other
boards/ASICs, update/recovery, profitability, or release readiness.
