# Parity work log

## 2026-08-15T19:21:15Z | selection and immutable attempt contract

- Selector: no open plan; THR-001 is the first unfinished candidate at
  checklist index 54. No earlier candidate was skipped.
- Inputs: clean synchronized source `756749d1`, pinned reference `c1915b0a`,
  consumed attempt-005 closure, and pushed marker/replay correction `6f637e87`.
- Scope: advance only the strict attempt bindings and run one bounded attempt-
  006 after a separate clean pushed implementation and exact-package gate.
- Safety: no detector, package, USB, serial, HTTP, device, sensor, NVS, reset,
  control, mining, OTA, erase, or hardware effect is eligible from the plan
  commit alone.

## 2026-08-15T19:25:00Z | terminal-lineage selector finding

- Signal: With the fresh open plan present, `next-item` failed because one pair
  of older THR-001 plans lacks a continuation link even though the older plan
  has a valid terminal closure. No other historical pair has this shape.
- Root cause: Reconciliation validates every historical adjacent pair whenever
  any same-row plan is open instead of beginning after the latest terminal
  closure. Editing immutable plans is prohibited.
- Planned correction: Treat the latest valid terminal closure as the lineage
  reset, retain strict linking for every later adjacent open plan, and prove
  both the production history and multiple-unclosed-plan failure cases.

## 2026-08-15T19:31:00Z | immutable-plan verification

- Plan SHA-256:
  `ae1b6c9a57d5a19d8c61b3bf73ab906eed1e05849ff9b86d50d4d98c5eb27542`.
- Verification: Ordered Cargo gates, Bright Builds, real ESP32-S3 firmware,
  all 45 Bazel tests, parity/progress, redaction, pinned reference, task shape,
  and diff checks passed without device access.
- Outcome: The selector correction and attempt-006 binding work may begin only
  after this plan/task commit is pushed. Hardware remains ineligible.
