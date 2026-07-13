# Project Research Summary

**Project:** Bitaxe Rust Firmware\
**Domain:** Ultra 205 trusted Stratum v1 production mining on Rust ESP-IDF firmware\
**Researched:** 2026-07-04\
**Confidence:** HIGH for scope, stack continuity, architecture boundaries, evidence governance, safety/redaction constraints; MEDIUM for live pool and hardware outcomes until accepted/rejected-share evidence exists.

## Executive Summary

v1.1 is a hardware-bound production-mining milestone, not a broad firmware expansion. The product is Rust ESP-IDF firmware for Bitaxe Ultra 205 owners that must graduate from the v1.0 controlled no-share harness to a real, safety-gated Stratum v1 mining session on BM1366 hardware. Experts should build it as a typed functional core with a thin ESP-IDF imperative shell: pure crates own Stratum state, ASIC command intent, safety gates, runtime counters, and API projections; firmware adapters own Wi-Fi, TCP sockets, UART, I2C, watchdog-friendly tasks, retained logs, and redacted evidence capture.

The recommended approach is conservative and exact-claim driven. Keep the current ESP-IDF `v5.5.4` Rust `std` stack, Bazel/`just` command surface, `espflash` hardware workflow, pinned ESP-Miner reference, and existing pure Rust cores. Add only the missing production pieces: a real Stratum v1 socket adapter, a production mining runtime task, BM1366 production init/work/result handling, fresh prerequisite safety observations, live telemetry/statistics/scoreboard updates, and v1.1-specific evidence/allow-list tooling.

The key risk is not missing technology; it is accidentally claiming more than the evidence proves or enabling unsafe hardware paths to get a share faster. Mitigate this by enforcing a claim ladder, detector-gated Ultra 205 runs, fail-closed mining prerequisite safety, bounded watchdog-aware loops, committed evidence redaction, and explicit non-claims for active voltage/fan/fault closure, OTAWWW/recovery fault injection, runtime display/input/BAP, non-205 boards, and Stratum v2.

## Key Findings

### Recommended Stack

The stack should remain stable for v1.1. ESP-IDF Rust `std`, ESP-IDF `v5.5.4`, `esp-idf-svc 0.52.1`, `esp-idf-sys 0.37.2`, Bazel `9.1.1`, `rules_rust 0.70.0`, `just`, `espflash`, and the current Cargo/Bazel split already fit the work. The milestone should extend current firmware adapters and host evidence tooling rather than introducing async runtimes, TLS, MQTT, a new network stack, or a parallel build system.

**Core technologies:**

- ESP-IDF `v5.5.4` - production firmware platform for Wi-Fi, lwIP sockets, HTTP/WebSocket, NVS, SPIFFS, OTA, FreeRTOS, logging, and ESP image conventions.
- `esp-idf-svc 0.52.1` / `esp-idf-sys 0.37.2` - Rust ESP-IDF service wrappers, HAL/sys access, and pinned build integration.
- Rust `std::net::TcpStream` over ESP-IDF/lwIP - smallest viable real Stratum v1 TCP adapter with bounded read/write timeouts.
- Rust `std::thread` / FreeRTOS-backed tasks - simple blocking runtime model with explicit stack sizes, sleeps, yields, watchdog checkpoints, and safe stop.
- `esp_idf_svc::hal` UART/GPIO/I2C adapters - effectful boundary for BM1366 UART/reset and INA260/EMC2101/DS4432U observations.
- Bazel + Bzlmod + `rules_rust` - canonical graph for pure crates, host tools, tests, package wrappers, and evidence commands.
- `just` - stable human command surface that delegates to repo-owned Bazel/scripts.
- `espflash` - detector, board-info, flash, monitor, and image backend for Ultra 205 evidence runs.

**Critical version and dependency requirements:**

