# Parity work log

## 2026-08-04T13:30:30Z | selection and plan

- Source commit: `0bc16ca966d91ed3f87b22ed6d88534dd3c47851`.
- Actions: Continued deterministic selection, retained every earlier effectful
  blocker, inventoried all 20 numbered upstream config seeds plus the custom
  seed, and confirmed the existing hardware catalog already covers their board
  versions and ASIC families.
- Verification: Clean synchronized branch, no open plan, exact reference seed
  inventory, and current catalog/defaults ownership confirmed.
- Evidence: Immutable plan and active task record.
- Outcome: Bounded pure matrix plan ready for its planning-commit gate.
- Blocker or next safe action: Commit the plan before implementation.

## 2026-08-04T13:44:00Z | typed matrix and focused verification

- Source commit: `7fe24e10`.
- Actions: Added a typed 21-entry defaults matrix for all numbered seeds and
  the explicit custom override, carried exact source-path provenance, and bound
  the public API to a pinned-reference golden fixture and catalog cross-checks.
- Verification: `cargo fmt --all`, focused strict Clippy, 51 `bitaxe-config`
  Cargo tests, the Bazel `bitaxe-config` target, and diff checks passed.
- Evidence: Exact matrix order and field equality, 20 selectable numbered
  seeds, one non-selectable custom seed, the custom pool-port exception, and
  catalog family, ASIC, frequency, voltage, and evidence-scope equality.
- Outcome: Pure implementation is complete; no runtime selection or hardware
  behavior changed.
- Blocker or next safe action: Commit the implementation, run the full
  mandatory gate, and transition only `CFG-006` to `implemented`.

## 2026-08-04T13:50:00Z | accepted and transitioned

- Source commit: `1583feb3966f32be73782ed22489b7a79a0dc248`.
- Actions: Completed the mandatory Rust sequence, Bright Builds, all 28 Bazel
  test targets, parity/progress, redaction, reference-cleanliness, and diff
  checks; recorded the typed result; transitioned only `CFG-006`.
- Verification: All gates passed with no parity validation errors and no
  redaction or reference-cleanliness findings.
- Evidence: Transition `20260804T135000Z-CFG-006`, `unit,golden`.
- Outcome: `CFG-006` is `implemented`; verified count remains 38/94 (40.4%).
- Blocker or next safe action: Non-205 seeded defaults and runtime behavior
  remain unverified pending separately scoped, admitted hardware evidence.
