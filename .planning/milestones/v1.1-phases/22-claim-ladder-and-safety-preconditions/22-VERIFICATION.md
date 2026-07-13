---
phase: 22-claim-ladder-and-safety-preconditions
verified: 2026-07-04T20:51:23Z
status: passed
score: "8/8 must-haves verified"
generated_by: gsd-verifier
lifecycle_mode: yolo
phase_lifecycle_id: 22-2026-07-04T20-10-36
generated_at: 2026-07-04T20:51:23Z
lifecycle_validated: true
overrides_applied: 0
requirements:
  - EVD-06
  - SAFE-10
  - SAFE-11
must_haves:
  truths:
    - "Operator can distinguish v1.0 controlled no-share evidence from v1.1 live production-mining claim tiers."
    - "Phase 21 approved controlled no-share soak remains controlled no-share closure and is not promoted to accepted/rejected production-share proof."
    - "Parity tooling has a testable claim-ladder source of truth with stable tier identifiers and overclaim guardrails."
    - "Production mining readiness requires fresh or explicitly bounded power, thermal, fan, voltage, and safety observations before BM1366 work dispatch."
    - "Missing, stale, unavailable, unsafe, ambiguous, or undocumented prerequisites produce stable user-visible blocker reason strings."
    - "A blocked mining-loop decision keeps work submission blocked, mining safe-blocked, and hardware-control suppression represented in the fail-closed safety plan without a force or shell bypass."
    - "Operator evidence and checklist rows promote only the evidence level supported by Phase 22 docs, tests, workflow checks, and redaction review."
    - "Parity materials preserve explicit non-claims for full active voltage, fan, thermal, self-test, and fault-stimulus closure."
  artifacts:
    - path: "docs/parity/evidence/phase-22-claim-ladder-and-safety-preconditions/claim-ladder.md"
      provides: "Operator-visible claim ladder"
    - path: "tools/parity/src/claim_ladder.rs"
      provides: "Claim ladder tier source and validator tests"
    - path: "crates/bitaxe-safety/src/mining_preconditions.rs"
      provides: "Typed production-mining prerequisite contract"
    - path: "crates/bitaxe-stratum/src/v1/mining_loop.rs"
      provides: "Fail-closed mining-loop dispatch gate"
    - path: "crates/bitaxe-stratum/src/v1/state.rs"
      provides: "Exact blocked reason runtime storage"
    - path: "crates/bitaxe-api/src/mining.rs"
      provides: "API blockedReason projection"
    - path: "docs/parity/evidence/phase-22-claim-ladder-and-safety-preconditions/summary.md"
      provides: "Phase 22 exact-claim closure summary"
    - path: "docs/parity/checklist.md"
      provides: "Conservative EVD-06, SAFE-10, and SAFE-11 checklist rows"
  key_links:
    - from: "tools/parity/src/main.rs"
      to: "tools/parity/src/claim_ladder.rs"
      via: "mod claim_ladder"
    - from: "tools/parity/src/claim_ladder.rs"
      to: "docs/parity/evidence/phase-22-claim-ladder-and-safety-preconditions/claim-ladder.md"
      via: "include_str! fixture validation"
    - from: "crates/bitaxe-stratum/src/v1/mining_loop.rs"
      to: "crates/bitaxe-safety/src/mining_preconditions.rs"
      via: "ProductionMiningPreconditionDecision consumed before dispatch"
    - from: "crates/bitaxe-api/src/mining.rs"
      to: "crates/bitaxe-stratum/src/v1/state.rs"
      via: "state.maybe_blocked_reason"
---

# Phase 22: Claim Ladder And Safety Preconditions Verification Report

**Phase Goal:** Ultra 205 operators can tell exactly which v1.1 production-mining claims are allowed, and firmware can fail closed before BM1366 work dispatch when prerequisite safety evidence is missing or unsafe.
**Verified:** 2026-07-04T20:51:23Z
**Status:** passed
**Re-verification:** No - no prior Phase 22 verification report existed.

## Goal Achievement