- Keep ESP-IDF `v5.5.4`; do not migrate to ESP-IDF `v6.x` during the hardware/evidence-sensitive v1.1 milestone.
- Pin ESP-IDF through `esp-idf-sys` metadata; do not rely on crate defaults.
- Do not add `tokio`, `async-std`, `smoltcp`, MQTT, TLS, Stratum v2, or a new mining daemon framework unless a phase-specific implementation proves the current blocking socket model cannot meet responsiveness requirements.
- Reuse `crates/bitaxe-stratum`, `crates/bitaxe-asic`, `crates/bitaxe-safety`, `crates/bitaxe-api`, and `tools/parity`; add pure state-machine pieces only where production runtime decisions need deterministic tests.

### Expected Features

v1.1 succeeds only if an Ultra 205 owner can configure local Wi-Fi and pool credentials, flash firmware, start a bounded safety-gated production mining run, observe real pool lifecycle and live telemetry, and capture either one real accepted/rejected share response from live ASIC-derived work or an explicit safe blocker. Controlled no-share evidence remains useful as regression/preflight evidence but cannot be promoted to production-share proof.

**Must have (table stakes):**

- Real Stratum v1 socket lifecycle - connect, subscribe, authorize, difficulty/extranonce, notify, submit response, reconnect/block, and safe stop with all pool values redacted.
- Redacted pool credential handling - local owner-supplied `pool-credentials*.json` may be used as runtime input, but committed evidence may record only category labels such as `pool_config: local-owner-supplied`.
- Trusted BM1366 initialization gate - full production work requires documented board `205`, source/reference commits, safety prerequisites, init markers, and final go/no-go state.
- Pool-derived work dispatch - live `mining.notify` data must feed typed BM1366 work dispatch, not fixed diagnostic jobs.
- Live ASIC result and nonce parsing - share submission claims require live BM1366 results tied to active pool jobs.
- Share submission and accepted/rejected outcome - at least one real parsed pool response to a live ASIC-derived `mining.submit`, or the milestone records a blocker/pending outcome.
- Mining prerequisite safety gate - fresh or explicitly bounded power/thermal/fan/safety evidence must allow work; missing, stale, unsafe, or unavailable prerequisites block work.
- Live API/WebSocket/statistics telemetry - `/api/system/info`, `/api/system/statistics`, `/api/system/scoreboard`, `/api/ws`, and `/api/ws/live` samples correlate to the same mining session.
- Watchdog and safe-stop behavior - bounded run stays responsive and exits with mining, hardware control, and work submission disabled.
- Evidence-governed claim promotion - parity checklist rows advance only to the exact level proven by redacted v1.1 artifacts.

**Should have (differentiators):**

- Exact-claim mining evidence ledger - one redacted v1.1 evidence root with package manifest, detector, board-info, commands, logs, API/WebSocket captures, share outcome, redaction review, safe-stop, and conclusion.
- Safety-gated production enablement - production mining is opt-in, prerequisite-bound, and explains blocked reasons.
- Redacted observability by default - logs and retained evidence use secret-free lifecycle markers and aggregate counters.
- Pure core, thin hardware shell - Stratum, ASIC, safety, share outcome, hashrate, and API behavior remain testable without hardware.
- Share outcome honesty - one real rejected share is a valid milestone proof if it came from live ASIC-derived work and a parsed pool response.
- Owner-ready operator workflow - one documented repo command flow handles detect, flash, credential seeding, bounded mining, telemetry capture, safe stop, and redaction.

**Defer (future milestones):**

- Full active voltage, fan, thermal fault-stimulus, recovery, and self-test hardware closure.
- OTA/recovery destructive or fault-injection validation, rollback, interrupted update, large erase, and OTAWWW parity.
- Runtime display/input parity and BAP accessory behavior.
- Non-205 boards and non-BM1366 ASIC families.
- Stratum v2.
- Unbounded production soak, stress mining, TLS hardening, or all-board release matrices.

### Architecture Approach

The existing functional-core/imperative-shell architecture should be preserved. Production mining should replace `controlled_mining_runtime` synthetic transcript publication with a live runtime shell that feeds real socket messages, real BM1366 results, and fresh safety observations through pure state/effect types. Firmware tasks should orchestrate bounded I/O and publish redacted snapshots; protocol parsing, gate decisions, share classification, and API statistics semantics belong in pure crates.

