# Phase 04: stratum-v1-and-first-mining-loop - Research

**Researched:** 2026-06-27
**Domain:** Stratum v1 protocol, deterministic fake pool, BM1366 mining work loop, Ultra 205 evidence gates
**Confidence:** HIGH for pure protocol and fake-pool planning, MEDIUM for live mining smoke because hardware/pool evidence must be captured separately

<user_constraints>
## User Constraints (from CONTEXT.md)

Source for this section: [VERIFIED: .planning/phases/04-stratum-v1-and-first-mining-loop/04-CONTEXT.md]

### Locked Decisions

- Implement Stratum v1 parsing and serialization in `crates/bitaxe-stratum` as a pure Rust core before firmware socket behavior owns side effects.
- Build deterministic fake-pool scenarios for subscribe, authorize, notify, set-difficulty, submit accepted/rejected, malformed/error responses, reconnect, fallback, and clean-jobs clearing.
- Convert notify/extranonce/difficulty state into typed BM1366 work fields and valid-job tracking by reusing Phase 3 ASIC types.
- Keep firmware as a thin imperative shell for sockets, TLS, FreeRTOS tasks, timers, logging, NVS reads, and hardware adapters.
- Require explicit ASIC, power, thermal, safety, and hardware-evidence gates before live production work submission is enabled.
- Expose typed mining runtime state for accepted/rejected shares, rejected reasons, pool difficulty, share difficulty inputs, lifecycle status, fallback-active status, and mining paused/safe-blocked status for Phase 5 reuse.
- Do not implement full Stratum v2, AxeOS HTTP/WebSocket handlers, safety controllers, OTA, or broader board verification in Phase 4.

</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
| --- | --- | --- |
| STR-01 | Stratum v1 message parsing and serialization match upstream-compatible request and response behavior. | Use typed JSON-RPC request/response/notification types for upstream methods in `stratum_api.h` and `stratum_api.c`. [VERIFIED: reference/esp-miner/components/stratum/include/stratum_api.h; reference/esp-miner/components/stratum/stratum_api.c] |
| STR-02 | Subscribe, authorize, notify, set-difficulty, and submit flows work against a deterministic fake pool harness. | Model pool transcripts as pure send/receive events and fake timing/state outcomes before firmware sockets. [VERIFIED: .planning/phases/04-stratum-v1-and-first-mining-loop/04-CONTEXT.md] |
| STR-03 | Mining job construction, coinbase decoding, extranonce handling, and work queue integration match reference-observable behavior. | Use `mining.c`, `coinbase_decoder.c`, `utils.c`, and `work_queue.c` as canonical behavior references. [VERIFIED: reference/esp-miner/components/stratum/mining.c; reference/esp-miner/main/work_queue.c] |
| STR-04 | Pool socket lifecycle, fallback pool behavior, reconnect behavior, and error logging match upstream user-visible behavior. | Firmware sockets should be planned as an adapter around pure lifecycle decisions; `stratum_socket.c` owns upstream socket/TCP details. [VERIFIED: reference/esp-miner/components/stratum/stratum_socket.c] |
| STR-05 | Accepted shares, rejected shares, share difficulty, hashrate inputs, and pool result counters update consistently across mining, API, and telemetry surfaces. | Add a shared typed runtime model now, leaving HTTP/WebSocket mapping to Phase 5. [VERIFIED: reference/esp-miner/main/system.h; reference/esp-miner/main/http_server/system_api_json.c] |
| STR-06 | The first Ultra 205 mining loop connects config, Stratum v1, BM1366 work dispatch, result parsing, and global state without bypassing safety gates. | Reuse Phase 3 `Bm1366InitPlan`, `Bm1366Command`, `Bm1366WorkFields`, and `Bm1366ValidJobIds`; firmware must fail closed when gates are absent. [VERIFIED: crates/bitaxe-asic/src/bm1366/init_plan.rs; crates/bitaxe-asic/src/bm1366/work.rs] |
| STR-07 | Mining parity has hardware-smoke and soak criteria recording command, board, port, firmware commit, reference commit, logs, observed result, and conclusion. | Keep fake-pool evidence separate from live mining evidence files and do not mark live mining verified without board-named logs. [VERIFIED: docs/adr/0012-parity-verification-evidence.md; docs/parity/checklist.md] |

