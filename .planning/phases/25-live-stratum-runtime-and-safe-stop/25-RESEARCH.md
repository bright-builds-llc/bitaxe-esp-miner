# Phase 25: Live Stratum Runtime And Safe Stop - Research

**Researched:** 2026-07-05  
**Domain:** ESP-IDF Rust firmware Stratum v1 runtime, safe stop, watchdog, and redacted evidence workflow  
**Confidence:** HIGH for local architecture and verification surfaces; MEDIUM-HIGH for ESP-IDF socket/watchdog guidance; MEDIUM for live pool outcome feasibility

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

#### Real Stratum Runtime Boundary

- **D-01:** Implement the live Stratum v1 path as a firmware socket adapter around the existing pure Stratum protocol/runtime core. ESP-IDF networking, TCP reads/writes, task yielding, timing, and shutdown stay in firmware; message parsing, lifecycle state, fake-pool behavior, submit mapping, and response classification stay in `crates/bitaxe-stratum`.
- **D-02:** Preserve the Phase 24 production BM1366 boundary: firmware interprets typed production actions and observations, but raw BM1366 work construction, result parsing, active-work correlation, submit intent construction, and redaction-safe result rendering stay in pure ASIC/Stratum modules.
- **D-03:** Live runtime startup must remain fail-closed behind Phase 22 prerequisite readiness. Missing, stale, unavailable, unsafe, ambiguous, or undocumented safety observations keep socket mining disabled or stopped with stable redaction-safe blocker reasons.

#### Submit Response Classification

- **D-04:** Classify accepted, rejected, blocked, timeout, reconnect, malformed, or no-observed share outcomes only when a pool response is tied to a live ASIC-derived submit intent from the active production work registry.
- **D-05:** A nonce/result observation plus submit intent is still not an accepted/rejected share claim until the real socket runtime observes and classifies the pool's `mining.submit` response. Implementation-only or fake-pool evidence remains below live accepted/rejected proof.
- **D-06:** Raw pool endpoints, ports, workers, owner addresses, passwords, targets, extranonces, share payloads, socket errors, device URLs, IPs, MACs, Wi-Fi values, tokens, NVS secrets, and raw BM1366 frames must not appear in committed logs, evidence, API captures, WebSocket captures, discussion artifacts, or parity updates.

#### Deterministic Fake-Pool Coverage

- **D-07:** Extend the existing deterministic fake-pool/fixture harness instead of creating a second protocol simulator. Coverage must prove subscribe, authorize, notify, set-difficulty, clean-jobs, submit response, reconnect, fallback, timeout, malformed response, and error classification behavior.
- **D-08:** Fake-pool tests should exercise both successful and fail-closed paths: clean-jobs invalidation, reconnect generation changes, stale work rejection, blocked prerequisite outcomes, accepted response classification, rejected response classification, and no-response timeout handling.
- **D-09:** Fake-pool or fixture tests can prove deterministic STR-11 behavior, but cannot by themselves promote STR-09 live pool response evidence.

#### Bounded Safe Stop

- **D-10:** Define safe stop as an explicit runtime postcondition: socket activity stopped, reads/writes no longer advancing mining state, work queues drained or invalidated, active production work invalidated, mining disabled, hardware control disabled, work submission blocked, and post-stop runtime/API-visible state refreshed.
- **D-11:** Safe-stop behavior must be callable from normal stop, reconnect/fallback exhaustion, prerequisite failure, operator cancellation, and verification cleanup paths without leaking secrets or leaving stale active-mining state.
- **D-12:** Committed evidence may record safe-stop categories and status labels, but any raw local logs used for diagnosis must remain ignored/local or be redacted before promotion.

#### Watchdog Responsiveness

- **D-13:** The live runtime must preserve watchdog responsiveness under bounded socket, ASIC, API/WebSocket, and evidence-capture load by using explicit checkpoints or yields around blocking-prone operations.
- **D-14:** Watchdog proof should combine pure budget/checkpoint tests with firmware or workflow evidence when hardware is available. A blocked or non-hardware path must keep SAFE-13 below hardware-verified status and record the exact non-claim.

#### Evidence And Allow-Manifest Integration

- **D-15:** Update Phase 23 evidence-root and allow/validation tooling deliberately for a Phase 25 live Stratum surface before hardware evidence is promoted. Do not bypass existing redaction, detector, package, safe-state, or prohibited-token validation.
- **D-16:** Hardware use must follow `just detect-ultra205`, board `205` selection, repo-owned commands, runtime-only local credentials, redaction review, and exact evidence recording. If detection, safe prerequisites, credentials, socket behavior, or share outcome proof is blocked, record the blocker instead of inferring success.
- **D-17:** Checklist promotion must be exact: STR-08, STR-09, STR-11, SAFE-12, and SAFE-13 advance only to the level supported by source, deterministic tests, workflow evidence, detector-gated hardware evidence, and redaction review actually produced in this phase.

### Claude's Discretion

Claude may choose exact module names, adapter trait names, fake-pool fixture structure, timeout budgets, retry limits, evidence filenames, redaction labels, and plan count. Those choices must preserve functional core / imperative shell structure, typed fail-closed behavior, redaction rules, Ultra 205 detector gating, repo-owned verification, and conservative parity semantics.

### Deferred Ideas (OUT OF SCOPE)

- API, WebSocket, statistics, scoreboard, and final parity checklist projection from v1.1 runtime events belong to Phase 26, except for post-stop state required by SAFE-12.
- Full active voltage/fan/thermal fault-stimulus, self-test hardware closure, OTA/recovery destructive or fault-injection evidence, runtime display/input, BAP, Stratum v2, non-205 boards, other ASIC families, and unbounded stress mining remain future work.
- Broad UI or AxeOS surface changes are out of scope for Phase 25.
</user_constraints>

## Summary

Phase 25 should be planned as a narrow transition from controlled no-share transcripts to a real Stratum v1 socket runtime. The existing pure core already owns Stratum v1 message parsing/serialization, fake-pool transcripts, runtime state, guarded mining gates, work queues, `ProductionWorkRegistry`, and `SubmitIntent`; the missing work is a live runtime state machine/classifier in `crates/bitaxe-stratum`, plus a firmware-owned ESP-IDF TCP adapter that feeds typed inbound/outbound events without logging raw pool values. [VERIFIED: `crates/bitaxe-stratum/src/v1/*.rs`; VERIFIED: `.planning/phases/25-live-stratum-runtime-and-safe-stop/25-CONTEXT.md`]

Plan the implementation in this order: extend pure runtime/classifier and fake-pool coverage, define safe-stop postconditions in pure types, add firmware socket shell with bounded timeouts/yields, update evidence/allow-manifest tools, then run detector-gated hardware evidence only after redaction and safety gates are in place. [VERIFIED: `standards/core/architecture.md`; VERIFIED: `tools/parity/src/mining_allow.rs`; VERIFIED: `docs/parity/evidence/phase-23-redacted-operator-evidence-workflow/evidence-contract.md`]

