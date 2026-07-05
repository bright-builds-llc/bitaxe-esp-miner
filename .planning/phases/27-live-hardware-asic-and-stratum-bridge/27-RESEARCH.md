# Phase 27: Live Hardware ASIC And Stratum Bridge - Research

**Researched:** 2026-07-05
**Domain:** BM1366 production UART bridge, Stratum v1 live socket runtime integration, detector-gated share-outcome evidence
**Confidence:** HIGH for repo-local integration gaps and locked decisions; MEDIUM for hardware timing/init sequencing because UART retention and production init paths are not yet implemented on device.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

### Live Bridge Boundary

- **D-01:** Extend `firmware/bitaxe/src/live_stratum_runtime.rs` as the sole live production bridge. Do not create a parallel production runtime module or route live socket mining back through `controlled_mining_runtime.rs`.
- **D-02:** Keep the functional-core / imperative-shell split: guarded dispatch plans, `ProductionWorkRegistry`, nonce correlation, submit intent, and submit classification stay in `crates/bitaxe-stratum`; ESP-IDF UART/GPIO effects, ASIC adapter execution, socket I/O, watchdog yields, and evidence markers stay in firmware.
- **D-03:** Preserve Phase 25 safe-stop, prerequisite, watchdog, and post-stop snapshot behavior. Phase 27 adds ASIC dispatch and nonce feedback inside the existing live runtime loop rather than replacing the socket shell.

### Production Command Dispatch In Live Runtime

- **D-04:** When the pure live runtime emits work-dispatch actions from pool-derived notify/clean-jobs state, the firmware bridge must translate them into `GuardedBm1366DispatchPlan` output and execute `maybe_production_command` through the existing ASIC adapter path used by the controlled runtime.
- **D-05:** Live dispatch must use `Bm1366ProductionCommand::SendProductionWork` and `Bm1366ProductionCommand::ReadProductionResult` only. Diagnostic `Bm1366Command::SendDiagnosticWork` must remain unreachable from the live socket path.
- **D-06:** Production ASIC status logs in the live path must reuse the Phase 24 redaction-safe keys: `asic_production_status=initialized`, `asic_production_status=work_dispatched`, `asic_production_status=result_correlated`, and `asic_production_status=fail_closed` with `reason={label} mining=disabled work_submission=disabled` for fail-closed cases.

### Nonce Observation Feedback And Submit Classification

- **D-07:** Hardware nonce/result reads must be wrapped as `ProductionNonceObservation { observed_generation, result }` at the firmware boundary, using the generation associated with the dispatch/read attempt rather than inferring session identity from parsed ASIC bytes alone.
- **D-08:** The live runtime must feed observations back into `ProductionWorkRegistry::correlate_nonce_result`, then only emit `mining.submit` when a current-generation `SubmitIntent` exists. Uncorrelated, stale-generation, malformed, duplicate, or blocked outcomes must fail closed without share claims.
- **D-09:** Accepted or rejected share classification remains tied to live pool response plus matching submit intent. Phase 27 may produce the first detector-gated live share-outcome artifact, but fake-pool-only or implementation-only paths must not promote STR-09 above the evidence actually captured.

### Distinct Phase 27 Evidence Mode

- **D-10:** Add a distinct compile-time opt-in mode for Phase 27 live hardware bridge evidence, analogous to Phase 25's mode/ack pair. Missing or mismatched mode/ack values must keep the bridge fail-closed and must not silently fall back to controlled or Phase 25-only behavior.
- **D-11:** Phase 27 mode must require Phase 22 prerequisite readiness before any ASIC dispatch, pool settings access, or socket connect attempt, preserving the existing fail-closed ordering from Phase 25.

### Hardware Evidence And Redaction

- **D-12:** Add a repo-owned Phase 27 evidence wrapper with blocked and hardware modes, detector-first hardware path, board-info gate, allow-manifest validation, redaction review, and exact non-claims when safe prerequisites or live share proof cannot proceed.
- **D-13:** Committed evidence must record share outcome as `accepted`, `rejected`, or `blocked_safe_prerequisite` using category labels only. Raw pool endpoints, ports, workers, owner addresses, passwords, targets, extranonces, share payloads, socket errors, device URLs, IPs, MACs, Wi-Fi values, tokens, NVS secrets, and raw BM1366 frames must not appear in committed artifacts.
- **D-14:** Update Phase 23/25 evidence-root slots and `tools/parity` only after exact Phase 27 artifacts exist. Preserve explicit non-claims when hardware prerequisites block live share proof.

### Verification And Checklist Semantics

