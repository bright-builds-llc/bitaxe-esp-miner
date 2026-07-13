# Phase 26: Telemetry And Parity Closure - Research

**Researched:** 2026-07-05
**Domain:** Rust firmware telemetry projection, AxeOS API/WebSocket parity, evidence-governed checklist closure
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

## Implementation Decisions

### Runtime Event Source And Counter Invariants

- **D-01:** Use a shared typed v1.1 runtime-event projection as the source of truth for API, WebSocket, statistics, scoreboard, and parity-visible mining state. Endpoint handlers should consume derived views rather than each inventing local counter or activity rules.
- **D-02:** Accepted and rejected share counters may advance only from a live ASIC-derived `SubmitIntent` plus matching parsed pool response for the current pool-session generation. Fake-pool-only responses, placeholder data, stale generations, missing submit intent, blocked prerequisites, stopped sockets, or implementation-only tests must not advance production-visible share counters.
- **D-03:** Hashrate inputs, mining activity, pool lifecycle, fallback status, blocked reasons, and post-stop state should be folded from the same typed runtime events or explicitly safe snapshots. Existing direct mutation helpers in `MiningRuntimeState` must be wrapped, constrained, or tested so Phase 26 call paths cannot bypass the event/projection gate.
- **D-04:** The projection should carry enough generation, session, or sequence metadata to prove stale events are ignored and safe-stop events reset active mining, work-submission, active-work, and lifecycle state before API or WebSocket serialization.

### API Projection Shape

- **D-05:** Keep public responses AxeOS-compatible. Preserve established wire fields and upstream-compatible shapes in `crates/bitaxe-api`, while deriving their values from the shared Phase 26 projection.
- **D-06:** `/api/system/info` should continue to use `ApiSnapshot` and `SystemInfoWire`, but its mining fields must reflect the Phase 26 projection rather than ad hoc controlled-evidence state replacement.
- **D-07:** `/api/system/statistics` should stop being only an empty placeholder when runtime-event evidence can support a bounded sample. Any emitted sample must name or encode that it came from the runtime projection; no historical series may be fabricated.
- **D-08:** `/api/system/scoreboard` should remain an upstream-shaped array. Populate it only from runtime events tied to parsed pool responses or active share outcomes. If no such events exist, return the compatible empty array and preserve the non-claim.
- **D-09:** Richer evidence semantics that do not fit upstream API fields should stay in internal projection types, tests, or evidence artifacts rather than adding non-upstream public API keys without a deliberate compatibility decision.

### WebSocket Session Correlation And Redaction

- **D-10:** `/api/ws/live` must serialize the same redacted projection used by the HTTP API, preserving the existing full-on-connect and 500 ms diff cadence semantics.
- **D-11:** `/api/ws` should remain compatible with the raw retained-log stream route. If Phase 26 needs mining lifecycle markers there, emit redaction-safe log markers from the shared projection instead of introducing raw pool, share, target, extranonce, device, Wi-Fi, NVS, or ASIC-frame values.
- **D-12:** Redaction must happen before telemetry fan-out. The projection and logs must never expose pool URLs, ports, workers, owner addresses, passwords, targets, extranonces, share payloads, socket errors, device URLs, IP addresses, MAC addresses, Wi-Fi values, tokens, NVS secrets, or raw BM1366 frames in committed evidence.
- **D-13:** WebSocket tests and evidence should prove no stale active-mining frame is emitted after safe stop, including connect-time full updates and cadence diffs after the safe-stop event has updated the projection.

### Parity Checklist And Evidence Promotion

- **D-14:** Promote checklist rows only to the exact level proven by Phase 26 artifacts. Code, unit tests, fake-pool fixtures, workflow evidence, and detector-gated hardware evidence each support different claim levels and must not be collapsed into a single "verified" claim.
- **D-15:** Add or update the Phase 26 evidence artifacts to map API-11, API-12, API-13, and EVD-08 to exact source artifacts. A checklist-first exact delta is enough for a narrow closure; add a machine-readable promotion manifest only if the planner finds reusable v1.1 closure validation is needed.
- **D-16:** Preserve explicit non-claims for accepted/rejected live share proof when Phase 25 evidence remains blocked, plus full active safety closure, OTA/recovery, non-205 boards, other ASIC families, Stratum v2, display/input, BAP, and unbounded stress.
- **D-17:** `tools/parity` should reject overbroad verified telemetry/statistics/scoreboard claims with pending/blocker language, missing Phase 26 artifacts, or absent redaction review.

### Verification Gate

- **D-18:** Unit tests must prove one concern at a time: event folding, counter gating, stale-generation rejection, safe-stop reset, API projection, statistics projection, scoreboard projection, WebSocket full/diff projection, and redaction-safe rendering.
- **D-19:** Repo-native verification should include targeted Rust tests for changed crates, firmware compile or relevant Bazel targets when firmware files change, script/helper tests when touched, `just parity`, `just verify-reference`, and lifecycle validation.
- **D-20:** Hardware verification is allowed only through the Ultra 205 detector gate and repo-owned commands. If detector, package identity, safe prerequisites, local credentials, live socket, share outcome, API, or WebSocket evidence is blocked, record the blocker and keep the affected claim below verified.

### Claude's Discretion

Claude may choose exact module names, event enum names, projection struct names, fixture shapes, route helper names, evidence filenames, and whether the projection lives primarily in `crates/bitaxe-stratum`, `crates/bitaxe-api`, or a narrow bridge between them. Those choices must preserve functional core / imperative shell structure, AxeOS public compatibility, runtime-only secrets, redaction-safe artifacts, Ultra 205 detector gating, and exact parity semantics.

### Deferred Ideas (OUT OF SCOPE)

## Deferred Ideas

- Accepted/rejected live share proof remains below verified when Phase 25 artifacts record `blocked_safe_prerequisite` instead of a detector-gated pool response.
- Full active voltage, fan, thermal fault-stimulus, load, recovery, self-test, non-205 board, and non-BM1366 ASIC hardware closure remain future work.
- OTAWWW, rollback, failed-update, large erase, interrupted-update, and destructive recovery evidence remain future work.
- Runtime display/input, LVGL-like UI flow, BAP, Stratum v2, and unbounded stress mining remain future work.
- Broad AxeOS UI changes remain out of v1.1.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
| --- | --- | --- |
| API-11 | `/api/system/info`, `/api/system/statistics`, and `/api/system/scoreboard` expose mining state, counters, hashrate inputs, share outcomes, and post-stop state derived from the same v1.1 runtime events. | Use a single typed runtime projection feeding `ApiSnapshot`, bounded statistics samples, and upstream-shaped scoreboard entries. [VERIFIED: `.planning/REQUIREMENTS.md`, `crates/bitaxe-api/src/snapshot.rs`, `crates/bitaxe-api/src/statistics.rs`, `crates/bitaxe-api/src/scoreboard.rs`] |
| API-12 | `/api/ws` and `/api/ws/live` stream redacted, session-correlated mining telemetry during bounded production mining without stale active-mining state after stop. | Reuse `WebSocketState` and `LiveTelemetryPlanner`, but serialize the Phase 26 redacted projection and add tests for connect-time and cadence frames after safe stop. [VERIFIED: `.planning/REQUIREMENTS.md`, `crates/bitaxe-api/src/telemetry.rs`, `crates/bitaxe-api/src/websocket_state.rs`, `firmware/bitaxe/src/http_api.rs`] |
| API-13 | Statistics, scoreboard, and share counters do not advance without corresponding runtime events and parsed pool responses. | Gate counter and scoreboard updates through `SubmitIntent`, current `PoolSessionGeneration`, request id, and parsed `StratumResponse`; keep empty statistics or scoreboard when evidence is absent. [VERIFIED: `.planning/REQUIREMENTS.md`, `crates/bitaxe-stratum/src/v1/production_work.rs`, `crates/bitaxe-stratum/src/v1/submit_response.rs`, `crates/bitaxe-stratum/src/v1/state.rs`] |
| EVD-08 | Committed parity checklist updates promote only exact claims proven by v1.1 artifacts and preserve explicit non-claims for deferred surfaces. | Update `docs/parity/checklist.md` and `tools/parity` only with exact Phase 26 evidence tiers and redaction-reviewed artifacts; preserve Phase 25 share-outcome blockers. [VERIFIED: `.planning/REQUIREMENTS.md`, `docs/parity/checklist.md`, `tools/parity/src/main.rs`, `docs/parity/evidence/phase-25-live-stratum-runtime-and-safe-stop/summary.md`] |
</phase_requirements>

