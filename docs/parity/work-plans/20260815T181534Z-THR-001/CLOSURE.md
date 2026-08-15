# Parity work closure

- Parity row: `THR-001`
- Final status: `implemented`
- Outcome: `blocked`
- Verification claimed: `no`
- Plan SHA-256: `2351951778835e6b27f4b61a0706128650a05d4b1ac9ea087cb98f9d014eb98c`
- Active task: `task-parity-thr001-emc2101-live-thermal`
- Implementation commit: `4fdd17db71c448d916eb866d58d0384c2f7a21b1`

## Closure reason

The exact production-order regression reproduced attempt-004's
`FaultProjectionMissing` in 0.00 seconds. The stimulus proved the first
reducer-published `thermal_reading_invalid` fault, but then required that fault
on every later one-second overlay. Ordinary producer policy intentionally ages
retained failed samples to stale after one second, so the redundant later check
rejected correct runtime truth.

The correction latches the first proven fault at the existing
`fault_observed` boundary, continues only when every underlying real EMC2101
read succeeds, counts exactly five invalid overlays, and retains the later
fresh-recovery proof. It removes one redundant phase and adds no public API,
test-only production branch, timeout, or evidence exception. Ordinary
fault/fresh/stale semantics are unchanged.

Focused tests, ordered Cargo gates, the real firmware build, Bright Builds, all
45 Bazel tests, parity/progress, redaction, reference cleanliness, and diff
review passed before the correction was committed and pushed.

## Next safe action

This software-only plan authorizes and claims no hardware evidence. THR-001
remains `implemented`. A separate immutable plan must bind any attempt-005
ordinal, exact package, effects, restoration, privacy, cleanup, and stop rules.

## Non-claims

This software-only closure does not claim hardware-regression evidence,
verification, physical sensor-fault behavior, mining, controls, or release
readiness. It authorizes no detector or device attempt.