- **D-15:** Unit and firmware tests must prove live-bridge dispatch, nonce observation stamping, correlation gating, fail-closed blockers, mode gating, and evidence-wrapper blocked paths without requiring hardware for every assertion.
- **D-16:** Hardware verification must follow `just detect-ultra205`, board `205`, repo-owned commands, runtime-only local credentials, redaction review, and exact evidence recording. If detection, safe prerequisites, ASIC dispatch, socket behavior, or share outcome proof is blocked, record the blocker instead of inferring success.
- **D-17:** STR-08, STR-09, ASIC-10, and ASIC-11 may advance only to the exact level supported by Phase 27 source, tests, workflow evidence, detector-gated hardware evidence, and redaction review actually produced.

### Claude's Discretion

Claude may choose exact module names, adapter helper names, evidence filenames, timeout budgets, mode/ack constant names, fixture shapes, and plan count. Those choices must preserve functional core / imperative shell structure, typed fail-closed behavior, redaction rules, Ultra 205 detector gating, repo-owned verification, and conservative parity semantics.

### Deferred Ideas (OUT OF SCOPE)

- Phase 28 checklist promotion from Phase 27 artifacts belongs to Phase 28, not Phase 27 closure work beyond producing the artifacts Phase 28 needs.
- Full active voltage, fan, thermal fault-stimulus, self-test hardware closure, OTA/recovery destructive evidence, runtime display/input, BAP, Stratum v2, non-205 boards, other ASIC families, and unbounded stress mining remain future work.
- Broad Phase 26 telemetry or API/WebSocket projection changes remain out of scope unless a minimal post-bridge runtime marker is required for evidence capture.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| STR-08 | Ultra 205 production mining uses a real Stratum v1 TCP socket lifecycle for connect, subscribe, authorize, difficulty/extranonce, notify, submit, response classification, reconnect, and safe stop. | Phase 25 socket shell exists; Phase 27 must wire notify→ASIC dispatch→submit inside that shell and prove detector-gated hardware lifecycle evidence. [VERIFIED: firmware/bitaxe/src/live_stratum_runtime.rs] |
| STR-09 | Ultra 205 production mining classifies at least one real pool response to a live ASIC-derived `mining.submit` as accepted, rejected, or explicitly blocked by a safe prerequisite. | `SendSubmitShare` handler and classifier exist; pure runtime never queues submit today; share-outcome slot is `blocked_safe_prerequisite`. Bridge must emit submit only after correlated nonce observation. [VERIFIED: docs/parity/evidence/phase-25-live-stratum-runtime-and-safe-stop/share-outcome.md] |
| ASIC-10 | Ultra 205 production mining dispatches BM1366 work derived from the active pool job, tracks job/extranonce/difficulty context, and invalidates stale work on clean-jobs or reconnect. | `ProductionWorkRegistry` and `LiveStratumRuntime::apply_notify` enqueue pool work; firmware pump loop never calls `dispatch_next` or executes `Bm1366ProductionCommand`. [VERIFIED: crates/bitaxe-stratum/src/v1/live_runtime.rs] |
| ASIC-11 | Ultra 205 production mining maps live BM1366 nonce or result observations back to active pool work before any share submission claim is recorded. | `ProductionNonceObservation` and `correlate_nonce_result` exist; controlled/fake-pool paths demonstrate pattern; live firmware lacks UART read→observation→submit wiring. [VERIFIED: crates/bitaxe-stratum/src/v1/fake_pool.rs correlate_runtime_submit_intent] |
</phase_requirements>

## Summary

Phase 27 is an integration phase, not a greenfield protocol phase. Phases 24–25 already delivered the pure production registry, guarded mining-loop dispatch plan, live Stratum socket adapter, submit classifier, prerequisite gate, safe-stop postconditions, and blocked share-outcome evidence workflow. The audit gap is explicit: `live_stratum_runtime.rs` drives TCP I/O and enqueues Stratum client messages but never executes `Bm1366ProductionCommand`, never reads BM1366 result frames, and never feeds `ProductionNonceObservation` back into the runtime to queue `mining.submit`. [VERIFIED: firmware/bitaxe/src/live_stratum_runtime.rs pump_live_socket_until_cleanup, write_runtime_action]

`controlled_mining_runtime.rs` is the reference for status publication and guarded-plan consumption, but it only counts `adapter_actions()` length—it does not execute UART effects. [VERIFIED: firmware/bitaxe/src/controlled_mining_runtime.rs adapter_action_count] Phase 27 must add real production UART execution while keeping live socket mining out of the controlled runtime path per D-01.

A second critical gap is peripheral lifetime: `main.rs` passes `uart1`, reset, and GPIO pins into `run_boot_gate_with_peripherals`, which constructs a local `AsicUart` and drops it when the boot gate returns. There is no retained production UART handle for the live socket loop. [VERIFIED: firmware/bitaxe/src/main.rs, firmware/bitaxe/src/asic_adapter.rs] The planner must schedule ASIC handle retention (or a production-runtime singleton) before bridge dispatch can work on hardware.

