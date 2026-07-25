---
status: awaiting_human_verify
trigger: "Phase 36 Plan 08 passed its exact-current software gate, package, and preflight, then its single authorized broker-owned Ultra 205 hardware command returned sealed_non_promotion with earliest failure flash_failed, secondary bounded restoration result recovery_failed, and cleanup complete. No candidate exists. Device restoration remains unresolved."
created: "2026-07-25T17:38:16Z"
updated: "2026-07-25T18:52:00Z"
---

## Current Focus

hypothesis: "Confirmed: an unsupported and conflicting flash evidence-option pair deterministically caused both broker failures before device access."
test: "Software verification is complete. The original real-device workflow can be verified only in a separately authorized future hardware plan; this debug session must not rerun Plan 36-08."
expecting: "A future authorized exact flash no longer fails at CLI validation, while all predeclared hardware safety, recovery, and evidence gates remain in force."
next_action: "Await separately authorized human/hardware verification; do not rerun the sealed Plan 36-08 command."

## Symptoms

expected: "The exact admitted board-205 package flashes successfully; if a partial-flash boundary occurs, the predeclared same-image bounded restoration restores bootability, while cleanup completes."
actual: "The sole authorized hardware invocation reached flash_failed; the predeclared same-image recovery then returned recovery_failed; cleanup completed; the attempt sealed non-promotional; Plan 36-07 remains blocked."
errors: "Primary flash_failed; secondary recovery_failed; disposition sealed_non_promotion."
reproduction: "Observed during the one and only Plan 36-08 Task 2 broker-owned hardware command. That command must not be rerun; this session is offline-only."
started: "First observed 2026-07-25 after exact source 3e52aa9080401336df55903a91f1c0f94d50bfd9 passed the graph and software gate; outcome committed at 96825555a5fd00b19886813a78fe4bd7def3505c and summarized/tracked at 7f11cd409c96ce5385238f56297fbb14dd732c0d."

## Eliminated

None yet.

## Evidence

- timestamp: "2026-07-25T17:38:16Z"
  checked: "Mandated instructions, active lessons, Plan 36-08 plan/summary, hardware-attempt policy, evidence policy, and debug knowledge base."
  found: "The investigation is offline-only; the sole hardware command and recovery are non-repeatable; the sealed private root is immutable; shareable reporting is limited to closed categories, booleans, bounded counts/durations, and public digests."
  implication: "Only deterministic repository source/tests and explicit fixed-handle sealed inputs may discriminate causes. Hardware or private-value speculation cannot justify a fix."

- timestamp: "2026-07-25T17:41:00Z"
  checked: "Knowledge-base keyword overlap for flash_failed, recovery_failed, sealed_non_promotion, flash, and recovery."
  found: "No entry has the required two-keyword overlap with this failure."
  implication: "There is no known-pattern diagnosis to test first; the investigation must trace the current production boundary."

- timestamp: "2026-07-25T17:43:00Z"
  checked: "Tracked literal occurrences of flash_failed, recovery_failed, and same-image recovery."
  found: "The closed strings appear in the Phase 36 process test and plan/summary, but not as production literals; the working tree contains only this new debug session."
  implication: "Production likely serializes typed enum variants. The next search must follow symbols and complete call paths rather than infer behavior from the summary."

- timestamp: "2026-07-25T17:48:00Z"
  checked: "Complete Phase 36 effect adapter, broker contract, transaction state machine, and deployed process boundary."
  found: "Exact flash and typed recovery both invoke the same repository flash command with the admitted manifest and factory image; only exact flash adds the credential input. The transaction maps every nonzero exact-flash result to flash_failed and every nonzero recovery result to recovery_failed. The process boundary nulls adapter stderr and retains no exit code, signal, timeout flag, or typed flash subcategory in the seal."
  implication: "The public seal proves ordering but not why either flash invocation failed. A permitted detailed artifact must contain an independent discriminator before a repository-owned root cause can be confirmed."

- timestamp: "2026-07-25T17:48:00Z"
  checked: "Recovery authorization branch in the transaction state machine."
  found: "TypedRecovery is invoked after every first failure, not only a typed partial-flash failure."
  implication: "This is a contract defect relative to the plan, but it does not by itself explain the observed primary flash failure. It is a separate candidate requiring regression evidence and must not be conflated with the incident root cause."

- timestamp: "2026-07-25T17:53:00Z"
  checked: "Deployed effect-adapter flash argument shape against tracked flash-tool option declarations and validation references."
  found: "The adapter renders both exact flash and typed recovery as the flash subcommand with evidence-mode dual and redact-evidence. The flash source declares those flags conflicting, and its validation text states dual evidence is supported only by flash-monitor."
  implication: "A deterministic pre-device CLI rejection is now the leading falsifiable repository-owned hypothesis for both observed categories."

- timestamp: "2026-07-25T18:00:00Z"
  checked: "Existing flash-parser unit regression for non-flash-monitor dual mode."
  found: "The focused offline test passed and proves the flash subcommand rejects dual evidence mode before environment or device execution."
  implication: "Combined with the adapter's exact argument construction and the broker's operation-to-category normalization, one repository-owned mechanism deterministically explains both flash_failed and recovery_failed."

- timestamp: "2026-07-25T18:04:00Z"
  checked: "Explicit sealed Plan 36-08 files resolved only through the fixed handle."
  found: "The seal is sealed_non_promotion with first flash_failed and secondary recovery_failed; the 20-record ledger contains exactly those two failed operations and completed cleanup; neither known flash-stage evidence directory exists; the candidate is absent."
  implication: "Absence of both stage directories is consistent with rejection before flash environment/evidence setup and contradicts a later monitor/capture failure. No sealed discriminator conflicts with the confirmed CLI-validation mechanism."

