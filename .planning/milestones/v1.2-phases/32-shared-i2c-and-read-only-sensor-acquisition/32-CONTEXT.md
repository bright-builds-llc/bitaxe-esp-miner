---
generated_by: gsd-discuss-phase
lifecycle_mode: yolo
phase_lifecycle_id: 32-2026-07-13T23-12-34
generated_at: 2026-07-13T23:12:34.029Z
---

# Phase 32: Shared I2C and Read-Only Sensor Acquisition - Context

**Gathered:** 2026-07-13
**Status:** Ready for planning
**Mode:** Yolo

<domain>
## Phase Boundary

Phase 32 establishes one firmware-owned, serialized, long-lived I2C0 lifecycle for the Ultra 205. The lifecycle preserves the existing startup display, then performs bounded read-only INA260 and EMC2101 acquisition whose successful results feed the Phase 31 observation contract with producer-owned stamps. Power facts publish atomically, temperature and tachometer remain independently truthful, and a failed sensor cannot block the API or unaffected observations.

This phase does not add runtime display behavior, persist settings, compose the Phase 34 coherent operator snapshot, admit Phase 35 evidence, write fan or configuration registers, change voltage, reset or sequence power, control the ASIC, inject faults, run self-tests, mine, consume credentials, access direct UART or pins, perform OTA, target another board, or reopen archived Phase 28.1.1 work.

</domain>

<decisions>
## Implementation Decisions

### Shared I2C0 lifecycle

- **D-01:** Construct exactly one bounded I2C0 bus for the normal Ultra 205 runtime. Borrow it for the existing startup display operation, then move the same bus into one long-lived sensor-producer task; do not create a second driver or expose a shared `Arc<Mutex<_>>` bus to arbitrary callers.
- **D-02:** Preserve the existing startup display as a bounded best-effort operation before periodic sensor acquisition begins. A display failure records its own explicit outcome but must return ownership intact and must not prevent the sensor producer from starting.
- **D-03:** Route display and sensor access through a bounded bus facade with real duration-to-ESP-tick conversion. Do not let embedded-hal defaults silently use an unbounded `BLOCK` timeout.
- **D-04:** Keep serialization structural: only the startup borrow and then the sole producer task may issue I2C0 transactions. API, WebSocket, retained-log, evidence, settings, and request paths may only read cloned stored observations.

### Producer cadence and failure isolation

- **D-05:** Start with one 500 ms producer sweep, matching the established live telemetry cadence. Each transaction has a finite deadline, the aggregate normal sweep must fit within its period, and the producer must avoid blocking retry loops or drift-amplifying catch-up bursts.
- **D-06:** Keep cadence and observation transitions in a pure reducer where practical; the firmware task owns the driver, monotonic clock, boot/session scope, source-local sequences, and publication into the shared observation store.
- **D-07:** A successful source acquisition alone may advance that source's sequence and monotonic acquisition time. Failed or invalid attempts preserve the prior last-good stamp and must never mint freshness.
- **D-08:** Attempted read failures become typed faults immediately, invalid samples become typed validation faults, and producer-owned elapsed-time processing may transition retained last-good data to stale when cadence expires. Query traffic never computes or refreshes those states.
- **D-09:** Continue the sweep after an individual sensor or fact fails, then publish one complete updated store snapshot. One failed sensor must not suppress later transactions, block the API, or erase unaffected observations.
- **D-10:** Use one-shot per-sweep attempts in Phase 32. Exact transaction deadlines, stale threshold, task name/stack size/priority, and overrun logging remain implementation discretion, provided they are bounded, testable, and do not alter the fixed ownership or truth rules.

### Read-only sensor admission

- **D-11:** Treat the INA260 current, bus-voltage, and power registers as one logical acquisition. Admit and publish the three facts atomically only after every required read succeeds and every decoded value passes validation; otherwise preserve the prior acquisition stamp and fault the power acquisition without partial freshness.
- **D-12:** Read EMC2101 external temperature and tachometer through separate read-only acquisition steps and reduce them independently. A missing, invalid, or failed tachometer must not invalidate a fresh temperature, and a temperature failure must not erase an independently valid tachometer.
- **D-13:** Use explicit typed decoding and plausibility checks before constructing Phase 31 `Fresh` observations. Reject impossible, sentinel, divide-by-zero, non-finite, overflow, and incomplete values with stable redaction-safe reasons.
- **D-14:** Enforce a closed read-only register allowlist for this producer. Do not call the existing EMC2101 initialization or fan-duty paths, DS4432U voltage paths, or any generic register-write surface from the Phase 32 runtime.
- **D-15:** Preserve independent source-local stamp semantics: one shared stamp for the atomic INA260 acquisition, a temperature-owned stamp, and a tachometer-owned stamp. Do not reuse one fact's stamp for another and do not introduce Phase 34's global operator-snapshot revision here.

### the agent's Discretion

- Exact module, task, command, and error type names.
- Exact bounded I2C timeout, stale threshold, and overrun threshold after research proves they fit the 500 ms sweep on the pinned ESP-IDF stack.
- Whether the startup display facade adapts the existing display helper or introduces a narrow transaction trait, provided it uses the same bus lifecycle and bounded calls.
- Exact validation ranges and raw-to-engineering-unit helpers, provided they are justified from the pinned device semantics, pure-testable, and do not transform invalid data into fresh compatibility values.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### v1.2 scope and observation contract