**Primary recommendation:** Extend the existing `pump_live_socket_until_cleanup` loop with an ASIC bridge sub-step (watchdog `StepKind::Asic`) that, after `WorkQueued`, runs `GuardedMiningLoopInputs::plan` against the live runtime's registry, executes `maybe_production_command` via a new `asic_adapter` production executor, stamps `ProductionNonceObservation` at the firmware boundary, calls back into pure runtime to queue `SendSubmitShare`, and preserves Phase 25 safe-stop ordering on any fail-closed blocker.

## Project Constraints

- No `.cursor/rules/` project-local files found. [VERIFIED: glob search]
- Follow `AGENTS.md` Ultra 205 detector gate, credential handling, redaction, and hardware evidence recording. [VERIFIED: AGENTS.md]
- GSD workflow: phase work through `/gsd-execute-phase`; research artifact is planning input only.
- Functional core / imperative shell per `standards/core/architecture.md`. [VERIFIED: standards/core/architecture.md]
- ESP-IDF `v5.5.4`, `esp-idf-svc 0.52.1`, Bazel 9.1.1, `just` command surface unchanged. [VERIFIED: AGENTS.md, firmware/bitaxe/Cargo.toml]

## Codebase Findings (Integration Gap)

### What exists and is reusable

| Asset | Location | Phase 27 use |
|-------|----------|--------------|
| Production command types | `crates/bitaxe-asic/src/bm1366/production.rs` | `SendProductionWork`, `ReadProductionResult`, `adapter_actions()` |
| Active-work registry | `crates/bitaxe-stratum/src/v1/production_work.rs` | enqueue, dispatch, correlate, invalidation |
| Guarded dispatch plan | `crates/bitaxe-stratum/src/v1/mining_loop.rs` | `GuardedMiningLoopInputs::plan` → `maybe_production_command`, `maybe_submit_intent` |
| Live socket shell | `firmware/bitaxe/src/live_stratum_runtime.rs` | TCP connect, read/write, prerequisite gate, safe-stop |
| Pure live runtime | `crates/bitaxe-stratum/src/v1/live_runtime.rs` | notify→registry enqueue, invalidation, `SendSubmitShare` action type |
| Submit classifier | `crates/bitaxe-stratum/src/v1/submit_response.rs` | intent+request-id tied accepted/rejected/blocked |
| Production status publishers | `firmware/bitaxe/src/asic_adapter/status.rs` | exact `asic_production_status=*` keys |
| UART read/parse reference | `firmware/bitaxe/src/asic_adapter.rs` `run_work_result_diagnostic` | `read_exact` + `parse_bm1366_result_frame` pattern |
| Fake-pool live correlation | `crates/bitaxe-stratum/src/v1/fake_pool.rs` | `correlate_runtime_submit_intent` host-side pattern |
| Phase 25 evidence wrapper | `scripts/phase25-live-stratum-evidence.sh` | template for Phase 27 blocked/hardware modes |
| Mining allow tiers | `tools/parity/src/mining_allow.rs` | strict wrapper command validation |

### What is missing (must implement)

1. **Live runtime never queues production bridge actions.** `LiveRuntimeAction` has `SendClientMessage` and `SendSubmitShare`, but `live_runtime.rs` only pushes subscribe/authorize messages—never `SendSubmitShare` and no dispatch/read intent. [VERIFIED: crates/bitaxe-stratum/src/v1/live_runtime.rs]
2. **Pump loop is socket-only.** `pump_live_socket_until_cleanup` alternates draining socket actions and reading JSON lines; no `StepKind::Asic` production step despite watchdog category existing. [VERIFIED: firmware/bitaxe/src/live_stratum_runtime.rs]
3. **No production UART executor.** `interpret_action` handles diagnostic actions but boot gate drops UART; no `execute_production_command(Bm1366ProductionCommand, valid_jobs)` API. [VERIFIED: firmware/bitaxe/src/asic_adapter.rs]
4. **Phase 27 evidence mode absent.** `MiningEvidenceMode` has FailClosed, LiveMiningRuntime, Phase25LiveStratumRuntime only. [VERIFIED: firmware/bitaxe/src/mining_evidence_mode.rs]
5. **Production entry uses Phase 25 gate.** `maybe_start_after_network_setup` checks `is_phase25_live_stratum_runtime()` only; Phase 27 needs distinct mode/ack and startup path. [VERIFIED: firmware/bitaxe/src/live_stratum_runtime.rs, firmware/bitaxe/src/main.rs]
6. **Share-outcome slot blocked.** Phase 25 committed `share_outcome: blocked_safe_prerequisite` with accepted/rejected non-claims. [VERIFIED: docs/parity/evidence/phase-25-live-stratum-runtime-and-safe-stop/share-outcome.md]

