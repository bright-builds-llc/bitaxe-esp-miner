# Phase 24: BM1366 Production Work Path - Research

**Researched:** 2026-07-05  
**Domain:** BM1366 production work modeling, Stratum v1 work correlation, firmware ASIC adapter boundaries  
**Confidence:** HIGH for repo-local architecture and required verification; MEDIUM for optional hardware execution details because hardware evidence is detector-gated and environment-dependent. [VERIFIED: .planning/phases/24-bm1366-production-work-path/24-CONTEXT.md] [VERIFIED: AGENTS.md]

<user_constraints>
## User Constraints (from CONTEXT.md)

The locked decisions, discretion area, and deferred ideas below are copied verbatim from `24-CONTEXT.md`; treat the whole block as sourced to that file. [VERIFIED: .planning/phases/24-bm1366-production-work-path/24-CONTEXT.md]

### Locked Decisions

## Implementation Decisions

### Production vs Diagnostic ASIC Modes

- **D-01:** Preserve a hard type-level boundary between diagnostic BM1366 chip/work paths and trusted production BM1366 initialization, work dispatch, and result handling. Diagnostic evidence may support readiness, but it must not automatically promote production work claims.
- **D-02:** Production mode requires the Phase 22 prerequisite readiness contract before work dispatch. Missing, stale, unavailable, unsafe, ambiguous, or undocumented safety observations keep mining disabled and work submission disabled with stable redaction-safe blocker reasons.
- **D-03:** Firmware may interpret typed production ASIC actions, but it must not construct raw BM1366 frames in the shell. Raw packet details, CRCs, work payload encoding, valid-job tracking, and result parsing stay in `crates/bitaxe-asic`.

### Pool-Derived Work Dispatch

- **D-04:** BM1366 production work must be derived from the active Stratum v1 pool job, including job id, extranonce context, difficulty or target context, clean-jobs generation, and enough metadata to later validate a nonce/result.
- **D-05:** Work dispatch should use a typed active-work registry or equivalent functional-core model that binds each sent BM1366 job to the current pool session generation. Clean-jobs, reconnect, authorization reset, or pool-session replacement invalidates older active work before another share claim can be recorded.
- **D-06:** Production work dispatch should be testable without hardware by replaying fake-pool or fixture jobs through the Stratum job model into BM1366 work commands and active-work records. Hardware evidence remains required before safety-critical behavior is marked verified.

### Result Correlation and Share Claim Gate

- **D-07:** A live BM1366 nonce or result observation is not a share claim until the functional core maps it to an active, non-stale pool-derived work record and computes the corresponding submit context.
- **D-08:** Uncorrelated, stale-generation, malformed, duplicate, invalid-job, wrong-session, or target-mismatched results must fail closed with stable blocker/status outcomes and no share submission claim.
- **D-09:** The production path may prepare redaction-safe submit intent data for Phase 25, but this phase should not claim accepted or rejected pool response behavior unless the later Stratum runtime actually observes and classifies the response.

### Fail-Closed Errors and Redaction

- **D-10:** Initialization, UART, reset, timeout, malformed frame, result-parse, job-correlation, stale-work, and prerequisite failures must produce typed, redaction-safe reasons suitable for logs, evidence summaries, API/WebSocket projection later, and parity checks.
- **D-11:** Committed evidence and planning artifacts must never include raw BM1366 frames, raw Stratum targets, extranonces, share payloads, pool endpoints, ports, workers, owner addresses, passwords, socket errors, device URLs, IPs, MACs, Wi-Fi values, tokens, or NVS secrets.
- **D-12:** The Phase 23 redacted evidence-root contract remains the committed evidence shape. Slots that depend on Phase 25 or Phase 26 should remain blocked or pending with exact non-claims instead of being inferred from Phase 24 implementation.

### Verification and Evidence Semantics

- **D-13:** Pure unit and fixture tests should cover diagnostic-vs-production mode separation, pool-job-to-BM1366-work derivation, clean-jobs/reconnect invalidation, active-work result correlation, duplicate/stale/malformed rejection, fail-closed reasons, and redaction-safe rendering.
- **D-14:** Hardware-capable verification must follow the Ultra 205 detector gate, use repo-owned commands, record board `205`, selected port, source commit, reference commit, package identity, exact commands, board-info output, captured logs, observed behavior, conclusion, and redaction review, and stop on ambiguous ports or unsafe prerequisites.
- **D-15:** Checklist promotion should be exact. `ASIC-09` through `ASIC-12` may advance only to the level supported by implemented code, deterministic tests, and any detector-gated evidence actually produced during the phase.

### Claude's Discretion

Claude may choose the exact module names, type names, plan count, fixture format, active-work registry shape, error enum names, helper script boundaries, and checklist wording. Those choices must preserve functional core / imperative shell structure, read-only reference policy, ESP-IDF tooling preference, conservative evidence semantics, typed fail-closed behavior, and redaction rules.

### Deferred Ideas (OUT OF SCOPE)

- Real Stratum v1 socket lifecycle, deterministic fake-pool production tests, live accepted/rejected share response classification, watchdog under bounded production load, and bounded safe-stop runtime proof belong to Phase 25.
- API/WebSocket/statistics/scoreboard promotion and final v1.1 parity closure belong to Phase 26.
- Full active voltage/fan/thermal fault-stimulus, self-test hardware closure, OTA/recovery destructive or fault-injection evidence, runtime display/input, BAP, Stratum v2, non-205 boards, other ASIC families, and unbounded stress mining remain future work.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| ASIC-09 | Ultra 205 production mining separates BM1366 diagnostic chip/work modes from trusted production initialization and work-result modes. | Use distinct diagnostic and production command/status/result types; do not reuse `SendDiagnosticWork` for production dispatch. [VERIFIED: .planning/REQUIREMENTS.md] [VERIFIED: crates/bitaxe-asic/src/bm1366/command.rs] |
| ASIC-10 | Ultra 205 production mining dispatches BM1366 work derived from the active pool job, tracks job/extranonce/difficulty context, and invalidates stale work on clean-jobs or reconnect. | Extend existing `MiningWork`, `MiningWorkQueue`, and valid-job tracking into a session-generation-aware production active-work registry. [VERIFIED: .planning/REQUIREMENTS.md] [VERIFIED: crates/bitaxe-stratum/src/v1/mining.rs] [VERIFIED: crates/bitaxe-stratum/src/v1/queue.rs] |
| ASIC-11 | Ultra 205 production mining maps live BM1366 nonce or result observations back to active pool work before any share submission claim is recorded. | Add a pure correlation gate that accepts parsed BM1366 nonce results only when the active-work registry proves current session, valid job, non-duplicate, and share-intent context. [VERIFIED: .planning/REQUIREMENTS.md] [VERIFIED: crates/bitaxe-asic/src/bm1366/result.rs] [VERIFIED: crates/bitaxe-stratum/src/v1/mining_loop.rs] |
| ASIC-12 | Ultra 205 production mining fails closed on BM1366 initialization, UART, reset, timeout, malformed result, or job-correlation failures without leaking raw frames in committed evidence. | Reuse stable safety blocker strings and add production ASIC blocker reasons rendered as category labels only; route hardware evidence through Phase 23 redacted slots. [VERIFIED: .planning/REQUIREMENTS.md] [VERIFIED: docs/parity/evidence/phase-22-claim-ladder-and-safety-preconditions/blocker-reasons.md] [VERIFIED: docs/parity/evidence/phase-23-redacted-operator-evidence-workflow/evidence-contract.md] |
</phase_requirements>