**Major components:**

1. `crates/bitaxe-stratum/src/v1/production_runtime.rs` - pure Stratum v1 session state, lifecycle transitions, share outcome planning, reconnect/fallback decisions, and redacted event summaries.
2. `firmware/bitaxe/src/production_mining_runtime.rs` - production task coordinator for socket, ASIC, safety, snapshots, watchdog checkpoints, channels, and safe stop.
3. `firmware/bitaxe/src/stratum_socket_adapter.rs` - ESP-IDF/lwIP TCP line I/O, DNS/connect/read/write timeouts, typed message parsing, local error classification, and credential-safe logs.
4. `firmware/bitaxe/src/asic_adapter.rs` plus `crates/bitaxe-asic` - full init interpretation, production work dispatch, bounded result reads, reset/fail-closed behavior, and typed BM1366 command/result contracts.
5. `firmware/bitaxe/src/safety_adapter.rs` plus `crates/bitaxe-safety` - fresh power/thermal/fan observations, evidence tokens, normal safety status, stale/unavailable blockers, and fail-closed effects.
6. `firmware/bitaxe/src/runtime_snapshot.rs`, `http_api.rs`, and `websocket_api.rs` plus `crates/bitaxe-api` - production-safe snapshot updates, live statistics, scoreboard, pause/resume command surface, and WebSocket frames.
7. Evidence scripts, `tools/parity`, and `docs/parity/checklist.md` - allow-manifest validation, redacted evidence packs, claim tiers, exact status promotion, and non-claim preservation.

**Key patterns to follow:**

- Model runtime steps as pure decisions from typed inputs to typed effects; firmware interprets effects.
- Require explicit gate tokens before hardware effects: detector, board-info, package/source/reference identity, fresh power/thermal/safety status, ASIC init, and hardware evidence acknowledgement.
- Emit redacted structured events with categories and booleans, never raw pool URLs, ports, workers, owner addresses, passwords, tokens, device URLs, Wi-Fi data, IPs, MACs, NVS secrets, raw targets, or raw BM1366 frames.
- Keep controlled no-share runtime as a regression harness, not the production source of truth.

### Critical Pitfalls

1. **Treating controlled no-share evidence as production proof** - prevent with a claim ladder that separates controlled-no-share, live-pool-smoke, accepted/rejected-share, bounded soak, and parity promotion. Stop if no real pool response and live ASIC result artifact exist.
2. **Reusing synthetic safety tokens to enable real hardware** - require fresh or explicitly bounded Ultra 205 power/thermal/fan/safety observations before full init or work dispatch. Stop on stale, unavailable, out-of-range, or undocumented active-control prerequisites.
3. **Bypassing the mining allow manifest** - extend `tools/parity` and repo-owned wrappers before new hardware procedures. Stop on detector mismatch, board-info failure, unapproved command surfaces, missing abort conditions, or missing recovery steps.
4. **Leaking pool credentials, owner identity, targets, or NVS secrets** - make redaction a deliverable before live pool use. Stop before commit/citation if any raw endpoint, port, user, worker, address, password, token, device URL, Wi-Fi value, IP, MAC, target, or NVS secret is present.
5. **Conflating diagnostic BM1366 work with production work** - split diagnostic and pool-derived paths. Stop if production still uses diagnostic job IDs, only timeout/no-result evidence exists, or nonce results are not tied to current jobs.
6. **Starving watchdogs, Wi-Fi, HTTP, or telemetry** - use bounded socket/UART waits, cooperative yields, watchdog checkpoints, and API/WebSocket responsiveness checks. Stop if TWDT/IWDT markers, stale telemetry, or delayed safe-stop appears.
7. **Reporting misleading hashrate, counters, or scoreboard state** - derive statistics from runtime events and parsed share responses only. Stop if counters move without submit responses or nonzero hashrate appears without valid runtime inputs.
8. **Expanding scope into deferred surfaces** - keep non-205, active controls, OTAWWW/recovery fault injection, display/input/BAP, and Stratum v2 below verified unless separate phases own evidence and recovery paths.

