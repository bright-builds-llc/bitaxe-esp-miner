---
phase: 25-live-stratum-runtime-and-safe-stop
verified: 2026-07-05T02:48:48Z
status: passed
score: 17/17 must-haves verified
generated_by: gsd-verifier
lifecycle_mode: yolo
phase_lifecycle_id: 25-2026-07-05T01-55-45
generated_at: 2026-07-05T02:48:48Z
lifecycle_validated: true
overrides_applied: 0
---

# Phase 25: Live Stratum Runtime And Safe Stop Verification Report

**Phase Goal:** A watchdog-responsive firmware shell drives a pure Stratum v1 production runtime with real TCP socket I/O, live ASIC-derived submit behavior, deterministic fake-pool coverage, and bounded safe-stop postconditions.
**Verified:** 2026-07-05T02:48:48Z
**Status:** passed
**Re-verification:** No - initial verification

## Goal Achievement

Phase 25 achieves the goal at the exact claim level allowed by the roadmap and phase context. The code implements the pure runtime, submit classifier, firmware socket shell, deterministic fake-pool coverage, safe-stop postconditions, watchdog categories, and evidence workflow. Committed evidence intentionally records `share_outcome: blocked_safe_prerequisite` and non-claims for accepted/rejected live shares and hardware watchdog proof because no detector-gated live pool response artifact is present.

### Observable Truths

| # | Truth | Status | Evidence |
| --- | --- | --- | --- |
| 1 | Operator can observe real Stratum v1 connect, subscribe, authorize, difficulty/extranonce, notify, submit, response classification, reconnect, and safe-stop lifecycle events. | VERIFIED | `firmware/bitaxe/src/live_stratum_runtime.rs` owns `TcpStream` connect/read/write/shutdown and drives `LiveStratumRuntime`; `docs/parity/checklist.md` keeps hardware live socket success below verified. |
| 2 | Tests prove deterministic fake-pool or fixture behavior for subscribe, authorize, notify, clean-jobs, submit response, reconnect, fallback, and error classification. | VERIFIED | `crates/bitaxe-stratum/src/v1/fake_pool.rs` includes `FakePoolTranscript::run_live_runtime` with accepted/rejected, blocked, timeout, malformed, reconnect, fallback, stale, and no-response classifications. |
| 3 | Operator can observe at least one real pool response to a live ASIC-derived `mining.submit` classified as accepted or rejected, or an explicit safe-prerequisite blocker. | VERIFIED | `docs/parity/evidence/phase-25-live-stratum-runtime-and-safe-stop/share-outcome.md` records `share_outcome: blocked_safe_prerequisite` and accepted/rejected non-claims. |
| 4 | Operator can stop production mining into a bounded safe state with socket activity stopped, work queues drained or invalidated, mining disabled, hardware control disabled, and post-stop API/WebSocket state updated. | VERIFIED | `LiveStratumRuntime::safe_stop`, `safe_stop_with_socket`, `phase25_safe_stop_state`, and `replace_mining_runtime_state_after_phase25_safe_stop` provide the required postconditions. |
| 5 | Operator can verify watchdog responsiveness under bounded socket, ASIC, API, WebSocket, and evidence-capture load. | VERIFIED | `crates/bitaxe-safety/src/watchdog.rs` adds Phase 25 step kinds and threshold tests; committed evidence records hardware watchdog proof as blocked/non-claim. |
| 6 | Pure Stratum runtime can model connect, subscribe, authorize, difficulty/extranonce, notify, submit, reconnect, and safe stop without socket I/O. | VERIFIED | `crates/bitaxe-stratum/src/v1/live_runtime.rs` contains the pure state machine and no `TcpStream`, `std::net`, or ESP-IDF ownership. |
| 7 | Accepted or rejected share classification requires a matching `SubmitIntent` and request id. | VERIFIED | `classify_submit_response` takes `&SubmitIntent` and request id; mismatched, absent, fake-only, and stale cases do not classify as accepted/rejected. |
| 8 | Fake-pool coverage proves deterministic accepted, rejected, blocked, timeout, reconnect, clean-jobs, malformed, and fallback behavior without making live STR-09 claims. | VERIFIED | Fake-pool tests and checklist text explicitly keep deterministic outcomes below live STR-09 proof. |
| 9 | Firmware live Stratum mode is a distinct compile-time opt-in and remains fail-closed when the Phase 25 mode and acknowledgment are absent. | VERIFIED | `MiningEvidenceMode::Phase25LiveStratumRuntime` requires `PHASE25_LIVE_STRATUM_MODE` plus `PHASE25_LIVE_STRATUM_ACK`; tests cover missing and mismatched values. |
| 10 | Firmware live Stratum startup evaluates Phase 22 typed prerequisite readiness before any `TcpStream` connect attempt or secret-bearing pool configuration access. | VERIFIED | `start_live_stratum_runtime_with_dependencies` gates on `ProductionMiningPreconditions::decision`; tests assert connect and settings access counters stay zero for blocked prerequisites. |
| 11 | Firmware owns real TCP socket connect/read/write/shutdown with bounded timeouts and feeds typed observations into the pure runtime. | VERIFIED | `FirmwareTcpConnector` uses `TcpStream`, 100 ms read/write timeouts, and `Shutdown::Both`; socket lines parse through `parse_server_message`. |
| 12 | Safe stop stops socket activity, invalidates work, disables mining and hardware-control surfaces, blocks submission, and refreshes API/WebSocket-visible post-stop state without implementing Phase 26 telemetry projection. | VERIFIED | Safe-stop markers and snapshot helper exist; no Phase 26 statistics or scoreboard projection was added. |
| 13 | Watchdog checkpoints cover socket, ASIC, API/WebSocket, and evidence-capture load categories. | VERIFIED | `PHASE25_LIVE_RUNTIME_STEP_KINDS` includes `Socket`, `Asic`, `Api`, `WebSocket`, and `EvidenceCapture`; firmware publishes category checkpoints. |
| 14 | Operator can run a repo-owned Phase 25 evidence wrapper that either records detector-gated live Stratum proof or an exact blocked safe-prerequisite non-claim. | VERIFIED | `scripts/phase25-live-stratum-evidence.sh` supports blocked and hardware modes; hardware mode starts with detector and board-info gates. |
| 15 | Mining allow validation deliberately permits only the Phase 25 wrapper command shape and continues to reject raw Stratum, unsafe hardware-control, erase, rollback, stale target, and secret-bearing evidence. | VERIFIED | `tools/parity/src/mining_allow.rs` adds `live-stratum-runtime` tiers and strict wrapper predicates while preserving prohibited command tokens. |
| 16 | Committed Phase 25 evidence records share outcome, safe stop, watchdog, redaction, and exact non-claims without raw credentials, endpoints, device URLs, IPs, MACs, targets, extranonces, share payloads, socket errors, or BM1366 frames. | VERIFIED | Forbidden-pattern scan over Phase 25/Phase 23 evidence docs returned no matches; evidence docs record category labels and raw-value non-claims. |
| 17 | Checklist rows for STR-08, STR-09, STR-11, SAFE-12, and SAFE-13 promote only to the status supported by the actual Phase 25 artifacts. | VERIFIED | `docs/parity/checklist.md` marks STR-11 verified from deterministic unit coverage and keeps STR-08, STR-09, SAFE-12, and SAFE-13 at implemented/workflow scope with hardware non-claims. |