## Summary

Phase 24 should be planned as a functional-core strengthening phase, not a live-pool runtime phase. [VERIFIED: .planning/ROADMAP.md] The existing code already parses Stratum v1 notify, extranonce, difficulty, reconnect, and submit-shaped messages; builds BM1366 work fields from a pool job; encodes diagnostic BM1366 work frames; parses BM1366 result frames; and tracks valid jobs in a queue. [VERIFIED: crates/bitaxe-stratum/src/v1/messages.rs] [VERIFIED: crates/bitaxe-stratum/src/v1/mining.rs] [VERIFIED: crates/bitaxe-asic/src/bm1366/work.rs] [VERIFIED: crates/bitaxe-asic/src/bm1366/result.rs] [VERIFIED: crates/bitaxe-stratum/src/v1/queue.rs]

The planner should create a hard production-mode path beside the diagnostic path. [VERIFIED: .planning/phases/24-bm1366-production-work-path/24-CONTEXT.md] The production path should derive typed BM1366 work from active Stratum job context, register dispatched work under a pool-session generation, invalidate older work on clean-jobs or reconnect, and convert parsed nonce results into redaction-safe submit intents only after correlation succeeds. [VERIFIED: crates/bitaxe-stratum/src/v1/mining.rs] [VERIFIED: crates/bitaxe-stratum/src/v1/queue.rs] [VERIFIED: crates/bitaxe-stratum/src/v1/mining_loop.rs]

Firmware work should remain a thin interpreter for typed production actions and redaction-safe statuses. [VERIFIED: AGENTS.md] [VERIFIED: standards/core/architecture.md] Raw frame construction, BM1366 CRC and payload layout, valid-job state, and result parsing should remain in `crates/bitaxe-asic`; live Stratum socket I/O and accepted/rejected pool response classification should remain Phase 25 non-claims. [VERIFIED: crates/bitaxe-asic/src/bm1366/command.rs] [VERIFIED: crates/bitaxe-asic/src/bm1366/result.rs] [VERIFIED: .planning/phases/24-bm1366-production-work-path/24-CONTEXT.md]

**Primary recommendation:** Plan two pure-core waves before any firmware shell work: first add typed production BM1366 work/result state in `bitaxe-asic` and `bitaxe-stratum`, then wire firmware/log/parity surfaces to those types without claiming Phase 25 live share responses. [VERIFIED: crates/bitaxe-asic/src/bm1366.rs] [VERIFIED: crates/bitaxe-stratum/src/v1.rs] [VERIFIED: docs/parity/checklist.md]

## Project Constraints

- No `.cursor/rules/`, `.cursor/skills/`, or `.agents/skills/` project-local files were found in this repository. [VERIFIED: Glob .cursor/rules/**] [VERIFIED: Glob .cursor/skills/**/SKILL.md] [VERIFIED: Glob .agents/skills/**/SKILL.md]
- Keep `reference/esp-miner` read-only; use it as behavioral evidence and breadcrumbs, not as an edit target. [VERIFIED: AGENTS.md] [VERIFIED: docs/adr/0005-read-only-reference-implementation.md]
- Preserve functional core / imperative shell: pure ASIC, Stratum, safety, redaction, and parity decisions belong in crates and tools; ESP-IDF UART, GPIO, NVS, socket I/O, serial capture, and hardware effects belong in firmware/tools/scripts. [VERIFIED: AGENTS.md] [VERIFIED: standards/core/architecture.md]
- Use ESP-IDF Rust, Bazel, and `just` as the canonical stack and command surface. [VERIFIED: AGENTS.md] [VERIFIED: Justfile] [VERIFIED: MODULE.bazel]
- Do not commit raw BM1366 frames, raw Stratum targets, extranonces, share payloads, pool endpoints, ports, workers, owner addresses, passwords, socket errors, device URLs, IPs, MACs, Wi-Fi values, tokens, or NVS secrets. [VERIFIED: AGENTS.md] [VERIFIED: docs/parity/evidence/phase-23-redacted-operator-evidence-workflow/evidence-contract.md]
- Hardware-capable verification must start with `just detect-ultra205` and stop on zero ports, multiple ports, board-info failure, non-205 target, unsafe prerequisites, or missing recovery/evidence instructions. [VERIFIED: AGENTS.md]
- Markdown planning artifacts parsed by GSD must not use standalone body `---` separators after YAML frontmatter. [VERIFIED: AGENTS.md]

## Standard Stack

### Core

| Library / Surface | Version | Purpose | Why Standard |
|-------------------|---------|---------|--------------|
| Rust workspace crates | edition 2021 | Own pure ASIC, Stratum, safety, API, parity, and host-tool logic. | The workspace is declared in `Cargo.toml`, and firmware plus host crates share Rust 2021. [VERIFIED: Cargo.toml] |
| `crates/bitaxe-asic` | workspace crate | Own BM1366 packet, work, result, init, observation, and production ASIC types. | Existing module layout already owns raw BM1366 frame encoding, valid-job parsing, and observations. [VERIFIED: crates/bitaxe-asic/src/bm1366.rs] |
| `crates/bitaxe-stratum` | workspace crate | Own Stratum v1 messages, pool-derived work, active-work registry, and share-intent mapping. | Existing code already parses Stratum v1, builds `MiningWork`, maintains queue state, and plans guarded dispatch. [VERIFIED: crates/bitaxe-stratum/src/v1/messages.rs] [VERIFIED: crates/bitaxe-stratum/src/v1/mining.rs] [VERIFIED: crates/bitaxe-stratum/src/v1/mining_loop.rs] |
| `crates/bitaxe-safety` | workspace crate | Own production precondition and stable blocker reason inputs. | Phase 22 safety gates already feed `MiningLoopGate` before work submission. [VERIFIED: crates/bitaxe-safety/src/mining_preconditions.rs] [VERIFIED: crates/bitaxe-stratum/src/v1/mining_loop.rs] |
| `firmware/bitaxe` | ESP-IDF Rust firmware | Interpret typed ASIC actions and publish redaction-safe statuses. | Firmware currently interprets `Bm1366AdapterAction` and avoids constructing Stratum/BM1366 domain decisions directly. [VERIFIED: firmware/bitaxe/src/asic_adapter.rs] |
| `tools/parity` | Rust CLI | Enforce checklist and evidence overclaim guardrails. | Parity tooling already validates mining allow manifests, claim tiers, redaction, and invalid verified rows. [VERIFIED: tools/parity/src/mining_allow.rs] [VERIFIED: tools/parity/BUILD.bazel] |