Phase 22 achieved the goal at the evidence level it explicitly scoped: operator-facing claim governance, typed fail-closed prerequisite decisions, exact blocker propagation, conservative checklist rows, and redaction-safe documentation. It does not claim live detector-gated prerequisite behavior, accepted/rejected shares, unbounded production mining, or full active voltage/fan/thermal/self-test/fault-stimulus closure.

### Observable Truths

| # | Truth | Status | Evidence |
| --- | --- | --- | --- |
| 1 | Operator can distinguish v1.0 controlled no-share evidence from v1.1 live production-mining claim tiers. | VERIFIED | `claim-ladder.md` defines `version_1_0_controlled_no_share`, `version_1_1_prerequisite_readiness`, `version_1_1_live_socket_runtime`, `version_1_1_live_asic_share_outcome`, and `explicit_deferred_non_claim`; `tools/parity/src/claim_ladder.rs` requires those ids. |
| 2 | Phase 21 approved controlled no-share soak remains controlled no-share closure and is not promoted to accepted/rejected production-share proof. | VERIFIED | `claim-ladder.md` states `approved_controlled_no_share_soak` supports controlled no-share closure only; the validator rejects same-paragraph accepted/rejected share overclaims. |
| 3 | Parity tooling has a testable claim-ladder source of truth with stable tier identifiers and overclaim guardrails. | VERIFIED | `claim_ladder_tiers()` and `validate_claim_ladder_document()` exist; `//tools/parity:tests` validates the committed Markdown fixture. |
| 4 | Production mining readiness requires fresh or explicitly bounded power, thermal, fan, voltage, and safety observations before BM1366 work dispatch. | VERIFIED | `ProductionMiningPreconditions` requires all five categories and returns `Ready` only when each is fresh or valid bounded Ultra 205 evidence; `MiningLoopGate::decision()` consumes blocked preconditions before legacy dispatch gates. |
| 5 | Missing, stale, unavailable, unsafe, ambiguous, or undocumented prerequisites produce stable user-visible blocker reason strings. | VERIFIED | `mining_preconditions.rs` defines fan, voltage, bounded-evidence, and safety reasons; existing power/thermal reasons pass through; `blocker-reasons.md` records the redaction-safe ledger. |
| 6 | A blocked mining-loop decision keeps work submission blocked, mining safe-blocked, and hardware-control suppression represented in the fail-closed safety plan without a force or shell bypass. | VERIFIED | Blocked preconditions include `SafetyEffectPlan::fail_closed(reason)`; `MiningRuntimeState::block_work_submission(reason)` sets blocked/safe-blocked state; bypass search found no force/unsafe override implementation in the touched surfaces. |
| 7 | Operator evidence and checklist rows promote only the evidence level supported by Phase 22 docs, tests, workflow checks, and redaction review. | VERIFIED | Checklist sets `EVD-06` to `verified` with `workflow`, while `SAFE-10` and `SAFE-11` remain `implemented` with `unit,workflow`; `summary.md` and `safety-preconditions.md` explicitly withhold live hardware verification claims. |
| 8 | Parity materials preserve explicit non-claims for full active voltage, fan, thermal, self-test, and fault-stimulus closure. | VERIFIED | `claim-ladder.md`, `summary.md`, `safety-preconditions.md`, and checklist SAFE rows preserve active safety, share-outcome, unbounded mining, non-205, Stratum v2, OTA/recovery, runtime UI/input, and BAP non-claims. |

**Score:** 8/8 truths verified

### Deferred Items

No Phase 22 verification gaps were deferred. Later phases must still provide detector-gated hardware/operator evidence before promoting live production mining, live share outcomes, redacted evidence root, safe-stop, watchdog-load behavior, API/WebSocket parity closure, or full active safety-control behavior.

### Required Artifacts

