---
phase: 24-bm1366-production-work-path
verified: 2026-07-05T01:22:09Z
status: passed
score: "21/21 must-haves verified"
generated_by: gsd-verifier
lifecycle_mode: yolo
phase_lifecycle_id: 24-2026-07-05T00-27-27
generated_at: 2026-07-05T01:22:09Z
lifecycle_validated: true
overrides_applied: 0
review_status: clean
---

# Phase 24: BM1366 Production Work Path Verification Report

**Phase Goal:** The functional core models trusted BM1366 production work decisions while the firmware shell initializes Ultra 205 BM1366 hardware, dispatches pool-derived work, parses live results, and fails closed on unsafe or uncorrelated ASIC behavior.
**Verified:** 2026-07-05T01:22:09Z
**Status:** passed
**Re-verification:** No - initial verification

## Goal Achievement

Phase 24 achieved its code, unit, workflow, evidence, checklist, and review-clean goals. Hardware execution, live Stratum socket success, accepted/rejected share outcomes, nonce-vs-target proof, share-hash validation, and Phase 26 telemetry promotion remain explicit non-claims rather than Phase 24 blockers.

### Observable Truths

| # | Truth | Status | Evidence |
| --- | --- | --- | --- |
| 1 | Operator can see diagnostic chip/work modes reported separately from trusted BM1366 production initialization and work-result modes. | VERIFIED | `Bm1366ProductionCommand`, `ProductionAsicStatus`, production ASIC status logs, and ASIC-09 checklist row are separate from diagnostic `SendDiagnosticWork`; fresh static scan found no `SendDiagnosticWork` in `crates/bitaxe-stratum/src/v1/mining_loop.rs`. |
| 2 | Firmware dispatches BM1366 work derived from the active pool job and tracks job, extranonce, and difficulty context. | VERIFIED | `ProductionWorkRegistry::enqueue_pool_work`, `dispatch_next`, `ProductionWorkRecord`, and `ProductionDispatch` preserve `MiningWork`, `PoolSessionGeneration`, job, extranonce, `ntime`, compact target, and pool difficulty context. |
| 3 | Firmware invalidates stale BM1366 work on clean-jobs or reconnect before another share claim can be recorded. | VERIFIED | `invalidate_for_clean_jobs`, `invalidate_for_reconnect`, authorization reset, and session replacement all call generation-advance-and-clear logic; unit tests cover queued, active, and valid-job invalidation. |
| 4 | Firmware maps live BM1366 nonce or result observations back to active pool work before any share submission claim is recorded. | VERIFIED | `ProductionNonceObservation` carries `PoolSessionGeneration`; `correlate_nonce_result` returns `SubmitIntent` only after current-generation active-work lookup and duplicate/context checks. |
| 5 | Firmware fails closed on BM1366 initialization, UART, reset, timeout, malformed result, or job-correlation failures without leaking raw frames in committed evidence. | VERIFIED | `ProductionAsicBlocker` and firmware publishers emit stable labels; fresh redaction/static scans passed; Phase 24 evidence files declare `raw_artifacts_committed: no` and `redaction_status: passed`. |
| 6 | Diagnostic BM1366 work and trusted production BM1366 work use distinct Rust types. | VERIFIED | `Bm1366Command::SendDiagnosticWork` remains outside `production.rs`; production uses `ProductionWorkPayload` and `Bm1366ProductionCommand::SendProductionWork`. |
| 7 | Production BM1366 commands can emit adapter actions without firmware constructing raw frames. | VERIFIED | `Bm1366ProductionCommand::adapter_actions()` returns `Bm1366AdapterAction`; firmware consumes command actions through `adapter_action_count` and does not construct production raw frames. |
| 8 | Production ASIC failures render stable redaction-safe category labels. | VERIFIED | `ProductionAsicBlocker::as_str()` defines stable lower-snake labels; tests assert labels and reject sensitive fragments. |
| 9 | Pool-derived BM1366 work is bound to a typed session generation before dispatch. | VERIFIED | Registry starts at `PoolSessionGeneration::initial()` and dispatches `ProductionDispatch { generation, work_payload, work }`. |
| 10 | Clean-jobs and reconnect events invalidate queued, active, and valid-job production work. | VERIFIED | Registry invalidation clears queue, active work, and `Bm1366ValidJobIds`; tests assert exact generation changes and stale absence. |
| 11 | Each active production work record carries job, extranonce, difficulty, and target context for later result correlation. | VERIFIED | `ProductionWorkRecord` stores stratum job id, ASIC job id, extranonce2, `ntime`, target context, original work, dispatch state, and result-seen state. |
| 12 | Raw-bearing production work records and dispatches cannot render job, extranonce, target, or payload details through Debug/display output. | VERIFIED | Custom Debug implementations redact record, dispatch, target context, registry, nonce observation, submit intent, and correlation outcome surfaces; tests check sentinel raw values are omitted. |
| 13 | A live BM1366 nonce/result observation produces a submit intent only after active-work correlation succeeds. | VERIFIED | `CorrelationOutcome::SubmitIntent` is produced only after generation match, active record lookup, stale/duplicate/context checks, and `ShareSubmission::from_nonce_result`. |
| 14 | Uncorrelated, stale-generation, duplicate, wrong-session, malformed, or target-mismatched results fail closed with stable redaction-safe reasons. | VERIFIED | Correlation returns `ProductionAsicBlocker::JobUncorrelated`, `WorkStale`, `DuplicateResult`, `WrongSession`, and `TargetMismatch`; malformed result handling is represented by production blocker taxonomy and result parse boundary. |
| 15 | The guarded mining loop dispatches production BM1366 commands, not diagnostic work commands, when the Phase 22 gate is ready. | VERIFIED | `GuardedMiningLoopInputs` owns `ProductionWorkRegistry` and emits `maybe_production_command: Some(Bm1366ProductionCommand::SendProductionWork(...))`; fresh scan found no diagnostic dispatch in the production loop. |
| 16 | The controlled firmware runtime consumes `maybe_production_command` and publishes production statuses instead of treating diagnostic work as the production route. | VERIFIED | Firmware inspects `dispatch.maybe_production_command`, counts typed adapter actions, and publishes initialized, dispatched, correlated, or fail-closed production ASIC statuses. |
| 17 | Firmware publishes production ASIC statuses without constructing raw BM1366 frames or leaking sensitive runtime values. | VERIFIED | `publish_production_asic_status` and `publish_production_asic_blocked_status` emit fixed category logs; planned forbidden-pattern scan passed on touched firmware files. |
| 18 | Operators can see exactly which Phase 24 BM1366 production work claims are implemented by code and tests. | VERIFIED | Evidence summary states the exact Phase 24 claim and lists implementation pointers plus verification commands. |
| 19 | Parity rows for ASIC-09 through ASIC-12 promote only exact Phase 24 evidence. | VERIFIED | Checklist rows are `implemented` with `unit,workflow`, link to Phase 24 summary, and do not promote hardware or live socket/share evidence. |
| 20 | Phase 25 live socket/share response and Phase 26 telemetry claims remain explicit non-claims. | VERIFIED | Checklist and evidence preserve non-claims for accepted/rejected share outcomes, live Stratum socket success, and API/WebSocket/statistics/scoreboard promotion. |
| 21 | Committed Phase 24 evidence contains no raw frames, raw Stratum fields, share payloads, credentials, device URLs, IPs, MACs, Wi-Fi values, tokens, or NVS secrets. | VERIFIED | Fresh forbidden-value scan over generated claim files returned no matches; evidence records `raw_artifacts_committed: no`. |