### Recommended data flow (target)

```
pool notify → LiveStratumRuntime::apply_notify (pure, enqueue registry)
           → firmware ASIC bridge step:
                GuardedMiningLoopInputs::plan (gate + registry + maybe_nonce_observation)
                → execute maybe_production_command (UART WriteFrame / ReadExact)
                → ProductionNonceObservation { observed_generation: dispatch.generation, result }
                → correlate_nonce_result → SubmitIntent
                → runtime.queue_submit_share(intent) → SendSubmitShare action
           → write_runtime_action (socket mining.submit)
           → handle_socket_line (classify pool response)
           → publish_submit_classification + evidence markers
```

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `bitaxe-stratum` | workspace | Live runtime, guarded loop, production registry, submit classifier | Locked functional core per D-02 [VERIFIED: crates/bitaxe-stratum/Cargo.toml] |
| `bitaxe-asic` | workspace | `Bm1366ProductionCommand`, result parse, adapter actions | Locked ASIC packet boundary per D-02 [VERIFIED: crates/bitaxe-asic] |
| `bitaxe-safety` | workspace | `ProductionMiningPreconditions`, `MiningLoopGate`, watchdog | Phase 22 gate ordering per D-11 [VERIFIED: crates/bitaxe-safety] |
| `esp-idf-svc` / `esp-idf-sys` | 0.52.1 / 0.37.2 via workspace | TCP socket, UART, GPIO, FreeRTOS | Project ESP-IDF Rust stack [VERIFIED: AGENTS.md] |
| `espflash` | 4.0.1 local | detect, flash-monitor, board-info | Repo flash backend [VERIFIED: shell probe] |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| Bazel `rules_rust` | 0.70.0 | Host crate tests | `bazel test //crates/bitaxe-stratum:tests` |
| `just` | 1.48.0 | Human commands | `just detect-ultra205`, evidence wrappers |
| Phase 25 evidence scripts | repo | Wrapper pattern | Mirror for Phase 27 share-outcome |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Extend `live_stratum_runtime.rs` (D-01) | Route through `controlled_mining_runtime.rs` | Violates locked decision; duplicates socket shell |
| New parallel bridge module | Single extended live runtime | Violates D-01 |
| Hand-roll BM1366 frames in firmware | `Bm1366ProductionCommand::adapter_actions()` | Violates architecture; GPL/provenance risk |

**No new Cargo dependencies required** for Phase 27 bridge work. [VERIFIED: existing crate graph covers all types]

## Architecture Patterns

### Recommended module structure

```
firmware/bitaxe/src/
├── live_stratum_runtime.rs      # extend pump loop + bridge helpers (D-01, D-03)
├── asic_adapter.rs              # add production executor; retain UART handle
├── asic_adapter/
│   ├── uart.rs                  # reuse read_exact/write_frame
│   ├── status.rs                # unchanged keys (D-06)
│   └── production.rs            # optional: execute_production_command helper
├── mining_evidence_mode.rs      # Phase 27 mode/ack pair (D-10)
└── main.rs                      # retain ASIC peripherals; Phase 27 startup gate

crates/bitaxe-stratum/src/v1/
├── live_runtime.rs              # queue_submit_share; maybe ProductionBridgePlan hook
└── mining_loop.rs               # GuardedMiningLoopSource::Live or reuse Notify source

scripts/
├── phase27-live-hardware-bridge-evidence.sh
└── phase27-live-hardware-bridge-evidence-test.sh

docs/parity/evidence/phase-27-live-hardware-asic-and-stratum-bridge/
├── share-outcome.md
├── summary.md
├── detector.md
├── board-info.md
├── redaction-review.md
└── ...
```

### Pattern 1: Guarded plan at firmware boundary

**What:** After pure runtime queues work (`WorkQueued`), firmware builds `GuardedMiningLoopInputs` from live runtime state and optional `ProductionNonceObservation`, calls `plan()`, executes shell effects from plan output.

**When to use:** Every notify-driven dispatch and every post-read correlation cycle.

**Example:**

```rust
// Pattern from controlled_runtime + mining_loop; firmware executes effects only
let inputs = GuardedMiningLoopInputs {
    gate: mining_loop_gate(preconditions.decision()),
    pool_defaults: ultra_205_defaults(),
    source: GuardedMiningLoopSource::Notify,
    production_registry: runtime.production_registry().clone(),
    runtime_state: runtime.state().clone(),
    maybe_nonce_observation,
};
let plan = inputs.plan()?;
if let Some(command) = plan.maybe_dispatch.and_then(|d| d.maybe_production_command) {
    production_executor.execute(command, runtime.production_registry().valid_jobs())?;
}
if let Some(intent) = plan.maybe_submit_intent {
    runtime.queue_submit_share(intent, next_request_id)?;
}
```

