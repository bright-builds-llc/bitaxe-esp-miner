---
phase: 04-stratum-v1-and-first-mining-loop
verified: 2026-06-27T15:43:05Z
status: passed
score: "16/16 must-haves verified"
generated_by: gsd-verifier
lifecycle_mode: yolo
phase_lifecycle_id: 04-2026-06-27T13-17-33
generated_at: 2026-06-27T15:43:05Z
lifecycle_validated: true
overrides_applied: 0
deferred:
  - truth: "Actual AxeOS API/WebSocket telemetry handlers expose mining runtime state to users."
    addressed_in: "Phase 5"
    evidence: "Phase 5 goal and success criteria explicitly cover upstream-compatible API, log, /api/ws, /api/ws/live, statistics, and mining-state surfaces; Phase 04 provides the typed runtime state those handlers will consume."
---

# Phase 4: Stratum V1 And First Mining Loop Verification Report

**Phase Goal:** Ultra 205 can mine through an upstream-compatible Stratum v1 loop using safe ASIC work dispatch and evidence-backed result reporting.
**Verified:** 2026-06-27T15:43:05Z
**Status:** passed
**Re-verification:** No - initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
| --- | --- | --- | --- |
| 1 | Developer can run deterministic fake pool scenarios covering subscribe, authorize, notify, set-difficulty, submit, fallback, reconnect, and error logging behavior. | ✓ VERIFIED | `FakePoolTranscript` processes typed `ExpectClient`, `SendServer`, `Disconnect`, and `Timeout` events; tests cover accepted submit, rejected submit, reconnect, fallback, and unexpected client paths. `cargo test -p bitaxe-stratum --all-features` passed 40 tests. |
| 2 | Firmware can construct jobs, decode coinbase/extranonce data, queue work, dispatch BM1366 work, parse results, and submit shares without bypassing safety gates. | ✓ VERIFIED | Pure bridge code builds `Bm1366WorkFields`, queues work, tracks valid/active jobs, converts valid nonce results into `ShareSubmission`, and emits dispatch only when `MiningLoopGate` is ready. Firmware main remains fail-closed and only publishes the blocked status. |
| 3 | User-facing mining/API/telemetry surfaces report share counters, difficulty/hashrate inputs, result counters, and lifecycle status consistently. | ✓ VERIFIED | Phase 04 owns the typed runtime state (`MiningRuntimeState`, `ShareCounters`, `HashrateInputs`, lifecycle/fallback/activity/work gate fields). Actual HTTP/WebSocket handler exposure is explicitly deferred to Phase 5. |
| 4 | Developer can review mining hardware-smoke and soak evidence before mining parity is claimed. | ✓ VERIFIED | `docs/parity/evidence/phase-04-stratum-v1-mining-loop.md` records command slots, smoke/soak criteria, and `not run - hardware evidence pending`; checklist splits live evidence into STR-008 `not-started`. |
| 5 | Stratum v1 parser/serializer tests cover subscribe, authorize, configure, suggest-difficulty, extranonce-subscribe, notify, set-difficulty, set-extranonce, set-version-mask, submit, result, and error behavior. | ✓ VERIFIED | `messages.rs` defines the client/server message enums, method names, parser, and tests for the planned method classes. |
| 6 | Raw JSON is parsed at the crate boundary into typed domain values before it mutates mining state. | ✓ VERIFIED | `parse_server_message` converts `serde_json::Value` into `StratumV1ServerMessage` variants and typed domain structs; fake-pool/state code consumes typed messages, not raw JSON. |
| 7 | Reference-derived protocol fixtures include source path, reference commit, license posture, derivation, and STR checklist IDs. | ✓ VERIFIED | `protocol-cases.json`, `fake-pool-transcripts.json`, and `mining-job-cases.json` are valid JSON with provenance metadata and 13/4/4 cases respectively. |
| 8 | Pool lifecycle state and share counters update from typed outcomes, not firmware sockets. | ✓ VERIFIED | `apply_server_message` and `apply_response` update `MiningRuntimeState` from typed `StratumV1ServerMessage` and `StratumResponse` values. |
| 9 | Runtime state includes API/telemetry-ready fields without implementing Phase 5 handlers. | ✓ VERIFIED | State includes accepted/rejected counters, rejected reasons, best difficulty, pool difficulty, fallback, hashrate inputs, lifecycle, activity, and work-submission gate. |
| 10 | Mining job construction covers coinbase/extranonce hashing, merkle root calculation, queue behavior, clean-jobs clearing, and BM1366 output fields. | ✓ VERIFIED | `coinbase.rs`, `mining.rs`, `queue.rs`, and `mining_loop.rs` implement and test these paths. |
| 11 | Stratum job construction reuses Phase 3 BM1366 work/result types and does not construct raw ASIC frame bytes. | ✓ VERIFIED | `bitaxe-stratum` uses `Bm1366WorkFields`, `Bm1366JobId`, `Bm1366NonceResult`, and `Bm1366ValidJobIds`; searches for `JobFrame`, `CommandFrame`, and `diagnostic_job_frame` under `crates/bitaxe-stratum/src/v1` returned no matches. |
| 12 | Clean-jobs clears queued work and valid-job tracking so stale ASIC nonces cannot submit shares. | ✓ VERIFIED | `MiningWorkQueue::enqueue_work` calls `clear_jobs()` for clean jobs, and `clear_jobs()` clears the queue, valid jobs, and active work. Tests cover clean-job replacement and invalidated nonce results. |
| 13 | Firmware-visible mining-loop status remains fail-closed unless explicit ASIC/safety/hardware evidence gates allow work submission. | ✓ VERIFIED | `MiningLoopGate::default()` blocks with `hardware_evidence_ack_missing`; `main.rs` calls `publish_mining_loop_blocked_status("hardware_evidence_ack_missing")`. |
| 14 | Phase 4 parity checklist distinguishes pure/fake-pool evidence from live hardware mining proof. | ✓ VERIFIED | STR-001 through STR-007 are `implemented` or `deferred`, not `verified`; STR-008 records live smoke/soak as `not-started | pending`. |
| 15 | Phase 4 evidence records run or skipped status for Ultra 205 smoke/soak criteria. | ✓ VERIFIED | Evidence file records pure/fake-pool commands as passed and hardware smoke/soak as `not run - hardware evidence pending`. |
| 16 | Fail-closed behavior remains in firmware main and live mining is not overclaimed. | ✓ VERIFIED | `rg` found no live socket/work-submission patterns in `firmware/bitaxe/src/main.rs`; evidence states live pool socket use and real BM1366 production work submission remain disabled. |