**Primary recommendation:** Build a single-owner firmware `TcpStream` adapter around a pure `LiveStratumRuntime`/`SubmitResponseClassifier` module, with short blocking timeouts, explicit stop checkpoints, redaction-safe markers, and a Phase 25 evidence wrapper that is allow-manifest-approved before hardware promotion. [CITED: https://docs.espressif.com/projects/esp-idf/en/v5.5.4/esp32s3/api-guides/lwip.html; VERIFIED: `firmware/bitaxe/src/mining_evidence_mode.rs`; VERIFIED: `.planning/phases/25-live-stratum-runtime-and-safe-stop/25-CONTEXT.md`]

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
| --- | --- | --- |
| STR-08 | Ultra 205 production mining uses a real Stratum v1 TCP socket lifecycle for connect, subscribe, authorize, difficulty/extranonce, notify, submit, response classification, reconnect, and safe stop. [VERIFIED: `.planning/REQUIREMENTS.md`] | ESP-IDF supports BSD sockets, `shutdown`, `select`, `SO_RCVTIMEO`/`SO_SNDTIMEO`, and non-blocking socket options; the repo already has message parsing and runtime state to consume socket events. [CITED: https://docs.espressif.com/projects/esp-idf/en/v5.5.4/esp32s3/api-guides/lwip.html; VERIFIED: `crates/bitaxe-stratum/src/v1/messages.rs`; VERIFIED: `crates/bitaxe-stratum/src/v1/state.rs`] |
| STR-09 | Ultra 205 production mining classifies at least one real pool response to a live ASIC-derived `mining.submit` as accepted, rejected, or explicitly blocked by a safe prerequisite. [VERIFIED: `.planning/REQUIREMENTS.md`] | Phase 24 provides `SubmitIntent` only after current-generation active work correlation; Phase 25 must tie socket responses to that intent before any accepted/rejected evidence claim. [VERIFIED: `crates/bitaxe-stratum/src/v1/production_work.rs`; VERIFIED: `docs/parity/evidence/phase-24-bm1366-production-work-path/result-correlation.md`] |
| STR-11 | Ultra 205 production mining has deterministic fake-pool or fixture tests for subscribe, authorize, notify, clean-jobs, submit response, reconnect, fallback, and error classification behavior. [VERIFIED: `.planning/REQUIREMENTS.md`] | `FakePoolTranscript` already covers subscribe/authorize/notify/submit, rejected response, disconnect, timeout, and unexpected client messages; it needs clean-jobs, reconnect generation, malformed/no-response classification, and blocker/fallback cases. [VERIFIED: `crates/bitaxe-stratum/src/v1/fake_pool.rs`; VERIFIED: `crates/bitaxe-stratum/src/v1/production_work.rs`] |
| SAFE-12 | Ultra 205 production mining can stop in a bounded safe state with socket activity stopped, work queues drained or invalidated, mining disabled, hardware control disabled, and post-stop API/WebSocket state updated. [VERIFIED: `.planning/REQUIREMENTS.md`] | Existing state has blocked/paused lifecycle fields and safety effects can disable ASIC/hardware control surfaces; a new safe-stop postcondition should compose these and refresh `runtime_snapshot`. [VERIFIED: `crates/bitaxe-stratum/src/v1/state.rs`; VERIFIED: `crates/bitaxe-safety/src/effects.rs`; VERIFIED: `firmware/bitaxe/src/runtime_snapshot.rs`] |
| SAFE-13 | Ultra 205 production mining preserves watchdog responsiveness under bounded socket, ASIC, API, WebSocket, and evidence-capture load. [VERIFIED: `.planning/REQUIREMENTS.md`] | Existing watchdog pure model defines a 25 ms step budget, 100 ms yield interval, and maximum consecutive steps before yield; ESP-IDF TWDT docs require subscribed tasks/users to reset periodically and detect tasks that run without yielding. [VERIFIED: `crates/bitaxe-safety/src/watchdog.rs`; CITED: https://docs.espressif.com/projects/esp-idf/en/v5.5.4/esp32s3/api-reference/system/wdts.html] |
</phase_requirements>

## Project Constraints (from .cursor/rules/)

No `.cursor/rules/` directory exists in this workspace, and no `.cursor/skills/` or `.agents/skills/` project skills were found. [VERIFIED: Glob `.cursor/rules/**/*`; VERIFIED: Glob `.cursor/skills/**/SKILL.md`; VERIFIED: Glob `.agents/skills/**/SKILL.md`]

Material repo constraints instead come from `AGENTS.md`, `AGENTS.bright-builds.md`, `standards-overrides.md`, and the managed standards pages. [VERIFIED: `AGENTS.md`; VERIFIED: `AGENTS.bright-builds.md`; VERIFIED: `standards/index.md`]

- Use GSD artifacts for repo-changing work and keep Phase 25 planning/execution under `.planning/phases/25-live-stratum-runtime-and-safe-stop/`. [VERIFIED: `AGENTS.md`; VERIFIED: GSD init output]
- Use functional core / imperative shell: pure protocol, safety, ASIC, evidence, and parity decisions belong in crates/tools; ESP-IDF sockets, task timing, credentials, NVS, HTTP/WebSocket capture, serial, and hardware effects stay in firmware/scripts. [VERIFIED: `standards/core/architecture.md`; VERIFIED: `.planning/phases/25-live-stratum-runtime-and-safe-stop/25-CONTEXT.md`]
- Hardware work must start with `just detect-ultra205`, board `205`, repo-owned commands, runtime-only local credentials, redaction review, and exact evidence recording. [VERIFIED: `AGENTS.md`; VERIFIED: `Justfile`]
- Do not read, print, summarize, commit, or expose real `wifi-credentials.json` or `pool-credentials*.json` contents; only pass them to repo-owned runtime commands when detector-gated prerequisites allow it. [VERIFIED: `AGENTS.md`]
- Committed evidence must not contain raw pool endpoints, ports, workers, owner addresses, passwords, targets, extranonces, share payloads, socket errors, device URLs, IPs, MACs, Wi-Fi values, tokens, NVS secrets, or raw BM1366 frames. [VERIFIED: `AGENTS.md`; VERIFIED: `docs/parity/evidence/phase-23-redacted-operator-evidence-workflow/evidence-contract.md`]
- New Rust modules should use `foo.rs` plus `foo/` if submodules are needed; optional internal values should use `maybe_`; unit tests should use Arrange/Act/Assert and focus on one concern. [VERIFIED: `standards/languages/rust.md`; VERIFIED: `standards/core/testing.md`]

## Standard Stack

### Core

| Library / Surface | Version | Purpose | Why Standard |
| --- | --- | --- | --- |
| `esp-idf-svc` / `esp-idf-sys` | `esp-idf-svc 0.52.1`, `esp-idf-sys 0.37.2`, ESP-IDF `tag:v5.5.4` | Firmware ESP-IDF services and sys bindings. | The workspace pins these versions and the firmware metadata pins ESP-IDF `v5.5.4`. [VERIFIED: `Cargo.toml`; VERIFIED: `firmware/bitaxe/Cargo.toml`] |
| Rust `std::net::TcpStream` over ESP-IDF lwIP BSD sockets | Rust `1.88.0-nightly` locally, target `xtensa-esp32s3-espidf` in project stack | Live Stratum TCP client socket. | ESP-IDF supports BSD sockets and the esp-idf-svc TCP example uses `std::net::TcpStream` for blocking TCP on ESP-IDF. [CITED: https://docs.espressif.com/projects/esp-idf/en/v5.5.4/esp32s3/api-guides/lwip.html; CITED: https://raw.githubusercontent.com/esp-rs/esp-idf-svc/master/examples/tcp.rs; VERIFIED: shell `rustc --version`] |
| `crates/bitaxe-stratum` | Local crate | Stratum v1 messages, fake-pool, production runtime state, submit intent classification, safe-stop pure state. | Existing project decisions place message parsing, lifecycle state, fake-pool behavior, submit mapping, and response classification in this crate. [VERIFIED: `.planning/phases/25-live-stratum-runtime-and-safe-stop/25-CONTEXT.md`; VERIFIED: `crates/bitaxe-stratum/src/v1.rs`] |
| `crates/bitaxe-asic` | Local crate | BM1366 production action/result modeling. | Phase 24 already owns BM1366 production commands, status labels, and redaction-safe blockers. [VERIFIED: `.planning/STATE.md`; VERIFIED: `firmware/bitaxe/src/asic_adapter/status.rs`] |
| `crates/bitaxe-safety` | Local crate | Mining prerequisite decisions, safety effects, watchdog checkpoint model. | SAFE-12/SAFE-13 depend on existing safety effects and watchdog budget primitives. [VERIFIED: `crates/bitaxe-safety/src/effects.rs`; VERIFIED: `crates/bitaxe-safety/src/watchdog.rs`] |
| `tools/parity` / `scripts` evidence wrappers | Local tools | Allow-manifest, operator evidence, redaction, and workflow validation. | Phase 23/24 evidence governance already routes hardware claims through these tools and Phase 25 must update them deliberately. [VERIFIED: `tools/parity/src/mining_allow.rs`; VERIFIED: `tools/parity/src/operator_evidence.rs`; VERIFIED: `scripts/BUILD.bazel`] |

### Supporting

| Surface | Version | Purpose | When to Use |
| --- | --- | --- | --- |
| `serde_json` | `1.0.150` | Parse and serialize Stratum JSON-RPC values. | Continue using existing typed message parsing instead of custom string parsing. [VERIFIED: `Cargo.toml`; VERIFIED: `crates/bitaxe-stratum/src/v1/messages.rs`] |
| `thiserror` / `anyhow` | `thiserror 2.0.18`, `anyhow 1.0.102` | Library and firmware/script-shell error paths. | Existing crates use `thiserror` for `StratumV1Error`, and firmware adapters use `anyhow`. [VERIFIED: `Cargo.toml`; VERIFIED: `crates/bitaxe-stratum/src/error.rs`; VERIFIED: `firmware/bitaxe/src/asic_adapter.rs`] |
| `just`, Bazel, `espflash`, Node, `rg` | `just 1.48.0`, Bazel `9.1.1`, `espflash 4.0.1`, Node `v24.13.0`, `rg 15.1.0` locally | Repo command surface, build/test/package/flash, evidence helpers, redaction scans. | These are available locally and already used by repo workflows. [VERIFIED: shell version audit; VERIFIED: `Justfile`; VERIFIED: `scripts/BUILD.bazel`] |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
| --- | --- | --- |
| `std::net::TcpStream` plus short timeouts | Raw `esp-idf-sys`/lwIP socket calls | Raw sockets expose more control but increase unsafe/error-handling surface; use them only if `std::net` cannot express required timeouts/shutdown. [CITED: https://docs.espressif.com/projects/esp-idf/en/v5.5.4/esp32s3/api-guides/lwip.html; CITED: https://raw.githubusercontent.com/esp-rs/esp-idf-svc/master/examples/tcp.rs] |
| Existing `fake_pool.rs` | New simulator process or external mock server | A second simulator would duplicate protocol semantics and violate the context decision to extend the deterministic harness. [VERIFIED: `.planning/phases/25-live-stratum-runtime-and-safe-stop/25-CONTEXT.md`; VERIFIED: `crates/bitaxe-stratum/src/v1/fake_pool.rs`] |
| Phase 25 evidence wrapper | Reusing Phase 21 wrapper as proof | Phase 21 wrapper records controlled no-share markers; it does not prove a real socket response to a live ASIC-derived submit. [VERIFIED: `scripts/phase21-live-mining-evidence.sh`; VERIFIED: `docs/parity/evidence/phase-24-bm1366-production-work-path/result-correlation.md`] |

**Installation:**

No new dependencies are required for planning or the first implementation wave. [VERIFIED: `Cargo.toml`; VERIFIED: `firmware/bitaxe/Cargo.toml`]

```bash
# Use existing repo commands.
just test
just build
just parity
```

**Version verification:** Local versions were checked with `node --version`, `cargo --version`, `rustc --version`, `bazel --version`, `just --version`, `espflash --version`, and `rg --version`. [VERIFIED: shell version audit]

## Architecture Patterns

### Recommended Project Structure

```text
crates/bitaxe-stratum/src/v1/
├── live_runtime.rs        # pure lifecycle transition, outbound intent, safe-stop postconditions
├── submit_response.rs     # pure classifier for submit intent + StratumResponse/timeout/malformed outcomes
├── fake_pool.rs           # extend existing deterministic transcript harness
├── production_work.rs     # retain active work and SubmitIntent ownership
└── state.rs               # add only reusable runtime state/status fields

firmware/bitaxe/src/
├── live_stratum_runtime.rs # firmware socket loop, timeouts, yields, redacted markers
├── controlled_mining_runtime.rs # keep or shrink controlled path without conflating evidence
└── runtime_snapshot.rs     # post-stop API-visible state refresh

scripts/
├── phase25-live-stratum-evidence.sh      # detector-gated redacted evidence wrapper
└── phase25-live-stratum-evidence-test.sh # deterministic wrapper/redaction tests
```

This layout follows the existing `foo.rs` plus `foo/` preference if submodules become necessary. [VERIFIED: `standards/languages/rust.md`]

### Pattern 1: Single-Owner Socket Adapter

**What:** Own the live TCP stream in one firmware task or loop; convert socket bytes into complete JSON lines; call pure parser/state-machine functions; serialize outbound `StratumV1ClientMessage` values; close/shutdown from the same owner during safe stop. [CITED: https://docs.espressif.com/projects/esp-idf/en/v5.5.4/esp32s3/api-guides/lwip.html; VERIFIED: `crates/bitaxe-stratum/src/v1/messages.rs`]

**When to use:** Use this for STR-08 because ESP-IDF warns that not all BSD socket operations are fully thread-safe and simultaneous reads/writes may require synchronization. [CITED: https://docs.espressif.com/projects/esp-idf/en/v5.5.4/esp32s3/api-guides/lwip.html]

**Example:**

```rust
// Source: ESP-IDF lwIP BSD socket docs + local Stratum message API.
fn run_socket_step(adapter: &mut LiveSocketAdapter, runtime: &mut LiveStratumRuntime) {
    let maybe_line = adapter.read_json_line_with_timeout();
    if let Some(line) = maybe_line {
        runtime.apply_server_message_line(&line);
    }

    for outbound in runtime.drain_outbound_messages() {
        adapter.write_json_line(&outbound);
    }

    adapter.checkpoint("socket_step");
}
```

### Pattern 2: Response Classification Requires Submit Intent

**What:** Classify share outcomes from `(SubmitIntent, request_id, response category)` instead of from socket response text alone. [VERIFIED: `.planning/phases/25-live-stratum-runtime-and-safe-stop/25-CONTEXT.md`; VERIFIED: `crates/bitaxe-stratum/src/v1/production_work.rs`]

**When to use:** Use this for STR-09 so accepted/rejected counters cannot advance unless Phase 24 produced a current-generation live ASIC-derived submit intent and the real socket runtime observes the matching `mining.submit` response. [VERIFIED: `docs/parity/evidence/phase-24-bm1366-production-work-path/result-correlation.md`; VERIFIED: `.planning/REQUIREMENTS.md`]

**Example:**

```rust
// Source: Phase 24 SubmitIntent boundary + StratumResponse message model.
pub enum SubmitClassification {
    Accepted,
    Rejected { reason: RedactedRejectReason },
    Timeout,
    Malformed,
    NoObservedShare,
    Blocked { reason: &'static str },
}
```

### Pattern 3: Safe Stop As Data, Then Firmware Effects

**What:** Model safe stop as a pure postcondition plan, then have firmware perform best-effort socket shutdown, invalidate work, disable submission/hardware control, and refresh runtime snapshot. [VERIFIED: `crates/bitaxe-stratum/src/v1/state.rs`; VERIFIED: `crates/bitaxe-safety/src/effects.rs`; VERIFIED: `firmware/bitaxe/src/runtime_snapshot.rs`]

**When to use:** Use this for normal stop, reconnect/fallback exhaustion, prerequisite failure, operator cancellation, and verification cleanup. [VERIFIED: `.planning/phases/25-live-stratum-runtime-and-safe-stop/25-CONTEXT.md`]

**Example:**

```rust
// Source: local state/effects model.
pub struct SafeStopPostconditions {
    pub socket_stopped: bool,
    pub active_work_invalidated: bool,
    pub mining_disabled: bool,
    pub hardware_control_disabled: bool,
    pub work_submission_blocked: bool,
    pub runtime_snapshot_refreshed: bool,
}
```

### Anti-Patterns to Avoid

- **Socket logic in pure crate:** `crates/bitaxe-stratum` should not own `TcpStream`, firmware sleeps, ESP-IDF sys calls, or local credential access. [VERIFIED: `.planning/phases/25-live-stratum-runtime-and-safe-stop/25-CONTEXT.md`; VERIFIED: `standards/core/architecture.md`]
- **Raw value logging:** Do not log raw endpoints, users, workers, extranonces, targets, share payloads, socket errors, IPs, or BM1366 frames. [VERIFIED: `AGENTS.md`; VERIFIED: `docs/parity/evidence/phase-23-redacted-operator-evidence-workflow/evidence-contract.md`]
- **Classifying fake-pool responses as live proof:** Fake-pool tests satisfy STR-11 but do not promote STR-09. [VERIFIED: `.planning/phases/25-live-stratum-runtime-and-safe-stop/25-CONTEXT.md`]
- **Continuing work after stop:** Socket reads/writes, queued/active work, dispatch, and submit generation must not advance after safe stop. [VERIFIED: `.planning/REQUIREMENTS.md`; VERIFIED: `crates/bitaxe-stratum/src/v1/production_work.rs`]
- **Ad hoc evidence command:** `tools/parity/src/mining_allow.rs` currently only approves Phase 15 and Phase 21 wrappers for mining evidence, so Phase 25 must update allow rules before promotion. [VERIFIED: `tools/parity/src/mining_allow.rs`; VERIFIED: `scripts/BUILD.bazel`]

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
| --- | --- | --- | --- |
| JSON-RPC parsing | String splits or substring checks for Stratum responses | `parse_server_message` and typed `StratumResponse` | Existing parser already validates methods, IDs, params, errors, notify, difficulty, extranonce, reconnect, ping, and response forms. [VERIFIED: `crates/bitaxe-stratum/src/v1/messages.rs`] |
| TCP/IP stack | Custom TCP framing or raw packet handling | ESP-IDF lwIP BSD sockets via Rust `std::net` unless a documented gap forces raw sys calls | ESP-IDF documents BSD sockets support and socket timeout/nonblocking options. [CITED: https://docs.espressif.com/projects/esp-idf/en/v5.5.4/esp32s3/api-guides/lwip.html] |
| Fake pool | Separate simulator CLI/service | Extend `FakePoolTranscript` | Context explicitly requires extending the existing deterministic harness. [VERIFIED: `.planning/phases/25-live-stratum-runtime-and-safe-stop/25-CONTEXT.md`; VERIFIED: `crates/bitaxe-stratum/src/v1/fake_pool.rs`] |
| Share outcome proof | Counter increments based on nonce or submit serialization | `SubmitIntent` + matching pool response classifier | Phase 24 evidence says submit intent is not live response proof. [VERIFIED: `docs/parity/evidence/phase-24-bm1366-production-work-path/result-correlation.md`] |
| Redaction | New one-off `sed` rules only in Phase 25 script | Existing redaction categories plus `operator_evidence`/`mining_allow` validation updated for Phase 25 | Phase 23 validator already encodes required slots and forbidden sentinels. [VERIFIED: `tools/parity/src/operator_evidence.rs`; VERIFIED: `docs/parity/evidence/phase-23-redacted-operator-evidence-workflow/evidence-contract.md`] |
| Safe stop | Log-only status marker | Pure postcondition + firmware effects + snapshot refresh + evidence slot | SAFE-12 requires actual bounded postconditions, not just a string. [VERIFIED: `.planning/REQUIREMENTS.md`; VERIFIED: `crates/bitaxe-safety/src/effects.rs`] |

**Key insight:** The hard problems are state ownership, redaction, and claim boundaries, not JSON or socket mechanics; reuse the existing typed core and evidence validators so Phase 25 cannot accidentally turn implementation evidence into live accepted/rejected proof. [VERIFIED: `.planning/STATE.md`; VERIFIED: `tools/parity/src/mining_allow.rs`; VERIFIED: `docs/parity/evidence/phase-24-bm1366-production-work-path/result-correlation.md`]

## Common Pitfalls

### Pitfall 1: Accepting/Rejection Counters Without Live Response Proof

**What goes wrong:** The runtime increments accepted/rejected shares after nonce correlation or submit serialization instead of after a real matching socket response. [VERIFIED: `crates/bitaxe-stratum/src/v1/controlled_runtime.rs`; VERIFIED: `.planning/phases/25-live-stratum-runtime-and-safe-stop/25-CONTEXT.md`]

**Why it happens:** Existing controlled-runtime code can apply a synthetic `maybe_submit_response`, and fake-pool tests already record counters. [VERIFIED: `crates/bitaxe-stratum/src/v1/controlled_runtime.rs`; VERIFIED: `crates/bitaxe-stratum/src/v1/fake_pool.rs`]

**How to avoid:** Require `SubmitIntent` identity, request ID, current `PoolSessionGeneration`, and real socket response classification before accepted/rejected labels. [VERIFIED: `crates/bitaxe-stratum/src/v1/production_work.rs`; VERIFIED: `.planning/phases/25-live-stratum-runtime-and-safe-stop/25-CONTEXT.md`]

**Warning signs:** Evidence says `share_submission_status=accepted` or `rejected` without a detector-gated socket runtime artifact and redaction review. [VERIFIED: `docs/parity/evidence/phase-23-redacted-operator-evidence-workflow/share-outcome.md`; VERIFIED: `tools/parity/src/mining_allow.rs`]

### Pitfall 2: Blocking Socket Reads Starve Watchdog

**What goes wrong:** A socket read, DNS/connect attempt, API probe, WebSocket capture, ASIC read, or evidence wrapper step blocks long enough that firmware cannot yield or feed watchdog paths. [CITED: https://docs.espressif.com/projects/esp-idf/en/v5.5.4/esp32s3/api-reference/system/wdts.html; VERIFIED: `.planning/REQUIREMENTS.md`]

**Why it happens:** ESP-IDF TWDT detects tasks running without yielding for a prolonged period, and the current pure watchdog model expects bounded steps/yields. [CITED: https://docs.espressif.com/projects/esp-idf/en/v5.5.4/esp32s3/api-reference/system/wdts.html; VERIFIED: `crates/bitaxe-safety/src/watchdog.rs`]

**How to avoid:** Use short socket read/write timeouts or nonblocking mode, perform one bounded operation per loop step, emit watchdog checkpoints, and treat missing progress as timeout/reconnect/safe-stop input. [CITED: https://docs.espressif.com/projects/esp-idf/en/v5.5.4/esp32s3/api-guides/lwip.html; VERIFIED: `crates/bitaxe-safety/src/watchdog.rs`]

**Warning signs:** The implementation uses `read_to_end`, indefinite `read`, unbounded loops over socket lines, or evidence scripts with long sleeps outside bounded windows. [CITED: https://raw.githubusercontent.com/esp-rs/esp-idf-svc/master/examples/tcp.rs; VERIFIED: `scripts/phase21-live-mining-evidence.sh`]

### Pitfall 3: Safe Stop Leaves Stale Work Active

**What goes wrong:** Socket stops but queued work, active work, valid jobs, or runtime snapshot still shows mining as active. [VERIFIED: `.planning/REQUIREMENTS.md`; VERIFIED: `crates/bitaxe-stratum/src/v1/production_work.rs`; VERIFIED: `firmware/bitaxe/src/runtime_snapshot.rs`]

**Why it happens:** Existing invalidation helpers are tied to clean-jobs, reconnect, authorization reset, and session replacement; Phase 25 needs a safe-stop-specific path or explicit reuse. [VERIFIED: `crates/bitaxe-stratum/src/v1/production_work.rs`]

**How to avoid:** Add pure safe-stop tests that assert socket stopped label, queue clear/invalidate, active work empty, work submission blocked, mining paused/safe-blocked, hardware control disabled, and snapshot refreshed. [VERIFIED: `.planning/REQUIREMENTS.md`; VERIFIED: `crates/bitaxe-safety/src/effects.rs`]

**Warning signs:** `safe_stop_status=complete` appears without postcondition tests and without `runtime_snapshot_status=updated`. [VERIFIED: `scripts/phase21-live-mining-evidence.sh`; VERIFIED: `.planning/REQUIREMENTS.md`]

### Pitfall 4: Evidence Tooling Blocks Phase 25

**What goes wrong:** Hardware run produces useful local logs but cannot be promoted because `mining_allow` rejects the new command/surface or redaction review lacks Phase 25 forbidden-token coverage. [VERIFIED: `tools/parity/src/mining_allow.rs`; VERIFIED: `tools/parity/src/operator_evidence.rs`]

**Why it happens:** `mining_allow` currently approves only Phase 15/Phase 21 mining wrappers and includes `stratum` as a prohibited command token. [VERIFIED: `tools/parity/src/mining_allow.rs`]

**How to avoid:** Plan a tooling wave before hardware promotion: add Phase 25 surfaces/claim tiers/commands, update redaction forbidden terms, test blocked and passed manifests, then run hardware. [VERIFIED: `.planning/phases/25-live-stratum-runtime-and-safe-stop/25-CONTEXT.md`; VERIFIED: `tools/parity/src/mining_allow.rs`]

**Warning signs:** Evidence wrapper runs via ad hoc shell command or raw `espflash` command that is not represented in an allow manifest. [VERIFIED: `AGENTS.md`; VERIFIED: `tools/parity/src/mining_allow.rs`]

### Pitfall 5: Accidental Credential or Endpoint Exposure

**What goes wrong:** Local pool credentials, endpoints, workers, socket errors, device URLs, IPs, or raw Stratum payloads leak through logs, API captures, WebSocket captures, redaction reviews, or failed command output. [VERIFIED: `AGENTS.md`; VERIFIED: `docs/parity/evidence/phase-23-redacted-operator-evidence-workflow/evidence-contract.md`]

**Why it happens:** Real Stratum socket errors and submit payloads are more sensitive than controlled no-share markers. [VERIFIED: `.planning/phases/25-live-stratum-runtime-and-safe-stop/25-CONTEXT.md`]

**How to avoid:** Log category labels only, keep raw diagnostics local/ignored, run deterministic scans, and make redaction pass a prerequisite for checklist citation. [VERIFIED: `tools/parity/src/operator_evidence.rs`; VERIFIED: `scripts/phase21-live-mining-evidence.sh`]

**Warning signs:** Logs contain `stratum+tcp://`, `poolUser`, raw `mining.submit`, `target`, `extranonce`, `nonce`, `socket_error=`, `device_url=`, raw IPs, or MACs. [VERIFIED: `docs/parity/evidence/phase-23-redacted-operator-evidence-workflow/evidence-contract.md`; VERIFIED: `tools/parity/src/operator_evidence.rs`]

## Code Examples

Verified patterns from local and official sources:

### Socket Adapter Loop Shape

```rust
// Source: ESP-IDF lwIP BSD socket docs and esp-idf-svc TCP example.
let mut stream = TcpStream::connect(pool_addr)?;
stream.set_read_timeout(Some(Duration::from_millis(100)))?;
stream.set_write_timeout(Some(Duration::from_millis(100)))?;

loop {
    if stop_requested() {
        let _ = stream.shutdown(Shutdown::Both);
        break;
    }

    runtime.checkpoint("socket_read");
    let maybe_line = read_one_line(&mut stream)?;
    runtime.apply_socket_observation(maybe_line);

    runtime.checkpoint("socket_write");
    for message in runtime.drain_outbound() {
        stream.write_all(message.to_json_line()?.as_bytes())?;
    }
}
```

### Pure Submit Classifier Shape

```rust
// Source: local SubmitIntent and StratumResponse models.
pub fn classify_submit_response(
    intent: &SubmitIntent,
    request_id: StratumRequestId,
    observation: SubmitResponseObservation,
) -> SubmitClassification {
    match observation {
        SubmitResponseObservation::Response(response)
            if response.maybe_id == Some(request_id) && response.success =>
        {
            SubmitClassification::Accepted
        }
        SubmitResponseObservation::Response(response)
            if response.maybe_id == Some(request_id) && !response.success =>
        {
            SubmitClassification::Rejected {
                reason: RedactedRejectReason::from_response(&response),
            }
        }
        SubmitResponseObservation::Timeout => SubmitClassification::Timeout,
        SubmitResponseObservation::Malformed => SubmitClassification::Malformed,
        SubmitResponseObservation::Disconnected => SubmitClassification::Reconnect,
        SubmitResponseObservation::Blocked(reason) => SubmitClassification::Blocked { reason },
        _ => SubmitClassification::NoObservedShare,
    }
}
```

### Safe Stop Postcondition Assertion

```rust
// Source: local state, registry invalidation, and safety effect models.
#[test]
fn safe_stop_invalidates_runtime_work_and_blocks_submission() {
    // Arrange
    let mut runtime = LiveStratumRuntime::with_active_work_fixture();

    // Act
    let postconditions = runtime.safe_stop("operator_cancelled");

    // Assert
    assert!(postconditions.socket_stopped);
    assert!(postconditions.active_work_invalidated);
    assert_eq!(runtime.state().work_submission, WorkSubmissionGate::Blocked);
    assert_eq!(runtime.state().mining_activity, MiningActivityStatus::SafeBlocked);
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
| --- | --- | --- | --- |
| Controlled no-share transcript/evidence harness | Real socket runtime with controlled/fake-pool tests below live-proof tier | Phase 25 boundary from Phase 21/24 evidence | Do not promote accepted/rejected outcomes without a live socket response to a `SubmitIntent`. [VERIFIED: `scripts/phase21-live-mining-evidence.sh`; VERIFIED: `.planning/phases/25-live-stratum-runtime-and-safe-stop/25-CONTEXT.md`] |
| Direct share counter updates from synthetic response | Submit-intent-tied classifier | Phase 24 to Phase 25 handoff | A nonce/result and submit intent are necessary but not sufficient for STR-09. [VERIFIED: `docs/parity/evidence/phase-24-bm1366-production-work-path/result-correlation.md`] |
| Phase 23 placeholder slots | Phase 25 share-outcome and safe-stop evidence or explicit blockers | Phase 25 ownership | `share-outcome.md` and `safe-stop.md` currently remain pending placeholders. [VERIFIED: `docs/parity/evidence/phase-23-redacted-operator-evidence-workflow/share-outcome.md`; VERIFIED: `docs/parity/evidence/phase-23-redacted-operator-evidence-workflow/safe-stop.md`] |
| Phase 21 wrapper allow rules | Phase 25-specific allow-manifest surface/command | Required before hardware promotion | Current `mining_allow` accepts Phase 15/21 command shapes, not a Phase 25 live Stratum wrapper. [VERIFIED: `tools/parity/src/mining_allow.rs`] |

**Deprecated/outdated for Phase 25:**

- Treating `STR-008` historical checklist `verified` row as enough for v1.1 `STR-08` is outdated for this milestone; v1.1 `STR-08` requires real socket lifecycle in the new Phase 25 requirements. [VERIFIED: `docs/parity/checklist.md`; VERIFIED: `.planning/REQUIREMENTS.md`]
- Treating Phase 21 `share_submission_status=pool_response_classification_deferred` as accepted/rejected proof is invalid; Phase 25 owns real response classification. [VERIFIED: `firmware/bitaxe/src/controlled_mining_runtime.rs`; VERIFIED: `.planning/phases/25-live-stratum-runtime-and-safe-stop/25-CONTEXT.md`]

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
| --- | --- | --- | --- |
| A1 | The research remains valid until 2026-08-04 for local architecture and repo constraints. [ASSUMED] | Metadata | A later dependency or project-policy change could make specific socket/tooling advice stale before implementation. |

All implementation recommendations in this research are grounded in local source, local GSD context, local standards, tool-version checks, or official documentation; the only `[ASSUMED]` item is the validity-window estimate in metadata. [VERIFIED: source list below]

## Resolved Questions

1. **Will detector-gated hardware and safe prerequisites be available during execution?**
   - What we know: hardware runs must start with `just detect-ultra205`, and missing/unsafe prerequisites must produce a blocker instead of a success claim. [VERIFIED: `AGENTS.md`; VERIFIED: `.planning/phases/25-live-stratum-runtime-and-safe-stop/25-CONTEXT.md`]
   - Resolution: Phase 25 plans both paths explicitly. Execution must first build deterministic blocked/non-claim coverage, then attempt detector-gated hardware only through `just detect-ultra205` and repo-owned Phase 25 evidence tooling. If hardware, safe prerequisites, credentials, socket behavior, ASIC result, pool response, watchdog evidence, or redaction review is unavailable, the correct outcome is a stable blocked-safe-prerequisite artifact and non-claim, not a failed phase. [VERIFIED: `.planning/REQUIREMENTS.md`; VERIFIED: `.planning/phases/25-live-stratum-runtime-and-safe-stop/25-CONTEXT.md`; VERIFIED: `AGENTS.md`]

2. **Should Phase 25 replace or coexist with the Phase 21 controlled runtime mode?**
   - What we know: existing firmware uses `BITAXE_MINING_EVIDENCE_MODE=live-mining-runtime` plus `BITAXE_HARDWARE_EVIDENCE_ACK` to enter the controlled runtime path. [VERIFIED: `firmware/bitaxe/src/mining_evidence_mode.rs`; VERIFIED: `firmware/bitaxe/src/controlled_mining_runtime.rs`]
   - Resolution: Phase 25 uses a distinct live-socket opt-in marker, planned as `phase25-live-stratum-runtime` with acknowledgment `ultra205-phase25-live-stratum-safe-stop`. The existing Phase 21 controlled runtime mode remains separate so controlled no-share evidence cannot be confused with real Stratum socket evidence. [VERIFIED: `docs/parity/evidence/phase-23-redacted-operator-evidence-workflow/evidence-contract.md`; VERIFIED: `.planning/phases/25-live-stratum-runtime-and-safe-stop/25-CONTEXT.md`; VERIFIED: `.planning/phases/25-live-stratum-runtime-and-safe-stop/25-02-PLAN.md`]

3. **How much Phase 26 telemetry should be touched for SAFE-12?**
   - What we know: Phase 26 owns API/WebSocket/statistics/scoreboard promotion except minimal post-stop state refresh for SAFE-12. [VERIFIED: `.planning/phases/25-live-stratum-runtime-and-safe-stop/25-CONTEXT.md`; VERIFIED: `.planning/ROADMAP.md`]
   - Resolution: SAFE-12 telemetry boundary is limited to `runtime_snapshot` post-stop state refresh and redaction-safe evidence markers proving socket stopped, work invalidated, mining disabled, hardware control disabled, work submission blocked, and post-stop snapshot updated. Phase 25 must not add or promote statistics, scoreboard, counter semantics, broad API/WebSocket projection, or Phase 26 telemetry rows. [VERIFIED: `.planning/ROADMAP.md`; VERIFIED: `.planning/phases/25-live-stratum-runtime-and-safe-stop/25-CONTEXT.md`; VERIFIED: `firmware/bitaxe/src/runtime_snapshot.rs`]

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
| --- | --- | --- | --- | --- |
| Node | Evidence scripts and GSD tools | Yes | `v24.13.0` | None needed. [VERIFIED: shell version audit] |
| Cargo/Rust | Rust crate and firmware builds | Yes | Cargo `1.88.0-nightly`, rustc `1.88.0-nightly` | Use repo Bazel wrappers for canonical checks. [VERIFIED: shell version audit; VERIFIED: `Justfile`] |
| Bazel | Canonical build/test/package graph | Yes | `9.1.1` | None needed. [VERIFIED: shell version audit; VERIFIED: `.bazelversion` via command output] |
| `just` | Human command surface | Yes | `1.48.0` | Direct Bazel/script invocation if a `just` recipe is unavailable. [VERIFIED: shell version audit; VERIFIED: `Justfile`] |
| `espflash` | Ultra 205 board-info/flash/monitor | Yes | `4.0.1` | Hardware evidence blocks if `espflash` or device detection fails. [VERIFIED: shell version audit; VERIFIED: `AGENTS.md`] |
| `rg` | Redaction scans and source inspection | Yes | `15.1.0` | None needed. [VERIFIED: shell version audit; VERIFIED: `tools/parity/src/operator_evidence.rs`] |
| Ultra 205 hardware/USB port | STR-09/SAFE-13 hardware promotion | Unknown in research | Not probed | Record blocker if `just detect-ultra205` fails or is ambiguous. [VERIFIED: `AGENTS.md`] |
| Local pool credentials | Live pool submit response | Unknown in research | Runtime-only file if present | Record `pool_config: local-owner-supplied` or blocker; do not inspect file contents. [VERIFIED: `AGENTS.md`; VERIFIED: `scripts/phase21-pool-credentials-json.mjs`] |

**Missing dependencies with no fallback:**

- Detector-gated Ultra 205 hardware evidence has no safe substitute for STR-09 live accepted/rejected proof or hardware-level SAFE-13 promotion. [VERIFIED: `.planning/REQUIREMENTS.md`; VERIFIED: `AGENTS.md`]

**Missing dependencies with fallback:**

- Missing pool credentials, missing safe prerequisites, missing `DEVICE_URL`, or pool no-response can fall back to explicit blocked/non-claim evidence while deterministic STR-11 and implementation-level SAFE-12 work proceeds. [VERIFIED: `AGENTS.md`; VERIFIED: `.planning/phases/25-live-stratum-runtime-and-safe-stop/25-CONTEXT.md`; VERIFIED: `scripts/phase21-live-mining-evidence.sh`]

## Validation Architecture

### Test Framework

| Property | Value |
| --- | --- |
| Framework | Bazel `rust_test` and `sh_test` targets over Rust crates and scripts. [VERIFIED: `crates/bitaxe-stratum/BUILD.bazel`; VERIFIED: `scripts/BUILD.bazel`] |
| Config file | `MODULE.bazel`, per-package `BUILD.bazel`, `Justfile`. [VERIFIED: Glob `**/BUILD.bazel`; VERIFIED: `Justfile`] |
| Quick run command | `bazel test //crates/bitaxe-stratum:tests //crates/bitaxe-safety:tests` for pure runtime/safe-stop/watchdog work. [VERIFIED: `crates/bitaxe-stratum/BUILD.bazel`; VERIFIED: `crates/bitaxe-safety/BUILD.bazel`] |
| Full suite command | `just test` or `bazel test //...` before phase closure when practical. [VERIFIED: `Justfile`] |

### Phase Requirements -> Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
| --- | --- | --- | --- | --- |
| STR-08 | Live Stratum lifecycle transitions and socket-adapter-facing outbound/inbound events. [VERIFIED: `.planning/REQUIREMENTS.md`] | Unit + firmware build | `bazel test //crates/bitaxe-stratum:tests && bazel build //firmware/bitaxe:firmware` | Partial; new `live_runtime.rs` likely needed. [VERIFIED: `crates/bitaxe-stratum/src/v1.rs`; VERIFIED: `firmware/bitaxe/BUILD.bazel`] |
| STR-09 | Submit response classification tied to live ASIC-derived `SubmitIntent`. [VERIFIED: `.planning/REQUIREMENTS.md`] | Unit + hardware evidence wrapper | `bazel test //crates/bitaxe-stratum:tests` plus detector-gated Phase 25 wrapper | Partial; `SubmitIntent` exists, classifier missing. [VERIFIED: `crates/bitaxe-stratum/src/v1/production_work.rs`] |
| STR-11 | Fake-pool coverage for lifecycle, clean-jobs, reconnect, fallback, submit response, timeout, malformed, errors. [VERIFIED: `.planning/REQUIREMENTS.md`] | Unit | `bazel test //crates/bitaxe-stratum:tests` | Partial; existing fake-pool tests need expansion. [VERIFIED: `crates/bitaxe-stratum/src/v1/fake_pool.rs`] |
| SAFE-12 | Safe-stop postconditions and post-stop runtime snapshot refresh. [VERIFIED: `.planning/REQUIREMENTS.md`] | Unit + firmware build + evidence wrapper test | `bazel test //crates/bitaxe-stratum:tests //crates/bitaxe-safety:tests && bazel build //firmware/bitaxe:firmware` | Partial; safe-stop postcondition type missing. [VERIFIED: `crates/bitaxe-stratum/src/v1/state.rs`; VERIFIED: `firmware/bitaxe/src/runtime_snapshot.rs`] |
| SAFE-13 | Watchdog responsiveness under bounded socket/ASIC/API/WebSocket/evidence load. [VERIFIED: `.planning/REQUIREMENTS.md`] | Unit + script test + hardware evidence when available | `bazel test //crates/bitaxe-safety:tests //scripts:phase25_live_stratum_evidence_test` after target exists | Partial; watchdog core exists, Phase 25 wrapper target missing. [VERIFIED: `crates/bitaxe-safety/src/watchdog.rs`; VERIFIED: `scripts/BUILD.bazel`] |

### Sampling Rate

- **Per task commit:** Run affected pure target, usually `bazel test //crates/bitaxe-stratum:tests` or `bazel test //crates/bitaxe-safety:tests`. [VERIFIED: `standards/core/verification.md`; VERIFIED: `crates/bitaxe-stratum/BUILD.bazel`]
- **Per firmware adapter wave:** Run pure tests plus `bazel build //firmware/bitaxe:firmware`. [VERIFIED: `firmware/bitaxe/BUILD.bazel`; VERIFIED: `Justfile`]
- **Per evidence tooling wave:** Run script test target and parity report target for affected validators, then `just parity`. [VERIFIED: `scripts/BUILD.bazel`; VERIFIED: `Justfile`]
- **Phase gate:** Full relevant repo-native verification and detector-gated hardware evidence or explicit blocked non-claim before `/gsd-verify-work`. [VERIFIED: `standards/core/verification.md`; VERIFIED: `AGENTS.md`; VERIFIED: `.planning/config.json`]

### Wave 0 Gaps

- [ ] `crates/bitaxe-stratum/src/v1/live_runtime.rs` - pure live lifecycle, outbound message sequencing, stop/reconnect/fallback transitions for STR-08/SAFE-12. [VERIFIED: `crates/bitaxe-stratum/src/v1.rs`]
- [ ] `crates/bitaxe-stratum/src/v1/submit_response.rs` - pure classifier for STR-09 and STR-11. [VERIFIED: `crates/bitaxe-stratum/src/v1/production_work.rs`]
- [ ] Expanded `crates/bitaxe-stratum/src/v1/fake_pool.rs` tests - clean-jobs, reconnect generation, stale work, timeout, malformed, blocked, accepted, rejected. [VERIFIED: `crates/bitaxe-stratum/src/v1/fake_pool.rs`]
- [ ] `firmware/bitaxe/src/live_stratum_runtime.rs` - socket shell with timeouts/yields and redacted markers. [VERIFIED: `firmware/bitaxe/src/controlled_mining_runtime.rs`; CITED: https://docs.espressif.com/projects/esp-idf/en/v5.5.4/esp32s3/api-guides/lwip.html]
- [ ] `scripts/phase25-live-stratum-evidence.sh` and `scripts/phase25-live-stratum-evidence-test.sh` - detector-gated evidence wrapper and redaction tests. [VERIFIED: `scripts/BUILD.bazel`; VERIFIED: `scripts/phase21-live-mining-evidence.sh`]
- [ ] `tools/parity/src/mining_allow.rs` updates - Phase 25 surface/claim-tier/allowed command. [VERIFIED: `tools/parity/src/mining_allow.rs`]
- [ ] Phase 23 evidence-root slot updates for `share-outcome.md` and `safe-stop.md` when Phase 25 evidence exists. [VERIFIED: `docs/parity/evidence/phase-23-redacted-operator-evidence-workflow/share-outcome.md`; VERIFIED: `docs/parity/evidence/phase-23-redacted-operator-evidence-workflow/safe-stop.md`]

## Security Domain

Security enforcement is enabled because `.planning/config.json` has no `security_enforcement: false` override. [VERIFIED: `.planning/config.json`]

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
| --- | --- | --- |
| V1 Encoding and Sanitization | Yes | Redact logs/evidence at boundaries and avoid raw Stratum/BM1366/socket values in committed artifacts. [CITED: https://owasp.org/www-project-application-security-verification-standard/; VERIFIED: `tools/parity/src/operator_evidence.rs`] |
| V2 Validation and Business Logic | Yes | Use typed parsers, state machines, `SubmitIntent`, `PoolSessionGeneration`, and safe-stop postconditions. [CITED: https://owasp.org/www-project-application-security-verification-standard/; VERIFIED: `crates/bitaxe-stratum/src/v1/messages.rs`; VERIFIED: `crates/bitaxe-stratum/src/v1/production_work.rs`] |
| V4 API and Web Service | Limited | Phase 25 only refreshes post-stop API/WebSocket-visible state; full telemetry projection belongs to Phase 26. [CITED: https://owasp.org/www-project-application-security-verification-standard/; VERIFIED: `.planning/phases/25-live-stratum-runtime-and-safe-stop/25-CONTEXT.md`] |
| V11 Cryptography | No new application crypto | Do not add custom crypto for Stratum response or share logic in this phase; existing SHA/mining primitives stay in pure modules. [CITED: https://owasp.org/www-project-application-security-verification-standard/; VERIFIED: `crates/bitaxe-stratum/src/v1/coinbase.rs`; VERIFIED: `.planning/phases/25-live-stratum-runtime-and-safe-stop/25-CONTEXT.md`] |
| V12 Secure Communication | Limited/blocked by pool protocol | Stratum v1 pool connections may be plaintext depending on operator pool; committed evidence must redact endpoints and credentials and should not claim TLS unless implemented and observed. [CITED: https://owasp.org/www-project-application-security-verification-standard/; VERIFIED: `AGENTS.md`; VERIFIED: `docs/parity/evidence/phase-23-redacted-operator-evidence-workflow/evidence-contract.md`] |

### Known Threat Patterns for Phase 25

| Pattern | STRIDE | Standard Mitigation |
| --- | --- | --- |
| Secret or endpoint disclosure in logs/evidence | Information Disclosure | Category-label logs, redaction filters, forbidden-sentinel tests, and no raw artifact commits. [VERIFIED: `tools/parity/src/operator_evidence.rs`; VERIFIED: `scripts/phase21-live-mining-evidence.sh`] |
| Share overclaim from synthetic/fake-pool response | Repudiation / Integrity | Require detector-gated live socket response tied to `SubmitIntent`; fake-pool evidence remains deterministic only. [VERIFIED: `.planning/phases/25-live-stratum-runtime-and-safe-stop/25-CONTEXT.md`; VERIFIED: `docs/parity/evidence/phase-24-bm1366-production-work-path/result-correlation.md`] |
| Stale work submitted after clean-jobs/reconnect/stop | Tampering / Safety | Invalidate generation, queue, valid jobs, and active work on clean-jobs/reconnect/safe stop. [VERIFIED: `crates/bitaxe-stratum/src/v1/production_work.rs`] |
| Watchdog starvation from blocking I/O | Denial of Service | Short timeouts, explicit checkpoints/yields, bounded evidence probes, and watchdog status evidence. [CITED: https://docs.espressif.com/projects/esp-idf/en/v5.5.4/esp32s3/api-reference/system/wdts.html; VERIFIED: `crates/bitaxe-safety/src/watchdog.rs`] |
| Unsafe hardware enablement without prerequisites | Elevation of Privilege / Safety | Keep live startup fail-closed behind Phase 22 prerequisite readiness and hardware evidence ack. [VERIFIED: `crates/bitaxe-stratum/src/v1/mining_loop.rs`; VERIFIED: `.planning/phases/25-live-stratum-runtime-and-safe-stop/25-CONTEXT.md`] |

## Sources

### Primary (HIGH confidence)

- `.planning/phases/25-live-stratum-runtime-and-safe-stop/25-CONTEXT.md` - locked decisions, scope, deferred surfaces, and canonical implementation surfaces. [VERIFIED]
- `.planning/REQUIREMENTS.md` - STR-08, STR-09, STR-11, SAFE-12, SAFE-13 requirement text. [VERIFIED]
- `.planning/ROADMAP.md` - Phase 25 goal and success criteria. [VERIFIED]
- `.planning/STATE.md` and `.planning/PROJECT.md` - v1.1 decisions, Phase 24 handoff, active milestone constraints. [VERIFIED]
- `AGENTS.md`, `AGENTS.bright-builds.md`, `standards/core/architecture.md`, `standards/core/testing.md`, `standards/core/verification.md`, `standards/languages/rust.md` - repo workflow and coding/testing constraints. [VERIFIED]
- `crates/bitaxe-stratum/src/v1/messages.rs`, `fake_pool.rs`, `production_work.rs`, `mining_loop.rs`, `state.rs`, `queue.rs`, `controlled_runtime.rs` - local pure Stratum/runtime implementation surfaces. [VERIFIED]
- `crates/bitaxe-safety/src/watchdog.rs`, `effects.rs` - local watchdog and safety effect model. [VERIFIED]
- `firmware/bitaxe/src/controlled_mining_runtime.rs`, `mining_evidence_mode.rs`, `runtime_snapshot.rs`, `network_stack.rs`, `http_api.rs` - firmware shell and post-stop/API touchpoints. [VERIFIED]
- `tools/parity/src/mining_allow.rs`, `operator_evidence.rs`, `scripts/phase21-live-mining-evidence.sh`, `scripts/phase23-redacted-operator-evidence.sh`, `scripts/BUILD.bazel`, `Justfile` - evidence, allow-manifest, redaction, and command surfaces. [VERIFIED]
- ESP-IDF v5.5.4 lwIP docs - BSD socket support, timeout/nonblocking options, `shutdown`, `select`, thread-safety warning. [CITED: https://docs.espressif.com/projects/esp-idf/en/v5.5.4/esp32s3/api-guides/lwip.html]
- ESP-IDF v5.5.4 watchdog docs - TWDT/IWDT behavior and reset/yield requirements. [CITED: https://docs.espressif.com/projects/esp-idf/en/v5.5.4/esp32s3/api-reference/system/wdts.html]
- OWASP ASVS project page - ASVS v5.0.0 availability and chapter referencing model. [CITED: https://owasp.org/www-project-application-security-verification-standard/]

### Secondary (MEDIUM confidence)

- esp-idf-svc TCP example on `master` - demonstrates blocking `std::net::TcpStream` and `TcpListener` with ESP-IDF services; useful as a current example but not pinned to the repo's exact crate tag. [CITED: https://raw.githubusercontent.com/esp-rs/esp-idf-svc/master/examples/tcp.rs]
- Web search results for esp-rs TCP usage - corroborated the `std::net` approach but not used as sole authority. [VERIFIED: WebSearch; cross-checked with ESP-IDF docs and esp-idf-svc example]

### Tertiary (LOW confidence)

- None used for recommendations. [VERIFIED: source review]

## Metadata

**Confidence breakdown:**

- Standard stack: HIGH - local `Cargo.toml`, firmware metadata, and shell version audit verified the versions and command availability. [VERIFIED: `Cargo.toml`; VERIFIED: `firmware/bitaxe/Cargo.toml`; VERIFIED: shell version audit]
- Architecture: HIGH - Phase 25 context and repo standards explicitly lock functional core / imperative shell and name implementation surfaces. [VERIFIED: `.planning/phases/25-live-stratum-runtime-and-safe-stop/25-CONTEXT.md`; VERIFIED: `standards/core/architecture.md`]
- Pitfalls: HIGH for local claim/evidence pitfalls and MEDIUM-HIGH for socket/watchdog pitfalls - local code proves current gaps and official ESP-IDF docs verify the runtime constraints. [VERIFIED: `tools/parity/src/mining_allow.rs`; CITED: https://docs.espressif.com/projects/esp-idf/en/v5.5.4/esp32s3/api-guides/lwip.html; CITED: https://docs.espressif.com/projects/esp-idf/en/v5.5.4/esp32s3/api-reference/system/wdts.html]
- Live STR-09 feasibility: MEDIUM - code/evidence paths are clear, but real pool response proof depends on detector-gated hardware, safe prerequisites, network, credentials, pool difficulty, and ASIC result behavior during execution. [VERIFIED: `.planning/STATE.md`; VERIFIED: `AGENTS.md`]

**Research date:** 2026-07-05  
**Valid until:** 2026-08-04 for local architecture and repo constraints; re-check ESP-IDF/esp-rs docs if dependencies change before implementation. [VERIFIED: `Cargo.toml`; ASSUMED: validity window estimate]
