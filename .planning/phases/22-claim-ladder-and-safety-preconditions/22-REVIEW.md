---
phase: 22-claim-ladder-and-safety-preconditions
reviewed: 2026-07-04T20:42:00Z
depth: standard
files_reviewed: 19
files_reviewed_list:
  - BUILD.bazel
  - tools/parity/BUILD.bazel
  - tools/parity/src/main.rs
  - tools/parity/src/claim_ladder.rs
  - docs/parity/evidence/phase-22-claim-ladder-and-safety-preconditions/claim-ladder.md
  - docs/parity/evidence/phase-22-claim-ladder-and-safety-preconditions/safety-preconditions.md
  - docs/parity/evidence/phase-22-claim-ladder-and-safety-preconditions/blocker-reasons.md
  - docs/parity/evidence/phase-22-claim-ladder-and-safety-preconditions/redaction-review.md
  - docs/parity/evidence/phase-22-claim-ladder-and-safety-preconditions/summary.md
  - docs/parity/checklist.md
  - crates/bitaxe-safety/BUILD.bazel
  - crates/bitaxe-safety/src/lib.rs
  - crates/bitaxe-safety/src/mining_preconditions.rs
  - crates/bitaxe-stratum/src/v1/mining_loop.rs
  - crates/bitaxe-stratum/src/v1/state.rs
  - crates/bitaxe-stratum/src/v1/controlled_runtime.rs
  - crates/bitaxe-stratum/src/v1/controlled_runtime/tests.rs
  - crates/bitaxe-api/src/mining.rs
  - firmware/bitaxe/src/controlled_mining_runtime.rs
findings:
  critical: 1
  warning: 2
  info: 1
  total: 4
status: issues_found
---

# Phase 22: Code Review Report

**Reviewed:** 2026-07-04T20:42:00Z
**Depth:** standard
**Files Reviewed:** 19
**Status:** issues_found

## Summary

Reviewed the Phase 22 claim ladder, parity guard, typed safety preconditions, mining-loop propagation, API projection, firmware controlled runtime shell, and evidence/checklist updates. The pure prerequisite model is generally conservative, but the controlled runtime evidence path still has two claim-safety gaps: rejected share details can leak through a redacted summary, and submit outcomes are accepted without verifying that the response belongs to the submit request. The firmware shell also still bypasses the new typed fan/voltage prerequisite contract with synthetic readiness.

## Critical Issues

### CR-01: Redacted Summary Can Leak Raw Pool Rejection Text

**File:** `crates/bitaxe-stratum/src/v1/controlled_runtime.rs:163-188`

**Issue:** `ControlledMiningRuntimeEvidence::redacted_summary()` formats `share_outcome` with `Debug`. For rejected shares, `apply_share_response()` stores `StratumResponseError.message` verbatim in `ControlledShareOutcome::Rejected { reason }` at lines 415-421, so a pool response that echoes a worker name, owner address, endpoint, token, or other raw pool payload can be emitted by a method explicitly named `redacted_summary`. That violates the Phase 22 redaction boundary if this summary is captured into evidence.

**Fix:**

```rust
fn redacted_share_outcome_label(outcome: &Option<ControlledShareOutcome>) -> &'static str {
    match outcome {
        Some(ControlledShareOutcome::Accepted) => "accepted",
        Some(ControlledShareOutcome::Rejected { .. }) => "rejected",
        Some(ControlledShareOutcome::NoShareObserved) => "no_share_observed",
        None => "none",
    }
}
```

Use the label in `redacted_summary()` instead of `Debug` formatting the full enum, and add a regression test where the rejection message contains the existing redaction sentinel strings.

## Warnings

### WR-01: Submit Outcome Trusts Any Stratum Response

**File:** `crates/bitaxe-stratum/src/v1/controlled_runtime.rs:388-421`

**Issue:** `apply_share_response()` records accepted or rejected share outcomes from `maybe_submit_response` without checking that `response.maybe_id` matches `ControlledMiningRuntimePlan::SUBMIT_REQUEST_ID`. A stale authorize/configure response, or any unrelated success response passed into this path, can be promoted into an accepted share outcome. This is especially risky because accepted/rejected shares are evidence-tier boundaries in the Phase 22 claim ladder.

**Fix:** Require the response id to match the submit request before recording an accepted or rejected share. Treat mismatched or missing ids as no verified submit outcome, or return a typed error if the caller expects strict correlation.

```rust
let Some(response) = maybe_response.filter(|response| {
    response.maybe_id == Some(ControlledMiningRuntimePlan::SUBMIT_REQUEST_ID)
}) else {
    return Some(ControlledShareOutcome::NoShareObserved);
};
```

Add tests for an accepted response with `AUTHORIZE_REQUEST_ID` and for a response with `maybe_id: None`; neither should increment accepted/rejected counters.

### WR-02: Firmware Runtime Bypasses Typed Fan And Voltage Preconditions

**File:** `firmware/bitaxe/src/controlled_mining_runtime.rs:207-226`

**Issue:** `controlled_runtime_gate()` hardcodes `production_preconditions: ProductionMiningPreconditionDecision::Ready` plus synthetic power, thermal, and safety evidence. It never constructs the Phase 22 `ProductionMiningPreconditions` from shell/runtime observations, so missing or stale fan and voltage prerequisites cannot fail closed with the new stable reasons before the runtime publishes `phase21_controlled_runtime_status=ready` and `bm1366_work_dispatch_status=typed_action_ready`. This contradicts the `fresh_or_explicitly_bounded` contract documented for Phase 22.

**Fix:** Build a `ProductionMiningPreconditions` value from fresh or explicitly bounded runtime observations before constructing `MiningLoopGate`. Until live fan/voltage/safety inputs exist, fail closed with stable blocker reasons such as `fan_observation_unavailable`, `voltage_observation_unavailable`, or `safety_preflight_evidence_missing`.

```rust
let preconditions = ProductionMiningPreconditions {
    power: power_prerequisite,
    thermal: thermal_prerequisite,
    fan: ProductionMiningPrerequisite::blocked(FAN_OBSERVATION_UNAVAILABLE),
    voltage: ProductionMiningPrerequisite::blocked(VOLTAGE_OBSERVATION_UNAVAILABLE),
    safety: safety_prerequisite,
};

MiningLoopGate {
    production_preconditions: preconditions.decision(),
    // existing legacy gates stay populated here
}
```

## Info

### IN-01: Thermal Pass-Through Test Exercises The Power Field

**File:** `crates/bitaxe-safety/src/mining_preconditions.rs:262-286`

**Issue:** The thermal cases in `production_mining_preconditions_pass_through_existing_safety_reasons()` are stored in a variable named `power_or_thermal`, but every case is assigned to the `power` field at lines 282-284. The invalid thermal observation still produces the expected reason, so the test passes, but it does not prove that the `thermal` prerequisite field preserves those thermal reasons when power is fresh.

**Fix:** Split the power and thermal cases, or include a field selector in each test case so thermal observations are assigned to `thermal` with `power` left fresh.

_Reviewed: 2026-07-04T20:42:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