**Score:** 16/16 truths verified

### Deferred Items

Items not yet met but explicitly addressed in later milestone phases.

| # | Item | Addressed In | Evidence |
| --- | --- | --- | --- |
| 1 | Actual AxeOS API/WebSocket telemetry handlers expose mining runtime state to users. | Phase 5 | Phase 5 goal and success criteria cover upstream-compatible API, log, `/api/ws`, `/api/ws/live`, statistics, and mining-state surfaces. |

### Required Artifacts

| Artifact | Expected | Status | Details |
| --- | --- | --- | --- |
| `crates/bitaxe-stratum/src/v1/messages.rs` | Typed Stratum v1 client/server parser and serializers | ✓ VERIFIED | Substantive parser/serializer code plus protocol tests. |
| `crates/bitaxe-stratum/src/v1/fake_pool.rs` | Deterministic fake-pool transcript runner | ✓ VERIFIED | Transcript runner mutates typed runtime state and rejects unexpected clients. |
| `crates/bitaxe-stratum/src/v1/state.rs` | Mining runtime lifecycle, counters, hashrate, activity, and gate state | ✓ VERIFIED | Defaults and mutation methods tested. |
| `crates/bitaxe-stratum/src/v1/coinbase.rs` | Coinbase/extranonce/merkle helpers | ✓ VERIFIED | Strict hex parsing, double SHA-256, merkle root, little-endian extranonce generation. |
| `crates/bitaxe-stratum/src/v1/mining.rs` | Stratum notify/extranonce to BM1366 work bridge | ✓ VERIFIED | Produces typed `Bm1366WorkFields`; non-zero version mask fails closed. |
| `crates/bitaxe-stratum/src/v1/queue.rs` | Bounded queue and valid-job tracking | ✓ VERIFIED | Capacity 12, FIFO, clean-jobs invalidation, active-work tracking. |
| `crates/bitaxe-stratum/src/v1/mining_loop.rs` | Fail-closed mining-loop gate and guarded planning | ✓ VERIFIED | Default block, ready gate, dispatch plan, valid nonce result conversion. |
| `firmware/bitaxe/src/main.rs` | Firmware-visible blocked mining-loop status | ✓ VERIFIED | Calls status publisher after ASIC boot gate; no live mining dispatch found. |
| `firmware/bitaxe/src/asic_adapter/status.rs` | Status log helper | ✓ VERIFIED | Logs `mining_loop_status=blocked reason={reason} work_submission=disabled`. |
| `docs/parity/checklist.md` | STR/STAT rows with implementation/evidence pointers | ✓ VERIFIED | STR-001 through STR-007 and STAT-004 point at Phase 4 evidence; STR-008 remains pending. |
| `docs/parity/evidence/phase-04-stratum-v1-mining-loop.md` | Phase 4 evidence and smoke/soak criteria | ✓ VERIFIED | Records pure checks and explicitly does not claim live mining proof. |