## Summary

Phase 26 should be planned as an internal projection closure, not as a new public API design. The repo already has AxeOS-compatible DTOs for `/api/system/info`, statistics, scoreboard, and live WebSocket envelopes; the missing piece is a single typed v1.1 runtime-event projection that owns mining activity, lifecycle, hashrate inputs, share counters, scoreboard material, safe-stop reset semantics, and redaction-safe evidence semantics before any HTTP or WebSocket adapter serializes state. [VERIFIED: `crates/bitaxe-api/src/wire.rs`, `crates/bitaxe-api/src/statistics.rs`, `crates/bitaxe-api/src/scoreboard.rs`, `crates/bitaxe-api/src/telemetry.rs`, `firmware/bitaxe/src/http_api.rs`]

The hardest invariant is counter legitimacy. `MiningRuntimeState` currently exposes direct `record_accepted_share` and `record_rejected_share` helpers, while Phase 25 classification already proves accepted/rejected only from a submit intent and matching parsed response; Phase 26 should wrap or bypass direct counter mutation on production-visible paths so fake-pool-only, stale, blocked, missing-intent, stopped, or placeholder observations cannot move counters, statistics, or scoreboard. [VERIFIED: `crates/bitaxe-stratum/src/v1/state.rs`, `crates/bitaxe-stratum/src/v1/submit_response.rs`, `.planning/phases/26-telemetry-and-parity-closure/26-CONTEXT.md`]

Evidence closure must remain conservative. Phase 25 committed evidence records `share_outcome: blocked_safe_prerequisite`, not detector-gated accepted/rejected share proof, so Phase 26 can close API/WebSocket/statistics/scoreboard projection mechanics but must preserve accepted/rejected live-share non-claims unless new detector-gated artifacts exist and pass redaction review. [VERIFIED: `docs/parity/evidence/phase-25-live-stratum-runtime-and-safe-stop/summary.md`, `docs/parity/evidence/phase-25-live-stratum-runtime-and-safe-stop/share-outcome.md`, `.planning/STATE.md`]

**Primary recommendation:** Implement a small pure Phase 26 projection module that folds typed runtime events into redacted API/statistics/scoreboard/WebSocket views, then wire firmware handlers to consume that projection and update parity artifacts only to the exact evidence tier proven. [VERIFIED: `.planning/phases/26-telemetry-and-parity-closure/26-CONTEXT.md`, `standards/core/architecture.md`, `standards/languages/rust.md`]

## Project Constraints (from .cursor/rules/)

No `.cursor/rules/` files were found in the workspace, and no repo-local `.claude/skills/` or `.agents/skills/` directories were found. [VERIFIED: glob search]

Material project constraints came from `AGENTS.md`, `AGENTS.bright-builds.md`, `standards/core/architecture.md`, `standards/core/code-shape.md`, `standards/core/testing.md`, `standards/core/verification.md`, `standards/languages/rust.md`, `standards-overrides.md`, and Phase 26 `CONTEXT.md`. [VERIFIED: repository files]

- Use GSD workflow artifacts and do not directly mutate `STATE.md` or `ROADMAP.md` for this phase plan. [VERIFIED: `AGENTS.md`, `.planning/STATE.md`]
- Keep upstream `reference/esp-miner` read-only and use it for breadcrumbs, behavior research, and fixtures without copying GPL-covered source expression into MIT-only Rust files. [VERIFIED: `PROVENANCE.md`, `docs/adr/0013-mit-first-with-gpl-guardrails.md`]
- Keep functional-core decisions in Rust crates/tools and ESP-IDF HTTP/WebSocket/socket/hardware work in thin firmware adapters. [VERIFIED: `standards/core/architecture.md`, `standards/languages/rust.md`]
- Unit-test pure business logic with one concern per test and Arrange/Act/Assert structure. [VERIFIED: `standards/core/testing.md`]
- Hardware use must start with `just detect-ultra205`, must require exactly one likely ESP USB serial port plus successful `espflash board-info --chip esp32s3 --port <port> --non-interactive`, and must stop on absent or ambiguous hardware. [VERIFIED: `AGENTS.md`]
- Committed evidence must not include pool endpoints, ports, workers, owner addresses, passwords, targets, extranonces, share payloads, socket errors, device URLs, IP addresses, MAC addresses, Wi-Fi values, tokens, NVS secrets, or raw BM1366 frames. [VERIFIED: `AGENTS.md`, `docs/parity/evidence/phase-23-redacted-operator-evidence-workflow/evidence-contract.md`]
- Frontmatter-parsed Markdown must not use standalone body `---` separators after YAML frontmatter. [VERIFIED: `AGENTS.md`]
- Before committing, run relevant repo-native verification; for Phase 26 this means targeted Rust/Bazel tests, `just parity`, `just verify-reference`, and lifecycle validation when touched paths require it. [VERIFIED: `AGENTS.bright-builds.md`, `standards/core/verification.md`, `.planning/phases/26-telemetry-and-parity-closure/26-CONTEXT.md`]

## Standard Stack

### Core

| Library / Target | Version | Purpose | Why Standard |
| --- | --- | --- | --- |
| Rust workspace crates | edition 2021 | Own pure API, Stratum, ASIC, safety, parity, and firmware logic. | The workspace is already organized around `crates/bitaxe-api`, `crates/bitaxe-stratum`, firmware adapters, and parity tooling. [VERIFIED: `Cargo.toml`] |
| `crates/bitaxe-stratum` | internal workspace crate | Own runtime events, pool session generation, submit intents, submit response classification, and mining runtime state. | Phase 24/25 work already placed production work, live runtime, and share classification here. [VERIFIED: `crates/bitaxe-stratum/src/v1/production_work.rs`, `crates/bitaxe-stratum/src/v1/live_runtime.rs`, `crates/bitaxe-stratum/src/v1/submit_response.rs`] |
| `crates/bitaxe-api` | internal workspace crate | Own AxeOS wire DTOs, `ApiSnapshot`, statistics, scoreboard, live telemetry envelopes, and WebSocket state planners. | Phase 05 established pure API DTOs and adapter input boundaries here. [VERIFIED: `crates/bitaxe-api/src/lib.rs`, `.planning/phases/05-axeos-api-logs-and-telemetry/05-CONTEXT.md`] |
| `firmware/bitaxe` | internal workspace package | Own ESP-IDF HTTP/WebSocket shells, runtime snapshot collection, safe-stop publication, and socket effects. | Existing firmware routes already use `collect_api_snapshot`, WebSocket session bridge, and live Stratum safe-stop state publication. [VERIFIED: `firmware/bitaxe/src/http_api.rs`, `firmware/bitaxe/src/runtime_snapshot.rs`, `firmware/bitaxe/src/live_stratum_runtime.rs`, `firmware/bitaxe/src/websocket_api.rs`] |
| `tools/parity` | internal workspace tool | Own checklist validation, exact verified-row guardrails, operator evidence validation, and mining allow-manifest checks. | `just parity` routes through this tool with `--fail-on-invalid-verified`. [VERIFIED: `justfile`, `tools/parity/src/main.rs`, `tools/parity/src/operator_evidence.rs`, `tools/parity/src/mining_allow.rs`] |

