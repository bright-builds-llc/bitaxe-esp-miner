# Parity work log

## 2026-08-11T19:51:44Z | selection and plan checkpoint

- Source commit: `a5328d1b72e06d24e9f3a151b55bd738881201da`
- Actions: Loaded active lessons within the deterministic budget, audited the
  ordered checklist candidates, and designed one normal exact-package OTA-slot
  transition using the existing same-device supervisor.
- Verification: Clean synchronized `main`, clean pinned reference, no open
  parity plan, and no existing public REL-001 attempt artifacts.
- Evidence: Immutable plan and matching active task contract.
- Outcome: `REL-001` selected as the first actionable row.
- Blocker or next safe action: Run the complete plan checkpoint gate, commit
  and push this immutable contract, then implement without editing `PLAN.md`.

## 2026-08-11T20:17:00Z | typed OTA evidence implementation

- Source commit: pending implementation commit.
- Actions: Added a bounded exact binary-body HTTP exchange, private typed OTA
  intent, `ota-live` same-device transaction with factory/`ota_0`
  postconditions, aggregate-only REL-001 contract and validator, private-first
  capture workflow, CLI/Just surfaces, and focused behavior, privacy, mode,
  failure-category, and real-child-process regressions. Existing fixture
  `reboot` and live `reboot-live` interfaces remain unchanged.
- Verification: Focused Cargo and Bazel HTTP transport, device-session,
  automation-contract, automation-orchestration, TypeScript, and real CLI
  tests pass. Bright Builds reports no findings, generated TypeScript matches
  the Rust-owned contract, and the new files remain within length limits.
- Evidence: Software-only typed capture and regressions; no public live
  evidence exists and no hardware interaction occurred.
- Outcome: Implementation is ready for the complete ordered software gate.
- Blocker or next safe action: Run every mandatory gate, review the full diff,
  commit and push the clean implementation, then build its exact package.

## 2026-08-11T20:18:00Z | complete implementation gate

- Source commit: pending implementation commit.
- Actions: Corrected the pre-effect artifact boundary so a mismatched OTA image
  is rejected before USB access or session-artifact creation, then reviewed the
  complete implementation for a single authoritative OTA transaction and no
  duplicated restart/readiness path.
- Verification: The focused real-CLI regression and device-session/automation
  Bazel tests passed. The complete ordered gate passed: `cargo fmt --all`,
  `cargo clippy --all-targets --all-features -- -D warnings`,
  `cargo build --all-targets --all-features`, `cargo test --all-features`,
  `bun scripts/bright-builds-check.ts all`, `just test`, `just parity`, and
  `just parity-progress`. Redaction, pinned-reference, whitespace, immutable
  plan, branch/upstream, and full-diff checks also passed.
- Evidence: Software-only validation of exact image binding, reader-before-one
  upload ordering, same-device recovery, typed terminal categories, aggregate
  evidence validation, private modes, redaction, and real process boundaries.
- Outcome: The clean implementation is ready to commit and push before any
  device effect.
- Blocker or next safe action: Commit and push this implementation, build its
  exact package, then consume the one detector and conditional capture attempt.

## 2026-08-11T20:20:00Z | attempt-001 pre-effect closure

- Source commit: `1e4c8a30d27e3f193d0b3f77faa157fb2b737309`
- Actions: Built and admitted the exact clean package, verified all six
  artifact digests and the pinned reference, consumed the one detector, and
  conditionally launched the one bounded capture.
- Verification: Detector admission succeeded for exactly one Ultra 205. The
  capture returned `evidence_invalid` before flash because the canonical
  partition comparator required `8K` while the exact checked-in table contains
  the ESP-IDF-equivalent token `8k`. The package digest matched, the private
  attempt root contains no captured device artifact, and no public projection
  was created. The complete ordered repository gate, redaction check, pinned
  reference check, parity report, and unchanged progress report passed on the
  closure state.
- Evidence: Closed aggregate terminal category and pre-effect absence only; no
  runtime or partition-transition evidence was accepted.
- Outcome: Attempt consumed without device mutation or REL-001 promotion.
- Blocker or next safe action: Close this immutable plan without verification.
  A fresh plan must normalize size suffix case, add a checked-in-table
  regression, pass the full gate, and allocate a fresh detector/attempt ordinal.