**Score:** 21/21 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
| --- | --- | --- | --- |
| `crates/bitaxe-asic/src/bm1366/production.rs` | Production BM1366 payload, command, blocker, and status taxonomy | VERIFIED | Exists, substantive, unit-tested, and referenced by Stratum and firmware production paths. |
| `crates/bitaxe-asic/src/bm1366.rs` | Public production module export | VERIFIED | Contains `pub mod production;`. |
| `crates/bitaxe-asic/BUILD.bazel` | Bazel registration for production module | VERIFIED | Includes `src/bm1366/production.rs`. |
| `crates/bitaxe-stratum/src/v1/production_work.rs` | Production active-work registry and result correlation | VERIFIED | Exists, substantive, unit-tested, and wired to mining work, ASIC production payloads, and guarded loop. |
| `crates/bitaxe-stratum/src/v1.rs` | Public production_work export | VERIFIED | Contains `pub mod production_work;`. |
| `crates/bitaxe-stratum/BUILD.bazel` | Bazel registration for production_work module | VERIFIED | Includes `src/v1/production_work.rs`. |
| `crates/bitaxe-stratum/src/v1/mining.rs` | Redaction-safe share submission surface | VERIFIED | `ShareSubmission` has custom Debug and remains the private payload inside `SubmitIntent`. |
| `crates/bitaxe-stratum/src/v1/mining_loop.rs` | Guarded production dispatch and correlation integration | VERIFIED | Owns `ProductionWorkRegistry`, consumes `ProductionNonceObservation`, and emits `maybe_production_command` / `maybe_submit_intent`. |
| `crates/bitaxe-stratum/src/v1/controlled_runtime.rs` | Controlled runtime production contract | VERIFIED | Builds `ProductionNonceObservation` at the result boundary and consumes guarded production outputs. |
| `firmware/bitaxe/src/controlled_mining_runtime.rs` | Firmware runtime production command/status consumption | VERIFIED | Consumes `maybe_production_command`, publishes production statuses, and preserves Phase 25 response non-claims. |
| `firmware/bitaxe/src/asic_adapter.rs` | Thin firmware export for production status publishers | VERIFIED | Re-exports production status publisher functions. |
| `firmware/bitaxe/src/asic_adapter/status.rs` | Redaction-safe production ASIC status publishers | VERIFIED | Emits exact `asic_production_status` labels and fail-closed reason label. |
| `docs/parity/evidence/phase-24-bm1366-production-work-path/production-work.md` | Production work and registry evidence | VERIFIED | Exists with board, source/reference commit metadata, exact non-claims, and redaction status. |
| `docs/parity/evidence/phase-24-bm1366-production-work-path/result-correlation.md` | Correlation and fail-closed evidence | VERIFIED | Describes `SubmitIntent`, `CorrelationOutcome`, production blockers, guarded dispatch, and controlled runtime evidence. |
| `docs/parity/evidence/phase-24-bm1366-production-work-path/redaction-review.md` | Phase 24 redaction review | VERIFIED | Documents scoped deterministic scan and passed artifact inventory. |
| `docs/parity/evidence/phase-24-bm1366-production-work-path/summary.md` | Exact claims, non-claims, and verification commands | VERIFIED | States exact implementation claim and preserves Phase 25/26 non-claims. |
| `docs/parity/checklist.md` | ASIC-09 through ASIC-12 checklist rows | VERIFIED | Rows are `implemented` with `unit,workflow` evidence and Phase 24 summary links. |
| `.planning/phases/24-bm1366-production-work-path/24-VALIDATION.md` | Completed validation metadata | VERIFIED | Frontmatter is `status: passed` and `wave_0_complete: true`; task rows cite observed commands. |