### Supporting

| Library / Tool | Version | Purpose | When to Use |
|----------------|---------|---------|-------------|
| `serde` / `serde_json` | `serde 1.0.228`, `serde_json 1.0.150` | Typed fixture, evidence, and message parsing. | Use for checked-in fixtures, parity manifests, and deterministic redaction tests. [VERIFIED: Cargo.toml] |
| `thiserror` | `2.0.18` | Library error enums. | Use for new ASIC/Stratum typed fail-closed errors instead of stringly error propagation. [VERIFIED: Cargo.toml] |
| `anyhow` | `1.0.102` | Firmware and CLI adapter errors. | Use only in shell/tool layers where context wrapping is more useful than exported domain variants. [VERIFIED: Cargo.toml] |
| `sha2` | `0.11.0` | Stratum coinbase and work hashing. | Reuse existing Stratum hashing helpers; do not add a new hashing library for Phase 24. [VERIFIED: Cargo.toml] [VERIFIED: crates/bitaxe-stratum/src/v1/mining.rs] |
| Bazel / `rules_rust` | Bazel `9.1.1`, `rules_rust 0.70.0` | Canonical build and test graph. | Add new Rust source files to the appropriate `BUILD.bazel` `srcs` list and verify via Bazel targets. [VERIFIED: .bazelversion] [VERIFIED: MODULE.bazel] |
| `just` | `1.48.0` available locally | Human command surface. | Use `just test`, `just package`, `just parity`, and `just detect-ultra205` rather than ad hoc command chains. [VERIFIED: Justfile] [VERIFIED: environment probe] |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Existing `bitaxe-asic` + `bitaxe-stratum` pure core | New production-mining crate | A new crate would add boundaries before the domain model proves it needs them; existing crates already own the relevant ASIC and Stratum concepts. [VERIFIED: crates/bitaxe-asic/BUILD.bazel] [VERIFIED: crates/bitaxe-stratum/BUILD.bazel] |
| Typed production action enum | Reusing `Bm1366Command::SendDiagnosticWork` | Reusing the diagnostic variant would blur the Phase 24 type-level diagnostic/production boundary. [VERIFIED: crates/bitaxe-asic/src/bm1366/command.rs] [VERIFIED: .planning/phases/24-bm1366-production-work-path/24-CONTEXT.md] |
| Explicit active-work registry | Only `Bm1366ValidJobIds` bitset | The bitset proves a BM1366 job id is valid but does not by itself carry pool session generation, extranonce, difficulty, duplicate, or submit-intent context. [VERIFIED: crates/bitaxe-asic/src/bm1366/result.rs] [VERIFIED: crates/bitaxe-stratum/src/v1/queue.rs] |
| Pure submit intent | Direct live `mining.submit` from Phase 24 | Live socket I/O and accepted/rejected response classification belong to Phase 25. [VERIFIED: .planning/phases/24-bm1366-production-work-path/24-CONTEXT.md] [VERIFIED: .planning/ROADMAP.md] |

**Installation:** No new dependency installation is recommended for Phase 24. [VERIFIED: Cargo.toml] The planner should add modules and Bazel `srcs` entries inside existing crates. [VERIFIED: crates/bitaxe-asic/BUILD.bazel] [VERIFIED: crates/bitaxe-stratum/BUILD.bazel]

**Version verification:** Versions above were verified from checked-in manifests and the local tool probe, not from training data. [VERIFIED: Cargo.toml] [VERIFIED: MODULE.bazel] [VERIFIED: .bazelversion] [VERIFIED: environment probe]

## Architecture Patterns

### Recommended Project Structure

```text
crates/bitaxe-asic/src/bm1366.rs
crates/bitaxe-asic/src/bm1366/
  production.rs            # production init/work/result mode types and typed actions
  work.rs                  # keep shared payload layout and diagnostic work helpers
  result.rs                # keep raw result-frame parsing and valid-job primitives

crates/bitaxe-stratum/src/v1.rs
crates/bitaxe-stratum/src/v1/
  production_work.rs       # pool-session generation, active registry, correlation gate
  mining.rs                # keep work-field and submit-message mapping helpers
  queue.rs                 # keep bounded queue or wrap it from production registry
  mining_loop.rs           # plan-level integration without live socket ownership

firmware/bitaxe/src/
  asic_adapter.rs          # interpret typed production ASIC actions
  asic_adapter/status.rs   # publish redaction-safe production statuses
```

The exact module names are discretionary, but the planner should keep `foo.rs` plus `foo/` layout for new multi-file Rust modules. [VERIFIED: standards/languages/rust.md] [VERIFIED: .planning/phases/24-bm1366-production-work-path/24-CONTEXT.md]

### Pattern 1: Type-Level Diagnostic vs Production Boundary

**What:** Add production-specific variants or wrapper types so diagnostic work cannot be accidentally treated as trusted production work. [VERIFIED: .planning/phases/24-bm1366-production-work-path/24-CONTEXT.md]

**When to use:** Use this boundary for initialization status, work dispatch, result correlation, and firmware log/status publication. [VERIFIED: crates/bitaxe-asic/src/bm1366/adapter_gate.rs] [VERIFIED: crates/bitaxe-asic/src/bm1366/observation.rs]

**Recommended shape:**

```rust
// Proposed pattern, source basis: existing diagnostic command and init status types.
pub enum Bm1366ProductionCommand {
    SendProductionWork(ProductionWorkPayload),
    ReadProductionResult,
}

pub enum ProductionAsicStatus {
    InitializedForProduction,
    WorkDispatched,
    ResultCorrelated,
    FailClosed { reason: ProductionAsicBlocker },
}
```

