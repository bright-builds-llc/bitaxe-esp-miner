# Phase 4: Stratum V1 And First Mining Loop - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md - this log preserves the alternatives considered.

**Date:** 2026-06-27T13:17:33.403Z
**Phase:** 4-Stratum V1 And First Mining Loop
**Mode:** Yolo
**Areas discussed:** Stratum V1 Protocol Surface, Fake Pool And Deterministic Coverage, Mining Job And Work Queue Integration, Firmware First Mining Loop, Counters Runtime State And Later API Surfaces

---

## Stratum V1 Protocol Surface

| Option | Description | Selected |
| --- | --- | --- |
| Pure Stratum v1 core first | Parse and serialize Stratum v1 messages in `crates/bitaxe-stratum`, with firmware sockets as adapters later. | yes |
| Firmware socket logic first | Build directly around ESP-IDF networking and extract pure logic later. | |
| Stratum v1 and v2 together | Expand protocol scope to both versions in this phase. | |

**User's choice:** Auto-selected pure Stratum v1 core first.
**Notes:** This matches the phase scope, prior functional-core decisions, and the deferred status of Stratum v2.

---

## Fake Pool And Deterministic Coverage

| Option | Description | Selected |
| --- | --- | --- |
| Deterministic fake-pool harness | Cover subscribe, authorize, notify, difficulty, submit, errors, reconnect, fallback, and clean-jobs behavior without requiring a live pool. | yes |
| Live pool smoke first | Prove behavior primarily through public or controlled pool interaction. | |
| Parser-only fixtures | Cover JSON parsing but defer lifecycle and submit behavior. | |

**User's choice:** Auto-selected deterministic fake-pool harness.
**Notes:** Fake-pool evidence can verify protocol and state logic while keeping live mining proof gated on hardware evidence.

---

## Mining Job And Work Queue Integration

| Option | Description | Selected |
| --- | --- | --- |
| Typed job and queue model | Convert notify/extranonce/difficulty into typed BM1366 work fields and valid-job tracking, with host-testable queue behavior. | yes |
| Raw ASIC frame construction in Stratum | Have Stratum code emit raw BM1366 frames. | |
| Leave queue behavior to firmware only | Rely on FreeRTOS queue behavior without a pure model. | |

**User's choice:** Auto-selected typed job and queue model.
**Notes:** This preserves Phase 3's raw-frame boundary and makes clean-jobs, stale-job rejection, and share submission decisions testable.

---

## Firmware First Mining Loop

| Option | Description | Selected |
| --- | --- | --- |
| Thin firmware shell with safety gates | Firmware owns sockets/tasks/logging and calls pure Stratum/ASIC cores only after explicit preflight and hardware-evidence gates. | yes |
| Enable work submission once protocol tests pass | Let pure protocol evidence unlock live mining. | |
| Keep firmware mining fully deferred | Implement only host tests in Phase 4. | |

**User's choice:** Auto-selected thin firmware shell with safety gates.
**Notes:** This is the only option consistent with Phase 3 fail-closed behavior and ADR-0012 hardware evidence requirements.

---

## Counters Runtime State And Later API Surfaces

| Option | Description | Selected |
| --- | --- | --- |
| Shared typed runtime state | Model share counters, pool difficulty, lifecycle status, fallback status, and mining paused/safe-blocked status for Phase 5 reuse. | yes |
| Firmware-local counters only | Keep counters private to the first mining loop until API work. | |
| Implement API handlers now | Expand into Phase 5 HTTP/WebSocket behavior. | |

**User's choice:** Auto-selected shared typed runtime state.
**Notes:** Phase 4 should expose reusable state but not implement the AxeOS API/WebSocket surfaces.

---

## the agent's Discretion

- Exact Rust module names, fixture file names, fake-pool transcript schema, queue representation, and plan count.
- Exact split between `bitaxe-stratum` and `bitaxe-core` for shared runtime state, provided the final ownership is documented by implementation pointers and tests.

## Deferred Ideas

- Full Stratum v2 behavior.
- AxeOS API and WebSocket handlers.
- Safety-controller enablement beyond required mining preflight gates.
- OTA, filesystem, and release packaging.