### Key Link Verification

| From | To | Via | Status | Details |
| --- | --- | --- | --- | --- |
| `crates/bitaxe-asic/src/bm1366/production.rs` | `crates/bitaxe-asic/src/bm1366/work.rs` | `ProductionWorkPayload` wraps `Bm1366WorkPayload` | VERIFIED | Manual check: field `payload: Bm1366WorkPayload` and constructor `Bm1366WorkPayload::new(job_id, fields)` are present. |
| `crates/bitaxe-asic/src/bm1366/production.rs` | `crates/bitaxe-asic/src/bm1366/command.rs` | `Bm1366ProductionCommand` emits `Bm1366AdapterAction` | VERIFIED | gsd-tools key-link check found the pattern. |
| `crates/bitaxe-stratum/src/v1/production_work.rs` | `crates/bitaxe-stratum/src/v1/mining.rs` | Registry accepts `MiningWork` | VERIFIED | gsd-tools key-link check found the pattern. |
| `crates/bitaxe-stratum/src/v1/production_work.rs` | `crates/bitaxe-asic/src/bm1366/production.rs` | Dispatch creates `ProductionWorkPayload` | VERIFIED | gsd-tools key-link check found the pattern. |
| `crates/bitaxe-stratum/src/v1/mining_loop.rs` | `crates/bitaxe-stratum/src/v1/production_work.rs` | Guarded loop owns `ProductionWorkRegistry` | VERIFIED | gsd-tools key-link check found the pattern. |
| `crates/bitaxe-stratum/src/v1/production_work.rs` | `crates/bitaxe-stratum/src/v1/mining.rs` | `SubmitIntent` wraps `ShareSubmission` | VERIFIED | Manual check: `SubmitIntent` owns private `submission: ShareSubmission`; regex-only key-link check missed the multi-line type declaration. |
| `firmware/bitaxe/src/asic_adapter/status.rs` | `crates/bitaxe-asic/src/bm1366/production.rs` | Production status/blocker labels | VERIFIED | gsd-tools key-link check found the pattern. |
| `firmware/bitaxe/src/controlled_mining_runtime.rs` | `crates/bitaxe-stratum/src/v1/mining_loop.rs` | Consumes guarded plan production outputs | VERIFIED | gsd-tools key-link check found `maybe_production_command` and `maybe_submit_intent`. |
| `firmware/bitaxe/src/controlled_mining_runtime.rs` | `firmware/bitaxe/src/asic_adapter/status.rs` | Publishes production ASIC status and blocker labels | VERIFIED | gsd-tools key-link check found publisher references. |
| `docs/parity/checklist.md` | `docs/parity/evidence/phase-24-bm1366-production-work-path/summary.md` | ASIC-09 through ASIC-12 evidence links | VERIFIED | gsd-tools key-link check passed. |
| `docs/parity/evidence/phase-24-bm1366-production-work-path/redaction-review.md` | `docs/parity/evidence/phase-24-bm1366-production-work-path/summary.md` | Redaction metadata agreement | VERIFIED | gsd-tools key-link check passed. |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
| --- | --- | --- | --- | --- |
| `crates/bitaxe-asic/src/bm1366/production.rs` | `ProductionWorkPayload.payload` | `Bm1366WorkPayload::new(job_id, fields)` | Yes | FLOWING |
| `crates/bitaxe-stratum/src/v1/production_work.rs` | `ProductionWorkRecord.work` and `target_context` | `MiningWork` from `MiningWorkBuilder` / active pool job input | Yes | FLOWING |
| `crates/bitaxe-stratum/src/v1/production_work.rs` | `SubmitIntent.submission` | `ShareSubmission::from_nonce_result(&record.work, observation.result)` after correlation | Yes | FLOWING |
| `crates/bitaxe-stratum/src/v1/mining_loop.rs` | `maybe_production_command` | `production_registry.dispatch_next()` and `Bm1366ProductionCommand::SendProductionWork` | Yes | FLOWING |
| `firmware/bitaxe/src/controlled_mining_runtime.rs` | Production status logs | `ControlledMiningRuntimePlan.guarded_plan` outputs and `ProductionAsicStatus` publishers | Yes | FLOWING |
| Phase 24 evidence docs | Checklist/evidence claim metadata | `24-01` through `24-04` summaries, fresh repo-native checks, and production code pointers | Yes | FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
| --- | --- | --- | --- |
| Phase 24 Rust core and parity tests pass | `bazel test //crates/bitaxe-asic/... //crates/bitaxe-stratum/... //crates/bitaxe-safety:tests //tools/parity:tests` | 4 tests pass; build completed successfully | PASS |
| Firmware still builds with production runtime/status wiring | `bazel build //firmware/bitaxe:firmware` | Built `bazel-bin/firmware/bitaxe/bitaxe-firmware.elf`; source commit `7bc7c7d3c822` | PASS |
| Parity checklist accepts ASIC-09 through ASIC-12 rows | `just parity` | `validation_errors: none`; ASIC-09 through ASIC-12 report `implemented` with `unit,workflow` | PASS |
| Reference implementation remains clean/read-only | `just verify-reference` | `reference clean: c1915b0a63bfabebdb95a515cedfee05146c1d50` | PASS |
| Lifecycle provenance is consistent | `node "$HOME/.cursor/get-shit-done/bin/gsd-tools.cjs" verify lifecycle 24 --expect-id 24-2026-07-05T00-27-27 --expect-mode yolo --require-plans` | Context, four plans, and four summaries valid; verification file was not yet present during pre-write check | PASS |
| Code review status is clean | `rg "status: clean\|critical: 0\|warning: 0\|info: 0\|total: 0" 24-REVIEW.md` | Clean review and zero findings confirmed | PASS |
| Review fix report closed findings | `rg "status: all_fixed\|findings_in_scope: 2\|fixed: 2\|skipped: 0" 24-REVIEW-FIX.md` | Two findings in scope, two fixed, zero skipped | PASS |
| Diagnostic dispatch is absent from production loop | `! rg -n "SendDiagnosticWork" crates/bitaxe-stratum/src/v1/mining_loop.rs` | No matches | PASS |
| Firmware production route does not mention diagnostic dispatch | `! rg -n "production.*SendDiagnosticWork\|SendDiagnosticWork.*production" firmware/bitaxe/src/controlled_mining_runtime.rs firmware/bitaxe/src/asic_adapter.rs firmware/bitaxe/src/asic_adapter/status.rs` | No matches | PASS |
| Production ASIC core avoids sensitive/static forbidden sentinels | `! rg -n "raw_bm1366_frame\|target=\|extranonce=\|share_payload=\|pool_config\|device_url\|password\|token" crates/bitaxe-asic/src/bm1366/production.rs` | No matches | PASS |
| Firmware touched files avoid sensitive/static forbidden sentinels | `! rg -n "raw_bm1366_frame\|target=\|extranonce=\|share_payload=\|socket_error=\|device_url=\|password\|token\|poolURL\|poolUser" firmware/bitaxe/src/controlled_mining_runtime.rs firmware/bitaxe/src/asic_adapter.rs firmware/bitaxe/src/asic_adapter/status.rs` | No matches | PASS |
| Generated evidence claim files are redaction-safe | `! rg -n -i "(stratum[+]tcp://\|bc1q[[:alnum:]]{20,}\|sentinel-(password\|token\|nvs\|share\|extra\|pool)\|192[.]0[.]2[.]\|[0-9a-f]{2}(:[0-9a-f]{2}){5})" production-work.md result-correlation.md summary.md` | No matches | PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
| --- | --- | --- | --- | --- |
| ASIC-09 | `24-01-PLAN.md`, `24-03-PLAN.md`, `24-04-PLAN.md` | Ultra 205 production mining separates BM1366 diagnostic chip/work modes from trusted production initialization and work-result modes. | SATISFIED | Production-only ASIC types, guarded production dispatch, production status publishers, clean review, and ASIC-09 checklist row. |
| ASIC-10 | `24-02-PLAN.md`, `24-03-PLAN.md`, `24-04-PLAN.md` | Ultra 205 production mining dispatches BM1366 work derived from the active pool job, tracks job/extranonce/difficulty context, and invalidates stale work on clean-jobs or reconnect. | SATISFIED | `ProductionWorkRegistry`, active records, invalidation tests, guarded dispatch, and ASIC-10 checklist row. |
| ASIC-11 | `24-03-PLAN.md`, `24-04-PLAN.md` | Ultra 205 production mining maps live BM1366 nonce or result observations back to active pool work before any share submission claim is recorded. | SATISFIED | `ProductionNonceObservation`, `CorrelationOutcome`, `SubmitIntent`, stale/wrong-session regression test, and ASIC-11 checklist row. |
| ASIC-12 | `24-01-PLAN.md`, `24-03-PLAN.md`, `24-04-PLAN.md` | Ultra 205 production mining fails closed on BM1366 initialization, UART, reset, timeout, malformed result, or job-correlation failures without leaking raw frames in committed evidence. | SATISFIED | `ProductionAsicBlocker`, status publishers, redaction-safe evidence, static scans, and ASIC-12 checklist row. |