### Supporting

| Library / Tool | Version | Purpose | When to Use |
| --- | --- | --- | --- |
| `serde` / `serde_json` | `serde 1.0.228`, `serde_json 1.0.150` | Serialize DTOs, fixtures, evidence manifests, and WebSocket frames. | Use for all structured wire/evidence payloads instead of string-building JSON. [VERIFIED: `Cargo.toml`, `crates/bitaxe-api/src/wire.rs`, `tools/parity/src/main.rs`] |
| `thiserror` | `2.0.18` | Typed library errors. | Use when a new pure projection or validation module needs reusable error types. [VERIFIED: `Cargo.toml`, `crates/bitaxe-api/src/wire.rs`] |
| `anyhow` | `1.0.102` | CLI/firmware-shell error context. | Use in shell-style firmware/tool code where contextual errors matter more than exported error enums. [VERIFIED: `Cargo.toml`, `firmware/bitaxe/src/http_api.rs`, `tools/parity/src/main.rs`] |
| `camino` | `1.2.3` | UTF-8 paths for host tools. | Use if Phase 26 adds parity/evidence manifests or CLI checks in `tools/parity`. [VERIFIED: `Cargo.toml`, `tools/parity/src/main.rs`] |
| Bazel / Just | Bazel `9.1.1`, `just 1.48.0` installed locally | Canonical build/test graph and human command surface. | Use `bazel test` targets for changed crates/scripts, plus `just parity` and `just verify-reference`. [VERIFIED: shell probe, `justfile`] |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
| --- | --- | --- |
| New public telemetry endpoint fields | Add non-upstream API keys to `/api/system/info` or `/api/ws/live` | Rejected unless separately decided; Phase 26 decisions require AxeOS-compatible public fields and richer semantics in internal projection/evidence types. [VERIFIED: `.planning/phases/26-telemetry-and-parity-closure/26-CONTEXT.md`] |
| Firmware-owned ad hoc counters | Mutate `MiningRuntimeState` directly from each handler or runtime branch | Rejected for production-visible paths because direct helper calls cannot prove submit-intent, generation, and parsed-response gating. [VERIFIED: `crates/bitaxe-stratum/src/v1/state.rs`, `.planning/phases/26-telemetry-and-parity-closure/26-CONTEXT.md`] |
| New protocol simulator | Build another fake pool or telemetry simulator | Rejected; Phase 25 already chose to extend the existing fake-pool/fixture harness for deterministic protocol behavior. [VERIFIED: `.planning/phases/25-live-stratum-runtime-and-safe-stop/25-CONTEXT.md`, `crates/bitaxe-stratum/src/v1/fake_pool.rs`] |
| Hand-written JSON strings | Build response bodies with string formatting | Rejected; existing API and parity surfaces use typed DTOs plus `serde_json`. [VERIFIED: `crates/bitaxe-api/src/wire.rs`, `crates/bitaxe-api/src/telemetry.rs`, `tools/parity/src/main.rs`] |

**Installation:**

No new dependencies should be installed for the primary Phase 26 implementation. Use the existing Rust workspace, Bazel targets, and Just commands. [VERIFIED: `Cargo.toml`, `justfile`]

**Version verification:** Workspace dependency versions were read from `Cargo.toml`, and local tool versions were probed with installed CLIs. [VERIFIED: `Cargo.toml`, shell probe]

## Architecture Patterns

### Recommended Project Structure

```text
crates/bitaxe-stratum/src/v1/
├── telemetry_projection.rs     # Typed v1.1 runtime events and fold logic
├── state.rs                    # Existing runtime state; constrain direct production-visible mutation
├── production_work.rs          # SubmitIntent and PoolSessionGeneration source
└── submit_response.rs          # Parsed response classification gate

crates/bitaxe-api/src/
├── snapshot.rs                 # ApiSnapshot adapter input consumes projection output
├── statistics.rs               # Existing wire shape plus projection-backed samples
├── scoreboard.rs               # Existing wire shape plus projection-backed entries
└── telemetry.rs                # Existing full/diff live WebSocket envelope

firmware/bitaxe/src/
├── runtime_snapshot.rs         # Thin bridge from projection to ApiSnapshot
├── http_api.rs                 # Route handlers consume projection-backed views
└── websocket_api.rs            # Session state only; no independent mining rules

docs/parity/evidence/phase-26-telemetry-and-parity-closure/
└── *.md                        # Redacted exact-claim artifacts and non-claims
```

This layout follows existing file ownership and Rust module conventions while keeping pure projection behavior outside ESP-IDF adapters. [VERIFIED: `Cargo.toml`, `crates/bitaxe-api/BUILD.bazel`, `crates/bitaxe-stratum/BUILD.bazel`, `firmware/bitaxe/src/runtime_snapshot.rs`, `standards/languages/rust.md`]

### Pattern 1: Typed Runtime Event Fold

**What:** Model Phase 26-owned events as a closed enum or small family of typed structs, then fold them into one projection state that can emit `MiningRuntimeState`, bounded `StatisticsSample`s, `ScoreboardEntry`s, WebSocket/log-safe labels, and evidence metadata. [VERIFIED: `.planning/phases/26-telemetry-and-parity-closure/26-CONTEXT.md`, `standards/core/architecture.md`]

**When to use:** Use this for every production-visible mining state transition: lifecycle, pool generation, hashrate input update, submit intent observation, parsed submit response, safe stop, and blocked prerequisite. [VERIFIED: `crates/bitaxe-stratum/src/v1/live_runtime.rs`, `crates/bitaxe-stratum/src/v1/production_work.rs`, `crates/bitaxe-stratum/src/v1/submit_response.rs`]

**Example:**

```rust
// Source: repo pattern from `crates/bitaxe-stratum/src/v1/submit_response.rs`
pub enum RuntimeTelemetryEvent {
    LifecycleChanged { sequence: u64, status: PoolLifecycleStatus },
    HashrateObserved { sequence: u64, inputs: HashrateInputs },
    SubmitClassified {
        sequence: u64,
        generation: PoolSessionGeneration,
        classification: SubmitClassification,
    },
    SafeStopped { sequence: u64, reason: &'static str },
}
```

### Pattern 2: Counter Gate Through Submit Classification

**What:** Derive accepted/rejected counter changes only after `classify_submit_response` or a narrow wrapper proves the current `SubmitIntent`, expected request id, matching pool session generation, and parsed `StratumResponse`. [VERIFIED: `crates/bitaxe-stratum/src/v1/submit_response.rs`, `crates/bitaxe-stratum/src/v1/production_work.rs`]