Existing diagnostic surfaces include `AsicAdapterMode::WorkResultDiagnostic`, `Bm1366Command::SendDiagnosticWork`, and `AsicInitStatus::InitializedNoMining`; production types should not reuse those names for trusted mining claims. [VERIFIED: crates/bitaxe-asic/src/bm1366/adapter_gate.rs] [VERIFIED: crates/bitaxe-asic/src/bm1366/command.rs] [VERIFIED: crates/bitaxe-asic/src/bm1366/observation.rs]

### Pattern 2: Session-Generation Active Work Registry

**What:** Model pool session generation as a typed value that changes on clean-jobs, reconnect, authorization reset, or pool-session replacement. [VERIFIED: .planning/phases/24-bm1366-production-work-path/24-CONTEXT.md]

**When to use:** Use the registry when enqueueing pool-derived work, dispatching BM1366 work, parsing nonce observations, and preparing submit intent. [VERIFIED: crates/bitaxe-stratum/src/v1/mining.rs] [VERIFIED: crates/bitaxe-stratum/src/v1/queue.rs] [VERIFIED: crates/bitaxe-stratum/src/v1/mining_loop.rs]

**Recommended shape:**

```rust
// Proposed pattern, source basis: MiningWorkQueue plus Bm1366ValidJobIds.
pub struct PoolSessionGeneration(u64);

pub struct ActiveProductionWork {
    pub generation: PoolSessionGeneration,
    pub work: MiningWork,
    pub dispatched: bool,
    pub result_seen: bool,
}

pub enum CorrelationOutcome {
    SubmitIntent(ShareSubmission),
    Blocked { reason: ProductionAsicBlocker },
}
```

The registry should contain enough context to later build `ShareSubmission` without exposing raw target, extranonce, pool identity, or raw frame bytes in logs or evidence. [VERIFIED: crates/bitaxe-stratum/src/v1/mining.rs] [VERIFIED: docs/parity/evidence/phase-23-redacted-operator-evidence-workflow/evidence-contract.md]

### Pattern 3: Parse at Boundaries, Then Correlate Domain Types

**What:** Parse BM1366 frames into `Bm1366ParsedResult` in `bitaxe-asic`, then correlate only `Bm1366NonceResult` against typed active production work in `bitaxe-stratum`. [VERIFIED: crates/bitaxe-asic/src/bm1366/result.rs] [VERIFIED: standards/core/architecture.md]

**When to use:** Use this whenever firmware receives bytes from UART or tests replay result fixtures. [VERIFIED: firmware/bitaxe/src/asic_adapter.rs] [VERIFIED: crates/bitaxe-asic/src/bm1366/result.rs]

**Key invariant:** A parsed nonce is not a share claim until registry correlation succeeds. [VERIFIED: .planning/phases/24-bm1366-production-work-path/24-CONTEXT.md]

### Pattern 4: Redaction-Safe Status Contracts

**What:** Render stable category labels such as blocker reasons and status names, not raw values. [VERIFIED: docs/parity/evidence/phase-22-claim-ladder-and-safety-preconditions/blocker-reasons.md] [VERIFIED: docs/parity/evidence/phase-23-redacted-operator-evidence-workflow/evidence-contract.md]

**When to use:** Use for firmware logs, evidence summaries, parity checklist notes, and later API/WebSocket projection. [VERIFIED: firmware/bitaxe/src/asic_adapter/status.rs] [VERIFIED: crates/bitaxe-stratum/src/v1/state.rs]

**Recommended blocker categories:** `production_asic_init_failed`, `production_uart_failed`, `production_reset_failed`, `production_result_timeout`, `production_result_malformed`, `production_work_stale`, `production_job_uncorrelated`, `production_duplicate_result`, and `production_target_mismatch`. [ASSUMED]

### Anti-Patterns to Avoid

- **Reusing diagnostic variants for production claims:** `SendDiagnosticWork` currently names a diagnostic surface; production dispatch should get a distinct type or variant. [VERIFIED: crates/bitaxe-asic/src/bm1366/command.rs]
- **Counting shares from ASIC results alone:** Share counters should not advance from nonce parsing; Phase 24 may create submit intent, while Phase 25 owns live pool response classification. [VERIFIED: crates/bitaxe-stratum/src/v1/fake_pool.rs] [VERIFIED: .planning/phases/24-bm1366-production-work-path/24-CONTEXT.md]
- **Keeping only a valid-job bitset:** Valid-job tracking is necessary but insufficient for session generation, duplicate rejection, and redaction-safe submit context. [VERIFIED: crates/bitaxe-asic/src/bm1366/result.rs] [VERIFIED: crates/bitaxe-stratum/src/v1/queue.rs]
- **Logging raw diagnostic or production details:** Firmware status should not print frame bytes, targets, extranonces, pool identity, socket errors, share payloads, or NVS secrets. [VERIFIED: AGENTS.md] [VERIFIED: docs/parity/evidence/phase-23-redacted-operator-evidence-workflow/redaction-review.md]

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| BM1366 raw payload or CRC handling in firmware | Ad hoc byte arrays or CRC in `firmware/bitaxe` | `crates/bitaxe-asic` packet, work, command, and result modules | Existing code owns typed BM1366 payload, frame, CRC, and result parsing. [VERIFIED: crates/bitaxe-asic/src/bm1366/work.rs] [VERIFIED: crates/bitaxe-asic/src/bm1366/packet.rs] [VERIFIED: crates/bitaxe-asic/src/bm1366/result.rs] |
| Stratum JSON parsing | String parsing in firmware or scripts | `crates/bitaxe-stratum/src/v1/messages.rs` | Existing parser handles notify, difficulty, extranonce, version mask, reconnect, response, and submit serialization. [VERIFIED: crates/bitaxe-stratum/src/v1/messages.rs] |
| Coinbase and merkle work derivation | New hashing helpers | Existing `coinbase.rs` and `MiningWorkBuilder` | Existing code already builds BM1366 work fields from notify/extranonce and rejects unsupported nonzero version masks. [VERIFIED: crates/bitaxe-stratum/src/v1/mining.rs] |
| Production safety gating | Firmware-local booleans | `ProductionMiningPreconditions`, `MiningLoopGate`, and `MiningRuntimeState` | Phase 22 already established typed readiness and stable blocker propagation. [VERIFIED: crates/bitaxe-safety/src/mining_preconditions.rs] [VERIFIED: crates/bitaxe-stratum/src/v1/mining_loop.rs] |
| Redaction review | One-off manual grep prose | Phase 23 evidence contract plus parity/operator-evidence validators | Required evidence slots and forbidden categories are already documented and testable. [VERIFIED: docs/parity/evidence/phase-23-redacted-operator-evidence-workflow/evidence-contract.md] [VERIFIED: tools/parity/src/operator_evidence.rs] |
| Accepted/rejected response claims | Phase 24 fake acceptance counters | Phase 25 Stratum runtime and fake-pool/response classification | Phase 24 scope explicitly excludes live accepted/rejected pool response behavior. [VERIFIED: .planning/phases/24-bm1366-production-work-path/24-CONTEXT.md] |

