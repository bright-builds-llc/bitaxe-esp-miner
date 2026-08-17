# Parity work closure

- Parity row: `STAT-001`
- Final status: `implemented`
- Outcome: `blocked`
- Verification claimed: `no`
- Plan SHA-256: `c65c8436a981a871e615752f4ad4c2a607ad673889736ee7ae08ede7355469f5`
- Active task: `task-parity-stat001-hashrate-monitor`

## Closure reason

Pushed source `c274be943032db291ad7666d583c34ab9c2ff014`
corrects attempt-016's precise coherent-read contention boundary without
inferring protected facts. Owner inbox mapping, session evaluation, and effect
entry now reset the ESP Task WDT and publish the resulting observation plus
closed subphase inside one sequence-bracketed store transaction. Completion
feeds remain observation-only, and a failed or unavailable subscription still
publishes entry subphase without inventing a feed.

The coherent reader retains exactly eight attempts and every existing
fail-closed outcome, but yields the scheduler after an odd or changed sequence
instead of issuing CPU-only spin hints. A deterministic preempted-writer test
proves one handoff lets a finite odd publication complete and returns the exact
stable history. Separate regressions prove a permanently odd writer and a
sequence that changes after every history copy still consume all eight attempts
and return `retry_exhausted`; mutex poison remains `history_poisoned`.

Focused store, owner-progress, source-ownership, API, campaign, automation, and
parity tests; generated-contract verification; 18-source evaluator identity;
real ESP32-S3 firmware and canonical package; redaction/reference checks;
managed file-length checks; and the complete ordered Cargo/Bazel/parity gate
pass. No credentials, protected attempt, detector/device/network runtime,
hardware effect, public projection, checklist transition, progress-history
append, or README mutation occurred, so STAT-001 remains `implemented`.

## Next safe action

A separate immutable STAT-001 hardware plan may consider one fresh detector-
gated attempt-017 only as a progress-backed observation of this fused writer
and scheduler-aware eight-attempt reader. It must bind the pushed source to a
newly built exact package and restate the complete conservative-profile, unit,
safety, privacy, evidence, recovery, cleanup, retry, stop, and promotion
contract. Never retry or splice attempt-016. Any new precise boundary must
receive its own diagnosis and regression-backed correction before continuation.

## Non-claims

This closure does not prove the exact attempt-016 scheduler interleaving,
authorize hardware under this plan, verify STAT-001 or live hashrate accuracy,
complete twenty windows or 600 active seconds, establish work renewal or
terminal zero, or claim arbitrary profiles/pools, other boards/ASICs,
update/recovery, profitability, or release readiness.