**When to use:** Use for production-visible `ShareCounters`, scoreboard entries, statistics samples that include share outcomes, and any parity-visible share outcome artifact. [VERIFIED: `.planning/phases/26-telemetry-and-parity-closure/26-CONTEXT.md`, `crates/bitaxe-api/src/statistics.rs`, `crates/bitaxe-api/src/scoreboard.rs`]

**Example:**

```rust
// Source: repo pattern from `classify_maybe_submit_response`
let Some(intent) = maybe_intent else {
    return SubmitClassification::Blocked {
        reason: "submit_intent_missing",
    };
};

let classification = classify_submit_response(intent, request_id, observation);
```

### Pattern 3: Projection-Backed Wire DTOs

**What:** Keep `SystemInfoWire`, `StatisticsWire`, and `ScoreboardEntryWire` as the public AxeOS shapes, but replace route-local placeholder sources with data produced by the Phase 26 projection. [VERIFIED: `crates/bitaxe-api/src/wire.rs`, `crates/bitaxe-api/src/statistics.rs`, `crates/bitaxe-api/src/scoreboard.rs`, `firmware/bitaxe/src/http_api.rs`]

**When to use:** Use in `/api/system/info`, `/api/system/statistics`, `/api/system/scoreboard`, `/api/ws/live`, and any tests that serialize public telemetry. [VERIFIED: `firmware/bitaxe/src/http_api.rs`, `crates/bitaxe-api/src/telemetry.rs`]

**Example:**

```rust
// Source: repo pattern from `crates/bitaxe-api/src/statistics.rs`
let sample = StatisticsSample::from_snapshot(&snapshot, timestamp_ms, response_time);
let response = statistics_response(timestamp_ms, maybe_columns, &[sample]);
```

### Pattern 4: Exact Claim Evidence Artifacts

**What:** Write Phase 26 evidence artifacts that map API-11, API-12, API-13, and EVD-08 to exact tests, route observations, WebSocket observations, redaction review, and retained non-claims. [VERIFIED: `.planning/phases/26-telemetry-and-parity-closure/26-CONTEXT.md`, `docs/parity/evidence/phase-23-redacted-operator-evidence-workflow/evidence-contract.md`]

**When to use:** Use when updating `docs/parity/checklist.md` or `tools/parity` guardrails. [VERIFIED: `docs/parity/checklist.md`, `tools/parity/src/main.rs`]

### Anti-Patterns to Avoid

- **Endpoint-local counter logic:** Handler-specific counter, activity, or post-stop rules would violate the shared projection source-of-truth decision. [VERIFIED: `.planning/phases/26-telemetry-and-parity-closure/26-CONTEXT.md`]
- **Fabricated statistics history:** `StatisticsWire` can represent historical rows, but Phase 26 must not create rows unless projection-backed samples exist. [VERIFIED: `.planning/phases/26-telemetry-and-parity-closure/26-CONTEXT.md`, `crates/bitaxe-api/src/statistics.rs`]
- **Scoreboard from raw or unproven share context:** Upstream scoreboard fields include job id, extranonce2, ntime, nonce, and version bits, all of which are sensitive or claim-bearing unless tied to exact accepted share outcomes; keep the array empty when evidence is absent. [VERIFIED: `reference/esp-miner/main/tasks/scoreboard.h`, `crates/bitaxe-api/src/scoreboard.rs`, `.planning/phases/26-telemetry-and-parity-closure/26-CONTEXT.md`]
- **WebSocket-specific state invention:** `/api/ws/live` should serialize the same projection as HTTP instead of deriving independent mining state. [VERIFIED: `.planning/phases/26-telemetry-and-parity-closure/26-CONTEXT.md`, `firmware/bitaxe/src/http_api.rs`]
- **Checklist overpromotion:** `verified` rows with pending evidence or blocker language are already invalid in `tools/parity`; Phase 26 should extend that pattern for telemetry/statistics/scoreboard rows. [VERIFIED: `tools/parity/src/main.rs`]

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
| --- | --- | --- | --- |
| JSON response serialization | Manual JSON strings or string concatenation | Existing Serde DTOs and `serde_json` | Existing API DTOs already preserve AxeOS field names and mixed numeric/boolean encodings. [VERIFIED: `crates/bitaxe-api/src/wire.rs`, `crates/bitaxe-api/src/statistics.rs`] |
| WebSocket diff/cadence logic | New route-specific diff engine | `LiveTelemetryPlanner` and `live_telemetry_diff` | Existing planner sends full connect frames, diff-only cadence frames, and no frame for unchanged state. [VERIFIED: `crates/bitaxe-api/src/telemetry.rs`] |
| WebSocket session tracking | New session registry in firmware route code | `WebSocketState` through `firmware/bitaxe/src/websocket_api.rs` | Existing state enforces route-local sessions and hibernates inactive routes. [VERIFIED: `crates/bitaxe-api/src/websocket_state.rs`, `firmware/bitaxe/src/websocket_api.rs`] |
| Submit response classification | Boolean accepted/rejected parsing in route code | `classify_submit_response` / `classify_maybe_submit_response` | Existing classifier handles accepted, rejected, timeout, reconnect, malformed, no observed share, blocked, stale generation, missing intent, and stopped socket cases. [VERIFIED: `crates/bitaxe-stratum/src/v1/submit_response.rs`] |
| Production work correlation | Freeform job/session strings | `ProductionWorkRegistry`, `PoolSessionGeneration`, and `SubmitIntent` | Existing registry binds work to generations, invalidates on clean-jobs/reconnect/session replacement, and creates submit intents only after nonce correlation. [VERIFIED: `crates/bitaxe-stratum/src/v1/production_work.rs`] |
| Evidence-root validation | Manual checklist review only | Existing `operator-evidence`, `mining-allow`, and `just parity` guardrails, extended narrowly for Phase 26 if needed | Existing tools validate required slots, redaction status, safe-state markers, and invalid verified rows. [VERIFIED: `tools/parity/src/operator_evidence.rs`, `tools/parity/src/mining_allow.rs`, `tools/parity/src/main.rs`] |

**Key insight:** Phase 26 is vulnerable to overclaim by construction because the wire fields are simple counters and arrays; the protection must live in typed event/projection invariants and parity evidence gates, not in API route handlers. [VERIFIED: `.planning/phases/26-telemetry-and-parity-closure/26-CONTEXT.md`, `crates/bitaxe-api/src/wire.rs`, `tools/parity/src/main.rs`]

## Common Pitfalls

### Pitfall 1: Direct Counter Mutation Survives in Production Paths

**What goes wrong:** `record_accepted_share` or `record_rejected_share` is called from a path that did not prove live submit intent plus parsed current-generation response. [VERIFIED: `crates/bitaxe-stratum/src/v1/state.rs`, `.planning/phases/26-telemetry-and-parity-closure/26-CONTEXT.md`]

**Why it happens:** `MiningRuntimeState` exposes direct mutation helpers for tests and earlier controlled/fake-pool work. [VERIFIED: `crates/bitaxe-stratum/src/v1/state.rs`, `crates/bitaxe-stratum/src/v1/fake_pool.rs`]

**How to avoid:** Add a production-facing projection function that accepts typed `SubmitClassification` plus generation/request evidence and is the only Phase 26 path that mutates production-visible `ShareCounters`. [VERIFIED: `crates/bitaxe-stratum/src/v1/submit_response.rs`, `.planning/phases/26-telemetry-and-parity-closure/26-CONTEXT.md`]

