---
generated_by: gsd-discuss-phase
lifecycle_mode: yolo
phase_lifecycle_id: 04-2026-06-27T13-17-33
generated_at: 2026-06-27T13:17:33.403Z
---

# Phase 4: Stratum V1 And First Mining Loop - Context

**Gathered:** 2026-06-27
**Status:** Ready for planning
**Mode:** Yolo

<domain>
## Phase Boundary

Phase 4 delivers deterministic Stratum v1 behavior and the first Ultra 205 mining loop boundary. The phase replaces the current deferred Stratum marker with pure parsing, serialization, fake-pool, job construction, work queue, result-submission, and pool lifecycle logic, then connects that logic to the existing BM1366 typed command/result boundary without bypassing Phase 3 fail-closed safety gates.

This phase may produce an evidence-backed first mining-loop smoke path, but only where required Ultra 205 hardware evidence exists. Stratum v2 completeness, AxeOS API handlers, WebSocket telemetry, safety-controller enablement, OTA, and broader board/ASIC verification remain later phases.

</domain>

<decisions>
## Implementation Decisions

### Stratum V1 Protocol Surface

- **D-01:** Implement Stratum v1 parsing and serialization in `crates/bitaxe-stratum` as a pure Rust protocol core. Cover subscribe, authorize, notify, set-difficulty, set-extranonce, set-version-mask, submit, result, error, fallback, and reconnect-relevant message shapes before any firmware socket adapter owns behavior.
- **D-02:** Parse raw JSON at the boundary into typed protocol messages and domain values. Avoid passing unchecked strings, arrays, IDs, difficulty values, extranonce fields, or hex payloads deep into mining logic.
- **D-03:** Keep Stratum v2 represented as deferred or scoped config data only. Phase 4 targets Stratum v1 first-loop parity; full Stratum v2 behavior remains V2 or a later explicit scope.

### Fake Pool And Deterministic Coverage

- **D-04:** Build a deterministic fake-pool harness in host-testable code instead of requiring a real mining pool for protocol correctness. The fake pool should cover subscribe, authorize, notify, set-difficulty, submit accepted, submit rejected, malformed/error responses, reconnect, fallback, and clean-jobs queue clearing.
- **D-05:** Golden fixtures should be reference-derived when practical and record provenance. Use upstream test cases under `reference/esp-miner/components/stratum/test/` and self-test mock messages as source evidence, but keep Rust implementations independently structured and MIT-first unless fixture data or ported expression is intentionally labeled.
- **D-06:** Treat fake-pool tests as protocol and state evidence, not hardware mining proof. Hardware-smoke or soak evidence is still required before mining-loop parity is marked verified.

### Mining Job And Work Queue Integration

- **D-07:** Model mining job construction as pure transformations from Stratum notify/extranonce/difficulty state into BM1366 work fields and valid-job tracking. Reuse Phase 3 `Bm1366WorkFields`, `Bm1366WorkPayload`, `Bm1366JobId`, `Bm1366ValidJobIds`, and parsed result types rather than reintroducing raw ASIC frame logic in Stratum code.
- **D-08:** Implement queue behavior in a host-testable module, likely under `crates/bitaxe-core` or `crates/bitaxe-stratum` depending on ownership discovered during planning. Queue decisions must cover enqueue, dequeue, timeout or empty behavior, clean-jobs clearing, valid-job invalidation, and bounded capacity/backpressure if needed.
- **D-09:** Preserve Phase 3 boundaries: Stratum code may ask for typed diagnostic or mining work dispatch, but raw BM1366 packet bytes, UART, reset, baud, and direct ASIC side effects stay inside `crates/bitaxe-asic` and `firmware/bitaxe` adapters.

### Firmware First Mining Loop

