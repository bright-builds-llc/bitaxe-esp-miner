---
phase: 26-telemetry-and-parity-closure
verified: 2026-07-05T05:00:10Z
status: passed
score: 14/14 must-haves verified
generated_by: gsd-verifier
lifecycle_mode: yolo
phase_lifecycle_id: 26-2026-07-05T03-48-38
generated_at: 2026-07-05T05:00:10Z
lifecycle_validated: true
overrides_applied: 0
---

# Phase 26: Telemetry And Parity Closure Verification Report

**Phase Goal:** API, WebSocket, statistics, scoreboard, and parity checklist projections are derived from the same v1.1 runtime events and promote only exact claims proven by redacted Ultra 205 artifacts.
**Verified:** 2026-07-05T05:00:10Z
**Status:** passed
**Re-verification:** No - initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
| --- | --- | --- | --- |
| 1 | Operator can query `/api/system/info`, `/api/system/statistics`, and `/api/system/scoreboard` for mining state, counters, hashrate inputs, share outcomes, and post-stop state derived from v1.1 runtime events. | VERIFIED | `firmware/bitaxe/src/http_api.rs` routes system info/statistics/scoreboard through projection helpers; `firmware/bitaxe/src/runtime_snapshot.rs` projects `RuntimeTelemetryProjection` into API views; API tests and firmware build passed. |
| 2 | Operator can observe `/api/ws` and `/api/ws/live` stream redacted, session-correlated mining telemetry without stale active-mining state after stop. | VERIFIED | `/api/ws/live` connect and cadence paths use `projected_live_telemetry_payload`; tests reject stale `"active"` payload after safe stop. `/api/ws` remains compatible raw retained-log stream; detector-gated live share telemetry remains a non-claim. |
| 3 | Statistics, scoreboard, and share counters do not advance unless corresponding runtime events and parsed pool responses exist. | VERIFIED | `RuntimeTelemetryProjection::fold` gates accepted/rejected counters on current generation plus `SubmitClassification`; statistics samples require drained runtime sample markers; scoreboard remains empty without parsed share outcome material. |
| 4 | Parity checklist updates promote only exact v1.1 claims proven by artifacts and preserve explicit non-claims for deferred surfaces. | VERIFIED | `docs/parity/checklist.md`, Phase 26 evidence files, and `tools/parity` validator preserve exact non-claims; `just parity` returned `validation_errors: none`. |
| 5 | Production-visible mining counters are derived from typed Phase 26 runtime events. | VERIFIED | `RuntimeTelemetryEvent` variants fold into one `RuntimeTelemetryProjection`; firmware producers call publish helpers rather than mutating counters directly. |
| 6 | Accepted and rejected counters advance only from current-generation submit intent plus parsed pool response. | VERIFIED | Firmware retains `PendingSubmit { intent, request_id }`, classifies matching responses through `classify_maybe_submit_response`, publishes `pending_submit.intent.generation`, and projection rejects stale/non-share classifications. |
| 7 | Bounded statistics samples are marked only by runtime-event boundaries, never repeated request-time reads. | VERIFIED | `RuntimeProjectionSampleMarker` is created by producer-bound events and drained once; repeated `projected_statistics` calls produce one row then empty output. |
| 8 | Safe-stop events reset active mining, work submission, lifecycle, and stale event eligibility before serialization. | VERIFIED | `SafeStopped` advances generation, disconnects lifecycle, blocks work submission, and tests assert `SafeBlocked` state plus no stale counter updates. |
| 9 | API view builders serialize the same projection-backed mining state for system info, statistics, scoreboard, and live WebSocket payloads. | VERIFIED | `project_api_views` replaces `ApiSnapshot.mining` from the projection and builds statistics, scoreboard, and `SystemInfoWire` JSON from the same state. |
| 10 | Scoreboard remains an upstream-shaped empty array unless parsed-response-backed share outcome material exists. | VERIFIED | `project_api_views` keeps `scoreboard_entries` empty; tests assert upstream empty array output and Phase 26 evidence records `empty_without_parsed_share_outcome`. |
| 11 | Phase 25 runtime producers fold lifecycle, hashrate, submit classification, blocked prerequisite, and safe-stop events into `RuntimeTelemetryProjection`. | VERIFIED | `live_stratum_runtime.rs` publishes lifecycle, pool difficulty, hashrate, work-ready, submit classification, blocked, bounded sample, and safe-stop events through runtime snapshot helpers. |
| 12 | Firmware HTTP and WebSocket consumers serialize projection-backed views through the Plan 26-02 API contract. | VERIFIED | `handle_system_info`, `handle_statistics`, `handle_scoreboard`, live cadence, and live connect frame all consume projected helpers. |
| 13 | Committed Phase 26 evidence maps API-11, API-12, API-13, and EVD-08 to exact source artifacts. | VERIFIED | `docs/parity/evidence/phase-26-telemetry-and-parity-closure/summary.md` maps all four IDs to plan summaries, source files, evidence files, commands, and exact non-claims. |
| 14 | `just parity` rejects overbroad telemetry, statistics, scoreboard, or Phase 26 verified claims with missing artifacts, blocker language, or absent redaction review. | VERIFIED | `validate_phase26_telemetry_verified_row` checks Phase 26 rows; regression tests cover missing summary, blocker language, missing redaction evidence, missing exact non-claims, and conservative accepted rows. |