**Score:** 17/17 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
| --- | --- | --- | --- |
| `crates/bitaxe-stratum/src/v1/live_runtime.rs` | Pure runtime and safe-stop postconditions | VERIFIED | Exists, substantive, exported, tested, and wired to production registry/messages. |
| `crates/bitaxe-stratum/src/v1/submit_response.rs` | `SubmitIntent`-tied classifier | VERIFIED | Exists, substantive, exported, and rejects fake/stale/mismatched responses. |
| `crates/bitaxe-stratum/src/v1/fake_pool.rs` | Deterministic fake-pool coverage | VERIFIED | Drives live runtime and classifier without socket I/O. |
| `firmware/bitaxe/src/live_stratum_runtime.rs` | Firmware TCP adapter, safe-stop, watchdog markers | VERIFIED | Owns `TcpStream` shell and delegates protocol decisions to pure crate. |
| `firmware/bitaxe/src/mining_evidence_mode.rs` | Distinct Phase 25 mode gate | VERIFIED | Requires dedicated mode/ack pair and preserves Phase 21 path separation. |
| `firmware/bitaxe/src/runtime_snapshot.rs` | Post-stop state refresh | VERIFIED | Provides named Phase 25 safe-stop replacement helper only. |
| `crates/bitaxe-safety/src/watchdog.rs` | Phase 25 watchdog categories | VERIFIED | Adds socket/ASIC/API/WebSocket/evidence step kinds under existing thresholds. |
| `scripts/phase25-live-stratum-evidence.sh` | Detector-gated evidence wrapper | VERIFIED | Supports blocked/hardware modes, detector-first hardware path, redacted allow manifest, safe-stop markers. |
| `scripts/phase25-live-stratum-evidence-test.sh` | Wrapper/redaction tests | VERIFIED | Covers blocked mode, detector failure, live capture attempt, URL validation, and redaction sentinels. |
| `tools/parity/src/mining_allow.rs` | Phase 25 allow-manifest validation | VERIFIED | Accepts only approved live-or-blocked wrapper command shapes. |
| `docs/parity/evidence/phase-25-live-stratum-runtime-and-safe-stop/summary.md` | Exact claims and non-claims | VERIFIED | Records blocked-safe-prerequisite status, safe-stop completion, watchdog blocker, and raw-value non-claims. |
| `docs/parity/checklist.md` | Conservative checklist rows | VERIFIED | Rows are exact to the available evidence and avoid hardware overclaiming. |