- **D-10:** Firmware integration should be a thin imperative shell around the pure Stratum and ASIC cores. Network sockets, TLS if enabled, FreeRTOS/task orchestration, timers, logging, and NVS/config reads belong in firmware adapters; protocol, queue, job, and counter decisions belong in pure crates.
- **D-11:** The first live Ultra 205 mining-loop path must require explicit safety and evidence gates before enabling production work submission. Missing chip-detect, power, thermal, safety, or hardware-evidence acknowledgment should fail closed with visible logs/status and no mining.
- **D-12:** A first-loop smoke may use a controlled/public pool only after the fake-pool suite and ASIC gates pass. Evidence must record command, board, port, firmware commit, reference commit, pool target when safe to disclose, relevant logs, accepted/rejected share result, reconnect/fallback observations when exercised, and conclusion.

### Counters, Runtime State, And Later API Surfaces

- **D-13:** Add a typed mining runtime state model for accepted shares, rejected shares, rejected reasons, pool difficulty, share difficulty/hashrate inputs, pool lifecycle status, fallback-active status, and mining paused/active/safe-blocked status.
- **D-14:** Keep user-facing API and WebSocket handler implementation out of Phase 4 unless the planner needs minimal model fields for future Phase 5 integration. Phase 4 should expose reusable state/model types so Phase 5 can map them into AxeOS-compatible responses without duplicating mining logic.
- **D-15:** Ensure status and logs do not overclaim parity. Rows can advance to `implemented` or `verified` only according to evidence type: unit/golden/fake-pool evidence for pure behavior, hardware-smoke or soak evidence for live mining behavior.

### the agent's Discretion

The agent may choose exact Rust module names, error enum names, fixture formats, fake-pool transcript schema, bounded queue representation, and plan count. Those choices must preserve the functional-core/imperative-shell boundary, keep raw socket and ASIC effects in adapters, use typed domain values, and preserve the existing safety/evidence gate before any live mining claims.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase Scope And Requirements

- `.planning/ROADMAP.md` - Phase 4 goal, dependencies, success criteria, verification expectations, and research flags.
- `.planning/REQUIREMENTS.md` - STR-01 through STR-07 plus related ASIC, API, safety, and evidence requirements.
- `.planning/PROJECT.md` - Ultra 205 BM1366 first target, architecture constraints, provenance constraints, and seed layout.
- `.planning/STATE.md` - Completed Phase 3 decisions, safety evidence blockers, and current milestone state.
- `.planning/phases/01-foundation-and-gamma-601-boot-log/01-CONTEXT.md` - Safe boot/log boundary and disabled mining/hardware-control decisions.
- `.planning/phases/02-ultra-205-config-and-nvs-model/02-CONTEXT.md` - Ultra 205 Stratum defaults, NVS settings, fallback pool settings, and validation boundary.
- `.planning/phases/03-bm1366-asic-protocol-and-safe-initialization/03-CONTEXT.md` - BM1366 semantic command/observation boundary, init gate, and hardware evidence rules.

### Existing Rust Integration Points

- `crates/bitaxe-stratum/src/lib.rs` - Current deferred Stratum placeholder to replace with Phase 4 pure protocol and mining-loop modules.
- `crates/bitaxe-stratum/BUILD.bazel` - Bazel target that must expose new Stratum sources, fixtures, and tests.
- `crates/bitaxe-config/src/defaults.rs` - Ultra 205 primary/fallback pool defaults and Stratum defaults.
- `crates/bitaxe-config/src/nvs.rs` - Stratum NVS keys, fallback keys, protocol migration, and REST/API field-name mapping.
- `crates/bitaxe-config/src/settings.rs` - Existing settings update mapping for Stratum-related fields.
- `crates/bitaxe-config/src/validation.rs` - Stratum protocol, TLS, port, and related validation types.
- `crates/bitaxe-asic/src/bm1366/work.rs` - BM1366 work payload, job ID, and diagnostic frame types to reuse for mining jobs.
- `crates/bitaxe-asic/src/bm1366/result.rs` - BM1366 result parsing and valid-job tracking to reuse for share submission decisions.
- `crates/bitaxe-asic/src/bm1366/command.rs` - Typed adapter actions and BM1366 command boundary.
- `crates/bitaxe-asic/src/bm1366/init_plan.rs` - Full-init and evidence-gated initialized-no-mining state that Phase 4 must not bypass.
- `firmware/bitaxe/src/asic_adapter.rs` - Current firmware interpreter for typed ASIC adapter actions and chip-detect gate.
- `firmware/bitaxe/src/asic_adapter/status.rs` - Visible ASIC/mining-disabled status logging to extend or preserve.
- `crates/bitaxe-core/src/lib.rs` - Existing safe-state and runtime identity types that Phase 4 may evolve into a mining state model.
- `docs/parity/checklist.md` - STR rows, work queue row, STAT/API-adjacent rows, and evidence status rules to update.