Source: [VERIFIED: crates/bitaxe-stratum/src/v1/mining_loop.rs, crates/bitaxe-stratum/src/v1/controlled_runtime.rs, crates/bitaxe-stratum/src/v1/fake_pool.rs]

### Pattern 2: Production UART executor (diagnostic-derived)

**What:** Reuse `AsicUart::write_frame` / `read_exact` + `parse_bm1366_result_frame` from diagnostic path; map `Bm1366ProductionCommand` through `adapter_actions()`; never log raw frame bytes.

**When to use:** `SendProductionWork` after dispatch; `ReadProductionResult` before observation stamping.

**Example:**

```rust
// Derived from firmware/bitaxe/src/asic_adapter.rs run_work_result_diagnostic
for action in command.adapter_actions() {
    interpret_production_action(&action, uart, reset)?;
}
let frame = uart.read_exact(BM1366_RESULT_FRAME_LEN, RESULT_WORK_TIMEOUT_MS)?;
let result = parse_bm1366_result_frame(&frame, &valid_jobs, extranonce2_len)?;
```

Source: [VERIFIED: firmware/bitaxe/src/asic_adapter.rs, crates/bitaxe-asic/src/bm1366/result.rs]

### Pattern 3: Phase 25 evidence wrapper mirror

**What:** `scripts/phase27-*-evidence.sh` with `--mode blocked|hardware`, detector-first hardware path, category-only pool/wifi labels, share-outcome slot with `accepted|rejected|blocked_safe_prerequisite`.

**When to use:** Phase 27 closure and mining-allow manifest generation.

Source: [VERIFIED: scripts/phase25-live-stratum-evidence.sh]

### Anti-Patterns to Avoid

- **Routing live socket mining through `controlled_mining_runtime.rs`:** Violates D-01; controlled path uses fixture transcript, not live TCP.
- **Inferring `PoolSessionGeneration` from parsed ASIC bytes:** Violates D-07; stamp from dispatch generation.
- **Promoting STR-09 from fake-pool or unit tests alone:** Violates D-09, D-17.
- **Reusing Phase 25 mode/ack for Phase 27 hardware bridge:** Violates D-10.
- **Logging raw BM1366 frames or Stratum payloads:** Violates D-13 and AGENTS.md redaction rules.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| BM1366 job frames | Custom UART byte packing in firmware | `Bm1366ProductionCommand::adapter_actions()` | CRC/layout lives in `bitaxe-asic` |
| Nonce→share mapping | Ad hoc job-id matching | `ProductionWorkRegistry::correlate_nonce_result` | Generation, duplicate, stale guards |
| Submit accepted/rejected | String matching pool JSON | `classify_submit_response` + `SubmitIntent` | Request-id and generation binding |
| Evidence redaction | Ad hoc sed scripts | Phase 25 wrapper pattern + `mining_allow` validation | Prohibited-token guards already exist |
| Prerequisite decisions | Shell string checks | `ProductionMiningPreconditions::decision` + `MiningLoopGate` | Phase 22 typed blockers |

## Common Pitfalls

### Pitfall 1: Dropped ASIC UART after boot gate

**What goes wrong:** Live bridge calls production dispatch but UART driver no longer exists.
**Why it happens:** `run_boot_gate_with_peripherals` owns peripherals locally and returns.
**How to avoid:** Refactor boot path to initialize ASIC into a retained `ProductionAsicHandle` (static `OnceLock` or struct passed to live runtime) before Wi-Fi/socket startup.
**Warning signs:** Compile errors on moved peripherals or runtime `uart_unavailable` blockers on every hardware run.

### Pitfall 2: Submit queued without correlated intent

**What goes wrong:** Pool sees `mining.submit` not tied to live ASIC work; STR-09 overclaim.
**Why it happens:** Socket loop emits submit from notify alone without `correlate_nonce_result`.
**How to avoid:** Only queue `SendSubmitShare` when `maybe_submit_intent` is `Some` after observation feedback (D-08).
**Warning signs:** Unit tests pass socket-only path; hardware shows submits with `result_receive_status=bounded_no_result`.

### Pitfall 3: Phase 25 vs Phase 27 mode collision

**What goes wrong:** Wrong runtime starts or fail-closed silent fallback.
**Why it happens:** Shared `maybe_start_after_network_setup` gate.
**How to avoid:** Distinct `PHASE27_*` mode/ack; explicit mutually exclusive startup branches in `main.rs` (D-10).
**Warning signs:** Phase 25 firmware package runs bridge code without Phase 27 ack.

### Pitfall 4: Bounded pump exits before ASIC+submit completes