### Key Link Verification

| From | To | Via | Status | Details |
| --- | --- | --- | --- | --- |
| `src/lib.rs` | `src/v1.rs` | `pub mod v1` | ✓ WIRED | Export present; runtime status is `ActiveV1Core`. |
| `src/v1.rs` | v1 child modules | module facade | ✓ WIRED | Exports `messages`, `state`, `fake_pool`, `coinbase`, `mining`, `queue`, and `mining_loop`; no `v1/mod.rs`. |
| `messages.rs` | protocol fixtures | method tests and fixture metadata | ✓ WIRED | Fixtures are Bazel compile data; method tests cover parser/serializer behavior. |
| `fake_pool.rs` | `state.rs` | typed state mutations | ✓ WIRED | Applies typed server messages to lifecycle, fallback, counters, and gate state. |
| `mining.rs` | `bitaxe-asic` work/result types | typed BM1366 bridge | ✓ WIRED | Uses `Bm1366WorkFields`, `Bm1366JobId`, and `Bm1366NonceResult`. |
| `queue.rs` | `bitaxe-asic` valid jobs | stale nonce prevention | ✓ WIRED | Uses `Bm1366ValidJobIds` and active-work tracking. |
| `mining_loop.rs` | `firmware/bitaxe/src/main.rs` | shared blocked reason/status | ✓ WIRED | Firmware publishes `hardware_evidence_ack_missing`. |
| `checklist.md` | Phase 4 evidence file | STR/STAT evidence links | ✓ WIRED | `just parity` passed with `validation_errors: none`. |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
| --- | --- | --- | --- | --- |
| `messages.rs` | `StratumV1ServerMessage` | JSON input through `parse_server_message` | Yes, typed values or typed errors | ✓ FLOWING |
| `fake_pool.rs` | `MiningRuntimeState` | Typed transcript events and responses | Yes, lifecycle/counters/fallback mutate from typed outcomes | ✓ FLOWING |
| `mining.rs` | `Bm1366WorkFields` / `ShareSubmission` | `MiningNotify`, `ExtranonceAssignment`, `Bm1366NonceResult` | Yes, strict hashing/hex conversion and job-id validation | ✓ FLOWING |
| `queue.rs` | queued/active work and valid jobs | `MiningWork` enqueue/dequeue/clean-jobs | Yes, clean-jobs clears queue, valid jobs, and active work | ✓ FLOWING |
| `mining_loop.rs` | dispatch/share plan | gate + queue + optional nonce result | Yes, only ready gate emits typed dispatch/share submission | ✓ FLOWING |
| `main.rs` | blocked status reason | constant string passed to status helper | Yes, firmware logs blocked status and no live mining | ✓ FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
| --- | --- | --- | --- |
| Stratum crate behavior | `cargo test -p bitaxe-stratum --all-features` | 40 tests passed | ✓ PASS |
| Bazel-visible Stratum tests | `bazel test //crates/bitaxe-stratum:tests` | passed, cached | ✓ PASS |
| Reference tree remains clean | `just verify-reference` | `reference clean: c1915b0a63bfabebdb95a515cedfee05146c1d50` | ✓ PASS |
| Parity checklist validity | `just parity` | `validation_errors: none` | ✓ PASS |
| Fixtures are parseable | Node JSON parse of three fixture files | protocol=13 cases, fake-pool=4 cases, mining-job=4 cases | ✓ PASS |
| Firmware main has no live mining dispatch | `rg "esp_transport|STRATUM_V1|submit_share|SendDiagnosticWork|SendWork|work_submission=enabled|socket" firmware/bitaxe/src/main.rs` | no matches | ✓ PASS |
| Stratum code does not construct raw ASIC frames | `rg "JobFrame|CommandFrame|diagnostic_job_frame|frame bytes|to_bytes" crates/bitaxe-stratum/src/v1` | no matches | ✓ PASS |
| Live smoke/soak not overclaimed | `rg "STR-00[6-8].*verified|Live mining.*verified|hardware-smoke.*verified" docs/parity/...` | no matches | ✓ PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
| --- | --- | --- | --- | --- |
| STR-01 | 04-01, 04-04 | Stratum v1 parsing/serialization | ✓ SATISFIED | Typed messages, parser, serializer, fixtures, tests. |
| STR-02 | 04-02, 04-04 | Fake pool subscribe/authorize/notify/difficulty/submit flows | ✓ SATISFIED | `FakePoolTranscript` tests accepted/rejected/reconnect/fallback/error paths. |
| STR-03 | 04-03, 04-04 | Mining job, coinbase, extranonce, queue integration | ✓ SATISFIED | Coinbase/mining/queue modules and tests. |
| STR-04 | 04-02, 04-04 | Fallback/reconnect/error lifecycle | ✓ SATISFIED | Fake-pool disconnect/timeout/error behavior and checklist note that live socket adapter is not claimed. |
| STR-05 | 04-02, 04-04 | Share counters, difficulty/hashrate, lifecycle state | ✓ SATISFIED | `MiningRuntimeState` and runtime tests; Phase 5 handler exposure deferred. |
| STR-06 | 04-03, 04-04 | First mining loop connects config, Stratum, BM1366, result parsing, safety gates | ✓ SATISFIED | Guarded planner uses `Ultra205Defaults`, typed work queue, BM1366 work/result types, and fail-closed gate. |
| STR-07 | 04-04 | Hardware smoke/soak criteria | ✓ SATISFIED | Evidence file records criteria and skipped live status; live proof not claimed. |

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| --- | --- | --- | --- | --- |
| `crates/bitaxe-stratum/src/v1/mining.rs` | 167 | `version rolling work generation is not implemented` | ℹ️ Info | Intentional fail-closed limitation: non-zero version masks return `InvalidField` and are covered by a unit test. |
| `firmware/bitaxe/src/asic_adapter.rs` / `fake_pool.rs` | multiple | empty match arms | ℹ️ Info | Inspected; these are intentional no-op branches for continue/unsupported status events, not user-visible stubs. |

### Human Verification Required

None for Phase 04 pass. Live Ultra 205 mining smoke and soak are explicitly pending and not claimed by this phase.

### Gaps Summary

No blocking gaps found. The phase achieves the pure Stratum v1, fake-pool, mining-job, queue, guarded-loop, firmware-blocked-status, and evidence-boundary goals. Actual API/WebSocket surfacing of the runtime state is deferred to Phase 5, and live mining smoke/soak remains pending without being overclaimed.

Local guidance materially considered: `AGENTS.md`, `AGENTS.bright-builds.md`, `standards-overrides.md`, `standards/index.md`, `standards/core/architecture.md`, `standards/core/code-shape.md`, `standards/core/verification.md`, `standards/core/testing.md`, and `standards/languages/rust.md`. No project skills were present under `.claude/skills/` or `.agents/skills/`.

---

_Verified: 2026-06-27T15:43:05Z_
_Verifier: the agent (gsd-verifier)_