**Key insight:** Phase 24’s hardest bug class is false correlation, not byte encoding. [VERIFIED: .planning/phases/24-bm1366-production-work-path/24-CONTEXT.md] The plan should spend most implementation effort on session-generation state, invalidation, duplicate/stale rejection, and redaction-safe outcomes. [VERIFIED: crates/bitaxe-stratum/src/v1/queue.rs] [VERIFIED: crates/bitaxe-stratum/src/v1/mining_loop.rs]

## Common Pitfalls

### Pitfall 1: Diagnostic Evidence Accidentally Promotes Production Claims

**What goes wrong:** A diagnostic work-result path is treated as production work dispatch. [VERIFIED: docs/parity/checklist.md]  
**Why it happens:** Existing types can already create BM1366 work payloads and diagnostic commands, so it is tempting to reuse them for production. [VERIFIED: crates/bitaxe-asic/src/bm1366/work.rs] [VERIFIED: crates/bitaxe-asic/src/bm1366/command.rs]  
**How to avoid:** Introduce production-named types and statuses and update tests to prove diagnostic and production modes are not interchangeable. [VERIFIED: .planning/phases/24-bm1366-production-work-path/24-CONTEXT.md]  
**Warning signs:** `SendDiagnosticWork` appears in a production work-dispatch assertion or production firmware status. [VERIFIED: crates/bitaxe-stratum/src/v1/mining_loop.rs]

### Pitfall 2: Clean-Jobs Clears Queue But Not Active Results

**What goes wrong:** A stale ASIC nonce maps to a previously dispatched job after clean-jobs or reconnect. [VERIFIED: reference/esp-miner/main/system.c via rg]  
**Why it happens:** Queue clearing and valid-job invalidation can be implemented without also invalidating active dispatched work. [VERIFIED: crates/bitaxe-stratum/src/v1/queue.rs]  
**How to avoid:** Treat clean-jobs and reconnect as generation-changing events that clear queued, valid, and active production work before another submit intent can be emitted. [VERIFIED: .planning/phases/24-bm1366-production-work-path/24-CONTEXT.md]  
**Warning signs:** Tests clear only the FIFO queue while `maybe_active_work` can still return old work. [VERIFIED: crates/bitaxe-stratum/src/v1/queue.rs]

### Pitfall 3: Parsed Nonce Becomes a Share Claim Too Early

**What goes wrong:** A nonce result creates a submit message or increments counters before pool-session correlation and response classification. [VERIFIED: crates/bitaxe-stratum/src/v1/mining_loop.rs] [VERIFIED: crates/bitaxe-stratum/src/v1/fake_pool.rs]  
**Why it happens:** Existing `ShareSubmission::from_nonce_result` can serialize a submit-shaped message once given work and a nonce. [VERIFIED: crates/bitaxe-stratum/src/v1/mining.rs]  
**How to avoid:** Rename Phase 24 outputs as `SubmitIntent` or equivalent, and keep accepted/rejected counters owned by Phase 25 response handling. [VERIFIED: .planning/phases/24-bm1366-production-work-path/24-CONTEXT.md]  
**Warning signs:** A Phase 24 test asserts `record_accepted_share` or `record_rejected_share` from a BM1366 result alone. [VERIFIED: crates/bitaxe-stratum/src/v1/state.rs]

### Pitfall 4: Redaction-Safe Reasons Drift Into Sensitive Details

**What goes wrong:** Evidence includes raw targets, extranonces, share payloads, socket errors, raw BM1366 frames, or local network/device values. [VERIFIED: AGENTS.md]  
**Why it happens:** Debugging ASIC/Stratum failures often tempts frame or payload logging. [ASSUMED]  
**How to avoid:** Log stable category labels and keep any raw runtime artifacts local and ignored; run deterministic redaction checks before promotion. [VERIFIED: docs/parity/evidence/phase-23-redacted-operator-evidence-workflow/evidence-contract.md] [VERIFIED: docs/parity/evidence/phase-23-redacted-operator-evidence-workflow/redaction-review.md]  
**Warning signs:** Evidence mentions raw frame bytes, pool identity, device target, or unredacted share fields. [VERIFIED: docs/parity/evidence/phase-23-redacted-operator-evidence-workflow/redaction-review.md]

### Pitfall 5: Version Rolling Gets Overclaimed

**What goes wrong:** Production work claims imply full version-rolling support even though current Rust work generation rejects nonzero version masks. [VERIFIED: crates/bitaxe-stratum/src/v1/mining.rs]  
**Why it happens:** Upstream reference has multi-midstate version-mask behavior, while current Rust intentionally supports only zero-mask work generation. [VERIFIED: reference/esp-miner/components/stratum/mining.c via rg] [VERIFIED: crates/bitaxe-stratum/src/v1/mining.rs]  
**How to avoid:** Plan explicit non-claim language or a narrow task if Phase 24 must handle nonzero masks. [VERIFIED: .planning/phases/24-bm1366-production-work-path/24-CONTEXT.md]  
**Warning signs:** Tests use nonzero `VersionMask` and expect production work to build without implementing rolled midstate behavior. [VERIFIED: crates/bitaxe-stratum/src/v1/mining.rs]

## Code Examples

Verified patterns from current sources:

### Existing BM1366 Result Parse Boundary

```rust
// Source: crates/bitaxe-asic/src/bm1366/result.rs
pub fn parse_bm1366_result_frame(
    bytes: &[u8],
    valid_jobs: &Bm1366ValidJobIds,
    address_interval: u8,
) -> Result<Bm1366ParsedResult, Bm1366ProtocolFault>
```

The production plan should call this parse boundary before correlation and should not pass raw frame bytes into Stratum or parity logic. [VERIFIED: crates/bitaxe-asic/src/bm1366/result.rs] [VERIFIED: standards/core/architecture.md]

### Existing Pool-Derived Work Builder

```rust
// Source: crates/bitaxe-stratum/src/v1/mining.rs
pub fn build(self, asic_job_id: Bm1366JobId) -> Result<MiningWork, StratumV1Error>
```