**Score:** 14/14 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
| --- | --- | --- | --- |
| `crates/bitaxe-stratum/src/v1/telemetry_projection.rs` | Shared runtime-event projection, sequence/session guards, counter gate, sample marker, safe-stop reset | VERIFIED | Exists, substantive, exported, tested, and wired through firmware producer helpers. |
| `crates/bitaxe-stratum/src/v1.rs` | Public module export | VERIFIED | Exports `pub mod telemetry_projection;`. |
| `crates/bitaxe-stratum/BUILD.bazel` | Bazel source registration | VERIFIED | Includes `src/v1/telemetry_projection.rs`. |
| `crates/bitaxe-api/src/runtime_projection.rs` | Projection-to-API view builder | VERIFIED | Defines `ProjectedApiViews` and `project_api_views`; maps projection state into snapshot/statistics/scoreboard/live JSON. |
| `crates/bitaxe-api/src/statistics.rs` | Upstream-compatible statistics response | VERIFIED | Consumed through `statistics_response` only with projection-supplied samples. |
| `crates/bitaxe-api/src/scoreboard.rs` | Upstream-compatible scoreboard response | VERIFIED | Consumed through projected empty entries when share outcome material is absent. |
| `crates/bitaxe-api/src/telemetry.rs` | Live WebSocket planning | VERIFIED | Existing planner is exercised with projection-backed JSON in tests. |
| `firmware/bitaxe/src/live_stratum_runtime.rs` | Runtime producer wiring | VERIFIED | Publishes typed projection events from lifecycle, work, submit response, blocked, and safe-stop paths. |
| `firmware/bitaxe/src/runtime_snapshot.rs` | Firmware projection bridge | VERIFIED | Stores projection state, assigns monotonic sequence values, drains sample markers, and exposes projected route helpers. |
| `firmware/bitaxe/src/http_api.rs` | HTTP and live WebSocket consumer wiring | VERIFIED | Routes call projected helper functions before serialization. |
| `firmware/bitaxe/src/websocket_api.rs` | WebSocket session bridge | VERIFIED | Preserves session frame planning; receives projection-backed payloads from `http_api.rs`. |
| `docs/parity/evidence/phase-26-telemetry-and-parity-closure/api.md` | API projection evidence | VERIFIED | Present, redaction reviewed, maps API-11/API-13/EVD-08. |
| `docs/parity/evidence/phase-26-telemetry-and-parity-closure/websocket.md` | WebSocket projection evidence | VERIFIED | Present, records projection-backed full/cadence behavior and stale-active rejection. |
| `docs/parity/evidence/phase-26-telemetry-and-parity-closure/statistics-scoreboard.md` | Statistics/scoreboard invariant evidence | VERIFIED | Present, records no request-time fabrication and empty scoreboard non-claim. |
| `docs/parity/evidence/phase-26-telemetry-and-parity-closure/redaction-review.md` | Denylist/redaction review | VERIFIED | Present, records `redaction_status: passed`, `raw_artifacts_committed: no`, and `raw_pool_values_committed: no`. |
| `docs/parity/evidence/phase-26-telemetry-and-parity-closure/summary.md` | Requirement mapping and exact non-claims | VERIFIED | Present, maps API-11/API-12/API-13/EVD-08 and preserves hardware/live-share non-claims. |
| `tools/parity/src/main.rs` | Phase 26 overclaim guardrail | VERIFIED | Validator and regression tests are present; `just parity` passes. |

### Key Link Verification