**What goes wrong:** `LIVE_SOCKET_PUMP_ITERATIONS` (16) stops loop at `VerificationCleanup` before share response.
**Why it happens:** Phase 25 bounded verification cleanup path.
**How to avoid:** Phase 27 hardware mode needs explicit iteration budget or stop condition that waits for submit classification or blocker; document timeout in evidence wrapper.
**Warning signs:** Hardware logs show `active` but never `share_submission_status` or classifier markers.

### Pitfall 5: Production init assumed without evidence

**What goes wrong:** Work dispatched to uninitialized ASIC; `production_asic_init_failed` or silent UART timeout.
**Why it happens:** `mining_loop_gate` hardcodes `asic_initialized: true` in live runtime tests/production path. [VERIFIED: live_stratum_runtime.rs mining_loop_gate]
**How to avoid:** Publish `asic_production_status=initialized` only after documented init boundary; fail-closed with `ProductionAsicBlocker::AsicInitFailed` when boot gate did not reach production-ready state.
**Warning signs:** Chip-detect-only boot package used for live mining evidence.

## Code Examples

### Stamp observation at firmware boundary (D-07)

```rust
// After successful dispatch + result read
let observation = ProductionNonceObservation {
    observed_generation: dispatch.generation,
    result: parsed_nonce_result,
};
// Pass to guarded plan or direct correlate_nonce_result on registry
```

Source: [VERIFIED: crates/bitaxe-stratum/src/v1/production_work.rs, crates/bitaxe-stratum/src/v1/controlled_runtime.rs]

### Production status keys (D-06)

```rust
asic_adapter::publish_production_asic_status(ProductionAsicStatus::WorkDispatched);
// Logs: asic_production_status=work_dispatched
asic_adapter::publish_production_asic_blocked_status(ProductionAsicBlocker::ResultTimeout);
// Logs: asic_production_status=fail_closed reason=production_result_timeout mining=disabled work_submission=disabled
```

Source: [VERIFIED: firmware/bitaxe/src/asic_adapter/status.rs]

### Phase 25 prerequisite gate ordering (preserve in Phase 27)

```rust
let decision = preconditions.decision();
let gate = mining_loop_gate(decision);
if let MiningLoopDecision::Blocked { reason } = gate.decision() {
    publish_blocked(reason);
    return LiveStartOutcome::Blocked { reason };
}
// Only then read pool settings and connect socket
```

Source: [VERIFIED: firmware/bitaxe/src/live_stratum_runtime.rs start_live_stratum_runtime_with_dependencies]

## File Touch List (planned)

| File | Action | Rationale |
|------|--------|-----------|
| `firmware/bitaxe/src/live_stratum_runtime.rs` | **Modify** | Primary bridge: pump loop ASIC step, prerequisite source for hardware, Phase 27 entry |
| `firmware/bitaxe/src/asic_adapter.rs` | **Modify** | Retain UART; `execute_production_command`; production `interpret_action` path |
| `firmware/bitaxe/src/asic_adapter/production.rs` | **Add (optional)** | Keep `asic_adapter.rs` under size guidance |
| `firmware/bitaxe/src/asic_adapter/status.rs` | **Reuse** | No key changes if D-06 followed |
| `firmware/bitaxe/src/mining_evidence_mode.rs` | **Modify** | Phase 27 mode/ack + tests |
| `firmware/bitaxe/src/main.rs` | **Modify** | Peripheral retention; Phase 27 startup branch |
| `firmware/bitaxe/src/controlled_mining_runtime.rs` | **Minimal** | Ensure Phase 27 mode no-op; no live routing |
| `crates/bitaxe-stratum/src/v1/live_runtime.rs` | **Modify** | `queue_submit_share`, bridge plan hook, tests |
| `crates/bitaxe-stratum/src/v1/mining_loop.rs` | **Maybe modify** | `GuardedMiningLoopSource::Live` if needed for evidence labels |
| `crates/bitaxe-stratum/src/v1/fake_pool.rs` | **Extend tests** | Live bridge deterministic coverage without hardware |
| `scripts/phase27-live-hardware-bridge-evidence.sh` | **Add** | D-12 evidence wrapper |
| `scripts/phase27-live-hardware-bridge-evidence-test.sh` | **Add** | Wrapper/redaction tests |
| `scripts/BUILD.bazel` | **Modify** | Bazel targets for Phase 27 scripts |
| `justfile` | **Modify** | `phase27-evidence` recipe |
| `tools/parity/src/mining_allow.rs` | **Modify** | Phase 27 allow tier after artifacts exist (D-14) |
| `tools/parity/src/operator_evidence.rs` | **Maybe modify** | Share-outcome slot promotion rules |
| `docs/parity/evidence/phase-27-live-hardware-asic-and-stratum-bridge/**` | **Add** | Committed evidence root |
| `docs/parity/evidence/phase-25-live-stratum-runtime-and-safe-stop/share-outcome.md` | **Update after proof** | Promote only with Phase 27 hardware artifact (D-14) |
| `docs/parity/checklist.md` | **Conservative update** | STR-08/09, ASIC-10/11 exact tier only (D-17) |
| `.planning/phases/27-live-hardware-asic-and-stratum-bridge/*-PLAN.md` | **Add** | Planner output |

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Diagnostic `SendDiagnosticWork` for mining | `Bm1366ProductionCommand::SendProductionWork` | Phase 24 | Live path must use production command only (D-05) |
| Controlled fixture transcript | Live TCP socket runtime | Phase 25 | Phase 27 bridges socket + ASIC |
| Share outcome blocked | Detector-gated accepted/rejected/blocked | Phase 27 target | Closes STR-09 gap |
| ASIC UART boot-only | Retained production executor | Phase 27 required | Enables hardware dispatch |

