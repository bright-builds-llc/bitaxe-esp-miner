# Architecture Research

**Domain:** Ultra 205 trusted production mining integration for Rust ESP-IDF Bitaxe firmware
**Researched:** 2026-07-04
**Confidence:** HIGH for project-local component boundaries and safety/evidence constraints; MEDIUM for exact ESP-IDF socket/task implementation details until the live adapter is implemented and hardware-evidenced.

## Standard Architecture

### System Overview

Trusted production mining should extend the v1.0 controlled no-share path into a real, safety-gated Stratum v1 and BM1366 runtime without moving business logic into firmware tasks. The existing split remains correct: pure crates decide protocol state, work dispatch, ASIC command intent, share submission, statistics, and safety gates; `firmware/bitaxe` performs ESP-IDF I/O, owns long-running tasks, and publishes redacted runtime/evidence snapshots.

```text
Operator settings / pool credentials in NVS
        |
        v
firmware/bitaxe imperative shell
  settings_adapter -> wifi_adapter -> stratum_socket_adapter -> asic_adapter
        |                 |                 |                 |
        v                 v                 v                 v
pure crates
  bitaxe-config     bitaxe-stratum     bitaxe-asic      bitaxe-safety
        \                 |                 |                 /
         \                v                 v                /
          +-------- mining_runtime core decisions ----------+
                              |
                              v
                  bitaxe-api snapshots and telemetry
                              |
                              v
             HTTP, WebSocket, retained logs, evidence files
```

The key architectural change is replacing `controlled_mining_runtime`'s synthetic transcript publication with a production runtime shell that feeds live socket messages and live BM1366 results through the same pure decision shapes. The current controlled path should remain as an evidence harness or test fixture path, but it must not be the production runtime source of truth.

### Component Responsibilities

| Component | Responsibility | Typical Implementation |
| --- | --- | --- |
| `firmware/bitaxe/src/production_mining_runtime.rs` | Own production runtime task lifecycle, bounded loops, channels, watchdog yields, safe stop, and state publication. | New firmware module that orchestrates adapters and pure plans; no protocol parsing or safety policy inline. |
| `firmware/bitaxe/src/stratum_socket_adapter.rs` | Resolve/connect/read/write Stratum v1 TCP lines, redact logs, classify socket failures, and expose typed events to the runtime. | New ESP-IDF/LwIP socket shell around `bitaxe_stratum::v1::messages`. |
| `crates/bitaxe-stratum/src/v1/production_runtime.rs` | Pure Stratum v1 session state machine: subscribe, authorize, difficulty, extranonce, notify, submit response, reconnect/fallback decisions, and redacted event summaries. | New pure module built from existing `messages`, `mining`, `mining_loop`, `queue`, and `state` modules. |
| `crates/bitaxe-stratum/src/v1/mining_loop.rs` | Convert gated queued work plus ASIC nonce results into BM1366 dispatch plans and share submissions. | Modify existing guarded plan to use production command names and preserve gate ordering. |
| `crates/bitaxe-stratum/src/v1/state.rs` | Track lifecycle, work gate, accepted/rejected shares, best difficulty, hashrate inputs, and activity status. | Modify with production-safe share outcome and hashrate/statistics fields only as needed. |
| `crates/bitaxe-asic/src/bm1366/init_plan.rs` | Decide when BM1366 full init is allowed and emit typed init actions. | Modify only if v1.1 needs a stronger production-ready status than `InitializedNoMining`. |
| `crates/bitaxe-asic/src/bm1366/command.rs` | Emit typed BM1366 adapter actions for init, frequency, nonce, and work commands. | Modify `SendDiagnosticWork` naming/semantics into a production work command while keeping bytes typed and non-logged. |
| `firmware/bitaxe/src/asic_adapter.rs` | Interpret typed ASIC adapter actions with UART/reset hardware and publish ASIC status. | Modify to support full init, production work dispatch, bounded result reads, and safe-stop reset handling. |
| `crates/bitaxe-safety` | Produce power, thermal, fan, voltage, and safety evidence tokens and fail-closed effect plans. | Modify minimally: add production-mining preflight decision wrappers if existing tokens are too scattered. |
| `firmware/bitaxe/src/safety_adapter.rs` | Collect live safety telemetry and interpret fail-closed effects against real peripherals or observe-only adapters. | Modify to expose a current `MiningSafetyGate` input and block work on stale/unavailable/faulted samples. |
| `firmware/bitaxe/src/runtime_snapshot.rs` | Store API-visible mining state and platform/safety snapshots. | Modify replacement API into production runtime state updates instead of evidence-only replacement. |
| `firmware/bitaxe/src/http_api.rs` and `websocket_api.rs` | Serve live production mining state, statistics, scoreboard, pause/resume, and retained logs. | Modify route handlers to consume production runtime snapshots and command channel effects. |
| Evidence scripts and parity checklist | Capture redacted commands, logs, API/WebSocket samples, share outcome, safe-stop, and exact status promotions. | Modify `docs/parity/checklist.md` rows and add v1.1 evidence tooling around existing detector/flash-monitor flows. |