### Upstream Stratum And Mining Reference Files

- `reference/esp-miner/components/stratum/stratum_api.c` - Stratum v1 JSON parse/serialize behavior and message handling.
- `reference/esp-miner/components/stratum/include/stratum_api.h` - Upstream Stratum method, notify, result, extranonce, and state structs.
- `reference/esp-miner/components/stratum/stratum_socket.c` - Socket lifecycle, DNS/connect behavior, TCP options, reconnect/fallback-relevant behavior.
- `reference/esp-miner/components/stratum/include/stratum_socket.h` - Socket boundary types and connection info.
- `reference/esp-miner/components/stratum/mining.c` - Coinbase hashing, merkle root, BM job construction, extranonce generation, and difficulty handling.
- `reference/esp-miner/components/stratum/include/mining.h` - Upstream mining notify and BM job struct boundary.
- `reference/esp-miner/components/stratum/coinbase_decoder.c` - Coinbase output decoding behavior.
- `reference/esp-miner/components/stratum/include/coinbase_decoder.h` - Coinbase decoder API.
- `reference/esp-miner/components/stratum/utils.c` - Hex/bin, endian, hash, nBits, and difficulty utilities used by Stratum/mining behavior.
- `reference/esp-miner/components/stratum/test/test_stratum_json.c` - Upstream JSON parsing expectations for fixtures.
- `reference/esp-miner/components/stratum/test/test_mining.c` - Upstream mining construction expectations for fixtures.
- `reference/esp-miner/components/stratum/test/test_coinbase_decoder.c` - Upstream coinbase decoder expectations for fixtures.
- `reference/esp-miner/components/stratum/test/test_utils.c` - Upstream utility behavior useful for golden tests.
- `reference/esp-miner/main/work_queue.c` and `reference/esp-miner/main/work_queue.h` - Queue behavior for Stratum work.
- `reference/esp-miner/main/tasks/protocol_coordinator.c` and `reference/esp-miner/main/tasks/protocol_coordinator.h` - Protocol selection and lifecycle coordination.
- `reference/esp-miner/main/system.c` and `reference/esp-miner/main/system.h` - Accepted/rejected share counters and clean-jobs handling.
- `reference/esp-miner/main/global_state.h` - Upstream global state fields for pool, shares, queue, and mining status.
- `reference/esp-miner/main/http_server/system_api_json.c` - Future Phase 5-visible fields such as share counters, pool difficulty, fallback status, and mining pause state.
- `reference/esp-miner/main/self_test/self_test.c` - Mock Stratum messages used by upstream self-test and useful fixture source material.

### Architecture, Evidence, And Policy