</phase_requirements>

## Summary

Phase 4 should be planned as four boundaries:

1. A pure Stratum v1 JSON-RPC protocol core in `crates/bitaxe-stratum`.
2. A deterministic fake-pool and mining state harness that proves user-visible lifecycle and counter behavior without live sockets.
3. A pure mining job/work queue bridge that turns Stratum notify/extranonce/difficulty state into Phase 3 BM1366 work fields and share-submission decisions.
4. A firmware mining-loop adapter that stays fail-closed unless ASIC init, safety, and evidence gates explicitly allow live work submission.

The highest-risk planning issue is overclaiming. Parser, serializer, fake-pool, job construction, queue, and share-decision tests can verify pure behavior. Live mining parity still requires Ultra 205 smoke or soak evidence before checklist rows can become `verified`. [VERIFIED: docs/adr/0012-parity-verification-evidence.md]

## Project Constraints

- The reference tree under `reference/esp-miner` is read-only evidence; do not modify it. [VERIFIED: AGENTS.md]
- Use functional core plus imperative shell: pure protocol/mining decisions belong in crates, ESP-IDF sockets/tasks/UART/GPIO/logging belong in firmware adapters. [VERIFIED: standards/core/architecture.md]
- Preserve GPL provenance: reference-derived fixtures need source path, pinned reference commit, license posture, and derivation notes. [VERIFIED: docs/adr/0013-mit-first-with-gpl-guardrails.md; PROVENANCE.md]
- Hardware-control and mining behavior cannot be marked verified without evidence. [VERIFIED: docs/adr/0012-parity-verification-evidence.md]
- Rust tests must remain focused and use Arrange/Act/Assert when helpful. [VERIFIED: standards/core/testing.md; standards/languages/rust.md]
- Before final commit in this Rust repo, run `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features`. [VERIFIED: AGENTS.md]

## Standard Stack

| Component | Purpose | Planning Recommendation |
| --- | --- | --- |
| `serde` / `serde_json` | JSON-RPC and fixture parsing. | Add to `bitaxe-stratum` through workspace dependencies; use typed deserialization at protocol boundaries. [VERIFIED: Cargo.toml] |
| `sha2` | Coinbase, merkle, header, and share-difficulty hashing. | Workspace already has `sha2 = 0.11.0`; use it instead of hand-rolling SHA-256. [VERIFIED: Cargo.toml] |
| `thiserror` | Library errors. | Add to `bitaxe-stratum` for parse, protocol, queue, fake-pool, and job errors. [VERIFIED: AGENTS.md] |
| `bitaxe-config` | Pool defaults, Stratum settings, TLS, protocol, ports, fallback defaults. | Depend on it when typed config values are needed; do not duplicate defaults. [VERIFIED: crates/bitaxe-config/src/defaults.rs; crates/bitaxe-config/src/nvs.rs] |
| `bitaxe-asic` | BM1366 work fields, job IDs, result parsing, valid-job tracking, adapter actions. | Reuse Phase 3 types; Stratum must not construct raw ASIC packet bytes. [VERIFIED: crates/bitaxe-asic/src/bm1366/work.rs; crates/bitaxe-asic/src/bm1366/result.rs] |

## Architecture Patterns

### Recommended Project Structure

```text
crates/bitaxe-stratum/src/
├── lib.rs
├── error.rs
├── jsonrpc.rs
├── v1.rs
└── v1/
    ├── messages.rs
    ├── client.rs
    ├── fake_pool.rs
    ├── mining.rs
    ├── coinbase.rs
    ├── queue.rs
    └── state.rs

crates/bitaxe-stratum/fixtures/v1/
├── protocol-cases.json
├── fake-pool-transcripts.json
└── mining-job-cases.json
```