## Recommended Project Structure

```text
crates/bitaxe-stratum/src/v1/
|-- production_runtime.rs      # Pure live Stratum session and share outcome planning
|-- mining_loop.rs             # Existing gate/work/result/share bridge, tightened for production
|-- messages.rs                # Existing Stratum v1 parse/serialize contract
|-- mining.rs                  # Existing notify -> BM1366 work and nonce -> submit bridge
|-- queue.rs                   # Existing bounded work queue behavior
`-- state.rs                   # Existing runtime counters, lifecycle, hashrate state

firmware/bitaxe/src/
|-- production_mining_runtime.rs # New task orchestrator and safe-stop owner
|-- stratum_socket_adapter.rs    # New TCP/DNS/read/write adapter
|-- asic_adapter.rs              # Modified full init/work/result interpreter
|-- safety_adapter.rs            # Modified live gate and effect interpreter
|-- runtime_snapshot.rs          # Modified production telemetry publisher
|-- http_api.rs                  # Modified commands/statistics/scoreboard routes
`-- websocket_api.rs             # Existing frame planner fed by updated snapshots
```

### Structure Rationale

- `bitaxe-stratum` should own the production mining state machine because it already owns Stratum messages, work construction, guarded mining loop planning, and share serialization.
- `bitaxe-asic` should continue emitting semantic BM1366 actions, not performing UART or GPIO work. Rename or supplement `SendDiagnosticWork` so production code does not carry diagnostic semantics.
- `firmware/bitaxe` should own task lifetimes, blocking I/O, timing, sleeps, watchdog yields, and safe-stop effects because those are ESP-IDF and hardware concerns.
- `runtime_snapshot` should remain the API projection point, but v1.1 should replace the evidence-only setter with a production update path that can be fed repeatedly by runtime events.

## Architectural Patterns

### Pattern 1: Pure Runtime Plan, Effectful Interpreter

**What:** Represent each live runtime step as a pure decision from typed inputs to typed effects. The firmware task interprets those effects through sockets, UART, GPIO, NVS, logs, and snapshots.

**When to use:** Subscribe/authorize sequencing, notify ingestion, clean-jobs handling, queued work dispatch, nonce-result handling, share submission, pause/resume, reconnect, and safe stop.

**Trade-offs:** More event and command types up front, but it keeps protocol correctness testable without a pool, ASIC, Wi-Fi, or ESP-IDF runtime.

```rust
pub enum ProductionMiningEffect {
    SendStratum(StratumV1ClientMessage),
    DispatchBm1366(Bm1366Command),
    SubmitShare(ShareSubmission),
    PublishState(MiningRuntimeState),
    SafeStop { reason: &'static str },
}
```

### Pattern 2: Gate Tokens Before Hardware Effects

**What:** Production work dispatch should require an explicit gate assembled from current ASIC init state, fresh power token, fresh thermal token, normal safety status, and hardware evidence acknowledgement.

**When to use:** Full BM1366 init, voltage/frequency transition, work dispatch, result handling loops, and restart after fault.

**Trade-offs:** Conservative gates can block early bring-up until telemetry is wired. That is appropriate for v1.1: the milestone requires trusted mining, not bypassed mining.

### Pattern 3: Redacted Evidence Events