| From | To | Via | Status | Details |
| --- | --- | --- | --- | --- |
| `telemetry_projection.rs` | `production_work.rs` | `PoolSessionGeneration` and firmware-retained `SubmitIntent` | WIRED | Projection gates by generation; firmware only publishes accepted/rejected classifications after matching `PendingSubmit`. |
| `telemetry_projection.rs` | `submit_response.rs` | `SubmitClassification` | WIRED | Accepted/rejected counter branches are limited to `SubmitClassification::Accepted` and `Rejected`; other outcomes do not advance counters. |
| `telemetry_projection.rs` | `state.rs` | `MiningRuntimeState` | WIRED | Projection owns state mutation for lifecycle, hashrate, blocker, safe-stop, and counter updates. |
| `telemetry_projection.rs` | `runtime_projection.rs` | Sample marker and state adapter | WIRED | API projection consumes projection state and optional drained marker. |
| `runtime_projection.rs` | `statistics.rs` | `StatisticsSample::from_snapshot` | WIRED | Samples are only created when `maybe_sample_marker` is `Some`. |
| `runtime_projection.rs` | `scoreboard.rs` | `scoreboard_response` | WIRED | Projected scoreboard entries remain empty without parsed share outcome material. |
| `runtime_projection.rs` | `telemetry.rs` | `SystemInfoWire` JSON for planner | WIRED | Live telemetry tests pass projection-backed JSON through `WebSocketState`. |
| `live_stratum_runtime.rs` | `telemetry_projection.rs` | Runtime producer helpers | WIRED | Producer sites publish typed events, including safe-stop and submit classification. |
| `runtime_snapshot.rs` | `telemetry_projection.rs` | Stored projection and marker drain | WIRED | `CommandVisibleState` owns `RuntimeTelemetryProjection` and drains markers once. |
| `runtime_snapshot.rs` | `runtime_projection.rs` | `project_api_views` | WIRED | Snapshot helper calls `project_api_views` with cloned projection and optional marker. |
| `http_api.rs` | `runtime_snapshot.rs` | Projected route helpers | WIRED | HTTP and live WebSocket handlers call projected helpers. |
| `docs/parity/checklist.md` | Phase 26 evidence summary | Checklist evidence citation | WIRED | API/statistics/scoreboard/EVD rows cite Phase 26 summary and redaction review. |
| `tools/parity/src/main.rs` | `docs/parity/checklist.md` | `validate_phase26_telemetry_verified_row` | WIRED | `validate_rows` calls the Phase 26 validator. |
| `redaction-review.md` | Phase 23 evidence contract | Denylist categories | WIRED | Redaction review uses established forbidden committed value categories. |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
| --- | --- | --- | --- | --- |
| `RuntimeTelemetryProjection` | `MiningRuntimeState` | Typed runtime events folded by `fold` | Yes - lifecycle, difficulty, hashrate, work-ready, blocked, submit, sample, and safe-stop events mutate projection state | FLOWING |
| `firmware/bitaxe/src/live_stratum_runtime.rs` | Submit classification | `PendingSubmit` retains `SubmitIntent` and matching `request_id`; parsed response goes through `classify_maybe_submit_response` | Yes - accepted/rejected classifications require matching pending submit and parsed response | FLOWING |
| `firmware/bitaxe/src/runtime_snapshot.rs` | `maybe_sample_marker` | Runtime producer publishes `BoundedSampleReady`; projected statistics drains marker once | Yes - statistics rows appear only from marker drain | FLOWING |
| `crates/bitaxe-api/src/runtime_projection.rs` | `snapshot.mining` | `projection.state().clone()` | Yes - API snapshot mining fields are replaced from projection state | FLOWING |
| `firmware/bitaxe/src/http_api.rs` | HTTP response bodies | `projected_system_info`, `projected_statistics`, `projected_scoreboard` | Yes - route handlers consume projected helpers | FLOWING |
| `firmware/bitaxe/src/http_api.rs` and `websocket_api.rs` | Live telemetry frames | `projected_live_telemetry_payload` passed to connect/cadence frame planners | Yes - WebSocket frames use current projection-backed JSON | FLOWING |
| `docs/parity/checklist.md` | Phase 26 claim status | Evidence files plus `tools/parity` validation | Yes - checklist rows cite exact artifacts and are guarded by `just parity` | FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
| --- | --- | --- | --- |
| Plan artifacts and key links are present | `node "$HOME/.cursor/get-shit-done/bin/gsd-tools.cjs" verify artifacts/key-links ...` for all four plans | 17/17 artifacts passed; 14/14 key links verified | PASS |
| Projection/API/parity tests pass | `bazel test //crates/bitaxe-stratum:tests //crates/bitaxe-api:tests //tools/parity:tests` | 3/3 test targets passed | PASS |
| Firmware compiles with projected route wiring | `bazel build //firmware/bitaxe:firmware` | Build completed successfully; source commit `f1136d74e10d` | PASS |
| Checklist guard accepts exact claims | `just parity` | `validation_errors: none` | PASS |
| Reference tree remains clean | `just verify-reference` | `reference clean: c1915b0a63bfabebdb95a515cedfee05146c1d50` | PASS |
| Lifecycle metadata is consistent | `node "$HOME/.cursor/get-shit-done/bin/gsd-tools.cjs" verify lifecycle 26 --expect-id 26-2026-07-05T03-48-38 --expect-mode yolo --require-plans` | Pre-report lifecycle check was valid except for the not-yet-created verification artifact | PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
| --- | --- | --- | --- | --- |
| API-11 | 26-01, 26-02, 26-03, 26-04 | `/api/system/info`, `/api/system/statistics`, and `/api/system/scoreboard` expose mining state, counters, hashrate inputs, share outcomes, and post-stop state derived from the same v1.1 runtime events. | SATISFIED | Projection core, API view builder, firmware route helpers, evidence `api.md` and `statistics-scoreboard.md`, and passed firmware build. |
| API-12 | 26-02, 26-03, 26-04 | `/api/ws` and `/api/ws/live` stream redacted, session-correlated mining telemetry during bounded production mining without stale active-mining state after stop. | SATISFIED | Live telemetry payload derives from projection; connect/cadence tests reject stale active state; `websocket.md` preserves detector-gated non-claims. |
| API-13 | 26-01, 26-02, 26-03, 26-04 | Statistics, scoreboard, and share counters do not advance without corresponding runtime events and parsed pool responses. | SATISFIED | Projection counter gate, one-drain sample marker, empty scoreboard invariant, pending-submit classification path, and passed tests. |
| EVD-08 | 26-04 | Committed parity checklist updates promote only exact claims proven by v1.1 artifacts and preserve explicit non-claims for deferred surfaces. | SATISFIED | Checklist EVD-08 row, Phase 26 evidence summary/redaction review, `tools/parity` guardrails, and `just parity` pass. |