The production plan should extend or wrap `MiningWork` with session generation and dispatch/correlation state instead of rebuilding notify/extranonce hashing elsewhere. [VERIFIED: crates/bitaxe-stratum/src/v1/mining.rs]

### Proposed Correlation Gate

```rust
// Proposed Phase 24 pattern.
pub fn correlate_nonce_result(
    registry: &mut ProductionWorkRegistry,
    result: Bm1366NonceResult,
) -> CorrelationOutcome {
    let Some(active) = registry.maybe_active_work(result.job_id) else {
        return CorrelationOutcome::Blocked {
            reason: ProductionAsicBlocker::JobUncorrelated,
        };
    };

    if active.is_stale_for(registry.generation()) {
        return CorrelationOutcome::Blocked {
            reason: ProductionAsicBlocker::StaleWork,
        };
    }

    active.submit_intent(result)
}
```

The exact names may change, but the function should be pure, typed, unit-tested, and free of firmware or socket effects. [VERIFIED: standards/core/architecture.md] [VERIFIED: standards/core/testing.md] [VERIFIED: .planning/phases/24-bm1366-production-work-path/24-CONTEXT.md]

## State of the Art

| Old / Current Approach | Phase 24 Approach | When Changed | Impact |
|------------------------|-------------------|--------------|--------|
| Diagnostic `SendDiagnosticWork` command | Production-specific work dispatch command or typed action | Phase 24 | Prevents diagnostic work evidence from proving production dispatch. [VERIFIED: crates/bitaxe-asic/src/bm1366/command.rs] |
| Valid-job bitset plus active map | Session-generation active-work registry | Phase 24 | Makes clean-jobs and reconnect invalidation explicit across queued and dispatched work. [VERIFIED: crates/bitaxe-stratum/src/v1/queue.rs] |
| Optional nonce result in guarded loop | Correlation outcome with submit intent or blocker | Phase 24 | Prevents uncorrelated nonce observations from becoming share claims. [VERIFIED: crates/bitaxe-stratum/src/v1/mining_loop.rs] |
| Phase 23 share-outcome non-claim slot | Phase 24 can add implementation/test proof but still leaves live response classification pending | Phase 24 | Keeps Phase 25 ownership clear for accepted/rejected responses. [VERIFIED: docs/parity/evidence/phase-23-redacted-operator-evidence-workflow/evidence-contract.md] |

**Deprecated/outdated for Phase 24:**

- Treating diagnostic work-result mode as sufficient production evidence is outdated for v1.1 trusted production mining. [VERIFIED: docs/parity/checklist.md] [VERIFIED: .planning/REQUIREMENTS.md]
- Treating snapshot/golden/unit tests as hardware parity proof is insufficient for safety-critical hardware verification. [VERIFIED: AGENTS.md] [VERIFIED: docs/parity/checklist.md]
- Treating live accepted/rejected shares as Phase 24 scope contradicts the Phase 24 context and roadmap. [VERIFIED: .planning/phases/24-bm1366-production-work-path/24-CONTEXT.md] [VERIFIED: .planning/ROADMAP.md]

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Recommended production blocker reason names are suitable as final stable strings. | Architecture Patterns | Planner may need to rename them to align with existing safety/parity naming before implementation. |
| A2 | Debugging pressure is the likely reason sensitive raw data might leak into evidence. | Common Pitfalls | Low implementation risk; mitigation remains the same redaction-safe status contract. |

## Open Questions

1. **RESOLVED: Phase 24 will not implement nonzero version-mask / multi-midstate work generation.** [VERIFIED: crates/bitaxe-stratum/src/v1/mining.rs]
   - Decision: Preserve the current Rust behavior that rejects nonzero version masks during work generation, and record nonzero version-mask support as an exact Phase 24 non-claim rather than adding version-rolling scope. [VERIFIED: crates/bitaxe-stratum/src/v1/mining.rs] [VERIFIED: .planning/ROADMAP.md]
   - Plan reflection: Plan 24-02 keeps production work derived from existing `MiningWork` only, so unsupported nonzero masks fail before registry dispatch; Plan 24-04 records the non-claim in Phase 24 evidence and checklist notes. [VERIFIED: .planning/phases/24-bm1366-production-work-path/24-02-PLAN.md] [VERIFIED: .planning/phases/24-bm1366-production-work-path/24-04-PLAN.md]

2. **RESOLVED: Phase 24 will keep mandatory proof at code/test/workflow level and will not include a conditional hardware promotion path.** [VERIFIED: AGENTS.md]
   - Decision: Hardware-capable verification remains detector-gated by `just detect-ultra205`, but Phase 24 plans do not promote checklist rows beyond `implemented | unit,workflow`. Verified hardware promotion remains pending until a later detector-gated execution plan names the evidence directory, selected port, board-info capture, package identity, exact commands, redaction workflow, observed behavior, and conclusion. [VERIFIED: AGENTS.md] [VERIFIED: docs/parity/checklist.md]
   - Plan reflection: Plan 24-04 writes exact non-claims and conservative checklist rows only; it does not contain an optional hardware promotion branch. [VERIFIED: .planning/phases/24-bm1366-production-work-path/24-04-PLAN.md]

3. **RESOLVED: Production registry ownership is split by domain boundary.** [VERIFIED: crates/bitaxe-asic/src/bm1366/result.rs] [VERIFIED: crates/bitaxe-stratum/src/v1/queue.rs]
   - Decision: BM1366 production command, result, payload, status, and blocker primitives belong in `crates/bitaxe-asic`; pool-session generation, active-work registry, invalidation, result correlation, and submit-intent preparation belong in `crates/bitaxe-stratum`. [VERIFIED: crates/bitaxe-asic/src/bm1366/work.rs] [VERIFIED: crates/bitaxe-stratum/src/v1/mining.rs] [VERIFIED: standards/core/architecture.md]
   - Plan reflection: Plan 24-01 creates ASIC primitives in `bitaxe-asic`; Plans 24-02 and 24-03 create and consume the production registry in `bitaxe-stratum`; firmware remains a thin interpreter/status publisher. [VERIFIED: .planning/phases/24-bm1366-production-work-path/24-01-PLAN.md] [VERIFIED: .planning/phases/24-bm1366-production-work-path/24-02-PLAN.md] [VERIFIED: .planning/phases/24-bm1366-production-work-path/24-03-PLAN.md]

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|-------------|-----------|---------|----------|
| `just` | Repo command surface | yes | `just 1.48.0` | Use direct Bazel commands only for diagnosis. [VERIFIED: environment probe] |
| Bazel | Build and tests | yes | `bazel 9.1.1` | None for canonical verification. [VERIFIED: environment probe] [VERIFIED: .bazelversion] |
| Cargo / Rust | Host Rust crate checks and firmware wrapper internals | yes | `cargo 1.88.0-nightly`, `rustc 1.88.0-nightly` | Bazel remains canonical for planned verification. [VERIFIED: environment probe] |
| `espflash` | Optional detector-gated hardware evidence | yes | `espflash 4.0.1` | Skip hardware evidence and record pending if detector gate cannot run. [VERIFIED: environment probe] [VERIFIED: AGENTS.md] |
| Node | GSD tooling and some scripts | yes | `v24.13.0` | None needed for research; scripts already expect Node in several sh_tests. [VERIFIED: environment probe] [VERIFIED: scripts/BUILD.bazel] |
| Python 3 | ESP-IDF helper ecosystem and scripts | yes | `Python 3.14.4` | Use managed `.embuild` ESP-IDF tools when generated. [VERIFIED: environment probe] [VERIFIED: AGENTS.md] |