- timestamp: "2026-07-25T18:10:00Z"
  checked: "Complete Phase 36 deployed-process regression."
  found: "The test verifies deployed executables exist but drives hardware mode only through an intentionally failing detector, so the deployed effect adapter and real flash parser never meet. It also expects recovery after detector failure."
  implication: "The regression gap allowed both the invalid flash option shape and blind recovery after non-flash failures. New coverage must cross the adapter/parser boundary and assert recovery is not invoked without a partial-flash discriminator."

- timestamp: "2026-07-25T18:27:00Z"
  checked: "Focused corrected flash-parser regression and deployed Phase 36 process regression."
  found: "The real parser regression passed. The deployed process regression failed before full verification; its stable failure label has not yet been extracted."
  implication: "The parser-side repair is proven, but the OS-boundary fixture or adapter behavior still needs one-variable diagnosis before the fix can be accepted."

- timestamp: "2026-07-25T18:32:00Z"
  checked: "Bazel test log shape and the process test's ordered preflight."
  found: "The test log contains only the Bazel harness header and separator, with no FAIL/category output. The first supervisor preflight in the test requires an entirely clean source tree, while this repair is intentionally uncommitted."
  implication: "The no-cache process test cannot reach the new adapter regression until the source/test repair is committed. This is a workflow gate, not evidence against the fix."

- timestamp: "2026-07-25T18:39:00Z"
  checked: "Shell syntax plus the required Rust pre-commit sequence in order."
  found: "bash syntax validation, cargo fmt --all, cargo clippy --all-targets --all-features with warnings denied, cargo build --all-targets --all-features, and cargo test --all-features all exited successfully."
  implication: "The implementation and Rust regression are clean enough to create the local atomic commit required by the deployed process test's exact clean-tree preflight."

- timestamp: "2026-07-25T18:44:00Z"
  checked: "Post-commit clean-tree focused regressions."
  found: "The real flash-parser regression passed again. The no-cache deployed Phase 36 process regression still exited with code 2 before producing its stable success message."
  implication: "Source-tree dirtiness is eliminated. The new adapter fixture must now be isolated one operation at a time."

- timestamp: "2026-07-25T18:49:00Z"
  checked: "Current-package no-cache deployed Phase 36 process regression with stable failure labels."
  found: "The test reached its final planning-graph check and failed only with `incomplete Phase 36 graph is not the exact wave-ordered contract`; neither exact-flash nor typed-recovery adapter assertion failed."
  implication: "The corrected deployed adapter exercised both fake flash boundaries successfully. Remaining verification is blocked by an orthogonal planning-index expectation that must be compared with current tracked state."

- timestamp: "2026-07-25T18:50:00Z"
  checked: "Public Phase 36 incomplete-plan ID/wave projection from the repository-owned GSD index."
  found: "The current exact incomplete graph is 36-07 at wave 8 followed by 36-04 at wave 9; completed Plan 36-08 is absent."
  implication: "The process test's inclusion of 36-08 at wave 7 is stale. Removing only that entry preserves exact planning-order validation and is independent of the flash repair."

- timestamp: "2026-07-25T18:51:00Z"
  checked: "Shell syntax and required Rust pre-commit sequence after the final test correction."
  found: "Both shell scripts passed syntax validation; cargo fmt, clippy with warnings denied, build for all targets/features, and tests for all features all exited successfully."
  implication: "The final source/test/debug changes satisfy the mandatory pre-commit checks and can be amended into the existing local atomic commit."

- timestamp: "2026-07-25T18:52:00Z"
  checked: "Exact-current package build, focused deployed-process regression, full Bazel test suite, no-cache Phase 36 broker/evidence/process regressions, parity validation, reference cleanliness, redaction verification, and diff whitespace."
  found: "The package built successfully; the focused process regression passed; all 75 Bazel tests passed; all three no-cache Phase 36 targets passed; parity reported no validation errors; reference and redaction gates passed; git diff --check passed."
  implication: "The software repair is verified across the real parser, deployed adapter boundary, package graph, adjacent Phase 36 contracts, full repository suite, and data-handling gates. Real-device restoration remains deliberately unverified without separate authorization."

## Resolution

root_cause: "scripts/phase36-hardware-effect.sh passed --evidence-mode dual and --redact-evidence to the tools/flash flash subcommand for both exact-package-flash and typed-recovery. The real parser rejects dual outside flash-monitor and declares the two flags conflicting, so both operations exited before environment or device execution and the broker normalized those exits to flash_failed and recovery_failed."
fix: "Removed the flash-monitor-only dual evidence option from Phase 36 exact flash and typed recovery while retaining redacted evidence. Added deployed-adapter OS-boundary and real flash-parser regressions for the corrected command shape. Updated the process test's stale public incomplete-plan expectation after Plan 36-08 completed."
verification: "Offline software verification passed: shell syntax; ordered Rust fmt/clippy/build/test; exact-current package; focused parser and deployed process regressions; full 75-test Bazel suite; no-cache Phase 36 broker/evidence/process regressions; parity, reference, redaction, and diff checks. Real-device restoration requires a separately authorized future hardware plan."
files_changed:
  - scripts/phase36-hardware-effect.sh
  - scripts/phase36-substantive-evidence-test.sh
  - tools/flash/src/main.rs
  - .planning/debug/phase36-plan08-flash-and-recovery-failed.md