## Implications for Roadmap

Based on research, suggested phase structure:

### Phase 1: Baseline Evidence Contract And Claim Ladder

**Rationale:** This must come first because the main v1.1 failure mode is overclaiming existing v1.0 controlled no-share evidence. The roadmap needs claim tiers before risky hardware work starts.\
**Delivers:** v1.1 claim ladder, exact subclaims/non-claims, parity checklist target rows, evidence vocabulary, acceptance blockers, and baseline v1.0 controlled evidence freeze.\
**Addresses:** Evidence-governed claim promotion, share outcome honesty, explicit deferrals.\
**Avoids:** Controlled evidence overclaim, broad "production ready" language, deferred-scope leakage.\
**Research flag:** Standard GSD/evidence pattern; no extra research unless checklist status semantics are unclear.

### Phase 2: Mining Prerequisite Safety Gate

**Rationale:** Production work dispatch must not begin from fixture safety values. Safety prerequisites decide whether live ASIC work may be attempted at all.\
**Delivers:** Fresh or explicitly bounded INA260/EMC2101/fan/safety observations, `MiningSafetyGate` inputs, stale/unavailable/faulted blockers, safe-stop preconditions, and active-control non-claims.\
**Addresses:** Mining prerequisite safety gate, trusted BM1366 initialization gate, watchdog/safe-stop prerequisites.\
**Avoids:** Synthetic safety token reuse, unsafe voltage/fan shortcuts, full active safety overclaim.\
**Research flag:** Needs `/gsd-research-phase` for Ultra 205 sensor paths, safe bounds, hardware evidence protocol, and any required recovery procedure.

### Phase 3: Production Mining Allowlist And Evidence Wrapper

**Rationale:** Every live hardware run must be repeatable, detector-gated, scoped, stoppable, and redaction-ready before real pool credentials are used.\
**Delivers:** v1.1 mining allow-manifest schema, approved claim tiers, repo-owned evidence wrapper, detector/package/board-info/source/reference capture, abort conditions, recovery steps, target-lock rules, and redaction gates.\
**Addresses:** Redacted credential handling, owner-ready operator workflow, exact-claim evidence ledger.\
**Avoids:** Allow-manifest bypass, stale or ambiguous `DEVICE_URL`, unapproved commands, missing safe-state markers.\
**Research flag:** Standard repo tooling pattern with targeted implementation research for allow-list schema and evidence artifact layout.

### Phase 4: Redaction And Secret-Handling Hardening

**Rationale:** Live pool and device evidence cannot be committed or cited safely unless redaction exists before the first real run.\
**Delivers:** Raw/committed artifact separation, deterministic redaction fixtures, final committed-artifact scan, retained-log/API/WebSocket redaction rules, NVS/settings consumption checks without printing values, and accepted category labels.\
**Addresses:** Redacted pool credential handling, redacted observability, local runtime input policy.\
**Avoids:** Pool credential leakage, owner identity leakage, target leakage, NVS secret leakage.\
**Research flag:** Standard pattern; targeted research only if new Stratum fields or evidence formats introduce uncertain sensitive values.

### Phase 5: Trusted BM1366 Init, Work, And Result Path

**Rationale:** A share claim is impossible until diagnostic ASIC behavior is separated from pool-derived production init/work/result behavior.\
**Delivers:** Production BM1366 command semantics, full-init readiness status, pool-derived work dispatch, clean-jobs invalidation, valid job tracking, bounded result reads, nonce/result validation, timeout/fault fail-closed behavior, reset-low recovery markers, and hardware evidence.\
**Addresses:** Trusted BM1366 initialization gate, pool-derived work dispatch, live ASIC result and nonce parsing.\
**Avoids:** Diagnostic/production ASIC confusion, stale nonce mapping, raw frame leakage, unsafe init shortcuts.\
**Research flag:** Needs `/gsd-research-phase` for BM1366 production sequencing, UART timing, job/result correlation, reset behavior, and hardware smoke design.