| Artifact | Expected | Status | Details |
| --- | --- | --- | --- |
| `docs/parity/evidence/phase-22-claim-ladder-and-safety-preconditions/claim-ladder.md` | Operator-visible allowed, blocked, and explicit non-claims | VERIFIED | Contains all five stable tier ids, allowed/blocked/non-claim language, and promotion rules. |
| `tools/parity/src/claim_ladder.rs` | Tier table, document validator, and tests | VERIFIED | Defines `ClaimTier`, `claim_ladder_tiers()`, and `validate_claim_ladder_document()`; tests cover required ids, fixture validity, overclaim rejection, and missing tier rejection. |
| `crates/bitaxe-safety/src/mining_preconditions.rs` | Pure production-mining prerequisite contract | VERIFIED | Defines `BoundedObservationEvidence`, `ProductionMiningPrerequisite`, `ProductionMiningPreconditions`, and `ProductionMiningPreconditionDecision`; tests cover ready, missing, stale/unavailable/unsafe, bounded ambiguity, undocumented evidence, board mismatch, and fail-closed effects. |
| `crates/bitaxe-stratum/src/v1/mining_loop.rs` | Dispatch gate consumes prerequisite decision before BM1366 dispatch | VERIFIED | Checks `ProductionMiningPreconditionDecision::Blocked` first; blocked plans emit no dispatch and no share submission. |
| `crates/bitaxe-stratum/src/v1/state.rs` | Runtime storage for exact blocker reason | VERIFIED | `maybe_blocked_reason`, `block_work_submission(reason)`, and `clear_blocked_reason()` exist and are tested. |
| `crates/bitaxe-api/src/mining.rs` | API-visible exact blocker projection | VERIFIED | `blockedReason` returns `state.maybe_blocked_reason.unwrap_or("")` only for safe-blocked work submission. |
| `firmware/bitaxe/src/controlled_mining_runtime.rs` | Review-fix shell integration for typed prerequisites | VERIFIED | Actual controlled runtime now constructs `ProductionMiningPreconditions`; missing live fan, voltage, or safety inputs block publication instead of claiming ready. |
| `docs/parity/evidence/phase-22-claim-ladder-and-safety-preconditions/safety-preconditions.md` | SAFE-10 evidence ledger | VERIFIED | Records `fresh_or_explicitly_bounded`, board `205`, all five prerequisite categories, dispatch gate, API visibility, and no-hardware-verification boundary. |
| `docs/parity/evidence/phase-22-claim-ladder-and-safety-preconditions/blocker-reasons.md` | SAFE-11 blocker reason ledger | VERIFIED | Lists stable redaction-safe reason strings and their safety/runtime/API surfaces. |
| `docs/parity/evidence/phase-22-claim-ladder-and-safety-preconditions/redaction-review.md` | Redaction review | VERIFIED | Reports no committed pool URLs, ports, workers, owner addresses, passwords, tokens, device URLs, IPs, MACs, Wi-Fi values, NVS secrets, raw Stratum payloads, raw share payloads, or raw BM1366 frames. |
| `docs/parity/evidence/phase-22-claim-ladder-and-safety-preconditions/summary.md` | Final exact-claim closure summary | VERIFIED | Cites claim ladder, precondition contract, blocker ledger, targeted tests, `just parity`, `just verify-reference`, and exact non-claims. |
| `docs/parity/checklist.md` | Conservative EVD-06, SAFE-10, SAFE-11 rows | VERIFIED | Adds the v1.1 governance section with `EVD-06` verified workflow and SAFE rows implemented unit/workflow. |

### Key Link Verification