- `docs/adr/0001-device-user-parity.md` - Observable device-user parity definition.
- `docs/adr/0003-esp-idf-rust-production-stack.md` - ESP-IDF Rust stack and adapter boundary.
- `docs/adr/0005-read-only-reference-implementation.md` - Read-only upstream reference policy.
- `docs/adr/0006-parity-checklist-as-audit-evidence.md` - Checklist evidence policy.
- `docs/adr/0008-reference-breadcrumb-comments.md` - Breadcrumb placement policy.
- `docs/adr/0009-monorepo-package-layout.md` - Crate and firmware path ownership.
- `docs/adr/0010-axeos-api-and-asset-compatibility.md` - Later API/static asset compatibility boundary.
- `docs/adr/0012-parity-verification-evidence.md` - Verification evidence requirements and safety-critical hardware gate.
- `docs/adr/0013-mit-first-with-gpl-guardrails.md` - GPL guardrails for upstream-derived behavior and fixtures.
- `docs/adr/0014-pivot-to-ultra-205-bm1366-first-parity.md` - Ultra 205/BM1366 first parity target and deferred Gamma 601 scope.
- `PROVENANCE.md` - Provenance, SPDX, reference usage, and release review policy.
- `docs/parity/evidence/phase-03-ultra-205-bm1366-chip-detect.md` - Phase 3 chip-detect evidence status and hardware-verification limits.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- `crates/bitaxe-config/src/defaults.rs` and config fixtures already provide typed Ultra 205 primary/fallback pool defaults, suggested difficulty, TLS/cert flags, and extranonce-subscribe flags.
- `crates/bitaxe-config/src/nvs.rs`, `settings.rs`, and `validation.rs` already model Stratum-related NVS keys, REST names, legacy migrations, enum validation, ports, and TLS values.
- `crates/bitaxe-asic/src/bm1366/work.rs` already models BM1366 job IDs, payload layout, and diagnostic job frames.
- `crates/bitaxe-asic/src/bm1366/result.rs` already parses BM1366 job nonces/register reads and tracks valid job IDs.
- `crates/bitaxe-asic/src/bm1366/command.rs` already exposes typed adapter actions for frame writes and result reads.
- `firmware/bitaxe/src/asic_adapter.rs` already interprets typed BM1366 actions and fails closed on chip-detect/setup faults.

### Established Patterns

- Pure logic belongs in crates with fixture-backed tests, while firmware owns ESP-IDF effects and visible logging.
- Upstream-derived fixtures include source path, pinned reference commit, and license posture.
- Runtime firmware logs are explicit about disabled mining/work submission until evidence unlocks a later state.
- Parity checklist rows should distinguish implemented pure logic from verified hardware behavior.

### Integration Points

- Replace `StratumRuntimeStatus::DeferredUntilPhase4` with typed Stratum protocol, fake-pool, mining job, queue, runtime state, and test modules.
- Add Bazel-visible Stratum tests and fixtures under `crates/bitaxe-stratum`.
- Add or extend core runtime state in `crates/bitaxe-core` only when the model is shared by firmware, API, and telemetry surfaces.
- Connect firmware mining loop code only after pure fake-pool and ASIC gate tests are present, and keep live hardware behavior guarded by explicit compile/runtime evidence inputs.

</code_context>

<specifics>
## Specific Ideas

- First build the fake-pool suite and deterministic transcript/state tests, then connect the first firmware loop.
- Prefer explicit state-machine types for pool lifecycle and mining status over booleans scattered across firmware code.
- Preserve accepted/rejected-share and rejected-reason data for Phase 5 API compatibility even if Phase 4 does not expose HTTP handlers.
- Treat reconnect/fallback as a user-visible Stratum behavior, not merely a socket retry implementation detail.
- Keep public or controlled-pool smoke optional until fake-pool coverage and hardware gates are clean.

</specifics>

<deferred>
## Deferred Ideas

- Full Stratum v2 behavior remains deferred outside Phase 4.
- AxeOS HTTP/WebSocket handlers and static asset compatibility remain Phase 5.
- Safety controllers, fan/thermal/power enablement, and self-test mining behavior remain Phase 6 unless required as hard preflight gates.
- OTA, filesystem, and release packaging remain Phase 7.

</deferred>

---

*Phase: 04-stratum-v1-and-first-mining-loop*
*Context gathered: 2026-06-27*