**Warning signs:** Tests pass by calling counter helpers directly, or fake-pool reports become API-visible evidence. [VERIFIED: `crates/bitaxe-stratum/src/v1/fake_pool.rs`, `docs/parity/evidence/phase-25-live-stratum-runtime-and-safe-stop/share-outcome.md`]

### Pitfall 2: Empty Statistics Placeholder Becomes Fake History

**What goes wrong:** The current empty response is replaced with a fabricated row just to satisfy the route. [VERIFIED: `firmware/bitaxe/src/http_api.rs`, `crates/bitaxe-api/src/statistics.rs`]

**Why it happens:** `StatisticsSample::from_snapshot` can turn any `ApiSnapshot` into a row, even when no event-backed sample boundary exists. [VERIFIED: `crates/bitaxe-api/src/statistics.rs`]

**How to avoid:** Emit samples only when a projection event marks a bounded sample time/source; otherwise keep `empty_statistics_response`. [VERIFIED: `.planning/phases/26-telemetry-and-parity-closure/26-CONTEXT.md`]

**Warning signs:** Statistics history advances on repeated GET requests with no new runtime event. [VERIFIED: `.planning/phases/26-telemetry-and-parity-closure/26-CONTEXT.md`]

### Pitfall 3: Scoreboard Leaks Raw Share Context

**What goes wrong:** `ScoreboardEntry` is populated from active work or submit intent before a proven share outcome and then exposes job id, extranonce2, nonce, or version bits. [VERIFIED: `crates/bitaxe-api/src/scoreboard.rs`, `crates/bitaxe-stratum/src/v1/production_work.rs`, `docs/parity/evidence/phase-23-redacted-operator-evidence-workflow/evidence-contract.md`]

**Why it happens:** Upstream scoreboard shape contains raw-ish mining fields, while Phase 26 redaction rules prohibit targets, extranonces, share payloads, and raw protocol context in committed evidence. [VERIFIED: `reference/esp-miner/main/tasks/scoreboard.h`, `AGENTS.md`, `.planning/phases/26-telemetry-and-parity-closure/26-CONTEXT.md`]

**How to avoid:** Keep `/api/system/scoreboard` as `[]` unless the projection has a redaction-safe, parsed-response-backed share outcome that the plan explicitly allows to serialize. [VERIFIED: `.planning/phases/26-telemetry-and-parity-closure/26-CONTEXT.md`, `docs/parity/evidence/phase-25-live-stratum-runtime-and-safe-stop/share-outcome.md`]

**Warning signs:** Scoreboard tests assert non-empty entries using fixture-only or fake-pool-only data. [VERIFIED: `crates/bitaxe-api/src/scoreboard.rs`, `crates/bitaxe-stratum/src/v1/fake_pool.rs`]

### Pitfall 4: Stale Active-Mining WebSocket Frame After Safe Stop

**What goes wrong:** A connect-time full update or 500 ms cadence diff sends `active` mining state after safe stop. [VERIFIED: `.planning/phases/26-telemetry-and-parity-closure/26-CONTEXT.md`, `crates/bitaxe-api/src/telemetry.rs`]

**Why it happens:** `WebSocketState` correctly plans frames, but it serializes whatever current JSON it receives; stale projection input remains stale output. [VERIFIED: `crates/bitaxe-api/src/websocket_state.rs`, `firmware/bitaxe/src/http_api.rs`]

**How to avoid:** Apply safe-stop projection before `system_info_from_snapshot` is serialized for HTTP or WebSocket, and test both `live_connect_frame` and `live_cadence_frame` after safe stop. [VERIFIED: `firmware/bitaxe/src/live_stratum_runtime.rs`, `firmware/bitaxe/src/runtime_snapshot.rs`, `crates/bitaxe-api/src/telemetry.rs`]

**Warning signs:** Tests cover only `MiningRuntimeState` safe stop, not serialized `/api/ws/live` frames. [VERIFIED: `crates/bitaxe-stratum/src/v1/live_runtime.rs`, `crates/bitaxe-api/src/telemetry.rs`]

### Pitfall 5: Checklist Claims Outrun Artifacts

**What goes wrong:** API, WebSocket, statistics, or scoreboard rows are promoted to `verified` while notes still mention pending, blocked, no-share, or below-verified evidence. [VERIFIED: `tools/parity/src/main.rs`, `docs/parity/checklist.md`]

**Why it happens:** Existing rows already mix older Phase 17/20/21 evidence with residual non-claims. [VERIFIED: `docs/parity/checklist.md`]

**How to avoid:** Add Phase 26 evidence artifacts first, then make the checklist delta exact and let `just parity` reject overbroad verified claims. [VERIFIED: `docs/adr/0006-parity-checklist-as-audit-evidence.md`, `docs/adr/0012-parity-verification-evidence.md`, `tools/parity/src/main.rs`]

**Warning signs:** A row cites implementation files or unit tests as hardware/live proof without a matching redacted artifact. [VERIFIED: `docs/adr/0012-parity-verification-evidence.md`, `.planning/phases/26-telemetry-and-parity-closure/26-CONTEXT.md`]

## Code Examples

Verified patterns from repo sources:

### Current Submit-Response Gate

```rust
// Source: `crates/bitaxe-stratum/src/v1/submit_response.rs`
pub fn classify_maybe_submit_response(
    maybe_intent: Option<&SubmitIntent>,
    request_id: StratumRequestId,
    observation: SubmitResponseObservation,
) -> SubmitClassification {
    let Some(intent) = maybe_intent else {
        return SubmitClassification::Blocked {
            reason: "submit_intent_missing",
        };
    };

    classify_submit_response(intent, request_id, observation)
}
```

This pattern is the minimum gate Phase 26 should preserve before counters can move. [VERIFIED: `crates/bitaxe-stratum/src/v1/submit_response.rs`]

### Current Statistics Projection Shape

```rust
// Source: `crates/bitaxe-api/src/statistics.rs`
pub fn statistics_response(
    current_timestamp: u64,
    maybe_columns: Option<&str>,
    samples: &[StatisticsSample],
) -> StatisticsWire {
    let columns = selected_columns(maybe_columns);
    let mut labels = columns
        .iter()
        .map(|column| column.label().to_owned())
        .collect::<Vec<_>>();
    labels.push(TIMESTAMP_LABEL.to_owned());
    // ...
}
```

Reuse this response builder; Phase 26 should decide whether `samples` is empty or projection-backed. [VERIFIED: `crates/bitaxe-api/src/statistics.rs`]

### Current Scoreboard Wire Shape

```rust
// Source: `crates/bitaxe-api/src/scoreboard.rs`
pub fn scoreboard_response(entries: &[ScoreboardEntry]) -> Vec<ScoreboardEntryWire> {
    entries
        .iter()
        .map(|entry| ScoreboardEntryWire {
            difficulty: entry.difficulty,
            job_id: entry.job_id.clone(),
            extranonce2: entry.extranonce2.clone(),
            ntime: entry.ntime,
            nonce: uppercase_hex_u32(entry.nonce),
            version_bits: uppercase_hex_u32(entry.version_bits),
        })
        .collect()
}
```

Do not change this public shape unless compatibility is deliberately revisited. [VERIFIED: `crates/bitaxe-api/src/scoreboard.rs`, `.planning/phases/26-telemetry-and-parity-closure/26-CONTEXT.md`]