**Deprecated/outdated for this phase:**
- Treating `adapter_action_count` as hardware dispatch proof—implementation-only marker. [VERIFIED: Phase 24 verification notes]

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Chip-detect or diagnostic boot is sufficient ASIC state for first live production work dispatch | Pitfall 5 | Hardware timeouts; need `Bm1366InitPlan::full_init` in Phase 27 or pre-phase |
| A2 | Retained UART can coexist with boot gate without re-init ESP-IDF driver | Pitfall 1 | May need single init path owning driver for app lifetime |
| A3 | `LIVE_SOCKET_PUMP_ITERATIONS` budget can be raised or made conditional for Phase 27 hardware evidence | Pitfall 4 | Hardware runs stop before pool response |
| A4 | Real pool may return rejected shares more often than accepted on first live submit | STR-09 | Evidence may record `rejected` or `blocked_safe_prerequisite`—still valid per requirements |

## Open Questions

1. **Production ASIC initialization scope for Phase 27**
   - What we know: Boot gate supports chip-detect and work-result diagnostic only; `full_init` exists in pure `init_plan` but is not wired to firmware production mode. [VERIFIED: adapter_gate.rs, init_plan.rs]
   - What's unclear: Whether v1.1 Phase 27 hardware evidence requires `Bm1366InitPlan::full_init` or chip-detect + production dispatch is enough for first share proof.
   - Recommendation: Plan fail-closed `AsicInitFailed` when init boundary not met; attempt minimal production init only behind Phase 27 mode/ack and Phase 22 gate.

2. **Firmware host test execution in Bazel**
   - What we know: `live_stratum_runtime.rs` has extensive `#[cfg(test)]` modules but no `rust_test` Bazel target for firmware crate. [VERIFIED: firmware/bitaxe/BUILD.bazel]
   - What's unclear: Whether Wave 0 adds `//firmware/bitaxe:host_tests` or relies on `cargo test -p bitaxe-firmware` in evidence scripts.
   - Recommendation: Add Bazel `rust_test` host target or script-invoked `cargo test` for bridge unit tests (D-15).

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Bazelisk | `just test`, builds | ✓ | 9.1.1 | — |
| `espflash` | detect, flash-monitor, board-info | ✓ | 4.0.1 | — |
| `just` | repo commands | ✓ | 1.48.0 | direct script invoke |
| Rust host toolchain | crate tests | ✓ | 1.88.0-nightly | — |
| ESP Rust `esp` toolchain | firmware build | ✓ assumed | espup-managed | `just bootstrap-esp` |
| Ultra 205 USB | hardware evidence | ✗ not probed | — | blocked-mode evidence + explicit non-claims |
| Local pool credentials | live share proof | ✗ not read | — | `blocked_safe_prerequisite` artifact |

**Missing dependencies with no fallback:**
- Connected Ultra 205 with exactly one ESP port for detector-gated hardware share-outcome proof (D-16).