**Missing dependencies with no fallback:** None found during the environment audit. [VERIFIED: environment probe]

**Missing dependencies with fallback:** Hardware availability itself was not probed during research. Phase 24 plans do not include hardware promotion; any later hardware evidence plan must run `just detect-ultra205` first and follow AGENTS.md evidence/redaction requirements. [VERIFIED: AGENTS.md]

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Bazel `rust_test` and `sh_test`; Rust tests are inline module tests or dedicated Rust test modules. [VERIFIED: crates/bitaxe-asic/BUILD.bazel] [VERIFIED: crates/bitaxe-stratum/BUILD.bazel] [VERIFIED: scripts/BUILD.bazel] |
| Config file | `MODULE.bazel`, per-crate `BUILD.bazel`, and `Justfile`. [VERIFIED: MODULE.bazel] [VERIFIED: Justfile] |
| Quick run command | `bazel test //crates/bitaxe-asic:tests //crates/bitaxe-stratum:tests //crates/bitaxe-safety:tests //tools/parity:tests` [VERIFIED: crates/bitaxe-asic/BUILD.bazel] [VERIFIED: crates/bitaxe-stratum/BUILD.bazel] [VERIFIED: crates/bitaxe-safety/BUILD.bazel] [VERIFIED: tools/parity/BUILD.bazel] |
| Full suite command | `just test` [VERIFIED: Justfile] |

### Phase Requirements -> Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|--------------|
| ASIC-09 | Diagnostic BM1366 modes cannot be used as production initialization/work/result modes. | unit | `bazel test //crates/bitaxe-asic:tests //crates/bitaxe-stratum:tests` | Partial: diagnostic types exist; production module/tests are Wave 0 gap. [VERIFIED: crates/bitaxe-asic/src/bm1366/adapter_gate.rs] |
| ASIC-10 | Pool-derived work tracks job, extranonce, difficulty, session generation, and invalidates on clean-jobs/reconnect. | unit/fixture | `bazel test //crates/bitaxe-stratum:tests` | Partial: `MiningWork` and queue exist; generation-aware registry is Wave 0 gap. [VERIFIED: crates/bitaxe-stratum/src/v1/mining.rs] [VERIFIED: crates/bitaxe-stratum/src/v1/queue.rs] |
| ASIC-11 | Parsed BM1366 nonce/result maps to active non-stale work before submit intent exists. | unit | `bazel test //crates/bitaxe-asic:tests //crates/bitaxe-stratum:tests` | Partial: parser and guarded loop exist; explicit production correlation outcomes are Wave 0 gap. [VERIFIED: crates/bitaxe-asic/src/bm1366/result.rs] [VERIFIED: crates/bitaxe-stratum/src/v1/mining_loop.rs] |
| ASIC-12 | Init, UART, reset, timeout, malformed result, stale/uncorrelated job failures fail closed and render redaction-safe reasons. | unit/workflow | `bazel test //crates/bitaxe-asic:tests //crates/bitaxe-stratum:tests //tools/parity:tests` | Partial: diagnostic fail-closed reasons exist; production reason taxonomy and redaction tests are Wave 0 gap. [VERIFIED: firmware/bitaxe/src/asic_adapter/status.rs] [VERIFIED: docs/parity/evidence/phase-23-redacted-operator-evidence-workflow/redaction-review.md] |

### Sampling Rate

- **Per task commit:** Run crate-scoped Bazel tests for changed Rust crates. [VERIFIED: standards/core/verification.md]
- **Per wave merge:** Run `bazel test //crates/bitaxe-asic:tests //crates/bitaxe-stratum:tests //crates/bitaxe-safety:tests //tools/parity:tests` and any changed script `sh_test`s. [VERIFIED: crates/bitaxe-asic/BUILD.bazel] [VERIFIED: scripts/BUILD.bazel]
- **Phase gate:** Run `just test` and `just parity`; run `bazel build //firmware/bitaxe:firmware` or `just package` if firmware code changed. [VERIFIED: Justfile] [VERIFIED: firmware/bitaxe/BUILD.bazel]
- **Optional hardware gate:** Run `just detect-ultra205` first, then only repo-owned hardware commands named in the plan; record exact non-claims if detection fails or is skipped. [VERIFIED: AGENTS.md]

### Wave 0 Gaps