### Current WebSocket Cadence Planner

```rust
// Source: `crates/bitaxe-api/src/telemetry.rs`
pub fn cadence_frame(&mut self, current: Value) -> Option<Value> {
    if self.active_clients == 0 {
        self.maybe_baseline = None;
        return None;
    }

    let Some(baseline) = self.maybe_baseline.as_ref() else {
        self.maybe_baseline = Some(current);
        return None;
    };
    // ...
}
```

The planner already handles hibernation and diff suppression; Phase 26 should focus on the current payload being correct and redacted. [VERIFIED: `crates/bitaxe-api/src/telemetry.rs`]

## State of the Art

| Old / Current Approach | Phase 26 Approach | When Changed | Impact |
| --- | --- | --- | --- |
| `/api/system/statistics` returns `empty_statistics_response(timestamp_ms, None)` unconditionally. | Return empty when no projection-backed samples exist; otherwise emit bounded projection samples. | Phase 26 planning/implementation. [VERIFIED: `firmware/bitaxe/src/http_api.rs`, `.planning/phases/26-telemetry-and-parity-closure/26-CONTEXT.md`] | Avoids fake history while allowing exact runtime-event evidence. |
| `/api/system/scoreboard` returns `scoreboard_response(&[])` unconditionally. | Keep empty unless parsed-response-backed share outcome data exists. | Phase 26 planning/implementation. [VERIFIED: `firmware/bitaxe/src/http_api.rs`, `.planning/phases/26-telemetry-and-parity-closure/26-CONTEXT.md`] | Preserves non-claim for live shares when Phase 25 remains blocked. |
| `runtime_snapshot.rs` replaces mining state through evidence and Phase 25 safe-stop helper functions. | Route updates through a Phase 26 event/projection gate or narrow wrappers that encode invariants. | Phase 26 planning/implementation. [VERIFIED: `firmware/bitaxe/src/runtime_snapshot.rs`, `.planning/phases/26-telemetry-and-parity-closure/26-CONTEXT.md`] | Prevents endpoint or evidence helpers from bypassing counter and safe-stop rules. |
| Existing WebSocket live frames serialize `system_info_from_snapshot(&snapshot)`. | Continue using the same wire payload shape, but ensure the snapshot comes from the Phase 26 projection after safe stop and redaction. | Phase 26 planning/implementation. [VERIFIED: `firmware/bitaxe/src/http_api.rs`, `crates/bitaxe-api/src/telemetry.rs`] | Preserves AxeOS-compatible full/diff semantics while fixing stale-state risks. |
| Checklist rows cite Phase 17/20/21 evidence and retain Phase 26 non-claims. | Add exact Phase 26 artifacts and promote only supported claims. | Phase 26 evidence closure. [VERIFIED: `docs/parity/checklist.md`, `.planning/phases/26-telemetry-and-parity-closure/26-CONTEXT.md`] | Prevents v1.1 closure from overstating accepted/rejected shares or deferred surfaces. |

**Deprecated/outdated for Phase 26:**

- Treating route registration or existing DTO shape as telemetry closure is insufficient; Phase 26 needs runtime-event-derived projection evidence. [VERIFIED: `.planning/phases/26-telemetry-and-parity-closure/26-CONTEXT.md`, `docs/parity/checklist.md`]
- Treating fake-pool accepted/rejected classifications as production-visible live-share proof is insufficient; Phase 25 and Phase 26 decisions require detector-gated live evidence for that tier. [VERIFIED: `.planning/phases/25-live-stratum-runtime-and-safe-stop/25-CONTEXT.md`, `docs/parity/evidence/phase-25-live-stratum-runtime-and-safe-stop/share-outcome.md`]

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
| --- | --- | --- | --- |
| A1 | A new `telemetry_projection.rs` under `crates/bitaxe-stratum/src/v1/` is the likely best home for the core projection, but the phase context allows a bridge in `crates/bitaxe-api` if planning finds that safer. [ASSUMED] | Architecture Patterns | Module ownership could shift during planning; tests and boundaries still need the same invariants. |

## Open Questions

**Status:** RESOLVED during planner revision. The chosen answers below are now reflected in Plans 26-01 through 26-04.

1. **[RESOLVED] Should Phase 26 add a machine-readable promotion manifest?**
   - What we know: CONTEXT says checklist-first exact delta is enough unless reusable v1.1 closure validation is needed. [VERIFIED: `.planning/phases/26-telemetry-and-parity-closure/26-CONTEXT.md`]
   - What's unclear: Whether the planner wants reusable validation for future telemetry closure. [VERIFIED: `.planning/phases/26-telemetry-and-parity-closure/26-CONTEXT.md`]
   - Recommendation: Start with checklist/evidence artifacts and extend `tools/parity` only for overbroad telemetry/statistics/scoreboard claim rejection. [VERIFIED: `tools/parity/src/main.rs`, `.planning/phases/26-telemetry-and-parity-closure/26-CONTEXT.md`]
   - Chosen answer: Do not add a machine-readable promotion manifest in Phase 26. Plan 26-04 uses checklist-first evidence artifacts plus narrow `tools/parity` guardrails for overbroad telemetry/statistics/scoreboard claims. [RESOLVED: `.planning/phases/26-telemetry-and-parity-closure/26-04-PLAN.md`]

2. **[RESOLVED] Can any scoreboard entry be safely emitted without live accepted/rejected proof?**
   - What we know: Context says populate scoreboard only from runtime events tied to parsed pool responses or active share outcomes, and Phase 25 evidence keeps accepted/rejected proof blocked. [VERIFIED: `.planning/phases/26-telemetry-and-parity-closure/26-CONTEXT.md`, `docs/parity/evidence/phase-25-live-stratum-runtime-and-safe-stop/share-outcome.md`]
   - What's unclear: Whether a blocked-safe-prerequisite artifact can support any non-empty upstream-shaped scoreboard row. [VERIFIED: `docs/parity/evidence/phase-25-live-stratum-runtime-and-safe-stop/share-outcome.md`, `crates/bitaxe-api/src/scoreboard.rs`]
   - Recommendation: Plan for an empty scoreboard unless new detector-gated live share artifacts are available during execution. [VERIFIED: `.planning/phases/26-telemetry-and-parity-closure/26-CONTEXT.md`]
   - Chosen answer: No. A blocked-safe-prerequisite artifact does not support non-empty scoreboard rows. Plans 26-02 through 26-04 keep scoreboard output empty unless parsed-response-backed and redaction-allowed share outcome material exists. [RESOLVED: `.planning/phases/26-telemetry-and-parity-closure/26-02-PLAN.md`, `.planning/phases/26-telemetry-and-parity-closure/26-04-PLAN.md`]

