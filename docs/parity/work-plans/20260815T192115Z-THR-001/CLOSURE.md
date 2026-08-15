# Parity work closure

- Parity row: `THR-001`
- Final status: `implemented`
- Outcome: `blocked`
- Verification claimed: `no`
- Plan SHA-256: `ae1b6c9a57d5a19d8c61b3bf73ab906eed1e05849ff9b86d50d4d98c5eb27542`
- Active task: `task-parity-thr001-emc2101-live-thermal`
- Terminal category: `evidence_invalid`
- Attempt: `006` consumed

## Closure reason

The clean exact package from pushed implementation `f7011949` and one detector-
admitted Ultra 205 entered the sole attempt-006 transaction. The device-side
state machine succeeded: no stimulus abort marker exists, fault and recovery
were each observed directly once, and bounded retained replay emitted eleven
complete baseline/fault/recovery triplets. The prior projection-loss and late-
attachment defects therefore did not recur.

The host withheld evidence as `evidence_invalid` because the replay producer's
canonical ESP-IDF tag is `bitaxe_firmware::boot_evidence`, while the strict
payload parser admits only the direct producer tag `bitaxe_firmware`. Baseline
was available only through replay; fault and recovery each had one direct and
eleven replayed records. No marker used any other origin. The marker validator
therefore saw no complete triplet even though the complete retained witness was
present.

Ordinary exact-package recovery passed with a recovery flash and no secondary
failure. USB holders, process holders, protected wrapper modes, and evidence
withholding passed. No candidate or public attempt-006 projection exists.

## Next safe action

Create a software-only continuation that first reproduces the exact retained-
replay tag in the real-child late-attachment case. Admit only the closed pair
of canonical INFO origins, or preserve the root firmware target when replaying,
and prove arbitrary levels, timestamps, tags, payloads, and ordering still fail
closed. Commit and push the correction before a distinct immutable plan may
bind attempt-007. Never reuse attempt-006.

## Non-claims

This closure does not publish hardware-regression evidence, verify or promote
THR-001, or claim physical overheat, electrical sensor failure, calibration,
mining, controls, other boards, or release readiness. It does not authorize an
attempt-006 retry or attempt-007. The typed stimulus overlays are a bounded
software diagnostic, not a claim of physical fault injection.
