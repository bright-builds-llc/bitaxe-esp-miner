# Phase 22 Claim Ladder And Safety Preconditions Summary

## Status

phase22_status: complete
phase22_evidence_closure: claim_ladder_and_safety_preconditions
board: 205
redaction_status: passed
evidence_class: unit,workflow
hardware_evidence_added_in_phase22: no

## Purpose

Phase 22 closes the v1.1 claim-governance and prerequisite-safety foundation. Operators can distinguish v1.0 controlled no-share evidence from v1.1 prerequisite readiness, live socket/runtime evidence, live ASIC-derived share outcomes, and explicit deferred non-claims. Firmware and pure Rust surfaces now model production mining prerequisites as typed decisions that fail closed with stable blocker reasons before BM1366 work dispatch.

## Evidence Matrix

| Evidence | Supported claim | Evidence class | Verification |
| --- | --- | --- | --- |
| `docs/parity/evidence/phase-22-claim-ladder-and-safety-preconditions/claim-ladder.md` | Operator-visible claim tiers, allowed claims, blocked claims, explicit non-claims, and promotion rules for `EVD-06`. | workflow | `bazel test //tools/parity:tests` |
| `tools/parity/src/claim_ladder.rs` | Parity guard requires stable tier ids and rejects controlled no-share overclaims. | unit,workflow | `bazel test //tools/parity:tests` |
| `crates/bitaxe-safety/src/mining_preconditions.rs` | Typed `fresh_or_explicitly_bounded` production-mining prerequisites for power, thermal, fan, voltage, and safety. | unit | `bazel test //crates/bitaxe-safety:tests` |
| `crates/bitaxe-stratum/src/v1/mining_loop.rs` | Mining-loop dispatch consumes typed precondition decisions and blocks before BM1366 work dispatch. | unit | `bazel test //crates/bitaxe-stratum:tests` |
| `crates/bitaxe-stratum/src/v1/state.rs` | Runtime state stores exact safe-blocked work-submission reasons. | unit | `bazel test //crates/bitaxe-stratum:tests` |
| `crates/bitaxe-api/src/mining.rs` | API mining state projects the exact runtime `blockedReason`. | unit | `bazel test //crates/bitaxe-api:tests` |
| `docs/parity/evidence/phase-22-claim-ladder-and-safety-preconditions/safety-preconditions.md` | Evidence summary for `SAFE-10` typed prerequisite readiness. | workflow | `just parity` |
| `docs/parity/evidence/phase-22-claim-ladder-and-safety-preconditions/blocker-reasons.md` | Stable redaction-safe blocker reason ledger for `SAFE-11`. | workflow | `just parity` |
| `docs/parity/evidence/phase-22-claim-ladder-and-safety-preconditions/redaction-review.md` | Redaction review for committed Phase 22 docs and reason strings. | workflow | `just verify-reference`, `just parity` |

## Exact Supported Claims

- `EVD-06` is supported by the operator claim ladder, parity guard tests, and checklist citation to this summary.
- `SAFE-10` is implemented by typed preconditions that require power, thermal, fan, voltage, and safety to be fresh or explicitly bounded for board `205`.
- `SAFE-11` is implemented by fail-closed blocker reason strings and propagation through safety decision, mining loop, runtime state, and API mining output.
- The Phase 22 prerequisite contract is `fresh_or_explicitly_bounded`.
- The Phase 22 docs and reason strings passed redaction review for committed artifacts.

## Explicit Non-Claims

accepted/rejected shares remain non-claims for Phase 22.

full active voltage/fan/thermal/self-test/fault-stimulus closure remains a non-claim for Phase 22.

Phase 22 also does not claim unbounded production mining, full production mining, active DS4432U voltage control, fan actuation, thermal fault stimulus, self-test hardware closure, fault-stimulus closure, successful live ASIC nonce/result parsing, non-205 boards, Stratum v2, OTA/recovery trust, runtime display/input parity, BAP behavior, or unbounded stress mining.

No new hardware behavior was claimed by Phase 22 because no detector-gated hardware command was run in this phase. Prior Phase 20 and Phase 21 hardware evidence remains useful context only at its exact evidence level and does not promote Phase 22 prerequisite readiness into live share or full active safety proof.

## Verification Commands

- `bazel test //tools/parity:tests`
- `bazel test //crates/bitaxe-safety:tests //crates/bitaxe-stratum:tests //crates/bitaxe-api:tests`
- `just parity`
- `just verify-reference`
- `node "$HOME/.cursor/get-shit-done/bin/gsd-tools.cjs" verify lifecycle 22 --expect-id 22-2026-07-04T20-10-36 --expect-mode yolo --require-plans`

## Checklist Readiness

The Phase 22 checklist rows should remain conservative:

- `EVD-06`: `verified`, evidence `workflow`, because the claim ladder and parity guard prove the operator-visible exact-claim governance surface.
- `SAFE-10`: `implemented`, evidence `unit,workflow`, because typed prerequisites and tests exist but no detector-gated hardware run verified live prerequisite behavior.
- `SAFE-11`: `implemented`, evidence `unit,workflow`, because stable blocker reasons and exact projection exist but no detector-gated hardware run verified live stale/unavailable/unsafe prerequisite behavior.