3. **[RESOLVED] Should statistics samples include a source label publicly?**
   - What we know: Context says any emitted sample must name or encode that it came from the runtime projection, but D-09 says richer semantics that do not fit upstream fields should stay internal or in evidence. [VERIFIED: `.planning/phases/26-telemetry-and-parity-closure/26-CONTEXT.md`]
   - What's unclear: Whether source labeling belongs in internal fixture/evidence metadata or public statistics response. [VERIFIED: `crates/bitaxe-api/src/statistics.rs`, `.planning/phases/26-telemetry-and-parity-closure/26-CONTEXT.md`]
   - Recommendation: Keep public response upstream-shaped and record source in projection tests/evidence artifacts unless the planner makes a deliberate compatibility decision. [VERIFIED: `.planning/phases/26-telemetry-and-parity-closure/26-CONTEXT.md`]
   - Chosen answer: Keep source labeling internal. Plan 26-01 defines `RuntimeProjectionSampleMarker` and `RuntimeProjectionSampleSource`; Plan 26-02 consumes the marker without adding public JSON fields; Plan 26-04 records source semantics in evidence. [RESOLVED: `.planning/phases/26-telemetry-and-parity-closure/26-01-PLAN.md`, `.planning/phases/26-telemetry-and-parity-closure/26-02-PLAN.md`, `.planning/phases/26-telemetry-and-parity-closure/26-04-PLAN.md`]

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
| --- | --- | --- | --- | --- |
| `just` | Human command surface and verification | yes | `just 1.48.0` | None needed. [VERIFIED: shell probe] |
| `bazel` / Bazelisk | Build/test targets | yes | `bazel 9.1.1` | None needed. [VERIFIED: shell probe, `.bazelversion` implied by tool output] |
| `cargo` | Rust workspace diagnostics and firmware wrapper internals | yes | `cargo 1.88.0-nightly` | Prefer Bazel targets for canonical verification. [VERIFIED: shell probe, `justfile`] |
| `rustc` | Rust compilation | yes | `rustc 1.88.0-nightly` | Prefer Bazel targets for canonical verification. [VERIFIED: shell probe] |
| `espflash` | Ultra 205 detection/board-info and hardware evidence | yes | `espflash 4.0.1` | Hardware evidence remains blocked if detector gate fails. [VERIFIED: shell probe, `AGENTS.md`] |
| `node` | GSD tooling and existing JS helpers | yes | `v24.13.0` | None needed for research/planning. [VERIFIED: shell probe, scripts references] |
| `python3` | ESP-IDF helper ecosystem and scripts | yes | `Python 3.14.4` | None needed for planning. [VERIFIED: shell probe] |

**Missing dependencies with no fallback:** None found during environment probe. [VERIFIED: shell probe]

**Missing dependencies with fallback:** None found during environment probe. [VERIFIED: shell probe]

## Validation Architecture

### Test Framework

| Property | Value |
| --- | --- |
| Framework | Bazel `rust_test` / `sh_test` targets over Rust crates, tools, and scripts. [VERIFIED: `crates/bitaxe-api/BUILD.bazel`, `crates/bitaxe-stratum/BUILD.bazel`, `scripts/BUILD.bazel`] |
| Config file | `BUILD.bazel` files per crate/tool/script. [VERIFIED: glob search, `crates/bitaxe-api/BUILD.bazel`, `crates/bitaxe-stratum/BUILD.bazel`] |
| Quick run command | `bazel test //crates/bitaxe-stratum:tests //crates/bitaxe-api:tests` for projection/API changes. [VERIFIED: `crates/bitaxe-api/BUILD.bazel`, `crates/bitaxe-stratum/BUILD.bazel`] |
| Full suite command | `just test`, plus `just parity`, `just verify-reference`, and lifecycle validation before closure. [VERIFIED: `justfile`, `.planning/phases/26-telemetry-and-parity-closure/26-CONTEXT.md`] |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
| --- | --- | --- | --- | --- |
| API-11 | HTTP API surfaces derive system info, statistics, scoreboard, counters, hashrate inputs, and post-stop state from one projection. | unit + firmware compile | `bazel test //crates/bitaxe-stratum:tests //crates/bitaxe-api:tests` plus `bazel build //firmware/bitaxe:firmware` if firmware files change. [VERIFIED: `justfile`, BUILD files] | Existing crate tests yes; new projection tests are Wave 0 gap. [VERIFIED: glob search] |
| API-12 | `/api/ws` and `/api/ws/live` stream redacted, session-correlated telemetry without stale active-mining after stop. | unit + optional evidence workflow | `bazel test //crates/bitaxe-api:tests`; add script/helper tests if Phase 26 evidence capture script is introduced. [VERIFIED: `crates/bitaxe-api/BUILD.bazel`, `scripts/BUILD.bazel`] | Existing WebSocket tests yes; safe-stop projection frame tests are Wave 0 gap. [VERIFIED: `crates/bitaxe-api/src/telemetry.rs`, `crates/bitaxe-api/src/websocket_state.rs`] |
| API-13 | Statistics, scoreboard, and share counters advance only from runtime events and parsed pool responses. | unit | `bazel test //crates/bitaxe-stratum:tests //crates/bitaxe-api:tests` [VERIFIED: BUILD files] | Existing classifier tests yes; projection counter-gate tests are Wave 0 gap. [VERIFIED: `crates/bitaxe-stratum/src/v1/submit_response.rs`, `crates/bitaxe-stratum/src/v1/state.rs`] |
| EVD-08 | Checklist promotions are exact and preserve deferred non-claims. | tool/unit + workflow | `bazel test //tools/parity:tests`; `just parity`; `just verify-reference`; lifecycle validation. [VERIFIED: `tools/parity/src/main.rs`, `justfile`, `.planning/phases/26-telemetry-and-parity-closure/26-CONTEXT.md`] | Existing parity tests yes; Phase 26 overclaim tests are Wave 0 gap if tool guardrails change. [VERIFIED: `tools/parity/src/main.rs`] |

### Sampling Rate

- **Per task commit:** Run the narrow Bazel target for the touched crate/tool/script. [VERIFIED: `standards/core/verification.md`, BUILD files]
- **Per wave merge:** Run `bazel test //crates/bitaxe-stratum:tests //crates/bitaxe-api:tests //tools/parity:tests` and any touched script tests. [VERIFIED: BUILD files, `.planning/phases/26-telemetry-and-parity-closure/26-CONTEXT.md`]
- **Phase gate:** Run relevant targeted tests, firmware build if firmware changed, `just parity`, `just verify-reference`, and GSD lifecycle validation. [VERIFIED: `.planning/phases/26-telemetry-and-parity-closure/26-CONTEXT.md`, `justfile`]

### Wave 0 Gaps

- [ ] `crates/bitaxe-stratum/src/v1/telemetry_projection.rs` or equivalent projection tests for event folding, stale generation rejection, safe-stop reset, and counter gating. [ASSUMED]
- [ ] `crates/bitaxe-api` tests proving projection-backed `SystemInfoWire`, `StatisticsWire`, `ScoreboardEntryWire`, and `/api/ws/live` connect/cadence frames after safe stop. [VERIFIED: existing API crate test structure]
- [ ] `tools/parity` tests rejecting Phase 26 telemetry/statistics/scoreboard `verified` rows with missing artifacts, blocker language, pending evidence, or absent redaction review if planner adds guardrails. [VERIFIED: `tools/parity/src/main.rs`, `.planning/phases/26-telemetry-and-parity-closure/26-CONTEXT.md`]
- [ ] Phase 26 evidence directory with redaction-reviewed API/WebSocket/statistics/scoreboard artifacts or explicit blocked non-claims. [VERIFIED: `.planning/phases/26-telemetry-and-parity-closure/26-CONTEXT.md`, `docs/parity/evidence/phase-23-redacted-operator-evidence-workflow/evidence-contract.md`]

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
| --- | --- | --- |
| V2 Authentication | no for Phase 26 core projection | Phase 26 does not introduce authentication; keep existing route access/origin gates unchanged. [VERIFIED: `firmware/bitaxe/src/http_api.rs`, `.planning/phases/26-telemetry-and-parity-closure/26-CONTEXT.md`] |
| V3 Session Management | yes for WebSocket sessions | Use existing `WebSocketState` session registration, max-client cap, unregister, and route-local hibernation. [VERIFIED: `crates/bitaxe-api/src/websocket_state.rs`, `firmware/bitaxe/src/websocket_api.rs`] |
| V4 Access Control | yes for HTTP/WebSocket route access | Preserve `handle_with_access_gate`, `plan_websocket_upgrade`, and no network-scan target discovery rules. [VERIFIED: `firmware/bitaxe/src/http_api.rs`, `AGENTS.md`] |
| V5 Input Validation | yes | Parse and validate query/config/evidence inputs at boundaries; use typed projection events instead of raw primitives. [VERIFIED: `standards/core/architecture.md`, `crates/bitaxe-api/src/statistics.rs`, `scripts/phase25-live-stratum-evidence.sh`] |
| V6 Cryptography | limited | Do not add cryptography; protect local secrets through runtime-only inputs and redaction, not custom crypto. [VERIFIED: `AGENTS.md`, `docs/parity/evidence/phase-23-redacted-operator-evidence-workflow/evidence-contract.md`] |