**What:** Runtime emits structured, secret-free events that evidence scripts and API/WebSocket projections can consume. Event fields should record categories and booleans, not raw pool URLs, users, passwords, workers, Wi-Fi values, IPs, tokens, raw BM1366 frames, or NVS secret values.

**When to use:** Every production Stratum lifecycle marker, pool connection event, authorization attempt, share submission response, safe-stop marker, and evidence summary.

**Trade-offs:** Operators lose some direct debugging detail in committed evidence. Local uncommitted diagnostics can remain richer only when explicitly kept out of logs/evidence promotion.

## Data Flow

### Production Mining Flow

```text
settings_adapter::current_settings_snapshot
  -> parse pool settings into redacted ProductionPoolConfig
  -> stratum_socket_adapter connects to configured pool
  -> bitaxe-stratum plans subscribe/authorize messages
  -> stratum_socket_adapter writes JSON lines
  -> stratum_socket_adapter reads JSON lines
  -> bitaxe-stratum parses response/difficulty/extranonce/notify
  -> bitaxe-stratum builds MiningWork and updates MiningWorkQueue
  -> safety_adapter + asic_adapter status build MiningLoopGate
  -> bitaxe-stratum guarded plan emits typed BM1366 work command
  -> asic_adapter writes typed frame without logging raw bytes
  -> asic_adapter reads bounded BM1366 result frame
  -> bitaxe-asic parses nonce result
  -> bitaxe-stratum converts result to ShareSubmission
  -> stratum_socket_adapter sends mining.submit
  -> bitaxe-stratum records accepted/rejected/no-share outcome
  -> runtime_snapshot publishes state for HTTP/WebSocket/statistics
  -> evidence tooling records redacted lifecycle and safe-stop proof
```

### Safety Gate Flow

```text
Power sample from INA260 adapter
  -> bitaxe-safety PowerObservation
  -> optional PowerEvidenceToken only when fresh and safe

Thermal sample from EMC2101/thermal adapter
  -> bitaxe-safety ThermalObservation
  -> optional ThermalEvidenceToken only when fresh and below threshold

Safety status + evidence marker
  -> MiningLoopGate
  -> Ready only if power, thermal, safety evidence, ASIC initialized, and ack are present
  -> Blocked otherwise with stable reason
```

Gate ordering should stay fail-closed and predictable: power evidence, thermal evidence, safety status/evidence, hardware evidence acknowledgement, ASIC initialized. This matches the existing `MiningLoopGate` shape and avoids running work dispatch when a more basic prerequisite is missing.

### Task Boundaries

| Task/Loop | Owns | Must Delegate |
| --- | --- | --- |
| Production mining coordinator | Runtime lifecycle, channel select/poll order, watchdog yields, safe stop, snapshot publication. | Stratum parsing/serialization, job construction, safety decisions, BM1366 command construction. |
| Stratum socket loop | TCP connect/read/write, line framing, timeouts, reconnect sleep, local-only error classification. | Protocol state, share outcome classification, credential rendering. |
| ASIC UART loop | UART baud/write/read, reset GPIO, bounded result wait, adapter setup errors. | Init/work command planning, result parsing, share submission planning. |
| Safety supervisor | Sensor sampling cadence, effect interpretation, latest telemetry publication. | Fault classification, PID/fan/voltage decisions, mining gate policy. |
| HTTP/WebSocket server | Request transport, access/origin gate, cadence frames, pause/resume command ingress. | Mining command effects, telemetry DTO mapping, statistics/scoreboard semantics. |

### Evidence Flow

```text
repo command with board=205 and detector output
  -> package manifest and source/reference commits
  -> flash-monitor or monitor log capture
  -> production runtime redacted markers
  -> explicit API/WebSocket/statistics samples
  -> accepted_or_rejected_share observed OR milestone blocker recorded
  -> safe-stop marker after bounded run
  -> redaction review
  -> parity checklist exact status update
```

Evidence should promote only exact claims. A connected pool plus notify is not a share outcome. A typed work dispatch is not accepted/rejected production mining. An accepted or rejected share response should be recorded as the first v1.1 production-share claim; if no such outcome can be safely achieved, the milestone should preserve that as an explicit blocker rather than inflating no-share evidence.

## New Components