### Key Link Verification

| From | To | Via | Status | Details |
| --- | --- | --- | --- | --- |
| `crates/bitaxe-stratum/src/v1/live_runtime.rs` | `crates/bitaxe-stratum/src/v1/production_work.rs` | `ProductionWorkRegistry` invalidation and submit-intent ownership | WIRED | Verifier helper found the declared pattern and source reads confirm invalidation paths. |
| `crates/bitaxe-stratum/src/v1/live_runtime.rs` | `crates/bitaxe-stratum/src/v1/messages.rs` | Typed client/server messages | WIRED | Runtime queues `StratumV1ClientMessage` and consumes `StratumV1ServerMessage`. |
| `crates/bitaxe-stratum/src/v1/submit_response.rs` | `crates/bitaxe-stratum/src/v1/production_work.rs` | Classifier requires `SubmitIntent` | WIRED | Function signature and tests require submit intent plus request id. |
| `firmware/bitaxe/src/live_stratum_runtime.rs` | `crates/bitaxe-stratum/src/v1/live_runtime.rs` | Socket lines and outbound actions | WIRED | Firmware writes `to_json_line`, parses server lines, and applies typed runtime messages. |
| `firmware/bitaxe/src/live_stratum_runtime.rs` | `crates/bitaxe-safety/src/mining_preconditions.rs` | Phase 22 readiness gate | WIRED | Preconditions gate runs before settings access or socket connection. |
| `firmware/bitaxe/src/live_stratum_runtime.rs` | `crates/bitaxe-stratum/src/v1/submit_response.rs` | Submit response classifier | WIRED | Firmware imports and uses classification functions for response observations. |
| `firmware/bitaxe/src/live_stratum_runtime.rs` | `firmware/bitaxe/src/runtime_snapshot.rs` | Post-stop state refresh | WIRED | Safe-stop path calls `replace_mining_runtime_state_after_phase25_safe_stop`. |
| `scripts/phase25-live-stratum-evidence.sh` | `tools/parity/src/mining_allow.rs` | Wrapper-generated allow manifest | WIRED | Wrapper emits Phase 25 manifest and calls mining-allow validation. |
| `docs/parity/checklist.md` | `docs/parity/evidence/phase-25-live-stratum-runtime-and-safe-stop/summary.md` | Evidence citation | WIRED | Checklist rows cite the Phase 25 evidence summary. |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
| --- | --- | --- | --- | --- |
| `firmware/bitaxe/src/live_stratum_runtime.rs` | Pool settings | `settings_adapter::current_settings_snapshot()` after readiness gate | Yes, runtime NVS snapshot values are parsed after preconditions pass | FLOWING |
| `firmware/bitaxe/src/live_stratum_runtime.rs` | Socket observations | `TcpStream` read with 100 ms timeout | Yes, complete read buffers parse through `parse_server_message` into the pure runtime | FLOWING |
| `crates/bitaxe-stratum/src/v1/live_runtime.rs` | Mining work and lifecycle state | Typed Stratum responses/notifications and `ProductionWorkRegistry` | Yes, notify builds `MiningWork`, enqueues work, updates lifecycle and mining activity | FLOWING |
| `crates/bitaxe-stratum/src/v1/submit_response.rs` | Submit classification | `SubmitIntent`, request id, typed `StratumResponse` or observation category | Yes, accepted/rejected only flow from matching live intent plus response identity | FLOWING |
| `firmware/bitaxe/src/runtime_snapshot.rs` | Post-stop mining state | `phase25_safe_stop_state` after `runtime.safe_stop` or blocked non-start | Yes, API/WebSocket-visible state is replaced with blocked post-stop mining state | FLOWING |
| `scripts/phase25-live-stratum-evidence.sh` | Evidence status fields | Detector/board-info/live-capture outcome or blocked mode | Yes, wrapper writes either live observed categories or blocked-safe-prerequisite categories | FLOWING |
| `docs/parity/evidence/phase-25-live-stratum-runtime-and-safe-stop/*.md` | Committed evidence | Static blocked-safe-prerequisite closure | Yes, records safe blocker and exact non-claims; no live accepted/rejected proof is claimed | FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
| --- | --- | --- | --- |
| Plan 25-01 artifacts and links are substantive/wired | `node "$HOME/.cursor/get-shit-done/bin/gsd-tools.cjs" verify artifacts ...25-01-PLAN.md && node ... verify key-links ...25-01-PLAN.md` | 5/5 artifacts, 3/3 links passed | PASS |
| Plan 25-02 artifacts and links are substantive/wired | `node "$HOME/.cursor/get-shit-done/bin/gsd-tools.cjs" verify artifacts ...25-02-PLAN.md && node ... verify key-links ...25-02-PLAN.md` | 5/5 artifacts, 4/4 links passed | PASS |
| Plan 25-03 artifacts and links are substantive/wired | `node "$HOME/.cursor/get-shit-done/bin/gsd-tools.cjs" verify artifacts ...25-03-PLAN.md && node ... verify key-links ...25-03-PLAN.md` | 5/5 artifacts, 3/3 links passed | PASS |
| Lifecycle provenance matches context/plans/summaries | `node "$HOME/.cursor/get-shit-done/bin/gsd-tools.cjs" verify lifecycle 25 --expect-id 25-2026-07-05T01-55-45 --expect-mode yolo --require-plans` | Valid except this report was not yet written at command time | PASS |
| Final automated implementation checks | Orchestrator-provided checks: `bazel test //crates/bitaxe-stratum:tests //crates/bitaxe-safety:tests //scripts:phase25_live_stratum_evidence_test //tools/parity:tests`, `bazel build //firmware/bitaxe:firmware`, `just parity`, `just verify-reference`, `git diff --check` | Reported passed by orchestrator | PASS |
| Evidence redaction | `rg` forbidden pattern scan over Phase 25 evidence docs and updated Phase 23 slots | No matches | PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
| --- | --- | --- | --- | --- |
| STR-08 | 25-01, 25-02, 25-03 | Real Stratum v1 TCP socket lifecycle for connect, subscribe, authorize, difficulty/extranonce, notify, submit, response classification, reconnect, and safe stop | SATISFIED | Pure runtime, firmware `TcpStream` adapter, evidence wrapper, and conservative checklist row exist. |
| STR-09 | 25-01, 25-02, 25-03 | Classifies real pool response to live ASIC-derived submit as accepted/rejected or explicitly blocked by safe prerequisite | SATISFIED | Classifier enforces `SubmitIntent` plus request id; committed evidence records `blocked_safe_prerequisite` and accepted/rejected non-claims. |
| STR-11 | 25-01, 25-03 | Deterministic fake-pool/fixture tests for subscribe, authorize, notify, clean-jobs, submit response, reconnect, fallback, and error classification | SATISFIED | `fake_pool.rs` coverage and recent `//crates/bitaxe-stratum:tests` pass. |
| SAFE-12 | 25-01, 25-02, 25-03 | Bounded safe stop with socket stopped, work invalidated, mining disabled, hardware control disabled, and post-stop API/WebSocket state updated | SATISFIED | Pure safe-stop postconditions, firmware socket shutdown, snapshot refresh, evidence docs, and safe-stop markers. |
| SAFE-13 | 25-02, 25-03 | Watchdog responsiveness under bounded socket, ASIC, API, WebSocket, and evidence-capture load | SATISFIED | Watchdog step kinds and tests exist; evidence records hardware-level watchdog proof as blocked/non-claim. |