| From | To | Via | Status | Details |
| --- | --- | --- | --- | --- |
| `tools/parity/src/main.rs` | `tools/parity/src/claim_ladder.rs` | `mod claim_ladder;` | VERIFIED | Module declaration present. |
| `tools/parity/src/claim_ladder.rs` | `claim-ladder.md` | `include_str!` fixture | VERIFIED | Manual check: the helper literal-pattern verifier missed this because `include_str!` spans multiple lines, but the fixture path is present and tested. |
| `tools/parity/BUILD.bazel` | `claim-ladder.md` | `compile_data` | VERIFIED | Markdown fixture is declared through the root export and parity target compile data. |
| `crates/bitaxe-safety/src/lib.rs` | `crates/bitaxe-safety/src/mining_preconditions.rs` | public module export | VERIFIED | `pub mod mining_preconditions;` present and Bazel source includes the file. |
| `crates/bitaxe-stratum/src/v1/mining_loop.rs` | `crates/bitaxe-safety/src/mining_preconditions.rs` | `ProductionMiningPreconditionDecision` | VERIFIED | Mining-loop gate checks blocked preconditions before power, thermal, safety, hardware ack, ASIC init, dispatch, or share submission. |
| `crates/bitaxe-stratum/src/v1/mining_loop.rs` | `crates/bitaxe-stratum/src/v1/state.rs` | `block_work_submission(reason)` | VERIFIED | The helper artifact verifier missed the inherent method export pattern, but the method exists, is called, and is tested. |
| `crates/bitaxe-api/src/mining.rs` | `crates/bitaxe-stratum/src/v1/state.rs` | `state.maybe_blocked_reason` | VERIFIED | API mapping projects exact runtime blocker reasons. |
| `docs/parity/checklist.md` | `docs/parity/evidence/phase-22-claim-ladder-and-safety-preconditions/summary.md` | evidence notes | VERIFIED | Checklist row `EVD-06` cites the Phase 22 summary; SAFE rows cite the preconditions and blocker ledgers. |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
| --- | --- | --- | --- | --- |
| `tools/parity/src/claim_ladder.rs` | Markdown validation errors | `include_str!(...claim-ladder.md)` plus required tier/phrase/paragraph scans | Yes - validates the committed claim ladder fixture in `//tools/parity:tests` | VERIFIED |
| `crates/bitaxe-safety/src/mining_preconditions.rs` | `ProductionMiningPreconditionDecision` | `ProductionMiningPreconditions::decision()` over power, thermal, fan, voltage, and safety fields | Yes - returns exact blocker reason and fail-closed effect plan for blocked inputs | VERIFIED |
| `crates/bitaxe-stratum/src/v1/mining_loop.rs` | `MiningLoopDecision` and guarded plan | `MiningLoopGate::decision()` and `apply_to_state()` | Yes - blocked preconditions prevent dispatch/share submission and store exact reason | VERIFIED |
| `crates/bitaxe-stratum/src/v1/state.rs` | `maybe_blocked_reason` | `block_work_submission(reason)` and `clear_blocked_reason()` | Yes - state stores and clears exact blocker reasons in tests | VERIFIED |
| `crates/bitaxe-api/src/mining.rs` | `blockedReason` | `mining_state_from_runtime()` reads `state.maybe_blocked_reason` | Yes - API output reflects exact `voltage_observation_stale` and hardware-ack blocker tests | VERIFIED |
| `firmware/bitaxe/src/controlled_mining_runtime.rs` | runtime publication decision | `controlled_production_preconditions().decision()` | Yes - actual settings snapshot path blocks on `fan_observation_unavailable` until live fan/voltage/safety observations exist | VERIFIED |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
| --- | --- | --- | --- |
| Targeted Rust and parity tests | `bazel test //tools/parity:tests //crates/bitaxe-safety:tests //crates/bitaxe-stratum:tests //crates/bitaxe-api:tests` | 4 cached test targets passed | PASS |
| Parity checklist no-overclaim validation | `just parity` | `validation_errors: none`; rows include `EVD-06`, `SAFE-10`, and `SAFE-11` at conservative statuses | PASS |
| Reference cleanliness | `just verify-reference` | `reference clean: c1915b0a63bfabebdb95a515cedfee05146c1d50` | PASS |
| Lifecycle provenance | `node "$HOME/.cursor/get-shit-done/bin/gsd-tools.cjs" verify lifecycle 22 --expect-id 22-2026-07-04T20-10-36 --expect-mode yolo --require-plans` | Valid before this report existed; context, plans, and summaries share lifecycle id/mode | PASS |
| Schema drift | `node "$HOME/.cursor/get-shit-done/bin/gsd-tools.cjs" verify schema-drift 22` | `drift_detected: false`, `blocking: false` | PASS |
| Recent full gate evidence | `bazel test //...` | Provided gate evidence reports 33 tests passing after review fixes | PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
| --- | --- | --- | --- | --- |
| EVD-06 | 22-01, 22-03 | Operator can distinguish v1.0 controlled no-share evidence from v1.1 live production mining claims through a documented claim ladder. | SATISFIED | `claim-ladder.md`, `claim_ladder.rs`, `docs/parity/checklist.md`, `summary.md`, and `//tools/parity:tests` prove exact claim tiers and no controlled-no-share overclaim. |
| SAFE-10 | 22-02, 22-03 | Ultra 205 production mining requires fresh or explicitly bounded power, thermal, fan, voltage, and safety observations before BM1366 work dispatch is enabled. | SATISFIED at implemented/unit/workflow level | `mining_preconditions.rs` models all five prerequisite categories; `MiningLoopGate` consumes blocked preconditions before dispatch; firmware controlled runtime blocks missing live fan/voltage/safety inputs. Hardware verification remains required before promoting to verified live behavior. |
| SAFE-11 | 22-02, 22-03 | Ultra 205 production mining fails closed with user-visible blocker reasons when safety prerequisites are stale, unavailable, unsafe, ambiguous, or undocumented. | SATISFIED at implemented/unit/workflow level | Stable reason constants and pass-throughs are tested; runtime state and API projection preserve exact blocker strings; `blocker-reasons.md` records the redaction-safe taxonomy. Hardware verification remains required before promoting live prerequisite behavior. |