### Phase 6: Watchdog And Runtime Responsiveness Soak

**Rationale:** The production runtime must stay responsive while socket and UART paths block, retry, or stall. This should be proven before relying on long live-pool sessions.\
**Delivers:** Bounded task step budgets, socket/UART timeouts, watchdog checkpoints, API/WebSocket responsiveness checks, retained-log cadence, no unexpected reboot/panic/silence evidence, and safe-stop latency bound.\
**Addresses:** Watchdog and safe-stop behavior, runtime telemetry responsiveness, operator trust.\
**Avoids:** TWDT/IWDT failures, stale telemetry, delayed stop, Wi-Fi/HTTP starvation.\
**Research flag:** Needs `/gsd-research-phase` or implementation spike for ESP-IDF FreeRTOS scheduling, watchdog feeding, stack sizes, and blocking I/O behavior under mining load.

### Phase 7: Real Stratum Socket And Share Lifecycle

**Rationale:** Only after gates, redaction, ASIC production path, and responsiveness are in place should the firmware attempt real pool lifecycle and share submission evidence.\
**Delivers:** `TcpStream` Stratum adapter, pure production Stratum state machine, subscribe/authorize/difficulty/extranonce/notify handling, clean-jobs behavior, submit/response classification, reconnect/backoff/fallback, fake-pool tests, and live-pool evidence for accepted, rejected, or safely pending share outcome.\
**Addresses:** Real Stratum v1 socket lifecycle, share submission and accepted/rejected outcome, share outcome honesty.\
**Avoids:** Happy-path-only Stratum, TCP-connect-as-success, synthetic submit responses, uncontrolled pool logging.\
**Research flag:** Needs `/gsd-research-phase` for pool edge cases, fake-pool coverage, reconnect/fallback semantics, and share response evidence criteria.

### Phase 8: Mining Telemetry, Statistics, And API Projection

**Rationale:** User-visible API/WebSocket/statistics surfaces must reflect the same runtime events that produced socket, ASIC, and share evidence.\
**Delivers:** Event-sourced share counters, rejected reason categories, observed hashrate inputs, best-difficulty/scoreboard semantics, `/api/system/info`, `/api/system/statistics`, `/api/system/scoreboard`, `/api/ws`, `/api/ws/live` captures, and post-stop stale-data checks.\
**Addresses:** Live hashrate and share statistics, scoreboard population, API/WebSocket telemetry.\
**Avoids:** Expected-hashrate masquerading as observed hashrate, counter/API mismatches, stale active-mining state after stop.\
**Research flag:** Standard model/projection work; route-specific compare fixture planning may need targeted research.

### Phase 9: Safe-Stop And Recovery Evidence

**Rationale:** A production run is not trusted until normal and error stops prove the socket, queue, ASIC path, work submission, and API state all end bounded and observable.\
**Delivers:** Safe-stop state transition, post-stop artifacts, socket inactive/closed marker, queue drained/invalidated marker, mining disabled, hardware control disabled, work submission disabled, API/WebSocket stopped projection, and recovery notes.\
**Addresses:** Watchdog/safe-stop behavior, mining prerequisite safety, exact evidence closure.\
**Avoids:** Safe-stop as log-only, queue/socket continuing after stop, API still advertising active mining.\
**Research flag:** Needs focused planning for hardware-safe recovery steps; avoid destructive/fault-injection flows unless explicitly phase-gated.

### Phase 10: Evidence Closure, Rollout Limits, And Parity Promotion