No orphaned Phase 25 requirement IDs were found in `.planning/REQUIREMENTS.md`.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| --- | --- | --- | --- | --- |
| `crates/bitaxe-stratum/src/v1/fake_pool.rs` | 118, 276, 309 | Empty match arms | Info | Intentional no-op cases in state machine matching, not stubs. |
| `firmware/bitaxe/src/live_stratum_runtime.rs` | 255, 310 | Empty match arms | Info | Intentional no-progress/no-event handling, not stubs. |
| `firmware/bitaxe/src/main.rs` | 129 | Broad forbidden scan matched `rust_target` | Info | False positive from boot provenance marker, not a raw Stratum target or secret. |

### Human Verification Required

None for phase acceptance. Hardware accepted/rejected share proof and hardware-level watchdog proof are explicitly recorded as blocked/non-claims, which is an allowed Phase 25 outcome when detector-gated live evidence is not present.

### Gaps Summary

No blocking gaps found. The only partial-looking items are intentional claim boundaries: accepted/rejected live shares and hardware watchdog responsiveness remain non-claims until a future detector-gated run produces redaction-safe artifacts. Phase 26 still owns full API, WebSocket, statistics, scoreboard, and final parity projection.

_Verified: 2026-07-05T02:48:48Z_
_Verifier: Claude (gsd-verifier)_
