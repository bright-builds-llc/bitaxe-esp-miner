# Parity work closure

- Parity row: `THR-001`
- Final status: `implemented`
- Outcome: `blocked`
- Verification claimed: `no`
- Plan SHA-256: `df13633088f5471dc84d31439add7c4144732c5c4f153e9202af371c4d324187`
- Active task: `task-parity-thr001-emc2101-live-thermal`
- Implementation commit: `6f637e87557084aa9c7d34861d2c16f1e7a083b1`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`

## Closure reason

The software-only plan is complete, but it cannot verify THR-001 without a
fresh hardware-regression attempt. Attempt-005 proved the device reached both
fault and recovery while exposing two host-observation defects: the validator
required invented byte-zero marker lines instead of the canonical ESP-IDF INFO
envelope, and the post-flash reader could attach after the baseline marker.

The host now extracts only numeric-uptime INFO records with the exact
`bitaxe_firmware` tag and requires one contiguous baseline/fault/recovery
triplet. An admitted thermal-stimulus package requests the existing bounded
retained-log replay, whose strict allowlist now includes only complete,
redaction-safe thermal-stimulus state records. Ordinary packages do not request
this replay.

Production-shaped tests first reproduced the exact `evidence_invalid` failure.
They now pass canonical, late-attachment, duplicate-prefix, and real-child
cases while rejecting bare lines, malformed timestamps, wrong levels or tags,
missing states, wrong order, and timeout. The ESP32-S3 build, ordered Cargo
gates, Bright Builds, all 45 Bazel tests, parity/progress, redaction, reference
cleanliness, and diff checks passed. No hardware effect ran under this plan.

## Next safe action

Create and push a distinct immutable attempt-006 plan that advances every
attempt binding and preserves the existing exact-package, detector, one-shot
stimulus, restoration, cleanup, privacy, failure-precedence, and stop contract.
Only after its complete software gates pass may it build a clean exact package,
admit one Ultra 205, and run one bounded campaign. Never reuse attempt-005.

## Non-claims

This closure does not reinterpret attempt-005, publish hardware-regression
evidence, verify or promote THR-001, or authorize attempt-006. It makes no claim
about physical overheat, electrical sensor failure, calibration, mining,
hardware controls, other boards, or release readiness. The direct macOS-host
firmware test remains inapplicable because `esp-idf-sys` rejects the host
target; the canonical ESP32-S3 build and Bazel firmware tests passed instead.