No orphaned Phase 26 requirement IDs were found in `.planning/REQUIREMENTS.md`: Phase 26 maps exactly to `EVD-08`, `API-11`, `API-12`, and `API-13`.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| --- | --- | --- | --- | --- |
| `firmware/bitaxe/src/http_api.rs` | 471 | `otawww_update=gap ...` | Info | Existing OTAWWW non-claim text, unrelated to Phase 26 telemetry closure. |
| `docs/parity/evidence/phase-26-telemetry-and-parity-closure/statistics-scoreboard.md` | 34 | Mentions placeholders/fabrication prevention | Info | Documentation of the exact invariant, not a runtime placeholder. |
| `crates/bitaxe-stratum/src/v1/fake_pool.rs`, `crates/bitaxe-stratum/src/v1/controlled_runtime.rs`, `crates/bitaxe-api/src/system.rs` | Multiple | Direct `record_accepted_share` / `record_rejected_share` calls outside projection | Info | These are legacy fake-pool, controlled runtime, or fixture/test paths; firmware Phase 26 production-visible route wiring uses `RuntimeTelemetryProjection`. |

No blocker anti-patterns were found in the Phase 26 implementation or evidence.

### Human Verification Required

None for the exact Phase 26 projection/workflow scope verified here.

Detector-gated live hardware/API/WebSocket observations, accepted/rejected live-share proof, full active safety closure, OTA/recovery, non-205 boards, other ASIC families, Stratum v2, display/input, BAP, and unbounded stress remain explicit non-claims. They are not counted as Phase 26 gaps because the phase artifacts deliberately avoid promoting those claims without current detector-gated evidence.

### Gaps Summary

No gaps found. The codebase delivers the Phase 26 goal at the exact claimed tier: shared runtime-event projection, projection-backed HTTP/WebSocket/statistics/scoreboard serialization, current-generation parsed submit counter gating, one-drain statistics sample markers, redaction-reviewed evidence, conservative checklist updates, and `just parity` guardrails against overbroad verified claims.

_Verified: 2026-07-05T05:00:10Z_
_Verifier: Claude (gsd-verifier)_