| Component | Why New | Inputs | Outputs |
| --- | --- | --- | --- |
| `crates/bitaxe-stratum/src/v1/production_runtime.rs` | Controlled runtime currently consumes synthetic transcripts; production needs a live session state machine. | Parsed server messages, current queue/state, gate decisions, nonce results, submit responses. | Client messages, queue updates, dispatch plans, share outcomes, redacted evidence events. |
| `firmware/bitaxe/src/production_mining_runtime.rs` | Current `maybe_start_after_asic_gate` only publishes bounded evidence markers. Production needs a long-running task. | Settings snapshot, safety snapshots, ASIC status/results, socket events, command effects. | Runtime state updates, adapter calls, retained logs, watchdog markers, safe-stop. |
| `firmware/bitaxe/src/stratum_socket_adapter.rs` | Real Stratum v1 socket I/O is absent; Wi-Fi only brings up network services. | Redacted pool config and planned client messages. | Parsed server messages, submit responses, socket status, reconnect events. |
| `firmware/bitaxe/src/mining_runtime_channel.rs` or local channel types | Runtime, HTTP commands, safety supervisor, and ASIC/socket adapters need bounded communication without shared ad hoc globals. | Command and event enums. | Bounded queues or latest-state cells with stable drop/block policy. |
| Evidence helper for v1.1 live mining | Existing evidence scripts prove controlled no-share; v1.1 needs share outcome and redaction gates. | Detector, flash-monitor logs, API/WebSocket probes, optional local pool credentials. | Redacted evidence summary, share outcome classification, checklist-ready claims. |

## Modified Components

| Component | Required Change | Risk Boundary |
| --- | --- | --- |
| `firmware/bitaxe/src/main.rs` | Start production runtime only after settings snapshot, ASIC boot gate, safety supervisor, Wi-Fi, and HTTP services are ordered deliberately. | Do not start production mining by default without explicit compile/runtime gate and safety prerequisites. |
| `firmware/bitaxe/src/controlled_mining_runtime.rs` | Keep as controlled evidence harness or replace with production runtime behind a clearer mode enum. | Do not let synthetic transcripts feed production claims. |
| `firmware/bitaxe/src/asic_adapter.rs` | Add full init interpretation, production work dispatch, result read loop, and safe-stop reset/effect handling. | Never log raw frames; fail closed on setup, write, read, parse, or timeout errors. |
| `crates/bitaxe-asic/src/bm1366/command.rs` | Split `SendDiagnosticWork` from production `SendWork` or rename with migration tests. | Avoid production code implying diagnostic-only semantics; keep bytes opaque in logs. |
| `crates/bitaxe-asic/src/bm1366/init_plan.rs` | Define a production-ready init status if `InitializedNoMining` is insufficient for work dispatch gates. | Full init still requires power, thermal, safety, and hardware evidence. |
| `crates/bitaxe-stratum/src/v1/mining_loop.rs` | Reuse the existing gate but support live notify source, clean-jobs, repeated dispatch, and nonce-to-share cycles. | Do not bypass `MiningLoopGate` for faster share attempts. |
| `crates/bitaxe-stratum/src/v1/state.rs` | Extend counters/hashrate only where API/statistics require it. | Keep secret-bearing pool identity out of state that can be serialized or logged. |
| `firmware/bitaxe/src/runtime_snapshot.rs` | Convert evidence-only `replace_mining_runtime_state_for_evidence` into a production-safe update path. | Avoid holding locks across I/O or publishing partially-updated snapshots. |
| `firmware/bitaxe/src/http_api.rs` | Wire pause/resume to runtime command channel; return production statistics and scoreboard. | Keep access gate behavior and avoid exposing pool credentials. |
| `docs/parity/checklist.md` | Promote exact v1.1 rows only after evidence: live socket, BM1366 production dispatch/result, share outcome, safety gate, telemetry, safe stop. | Do not mark full active voltage/fan/thermal parity verified if only mining prerequisite safety is proven. |

## Integration Points

### External Services