All Phase 22 requirement IDs from PLAN frontmatter are accounted for in `.planning/REQUIREMENTS.md`; no orphaned Phase 22 requirements were found.

### Code Review Fix Outcomes

| Finding | Status | Verification Evidence |
| --- | --- | --- |
| CR-01 redacted summary could leak raw pool rejection text | FIXED | Commit `dc367f5`; `ControlledMiningRuntimeEvidence::redacted_summary()` now uses `redacted_share_outcome_label()`, and regression tests check sentinel rejection text does not appear in summaries. |
| WR-01 submit outcome trusted any Stratum response | FIXED | Commit `0282c0f`; `apply_share_response()` now requires `maybe_id == SUBMIT_REQUEST_ID`, with tests for authorize-response and missing-id cases. |
| WR-02 firmware runtime bypassed typed fan/voltage preconditions | FIXED | Commit `b80601b`; firmware controlled runtime now builds `ProductionMiningPreconditions` and blocks missing fan, voltage, and safety inputs before publishing ready/dispatch markers. |
| Review-fix report | PRESENT | Commit `1108b9a`; `.planning/phases/22-claim-ladder-and-safety-preconditions/22-REVIEW-FIX.md` records all three in-scope findings fixed. |

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| --- | --- | --- | --- | --- |
| None blocking | N/A | Stub/bypass scan | N/A | No TODO/FIXME/placeholders, empty implementations, or force/unsafe override paths were found in the touched Rust surfaces. |
| `crates/bitaxe-safety/src/mining_preconditions.rs` | 240 | Existing review info: thermal pass-through table stores thermal cases in the `power` field for one table test | Info | Not a Phase 22 blocker: thermal missing/unavailable behavior and production precondition field coverage are otherwise tested, and targeted Bazel tests pass. |
| GSD helper output | N/A | Literal-pattern false negatives | Info | `verify key-links` missed multi-line `include_str!`; `verify artifacts` missed inherent method `MiningRuntimeState::block_work_submission`. Manual and test evidence confirms both links. |

### Human Verification Required

None for Phase 22 closure. The phase's supported claims are docs, unit, workflow, and code-wiring claims.

Detector-gated Ultra 205 hardware evidence is still required before any later phase promotes SAFE-10 or SAFE-11 beyond `implemented` into verified live prerequisite behavior, or claims live accepted/rejected shares, active voltage/fan/thermal/self-test/fault-stimulus closure, unbounded production mining, non-205 board behavior, Stratum v2, OTA/recovery trust, runtime display/input parity, or BAP behavior.

### Gaps Summary

No blocking gaps found. Phase 22 achieved its goal while preserving the intended evidence boundary: EVD-06 is verified as workflow claim governance, SAFE-10 and SAFE-11 are implemented with unit/workflow evidence, and hardware-critical promotions remain future work.

_Verified: 2026-07-04T20:51:23Z_
_Verifier: Claude (gsd-verifier)_