### Known Threat Patterns for Phase 26

| Pattern | STRIDE | Standard Mitigation |
| --- | --- | --- |
| Sensitive pool or network values in telemetry/evidence | Information Disclosure | Redact before fan-out and commit only category labels or reviewed artifacts. [VERIFIED: `.planning/phases/26-telemetry-and-parity-closure/26-CONTEXT.md`, `AGENTS.md`] |
| Stale active-mining state after safe stop | Tampering / Repudiation | Fold safe-stop events into the projection before HTTP/WebSocket serialization and test connect/cadence frames. [VERIFIED: `firmware/bitaxe/src/live_stratum_runtime.rs`, `crates/bitaxe-api/src/telemetry.rs`] |
| Counter inflation from fake-pool, stale, or blocked observations | Tampering | Gate production-visible counter advances through current-generation submit intent and parsed pool response classification. [VERIFIED: `crates/bitaxe-stratum/src/v1/submit_response.rs`, `.planning/phases/26-telemetry-and-parity-closure/26-CONTEXT.md`] |
| Parity overclaim in checklist | Repudiation | Use exact artifacts, `just parity`, and guardrails rejecting invalid verified rows. [VERIFIED: `tools/parity/src/main.rs`, `docs/adr/0012-parity-verification-evidence.md`] |
| Raw scoreboard fields leaking share context | Information Disclosure | Keep scoreboard empty without parsed-response-backed share events and avoid committed raw job/extranonce/nonce artifacts. [VERIFIED: `crates/bitaxe-api/src/scoreboard.rs`, `docs/parity/evidence/phase-23-redacted-operator-evidence-workflow/evidence-contract.md`] |

## Sources

### Primary (HIGH confidence)

- `.planning/phases/26-telemetry-and-parity-closure/26-CONTEXT.md` - Locked decisions, phase boundary, exact non-claims, canonical implementation surfaces. [VERIFIED: file read]
- `.planning/REQUIREMENTS.md` - API-11, API-12, API-13, EVD-08 requirement text and v1.1 scope. [VERIFIED: file read]
- `.planning/STATE.md` - Phase 25 closure status, live-share blockers, and carried v1.1 decisions. [VERIFIED: file read]
- `AGENTS.md` and `AGENTS.bright-builds.md` - Ultra 205 detector gates, redaction rules, GSD workflow, and standards routing. [VERIFIED: project instructions and file read]
- `standards/core/architecture.md`, `standards/core/code-shape.md`, `standards/core/testing.md`, `standards/core/verification.md`, `standards/languages/rust.md` - Functional core, code shape, testing, verification, and Rust conventions. [VERIFIED: file reads]
- `crates/bitaxe-stratum/src/v1/state.rs`, `production_work.rs`, `submit_response.rs`, `live_runtime.rs`, `fake_pool.rs` - Runtime state, session generation, submit intent, classification, safe-stop, and fake-pool behavior. [VERIFIED: file reads and ripgrep]
- `crates/bitaxe-api/src/snapshot.rs`, `wire.rs`, `statistics.rs`, `scoreboard.rs`, `telemetry.rs`, `websocket_state.rs`, `mining.rs` - API snapshot, wire DTOs, mining projection, statistics, scoreboard, live telemetry, and WebSocket state. [VERIFIED: file reads]
- `firmware/bitaxe/src/http_api.rs`, `runtime_snapshot.rs`, `websocket_api.rs`, `live_stratum_runtime.rs` - Firmware route handlers, snapshot bridge, WebSocket bridge, and Phase 25 safe-stop publication. [VERIFIED: file reads and ripgrep]
- `tools/parity/src/main.rs`, `operator_evidence.rs`, `mining_allow.rs` - Checklist, operator evidence, mining allow-manifest, and verified-row validation. [VERIFIED: file reads and ripgrep]
- `docs/parity/evidence/phase-23-redacted-operator-evidence-workflow/evidence-contract.md` and Phase 25 evidence files - Evidence slots, redaction contract, safe-stop, share-outcome, summary, and non-claims. [VERIFIED: file reads]

### Secondary (MEDIUM confidence)

- `reference/esp-miner/main/http_server/system_api_json.c`, `websocket_api.c`, `websocket_log.c`, `http_server.c`, `tasks/statistics_task.h`, `tasks/hashrate_monitor_task.c`, `tasks/scoreboard.h`, `system.c`, `global_state.h` - Upstream behavioral breadcrumbs for AxeOS field shapes, statistics, scoreboard, WebSocket cadence, and runtime fields. These are read-only GPL reference materials used for behavior, not code copying. [CITED: local pinned reference]
- `docs/adr/0006-parity-checklist-as-audit-evidence.md`, `docs/adr/0012-parity-verification-evidence.md`, `docs/adr/0013-mit-first-with-gpl-guardrails.md`, `PROVENANCE.md` - Evidence, verification, and licensing policies. [VERIFIED: file reads]

### Tertiary (LOW confidence)

- No web-only or unverified external sources were used. [VERIFIED: research log]

## Metadata

**Confidence breakdown:**

- Standard stack: HIGH - Phase 26 reuses existing Rust crates, Bazel/Just commands, and internal tools with observed versions and file ownership. [VERIFIED: `Cargo.toml`, `justfile`, shell probe]
- Architecture: HIGH - Locked context and existing modules converge on functional core / imperative shell and typed projection boundaries. [VERIFIED: `.planning/phases/26-telemetry-and-parity-closure/26-CONTEXT.md`, standards files, codebase reads]
- Pitfalls: HIGH - Pitfalls map directly to existing mutable counters, placeholder handlers, redaction contracts, WebSocket planner behavior, and parity guardrails. [VERIFIED: codebase reads and evidence files]
- Evidence closure: HIGH for current non-claims and required guardrails; MEDIUM for exact Phase 26 artifact filenames because the planner still chooses them. [VERIFIED: Phase 26 context; ASSUMED for filenames]

**Research date:** 2026-07-05
**Valid until:** 2026-08-04 for internal architecture and evidence policy; revisit sooner if Phase 25 hardware evidence is rerun and produces accepted/rejected live share artifacts. [VERIFIED: Phase 25 evidence status]