| Service | Integration Pattern | Notes |
| --- | --- | --- |
| Stratum v1 pool | Firmware socket adapter exchanges newline-delimited JSON planned/parsed by `bitaxe-stratum`. | Pool URL, port, username, password, worker, endpoints, and responses must be redacted in committed logs/evidence. |
| ESP-IDF Wi-Fi/LwIP | Existing Wi-Fi adapter brings up STA; new Stratum adapter should depend on network-ready status. | Do not derive `DEVICE_URL` or pool target from scans or stale state; use settings and current session only. |
| BM1366 ASIC over UART1 | Existing ASIC adapter interprets typed actions; production adds repeated work/result operations. | UART read timeouts and invalid frames must safe-stop or remain blocked, not spin indefinitely. |
| INA260/thermal/fan/voltage peripherals | Safety adapter turns readings into evidence tokens and effect plans. | v1.1 needs mining prerequisite safety only; full active safety closure remains a non-claim unless separately evidenced. |

### Internal Boundaries

| Boundary | Communication | Notes |
| --- | --- | --- |
| `production_mining_runtime` to `bitaxe-stratum` | Pure function calls with typed events and state. | Runtime owns loop cadence; crate owns decisions. |
| `production_mining_runtime` to `stratum_socket_adapter` | Planned messages out, parsed messages/status events in. | Adapter must not log credentials or raw authorize lines. |
| `production_mining_runtime` to `asic_adapter` | Typed BM1366 adapter actions and parsed nonce results. | Adapter interprets hardware effects only after gate readiness. |
| `safety_adapter` to runtime | Latest gate snapshot or event channel. | Stale/unavailable safety telemetry should block work submission. |
| `http_api` to runtime | Command channel for pause/resume and snapshot reads for state. | API command response can be immediate; runtime effect is asynchronous but observable. |
| `runtime_snapshot` to `bitaxe-api` | Snapshot DTO mapping. | Keep API compatibility while adding production mining counters. |

## Scaling Considerations

| Scale | Architecture Adjustments |
| --- | --- |
| Single Ultra 205 | One production runtime task, one Stratum socket, one ASIC UART loop, shared latest-state snapshots are enough. |
| Longer soak on one device | Add bounded queue metrics, reconnect backoff state, watchdog checkpoints, and memory/heap telemetry in snapshots. |
| Multiple boards/ASICs later | Generalize production runtime over board/ASIC capabilities only after Ultra 205 evidence is complete. Do not abstract v1.1 prematurely. |

### Scaling Priorities

1. **First bottleneck:** Blocking socket or UART operations can starve telemetry/watchdog. Use bounded timeouts and explicit yield checkpoints before optimizing throughput.
2. **Second bottleneck:** Snapshot locking can contend with WebSocket cadence and runtime updates. Keep snapshots small, cloneable, and updated outside I/O-critical sections.
3. **Third bottleneck:** Evidence logs can leak or grow too much during soak. Emit compact structured markers and let scripts summarize.

## Anti-Patterns

### Anti-Pattern 1: Production Logic In Firmware Loops

**What people do:** Parse pool messages, decide share outcomes, update counters, and build BM1366 frames directly inside `production_mining_runtime.rs`.
**Why it's wrong:** It buries business logic in ESP-IDF I/O code and makes hardware-free tests weak.
**Do this instead:** Keep runtime loops thin and move decisions into `bitaxe-stratum`, `bitaxe-asic`, `bitaxe-safety`, and API mappers.

### Anti-Pattern 2: Reusing Synthetic Evidence As Production Truth

**What people do:** Extend `controlled_mining_runtime` markers until they look like production mining.
**Why it's wrong:** v1.0 controlled no-share evidence explicitly does not prove accepted/rejected production share behavior.
**Do this instead:** Build a live Stratum socket adapter and record actual pool responses, or record a milestone blocker if a safe share outcome cannot be observed.

### Anti-Pattern 3: Secret-Bearing Logs

**What people do:** Log raw Stratum authorize lines, pool URLs, worker names, endpoint addresses, NVS keys/values, or raw request bodies for debugging.
**Why it's wrong:** Evidence is intended for commit/review and must not expose credentials or private endpoints.
**Do this instead:** Log redacted lifecycle markers such as `stratum_authorize_status=sent redacted=true` and keep local secret inputs outside committed artifacts.

### Anti-Pattern 4: Safety Gate Bypass For Bring-Up