Use `v1.rs` plus `v1/` rather than `v1/mod.rs` to follow the repo Rust module rule. [VERIFIED: standards/languages/rust.md]

### Pattern 1: JSON-RPC Boundary Types

Define explicit message types rather than passing raw `serde_json::Value` after parsing:

```rust
pub enum StratumV1ServerMessage {
    Notify(MiningNotify),
    SetDifficulty(PoolDifficulty),
    SetExtranonce(ExtranoncePrefix),
    SetVersionMask(VersionMask),
    Result(StratumResponse),
    Error(StratumError),
}
```

This maps upstream `StratumApiV1Message` concepts without copying the C struct shape. [VERIFIED: reference/esp-miner/components/stratum/include/stratum_api.h]

### Pattern 2: Fake Pool Transcript

Use a transcript model with expected client sends and scripted pool receives:

```rust
pub enum FakePoolEvent {
    ExpectClient(StratumV1ClientMessage),
    SendServer(StratumV1ServerMessage),
    Disconnect,
    Timeout,
}
```

This lets STR-02 and STR-04 cover lifecycle behavior deterministically without a real pool.

### Pattern 3: Mining Job Bridge

Keep job construction pure:

```rust
pub struct MiningWork {
    pub stratum_job_id: StratumJobId,
    pub asic_job_id: Bm1366JobId,
    pub fields: Bm1366WorkFields,
    pub pool_difficulty: PoolDifficulty,
}
```

The bridge should consume parsed notify/extranonce/version-mask/difficulty state and produce BM1366 work fields plus valid-job tracking. Raw job frames remain in `bitaxe-asic`. [VERIFIED: crates/bitaxe-asic/src/bm1366/work.rs]

### Pattern 4: Runtime State As Shared Core

Create a typed state model that Phase 5 can read:

```rust
pub struct MiningRuntimeState {
    pub lifecycle: PoolLifecycleStatus,
    pub counters: ShareCounters,
    pub maybe_pool_difficulty: Option<PoolDifficulty>,
    pub fallback_active: bool,
    pub work_submission: WorkSubmissionGate,
}
```

Use `maybe_` naming for optional internal fields per repo standards. [VERIFIED: standards/languages/rust.md]

## Don't Hand-Roll

| Problem | Do Not Build | Use Instead |
| --- | --- | --- |
| SHA-256 / double-SHA-256 | Custom hash implementation | `sha2` workspace dependency |
| Stratum settings/defaults | New hardcoded pool defaults | `bitaxe-config` defaults/NVS schema |
| ASIC work frames | Raw frame bytes in Stratum | `bitaxe-asic::bm1366::{work, command, result}` |
| Firmware sockets in pure code | ESP-IDF transport in `bitaxe-stratum` | Pure transcript/state plus firmware adapter |
| Hardware parity proof | Fake-pool tests as live proof | `docs/parity/evidence/*.md` hardware-smoke/soak records |

## Common Pitfalls

### Pitfall 1: Treating Stratum JSON As Untyped Data

Raw JSON values make it easy to accept malformed notify params, IDs, difficulty, or extranonce lengths. Parse once into domain types and reject invalid shapes at the boundary. [VERIFIED: standards/core/architecture.md]

### Pitfall 2: Forgetting Clean-Jobs Clears Work

Upstream notify includes `clean_jobs`; when true, queued work and valid-job tracking need clearing so stale ASIC nonces do not submit against old jobs. [VERIFIED: reference/esp-miner/components/stratum/include/stratum_api.h; reference/esp-miner/main/system.h]

### Pitfall 3: Duplicating ASIC Job-ID Semantics

BM1366 job IDs advance by 8 and use masked lookup keys. Reuse `Bm1366JobId` and `Bm1366ValidJobIds`; do not create a generic incrementing Stratum-owned ASIC ID. [VERIFIED: crates/bitaxe-asic/src/bm1366/work.rs; crates/bitaxe-asic/src/bm1366/result.rs]

### Pitfall 4: Collapsing Fake Pool And Firmware Socket Behavior