**Missing dependencies with fallback:**
- Hardware: record `share_outcome: blocked_safe_prerequisite` with detector/board-info blocker (Phase 25 pattern).

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Bazel `rules_rust` 0.70.0 + `#[cfg(test)]` firmware modules |
| Config file | `crates/bitaxe-stratum/BUILD.bazel`, `scripts/BUILD.bazel` |
| Quick run command | `bazel test //crates/bitaxe-stratum:tests --test_output=errors` |
| Full suite command | `bazel test //...` and `bazel test //scripts:phase27_live_hardware_bridge_evidence_test` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| STR-08 | Live socket lifecycle with ASIC bridge steps | unit + firmware host | `bazel test //crates/bitaxe-stratum:tests --test_output=errors` | ✅ |
| STR-08 | Detector-gated hardware socket+ASIC | hardware smoke | `just detect-ultra205` then Phase 27 evidence `--mode hardware` | ❌ Wave 0 script |
| STR-09 | Submit classification tied to intent | unit | `bazel test //crates/bitaxe-stratum:tests --test_filter=submit` | ✅ |
| STR-09 | Share-outcome evidence slot | workflow | `bazel test //scripts:phase27_live_hardware_bridge_evidence_test` | ❌ Wave 0 |
| ASIC-10 | Notify→dispatch production command | unit + firmware host | extend `live_stratum_runtime` tests | ❌ Wave 0 |
| ASIC-11 | Observation stamping + correlation gate | unit | `production_work.rs` + new bridge tests | ✅ partial |
| D-10 | Phase 27 mode/ack fail-closed | unit | `mining_evidence_mode` tests | ❌ Wave 0 |
| D-12 | Blocked evidence path | workflow | phase27 evidence test `--mode blocked` | ❌ Wave 0 |

### Sampling Rate

- **Per task commit:** `bazel test //crates/bitaxe-stratum:tests --test_output=errors`
- **Per wave merge:** `bazel test //...` (scoped to touched targets if firmware build slow)
- **Phase gate:** Phase 27 evidence test green + verifier before `/gsd-verify-work`

### Wave 0 Gaps

- [ ] `scripts/phase27-live-hardware-bridge-evidence.sh` — hardware/blocked evidence wrapper
- [ ] `scripts/phase27-live-hardware-bridge-evidence-test.sh` — redaction and blocked-mode tests
- [ ] `scripts/BUILD.bazel` — `phase27_live_hardware_bridge_evidence` targets
- [ ] `firmware/bitaxe` host `rust_test` or documented `cargo test -p bitaxe-firmware` for `live_stratum_runtime` bridge tests
- [ ] `mining_evidence_mode.rs` Phase 27 mode tests
- [ ] `tools/parity/src/mining_allow.rs` Phase 27 tier (after wrapper exists)

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|------------------|
| V2 Authentication | no | N/A — pool authorize uses runtime-only credentials |
| V3 Session Management | partial | `PoolSessionGeneration` invalidation on reconnect/clean-jobs |
| V4 Access Control | yes | Phase 22 prerequisite gate before dispatch/connect |
| V5 Input Validation | yes | `parse_server_message`, `parse_bm1366_result_frame`, JSON line bounds |
| V6 Cryptography | no | Stratum/password handling is category-label only in evidence |

### Known Threat Patterns

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Secret leakage in committed evidence | Information disclosure | Category labels only; `mining_allow` prohibited tokens (D-13) |
| Stale-work share submission | Tampering | `correlate_nonce_result` generation + active-work checks (D-08) |
| Diagnostic work smuggled as production | Elevation | Type-level `Bm1366ProductionCommand` only in live path (D-05) |
| Ungated hardware mining | Denial of service / safety | Phase 22 gate + Phase 27 mode/ack fail-closed (D-10, D-11) |

## Sources

### Primary (HIGH confidence)

- `.planning/phases/27-live-hardware-asic-and-stratum-bridge/27-CONTEXT.md` — locked decisions and scope
- `firmware/bitaxe/src/live_stratum_runtime.rs` — socket shell and integration gap
- `firmware/bitaxe/src/controlled_mining_runtime.rs` — guarded plan consumption reference
- `crates/bitaxe-stratum/src/v1/live_runtime.rs` — pure runtime and registry
- `crates/bitaxe-stratum/src/v1/mining_loop.rs` — `GuardedMiningLoopInputs::plan`
- `crates/bitaxe-asic/src/bm1366/production.rs` — production commands
- `docs/parity/evidence/phase-25-live-stratum-runtime-and-safe-stop/share-outcome.md` — current blocker state
- `AGENTS.md` — detector gate, redaction, hardware evidence

### Secondary (MEDIUM confidence)

- `reference/esp-miner/main/tasks/protocol_coordinator.c` — upstream coordinator/watchdog breadcrumb [CITED: reference tree]
- `reference/esp-miner/components/stratum/mining.c` — coinbase/job/submit breadcrumb [CITED: 27-CONTEXT.md canonical refs]

### Tertiary (LOW confidence — needs hardware validation)

- Production init sufficiency with chip-detect-only boot (Assumption A1)
- Optimal pump iteration budget for live share response (Assumption A3)

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — reuse existing workspace crates; no new dependencies
- Architecture: HIGH — clear gap between Phase 24/25 artifacts; locked bridge location
- Pitfalls: MEDIUM — UART retention and init sequencing need hardware validation

**Research date:** 2026-07-05
**Valid until:** 2026-08-05 (stable integration); 2026-07-12 for hardware timing assumptions