No orphaned Phase 24 requirements were found in `.planning/REQUIREMENTS.md`; ASIC-09 through ASIC-12 are the complete Phase 24 requirement set and are all accounted for in plan frontmatter.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| --- | --- | --- | --- | --- |
| None | N/A | No blocker stubs, missing artifact wiring, diagnostic-production reuse, or sensitive evidence leaks found in Phase 24 verification scope. | N/A | N/A |

### Human Verification Required

None for Phase 24 pass. Detector-gated hardware execution, live Stratum socket behavior, accepted/rejected share response classification, safe-stop runtime proof, and API/WebSocket/statistics/scoreboard promotion are preserved as later-phase non-claims rather than Phase 24 human-verification blockers.

### Gaps Summary

No gaps found. The two automated key-link false negatives were manually resolved as present because the Rust type relationships span multiple lines:

- `ProductionWorkPayload` owns `Bm1366WorkPayload`.
- `SubmitIntent` owns private `ShareSubmission`.

The code review closure was also verified: `24-REVIEW.md` is clean, and `24-REVIEW-FIX.md` records two findings fixed and zero skipped. The WR-01 target-proof concern is intentionally preserved as an explicit Phase 24 non-claim, with Phase 24 limited to stored work-context drift checks and submit-intent correlation.

_Verified: 2026-07-05T01:22:09Z_
_Verifier: Claude (gsd-verifier)_
