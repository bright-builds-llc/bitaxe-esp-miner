# Parity work log

## 2026-08-15T20:17:54Z | attempt-007 selection

- Selection: Clean synchronized source `6117e2e7`; THR-001 is the first
  unfinished candidate and no earlier row was skipped.
- Prerequisite: The exact attempt-006 replay-origin failure is reproduced and
  fixed at `9fa31503`; focused and complete gates passed, and the software-only
  predecessor is terminal-closed.
- Action: Freeze one fresh attempt-007 contract. No binding change, package,
  detector, USB, or device effect is authorized until this immutable plan is
  verified, committed, and pushed.

## 2026-08-15T20:23:00Z | immutable-plan digest

- Plan SHA-256:
  `f461fb0e926309b280f49821d3e1a6725c547623f729bf9ea9e9a2cb1f47eac1`.
- Scope: One attempt-007 binding and, only after a separately pushed clean
  implementation and exact package, one detector and one bounded campaign.

## 2026-08-15T20:31:00Z | immutable-plan verification

- Verification: Ordered Cargo gates, Bright Builds, the real firmware build,
  all 45 Bazel tests, parity/progress, redaction, pinned-reference cleanliness,
  exact live-plan selection, and diff checks pass without hardware.
- Outcome: The attempt-007 binding work may begin only after this plan commit is
  pushed. Packaging, detection, USB, and device access remain ineligible.