**Rationale:** Checklist and release language should change only after artifacts prove each exact v1.1 subclaim and redaction passes.\
**Delivers:** Final redacted evidence pack, `just parity` pass, checklist updates for STR/ASIC/STAT/API/SAFE rows, explicit non-claims, release/roadmap notes, and blocked/pending items for unobserved shares or deferred surfaces.\
**Addresses:** Evidence-governed claim promotion, exact-claim mining ledger, explicit deferrals.\
**Avoids:** Scope creep into non-205 boards, OTAWWW/recovery fault injection, runtime display/input/BAP, active safety closure, Stratum v2, and all-board language.\
**Research flag:** Standard evidence closure; no extra research unless a claim depends on a still-ambiguous artifact.

### Phase Ordering Rationale

- The claim ladder comes first because implementation without exact claim tiers invites v1.0 controlled evidence overpromotion.
- Safety, allow-list, and redaction precede live pool work because the first real mining run touches hardware risk and owner secrets.
- BM1366 production work precedes Stratum share evidence because a valid share must come from live ASIC-derived work, not a socket transcript.
- Watchdog responsiveness precedes longer live-pool evidence because blocking socket/UART loops can otherwise invalidate telemetry, safe-stop, and service availability claims.
- Stratum and share lifecycle precede statistics/API promotion because counters, hashrate, scoreboard, and WebSocket data must derive from real runtime events.
- Safe-stop and evidence closure finish the milestone by proving postconditions, preserving exact non-claims, and preventing deferred scope from leaking into v1.1.

### Research Flags

Phases likely needing `/gsd-research-phase` during planning:

- **Phase 2:** Ultra 205 power/thermal/fan sensor paths, safe bounds, hardware evidence protocol, and any recovery constraints.
- **Phase 5:** BM1366 production init sequencing, UART timing, job/result correlation, reset/fail-closed behavior, and hardware smoke evidence.
- **Phase 6:** ESP-IDF FreeRTOS scheduling, watchdog feeding, blocking socket/UART timeouts, task stack sizing, and responsiveness evidence.
- **Phase 7:** Stratum v1 live-pool edge cases, fake-pool scenarios, reconnect/fallback semantics, and accepted/rejected-share evidence criteria.
- **Phase 9:** Safe-stop hardware recovery proof if postcondition checks require hardware effects beyond passive observation.

Phases with standard patterns that can usually skip a standalone research phase:

- **Phase 1:** Evidence contract and claim ladder are governance work using existing parity conventions.
- **Phase 3:** Allow-list and wrapper upgrade should extend existing `tools/parity` and evidence script patterns.
- **Phase 4:** Redaction hardening follows existing local secret-handling policy and deterministic fixture patterns.
- **Phase 8:** Runtime DTO/API/WebSocket projection is standard once event sources are defined.
- **Phase 10:** Evidence closure and checklist promotion are established exact-claim workflows.

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | Strong agreement across local repo state, official ESP-IDF/esp-rs docs, and prior v1.0 validation. No dependency expansion is needed for v1.1. |
| Features | HIGH | Table stakes and deferrals are well grounded in v1.0 evidence boundaries, parity checklist state, user-visible mining expectations, and milestone scope. |
| Architecture | HIGH | Functional core / imperative shell is already the repo standard and maps cleanly to existing Stratum, ASIC, safety, API, firmware adapter, and evidence boundaries. |
| Pitfalls | HIGH | Evidence, safety, redaction, watchdog, and scope risks are strongly supported by repo rules, current implementation boundaries, and ESP-IDF docs. |
| Live mining outcome | MEDIUM | Real accepted/rejected share behavior remains unproven until detector-gated hardware evidence observes a live ASIC-derived submit and parsed pool response. |

**Overall confidence:** HIGH for roadmap structure and constraints; MEDIUM for final share-outcome feasibility until hardware evidence exists.

### Gaps to Address