**What people do:** Add a temporary "force mining" flag that skips power, thermal, ASIC init, or evidence acknowledgement gates.
**Why it's wrong:** It creates the exact unsafe path v1.1 is supposed to avoid and can accidentally become release behavior.
**Do this instead:** Add explicit, documented evidence modes that still publish blocked reasons and require detector-gated hardware procedures.

## Suggested Build Order

| Order | Build Item | Why This Order | Primary Evidence |
| --- | --- | --- | --- |
| 1 | Define pure production runtime contract in `bitaxe-stratum`. | Establish state/effect vocabulary before firmware tasks exist. | Unit tests for subscribe/authorize/notify/share accepted/rejected/no-share transitions. |
| 2 | Split production BM1366 work command semantics from diagnostic work. | Prevent production runtime from depending on diagnostic-only naming and evidence. | Unit/golden command tests, no raw frame logs. |
| 3 | Add mining prerequisite safety gate aggregator. | Work dispatch and full init need one stable gate input. | Unit tests for fresh/stale/missing/faulted power and thermal cases. |
| 4 | Extend ASIC adapter for full init, work dispatch, result reads, and safe stop. | Production runtime needs a hardware interpreter before live pool work can be trusted. | Detector-gated hardware smoke for init blocked/ready paths and bounded result behavior. |
| 5 | Add Stratum socket adapter with redaction tests. | Real pool I/O should be isolated before long-running mining orchestration. | Host tests for rendering/redaction; firmware smoke for connect failure without secrets. |
| 6 | Add production mining runtime task behind explicit v1.1 gate. | Coordinator can now compose typed socket, ASIC, safety, and snapshot boundaries. | Controlled host tests plus hardware smoke showing blocked until gates ready. |
| 7 | Wire runtime snapshots, statistics, scoreboard, API, and WebSocket. | User-visible telemetry should reflect runtime state before share evidence runs. | API/WebSocket samples correlated with runtime markers. |
| 8 | Run bounded live mining evidence with local pool credentials. | Only after gates, redaction, snapshots, and safe-stop are in place. | Accepted/rejected share outcome or explicit safe blocker; safe-stop proof; redaction pass. |
| 9 | Update parity checklist with exact claims. | Status changes should follow evidence, not implementation completion. | `just parity` and checklist diffs tied to evidence paths. |

## Sources

- `.planning/PROJECT.md` - v1.1 goal, active requirements, constraints, and out-of-scope boundaries. Confidence: HIGH.
- `standards/core/architecture.md` - functional core / imperative shell and parse-at-boundaries rules. Confidence: HIGH.
- `docs/parity/checklist.md` - current v1.0 evidence state and non-claims for production share outcomes, active safety, and BM1366 full production mining. Confidence: HIGH.
- `firmware/bitaxe/src/main.rs` - current boot order and controlled runtime startup point. Confidence: HIGH.
- `firmware/bitaxe/src/controlled_mining_runtime.rs` - current compile-gated synthetic/no-share runtime harness and redaction markers. Confidence: HIGH.
- `firmware/bitaxe/src/asic_adapter.rs` - current BM1366 UART/reset interpreter and diagnostic work/result boundaries. Confidence: HIGH.
- `firmware/bitaxe/src/runtime_snapshot.rs`, `http_api.rs`, and `websocket_api.rs` - current API/WebSocket snapshot projection and command surfaces. Confidence: HIGH.
- `firmware/bitaxe/src/wifi_adapter.rs`, `network_stack.rs`, and `settings_adapter.rs` - current Wi-Fi, network initialization, and NVS snapshot boundaries. Confidence: HIGH.
- `crates/bitaxe-stratum/src/v1/mining_loop.rs`, `controlled_runtime.rs`, `messages.rs`, `mining.rs`, and `state.rs` - current pure Stratum, work queue, share, and mining state contracts. Confidence: HIGH.
- `crates/bitaxe-asic/src/bm1366/init_plan.rs` and `command.rs` - current pure BM1366 init and adapter action contracts. Confidence: HIGH.
- `crates/bitaxe-safety/src/power.rs` and `thermal.rs` - current power/thermal evidence token and fail-closed decision contracts. Confidence: HIGH.

*Architecture research for: Ultra 205 trusted production mining*
*Researched: 2026-07-04*
