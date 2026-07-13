---
phase: 22-claim-ladder-and-safety-preconditions
fixed_at: 2026-07-04T20:48:12Z
review_path: .planning/phases/22-claim-ladder-and-safety-preconditions/22-REVIEW.md
iteration: 1
findings_in_scope: 3
fixed: 3
skipped: 0
status: all_fixed
---

# Phase 22: Code Review Fix Report

**Fixed at:** 2026-07-04T20:48:12Z
**Source review:** .planning/phases/22-claim-ladder-and-safety-preconditions/22-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 3
- Fixed: 3
- Skipped: 0

## Fixed Issues

### CR-01: Redacted Summary Can Leak Raw Pool Rejection Text

**Status:** fixed
**Files modified:** `crates/bitaxe-stratum/src/v1/controlled_runtime.rs`, `crates/bitaxe-stratum/src/v1/controlled_runtime/tests.rs`
**Commit:** dc367f5
**Applied fix:** Replaced debug formatting of `ControlledShareOutcome` in `redacted_summary()` with stable labels, and added regression coverage for rejected shares whose raw pool reason contains redaction sentinel strings.
**Verification:** Re-read affected sections; `bazel test //crates/bitaxe-stratum:tests` passed.

### WR-01: Submit Outcome Trusts Any Stratum Response

**Status:** fixed: requires human verification
**Files modified:** `crates/bitaxe-stratum/src/v1/controlled_runtime.rs`, `crates/bitaxe-stratum/src/v1/controlled_runtime/tests.rs`
**Commit:** 0282c0f
**Applied fix:** Required share submit responses to match `ControlledMiningRuntimePlan::SUBMIT_REQUEST_ID` before recording accepted or rejected share outcomes, with regression coverage for authorize-response and missing-ID cases.
**Verification:** Re-read affected sections; `bazel test //crates/bitaxe-stratum:tests` passed.

### WR-02: Firmware Runtime Bypasses Typed Fan And Voltage Preconditions

**Status:** fixed: requires human verification
**Files modified:** `firmware/bitaxe/src/controlled_mining_runtime.rs`
**Commit:** b80601b
**Applied fix:** Constructed Phase 22 typed production mining preconditions before building the firmware mining gate, fail-closed missing live fan/voltage/safety inputs with stable redaction-safe blocker reasons, and preserved blocked precondition reasons in the firmware publication path.
**Verification:** Re-read affected sections; `bazel build //firmware/bitaxe:firmware` passed.

## Skipped Issues

None.

_Fixed: 2026-07-04T20:48:12Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