Fake-pool transcript behavior proves the state machine, not ESP-IDF DNS, TCP, TLS, or reconnect timing. Keep firmware socket smoke separate. [VERIFIED: reference/esp-miner/components/stratum/stratum_socket.c]

### Pitfall 5: Overclaiming API/Telemetry Compatibility

Phase 4 should create state fields used later by API/telemetry, but route handlers and WebSocket payload parity belong to Phase 5. [VERIFIED: .planning/ROADMAP.md]

### Pitfall 6: Enabling Live Work Before Safety Gates

Pure protocol correctness is not enough to mine. Firmware must require ASIC init and hardware/safety evidence gates before production work submission. [VERIFIED: crates/bitaxe-asic/src/bm1366/init_plan.rs; docs/adr/0012-parity-verification-evidence.md]

## Planning Guidance

Recommended plan split:

1. **Protocol and fixture foundation:** Replace `StratumRuntimeStatus::DeferredUntilPhase4` with typed errors, JSON-RPC IDs, Stratum v1 client/server messages, serializers, parsers, and provenance fixtures.
2. **Fake pool and lifecycle state:** Add deterministic transcript runner, subscribe/authorize/difficulty/notify/submit flows, reconnect/fallback decisions, and runtime counters/status.
3. **Mining job and queue bridge:** Implement coinbase/extranonce/merkle/job construction, bounded queue semantics, clean-jobs clearing, valid-job tracking, and BM1366 work-field output.
4. **Firmware gated mining-loop shell:** Add adapter-facing mining-loop planning/status that can run fake or safe-gated live work and records fail-closed statuses when gates are missing.
5. **Evidence and checklist:** Update `docs/parity/checklist.md` STR/STAT rows and add Phase 4 evidence records, distinguishing unit/golden/fake-pool evidence from live hardware proof.

The planner may merge these if task count gets too high, but every STR requirement must appear in at least one plan frontmatter `requirements` entry.

## Validation Architecture

### Test Framework

| Property | Value |
| --- | --- |
| Framework | Rust unit tests through Cargo and Bazel `rust_test`. |
| Config file | `crates/bitaxe-stratum/BUILD.bazel`, workspace `Cargo.toml`, `MODULE.bazel`. |
| Quick run command | `cargo test -p bitaxe-stratum --all-features` |
| Full suite command | `cargo test --all-features` plus `bazel test //crates/bitaxe-stratum:tests //crates/bitaxe-asic:tests //crates/bitaxe-config:tests //crates/bitaxe-core:tests` |
| Estimated runtime | ~60-180 seconds for targeted tests; full Rust pre-commit checks are longer. |

### Phase Requirements -> Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
| --- | --- | --- | --- | --- |
| STR-01 | Stratum v1 parser/serializer for subscribe, authorize, configure, suggest difficulty, extranonce subscribe, notify, set difficulty, set extranonce, version mask, submit, result/error. | unit/golden | `cargo test -p bitaxe-stratum stratum_v1_protocol --all-features` | missing, Wave 0 needed. |
| STR-02 | Deterministic fake-pool subscribe/authorize/notify/difficulty/submit/fallback/reconnect/error flows. | unit/fake integration | `cargo test -p bitaxe-stratum fake_pool --all-features` | missing, Wave 0 needed. |
| STR-03 | Coinbase decoding, extranonce generation, merkle root, mining job construction, queue integration. | unit/golden | `cargo test -p bitaxe-stratum mining_job --all-features` | missing, Wave 0 needed. |
| STR-04 | Pool lifecycle, fallback, reconnect, and error logging/status decisions. | unit/fake integration | `cargo test -p bitaxe-stratum pool_lifecycle --all-features` | missing, Wave 0 needed. |
| STR-05 | Share counters, rejected reasons, difficulty/hashrate inputs, fallback-active and lifecycle state. | unit | `cargo test -p bitaxe-stratum runtime_state --all-features` | missing, Wave 0 needed. |
| STR-06 | Gated first Ultra 205 mining-loop bridge to BM1366 work dispatch/result parsing without bypassing safety gates. | unit plus firmware build/smoke | `cargo test -p bitaxe-stratum mining_loop --all-features && cargo test -p bitaxe-asic --all-features` | pure bridge missing; hardware smoke deferred until evidence gate. |
| STR-07 | Hardware-smoke/soak evidence criteria and checklist updates. | static/evidence review | `rg -n "STR-00[1-7]|phase-04|mining" docs/parity docs/parity/evidence` | evidence file missing, plan should add skipped or run evidence record. |