- **Accepted/rejected share observation:** Planning must allow a real rejected share, accepted share, or explicit safe blocker; do not require "accepted only" or promote no-share evidence.
- **Fresh safety observations:** Confirm which Ultra 205 power, thermal, fan, voltage, and ASIC init observations are required to enable bounded work without claiming full active safety closure.
- **BM1366 production sequencing:** Separate diagnostic chip/work evidence from production pool-derived work and live result parsing.
- **Watchdog/runtime behavior:** Validate task budgets, socket/UART timeouts, yield cadence, and API/WebSocket responsiveness under mining load.
- **Redaction coverage:** Extend tests for pool URL, port, worker, BTC-address-like username, password, device URL, IP, MAC, NVS values, Stratum target/extranonce, share payloads, retained logs, WebSocket captures, and command summaries.
- **Device target provenance:** Use only explicit same-session `DEVICE_URL` or detector-gated fresh monitor derivation; no scans, stale logs, mDNS, ARP, or router state.
- **Deferred scope control:** Keep non-205 boards, OTAWWW/recovery fault injection, runtime display/input/BAP, Stratum v2, active voltage/fan/fault closure, and unbounded stress mining below verified unless later phases explicitly own them.

## Sources

### Primary (HIGH confidence)

- `.planning/research/STACK.md` - stack continuity, version pins, adapter boundaries, evidence tooling recommendations, and dependency non-additions.
- `.planning/research/FEATURES.md` - v1.1 table stakes, differentiators, anti-features, acceptance boundaries, dependency flow, and explicit deferrals.
- `.planning/research/ARCHITECTURE.md` - production runtime architecture, component boundaries, data flow, pure/effectful patterns, and suggested build order.
- `.planning/research/PITFALLS.md` - critical pitfalls, blockers, warning signs, phase mapping, technical debt patterns, and recovery strategies.
- `.planning/PROJECT.md`, `.planning/MILESTONES.md`, `.planning/milestones/v1.0-MILESTONE-AUDIT.md`, `docs/parity/checklist.md` - active milestone scope, v1.0 shipped state, exact non-claims, and parity evidence state.
- `AGENTS.md` and repo-local guidance - Ultra 205 detector gate, local credential policy, redaction rules, hardware evidence requirements, and destructive/fault-injection limits.
- `AGENTS.bright-builds.md`, `standards/core/architecture.md`, `standards/core/verification.md` - functional-core/imperative-shell, exact verification expectations, and pre-commit verification guidance.

### Implementation Evidence (HIGH confidence)

- `firmware/bitaxe/src/main.rs`, `controlled_mining_runtime.rs`, `asic_adapter.rs`, `safety_adapter.rs`, `runtime_snapshot.rs`, `http_api.rs`, `websocket_api.rs`, `wifi_adapter.rs`, `network_stack.rs`, `settings_adapter.rs` - current firmware shell, controlled runtime, ASIC/safety adapters, API/WebSocket snapshots, Wi-Fi, network, and settings boundaries.
- `crates/bitaxe-stratum/src/v1/*`, `crates/bitaxe-asic/src/bm1366/*`, `crates/bitaxe-safety/src/*`, `crates/bitaxe-api/src/*` - existing pure Stratum, BM1366, safety, API, statistics, and scoreboard contracts to preserve and extend.
- `scripts/phase21-live-mining-evidence.sh`, `scripts/phase21-pool-input-bridge.sh`, `scripts/phase21-live-mining-package.sh`, `tools/parity/src/mining_allow.rs` - current evidence, pool input, package, redaction, and allow-list patterns.

### Official / Vendor (HIGH to MEDIUM-HIGH confidence)

- ESP-IDF v5.5.4 release and docs - `https://github.com/espressif/esp-idf/releases/tag/v5.5.4`, lwIP sockets, watchdogs, FreeRTOS SMP, power management, and NVS behavior.
- esp-rs docs and crates - `esp-idf-svc`, `esp-idf-sys`, and `EspWifi` integration supporting Rust `std` TCP/UDP over ESP-IDF/lwIP.
- `espflash` docs/crate - ESP32-S3 board-info, flash, monitor, list-ports, and save-image workflow support.
- Bazel and `rules_rust` docs - Bazel 9/Bzlmod/rules_rust compatibility and current Cargo dependency mirroring pattern.

*Research completed: 2026-07-04*\
*Ready for roadmap: yes*