- [ ] `crates/bitaxe-asic/src/bm1366/production.rs` or equivalent — production-mode BM1366 command/status/result types for ASIC-09 and ASIC-12. [VERIFIED: crates/bitaxe-asic/src/bm1366.rs]
- [ ] `crates/bitaxe-stratum/src/v1/production_work.rs` or equivalent — session-generation registry, invalidation, duplicate detection, and correlation outcomes for ASIC-10 and ASIC-11. [VERIFIED: crates/bitaxe-stratum/src/v1.rs]
- [ ] Redaction tests for production blocker/status rendering — prove forbidden pool/device/share/BM1366 raw categories are absent. [VERIFIED: docs/parity/evidence/phase-23-redacted-operator-evidence-workflow/redaction-review.md]
- [ ] Parity/checklist update tests or fixture changes — ensure ASIC-09 through ASIC-12 advance only to implemented/verified levels proven by Phase 24 artifacts. [VERIFIED: tools/parity/src/main.rs] [VERIFIED: docs/parity/checklist.md]

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|------------------|
| V2 Authentication | no for Phase 24 core; Phase 25 owns live Stratum auth runtime. | Keep credentials runtime-only and redacted in evidence. [VERIFIED: .planning/phases/24-bm1366-production-work-path/24-CONTEXT.md] [VERIFIED: AGENTS.md] |
| V3 Session Management | yes for pool-session generation, not web sessions. | Use typed session generation and invalidate on clean-jobs/reconnect/session replacement. [VERIFIED: crates/bitaxe-stratum/src/v1/queue.rs] [VERIFIED: .planning/phases/24-bm1366-production-work-path/24-CONTEXT.md] |
| V4 Access Control | yes for mining enablement gates. | Require Phase 22 readiness and hardware evidence ack before production dispatch. [VERIFIED: crates/bitaxe-stratum/src/v1/mining_loop.rs] [VERIFIED: docs/parity/evidence/phase-22-claim-ladder-and-safety-preconditions/blocker-reasons.md] |
| V5 Input Validation | yes. | Parse Stratum messages and BM1366 frames into domain types before core decisions. [VERIFIED: crates/bitaxe-stratum/src/v1/messages.rs] [VERIFIED: crates/bitaxe-asic/src/bm1366/result.rs] |
| V6 Cryptography | limited. | Use existing `sha2` work hashing helpers; CRC remains BM1366 protocol integrity, not security cryptography. [VERIFIED: crates/bitaxe-stratum/src/v1/mining.rs] [VERIFIED: crates/bitaxe-asic/src/bm1366/crc.rs] |
| V7 Error Handling and Logging | yes. | Emit stable redaction-safe blocker/status labels and never raw frames or secrets in committed evidence. [VERIFIED: docs/parity/evidence/phase-22-claim-ladder-and-safety-preconditions/blocker-reasons.md] [VERIFIED: docs/parity/evidence/phase-23-redacted-operator-evidence-workflow/evidence-contract.md] |

### Known Threat Patterns for BM1366 Production Work

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Stale nonce replay after clean-jobs or reconnect | Tampering / Replay | Session-generation registry invalidates queued and active work before submit intent. [VERIFIED: reference/esp-miner/main/system.c via rg] [VERIFIED: crates/bitaxe-stratum/src/v1/queue.rs] |
| Uncorrelated ASIC result creates a share claim | Spoofing / Tampering | Correlate parsed nonce to active pool-derived work before submit intent; otherwise fail closed. [VERIFIED: .planning/phases/24-bm1366-production-work-path/24-CONTEXT.md] |
| Malformed BM1366 frame causes unsafe state | Tampering / Denial of Service | Parse with length, preamble, CRC, job-id, core-id, and address-interval checks; return typed faults. [VERIFIED: crates/bitaxe-asic/src/bm1366/result.rs] |
| Raw frame or share payload leaks in evidence | Information Disclosure | Use Phase 23 redacted evidence contract and stable status labels only. [VERIFIED: docs/parity/evidence/phase-23-redacted-operator-evidence-workflow/evidence-contract.md] |
| Production dispatch runs without safety prerequisites | Elevation of Privilege / Safety Hazard | Require Phase 22 readiness, hardware evidence ack, ASIC initialized gate, and fail-closed status. [VERIFIED: crates/bitaxe-stratum/src/v1/mining_loop.rs] |

## Sources

### Primary (HIGH confidence)

- `.planning/phases/24-bm1366-production-work-path/24-CONTEXT.md` — locked decisions, scope boundaries, current implementation surfaces, upstream breadcrumbs, and deferred ideas. [VERIFIED: ReadFile]
- `.planning/REQUIREMENTS.md` — ASIC-09 through ASIC-12 requirement text and v1.1 traceability. [VERIFIED: ReadFile]
- `.planning/ROADMAP.md` — Phase 24 goal and success criteria, Phase 25 and Phase 26 ownership boundaries. [VERIFIED: ReadFile]
- `AGENTS.md` and `AGENTS.bright-builds.md` — repo-local hardware, redaction, workflow, and standards-routing rules. [VERIFIED: ReadFile]
- `standards/core/architecture.md`, `standards/core/code-shape.md`, `standards/core/testing.md`, `standards/core/verification.md`, `standards/languages/rust.md` — functional-core, typed invariants, test, verification, and module-layout rules. [VERIFIED: ReadFile]
- `crates/bitaxe-asic/src/bm1366/*.rs` — existing BM1366 diagnostic work, command, init, observation, and result parsing surfaces. [VERIFIED: ReadFile]
- `crates/bitaxe-stratum/src/v1/*.rs` — existing Stratum v1 message, work, queue, mining-loop, fake-pool, and runtime-state surfaces. [VERIFIED: ReadFile]
- `firmware/bitaxe/src/asic_adapter.rs` and `firmware/bitaxe/src/asic_adapter/status.rs` — firmware ASIC adapter interpretation and redaction-safe status logging. [VERIFIED: ReadFile]
- `docs/parity/checklist.md` and Phase 22/23 evidence docs — current evidence levels and exact non-claim rules. [VERIFIED: ReadFile]

### Secondary (MEDIUM confidence)

- `reference/esp-miner/components/asic/bm1366.c`, `components/asic/asic_common.c`, `components/stratum/mining.c`, `components/stratum/stratum_api.c`, `main/system.c`, `main/work_queue.c`, and `main/tasks/protocol_coordinator.c` — upstream behavioral breadcrumbs found by search; use as behavior evidence only, not source-expression input. [VERIFIED: rg]

### Tertiary (LOW confidence)

- None. No unverified web sources were used because the phase is constrained to repo-local and upstream-reference behavior. [VERIFIED: research scope]

## Metadata

**Confidence breakdown:**

- Standard stack: HIGH — based on checked-in workspace manifests, Bazel files, Justfile, and local tool probe. [VERIFIED: Cargo.toml] [VERIFIED: MODULE.bazel] [VERIFIED: Justfile] [VERIFIED: environment probe]
- Architecture: HIGH — based on locked Phase 24 decisions and existing crate boundaries. [VERIFIED: .planning/phases/24-bm1366-production-work-path/24-CONTEXT.md] [VERIFIED: standards/core/architecture.md]
- Pitfalls: HIGH for scope/redaction/correlation pitfalls; MEDIUM for proposed blocker reason names because final naming is discretionary. [VERIFIED: docs/parity/checklist.md] [VERIFIED: docs/parity/evidence/phase-23-redacted-operator-evidence-workflow/evidence-contract.md] [ASSUMED]

**Research date:** 2026-07-05  
**Valid until:** 2026-08-04, or earlier if Phase 25 changes the Stratum runtime contract before Phase 24 planning completes. [ASSUMED]
