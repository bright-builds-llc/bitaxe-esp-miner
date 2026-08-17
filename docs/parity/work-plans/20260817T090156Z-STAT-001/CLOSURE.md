# Parity work closure

- Parity row: `STAT-001`
- Final status: `implemented`
- Outcome: `blocked`
- Verification claimed: `no`
- Plan SHA-256: `882eb354b4bdf5732fa03d1fe2fd8f72b3529674a4b4af982338da21bdd1b35b`
- Active task: `task-parity-stat001-hashrate-monitor`

## Closure reason

Pushed source `177fffe9554944cf1c57b7e595735bb20ba6be84`
corrects attempt-015's real owner-work diagnostic boundary without inferring
protected facts. Runtime health now carries one closed, value-free owner
subphase for inbox mapping, session evaluation, and every current production
effect category. The firmware store copies subphase with phase, wait, read
outcome, and watchdog history under the existing sequence-bracketed coherent
snapshot; top-level phase and receive-wait publications clear stale subphase.

The production feedback driver now emits entry as well as completion boundaries.
The owner feeds before session evaluation and before each classified effect,
so bounded work no longer inherits nearly the entire timeout age from preceding
work. The production-shaped regression starts with a 4,999 ms inherited age,
proves the handler entry feed prevents a false stale transition, and also proves
an effect that itself exceeds 5,000 ms remains observable as stale. The effect
mapping is exhaustive over `ProductionSessionEffect`, so a new effect cannot
silently bypass classification.

The subphase reaches HTTP, WebSocket, retained runtime-health records, campaign
aggregation, sealed network/result evidence, and the private-first wrapper.
Unknown subphases fail closed without republishing free text, and the earliest
watchdog tuple now retains read outcome, phase, subphase, wait, and failure
through terminal observations. Private campaign result/network schemas rotate
to v16/v10; the public hashrate evidence schema, 18-source evaluator identity,
checklist, progress history, and README remain unchanged.

Focused store/core/wire/feedback/campaign/wrapper/parity regressions, generated
contracts, real ESP32-S3 firmware, canonical package, redaction, pinned-reference
checks, managed file-length checks, and the complete ordered Cargo/Bazel/parity
gate pass. No credentials, protected attempt, detector/device/network runtime,
hardware effect, public projection, or checklist transition occurred, so
STAT-001 remains `implemented`.

## Next safe action

Attempt-015 cannot retrospectively reveal whether its stale interval occurred
during inbox mapping, session evaluation, or one specific effect. A fresh
immutable STAT-001 plan may consider one detector-gated attempt-016 only as a
progress-backed observation of the corrected v16/v10 boundary. It must bind the
pushed source to a newly built exact package and restate the complete
conservative-profile, unit, safety, privacy, evidence, recovery, cleanup, retry,
stop, and promotion contract. Never retry or splice attempt-015. If attempt-016
identifies a genuinely blocking subphase, diagnose and correct that exact
boundary before any further continuation.

## Non-claims

This closure does not identify attempt-015's hidden owner subphase, authorize
hardware under this plan, prove scheduler or transport behavior, verify
STAT-001 or live hashrate accuracy, complete twenty windows or 600 active
seconds, establish work renewal or terminal zero, or claim arbitrary
profiles/pools, other boards/ASICs, update/recovery, profitability, or release
readiness.