- `.planning/ROADMAP.md` — Phase 32 goal, dependency, success criteria, and separation from Phases 33-35.
- `.planning/PROJECT.md` — operator-ready observation-first goal and excluded active effects.
- `.planning/REQUIREMENTS.md` — OBS-02 through OBS-05 and explicit future/out-of-scope capabilities.
- `.planning/STATE.md` — Phase 31 decisions that power facts share one acquisition while temperature and tachometer remain independent.
- `.planning/phases/31-operator-claim-and-telemetry-contract/31-CONTEXT.md` — locked observation truth, producer-owned freshness, compatibility projection, and claim-admission semantics.

### Firmware and upstream behavioral evidence

- `firmware/bitaxe/src/main.rs` — current startup display versus Phase 27 I2C0 ownership branch.
- `firmware/bitaxe/src/display_adapter.rs` — current startup-only display initialization and rendering path.
- `firmware/bitaxe/src/safety_adapter/i2c_bus.rs` — current I2C0 driver, read/write primitives, and timeout handling.
- `firmware/bitaxe/src/safety_adapter/ina260.rs` — existing INA260 register reads and engineering-unit conversion.
- `firmware/bitaxe/src/safety_adapter/emc2101.rs` — existing thermal/tachometer reads plus prohibited initialization and fan-control writes that must remain unreachable.
- `reference/esp-miner/main/i2c_bitaxe.c` — upstream shared-bus lifecycle breadcrumb.
- `reference/esp-miner/main/power/INA260.c` — upstream INA260 register and conversion evidence.
- `reference/esp-miner/main/thermal/EMC2101.c` — upstream EMC2101 temperature and tachometer evidence; control writes are behavioral reference only and excluded from v1.2.

### Repository constraints

- `AGENTS.md` — ESP-IDF tooling, hardware authorization, archived-lineage prohibition, evidence redaction, and no-direct-UART/pin constraints.
- `AGENTS.bright-builds.md` — required workflow, architecture, testing, and verification rules.
- `standards/core/architecture.md` — functional core, imperative shell, and illegal-states-unrepresentable rules.
- `standards/core/code-shape.md` — guard clauses, naming, and local readability rules.
- `standards/core/testing.md` — behavior-focused Arrange/Act/Assert expectations.
- `standards/core/verification.md` — sync and clean verification requirements.
- `standards/languages/rust.md` — Rust modeling, error handling, testing, and pre-commit gates.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- `firmware/bitaxe/src/safety_adapter/i2c_bus.rs` already centralizes the ESP-IDF I2C0 driver and register transactions, but its timeout must be expressed in real tick units and its write capability must be narrowed for the runtime producer.
- `firmware/bitaxe/src/display_adapter.rs` already preserves the visible startup debug text and is the natural borrower of the bounded shared bus.
- `firmware/bitaxe/src/safety_adapter/ina260.rs` and `emc2101.rs` already contain raw read and conversion breadcrumbs that can be separated from active initialization/control functions.
- `crates/bitaxe-safety/src/observation.rs`, `power.rs`, and `thermal.rs` provide the Phase 31 truth/stamp/reducer types for successful, stale, unavailable, and faulted observations.
- `firmware/bitaxe/src/runtime_snapshot.rs` and the API snapshot/store paths already provide the consumer side that must remain clone-only and request-independent.

### Established Patterns

- Pure Rust crates own validation and state transitions; ESP-IDF and FreeRTOS code remains a thin imperative shell.
- Current normal startup lets either the display or the legacy Phase 27 bring-up path consume `peripherals.i2c0`; Phase 32 must replace that split with one normal-runtime lifecycle without reviving diagnostic or mining behavior.
- Phase 31 made compatibility numeric fields projections rather than freshness proof and requires producer-owned stamps for every real observation.

### Integration Points

- Introduce a sole normal-runtime I2C0 owner in the firmware startup path and adapt startup display rendering to borrow it.
- Extract/read-only sensor commands and pure decoding/reducer behavior from the existing safety adapters without invoking active initialization or control surfaces.
- Publish producer-owned power, temperature, and tachometer observations into the existing shared runtime snapshot/store consumed by API projections.
- Add build-time and test-time guards proving the normal Phase 32 producer cannot reach prohibited write helpers.

</code_context>

<specifics>
## Specific Ideas

- Preferred lifecycle: `construct bounded bus → render startup display with a temporary borrow → move bus into one 500 ms producer task`.
- Preferred sweep: atomic INA260 acquisition followed by independent EMC2101 temperature and tachometer attempts, with the sweep continuing after each failure.
- Preserve display observability and sensor availability as separate facts; neither is allowed to authenticate the other.
- Keep all hardware use read-only and detector-gated if a later plan requires real-device verification. Phase 35 remains the only owner of final correlated evidence admission and parity promotion.

</specifics>

<deferred>
## Deferred Ideas

- Runtime display refresh or additional display/UI behavior belongs in a future phase.
- Confirmed hostname persistence belongs to Phase 33.
- Global coherent operator-snapshot revision, provenance, and passive health composition belong to Phase 34.
- Correlated hardware evidence and exact parity promotion belong to Phase 35.
- Active fan/configuration-register, voltage, reset, power, ASIC, fault-stimulus, self-test, mining, credential, direct-UART/pin, OTA, other-board, and archived-lineage work remain excluded.

</deferred>

***

*Phase: 32-shared-i2c-and-read-only-sensor-acquisition*
*Context gathered: 2026-07-13*