### Sampling Rate

- **After every task commit:** Run the plan-specific targeted `cargo test -p bitaxe-stratum ... --all-features` command.
- **After every plan wave:** Run `cargo test -p bitaxe-stratum --all-features` and touched dependency crate tests.
- **Before phase verification:** Run the repo Rust pre-commit sequence required by `AGENTS.md`, plus `just test`, `just parity`, and `just verify-reference` when touched paths affect parity/reference claims.

### Wave 0 Gaps

- [ ] `crates/bitaxe-stratum/src/error.rs` and Stratum v1 module graph.
- [ ] `crates/bitaxe-stratum/fixtures/v1/` metadata-rich protocol and fake-pool fixtures.
- [ ] `crates/bitaxe-stratum/BUILD.bazel` updated with new source files, dependencies, and fixture compile data.
- [ ] Mining runtime state and queue tests that can run without firmware or real pool.
- [ ] Phase 4 evidence file under `docs/parity/evidence/` defining smoke/soak outcome or skipped hardware proof.

## Security Domain

| Pattern | STRIDE | Mitigation |
| --- | --- | --- |
| Malformed pool JSON influences mining state | Tampering | Typed parser rejects malformed method/params/IDs and never mutates state from invalid messages. |
| Pool or fake transcript causes stale share submission | Tampering / Repudiation | Clean-jobs clears queue and valid-job tracking; stale ASIC result cannot produce a submit command. |
| Credentials or pool details leak in evidence | Information Disclosure | Evidence should redact sensitive pool username/password and record only safe pool target details. |
| Work submission enabled without safety gates | Elevation of Privilege / DoS | Firmware checks explicit gate tokens before production work dispatch and logs fail-closed reason otherwise. |
| GPL reference expression copied into MIT source | Supply Chain | Use breadcrumbs and reference-derived fixtures with license posture; keep Rust independently structured. |

## Sources

### Primary (HIGH confidence)

- `.planning/phases/04-stratum-v1-and-first-mining-loop/04-CONTEXT.md` - locked decisions and canonical refs.
- `.planning/REQUIREMENTS.md` - STR-01 through STR-07.
- `.planning/ROADMAP.md` - Phase 4 success criteria and verification expectations.
- `reference/esp-miner/components/stratum/stratum_api.c` and `include/stratum_api.h` - Stratum v1 JSON-RPC methods, message shape, IDs, timings, parser/serializer boundary.
- `reference/esp-miner/components/stratum/mining.c` and `include/mining.h` - coinbase hash, merkle root, BM job construction, extranonce, nonce testing.
- `reference/esp-miner/components/stratum/coinbase_decoder.c` and tests - coinbase output decoding.
- `reference/esp-miner/components/stratum/utils.c` and tests - hex/bin, endian, hash, difficulty helpers.
- `reference/esp-miner/main/work_queue.c` and `work_queue.h` - queue size, enqueue/dequeue/timeout/clear behavior.
- `reference/esp-miner/main/system.c` and `system.h` - share counters and clean-jobs notifications.
- `reference/esp-miner/main/http_server/system_api_json.c` - later API-visible runtime fields.
- `crates/bitaxe-asic/src/bm1366/*` - existing BM1366 command, work, result, and init gate surfaces.
- `crates/bitaxe-config/src/defaults.rs`, `nvs.rs`, `settings.rs`, `validation.rs` - Stratum defaults and settings validation.
- `docs/adr/0012-parity-verification-evidence.md`, `docs/adr/0013-mit-first-with-gpl-guardrails.md`, `PROVENANCE.md` - evidence and provenance policy.

## RESEARCH COMPLETE
