# Phase 27: Live Hardware ASIC And Stratum Bridge - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-07-05
**Phase:** 27-live-hardware-asic-and-stratum-bridge
**Mode:** Yolo
**Areas discussed:** Live bridge boundary, Production command dispatch, Nonce observation feedback, Phase 27 evidence mode, Hardware evidence and redaction

---

## Live Bridge Boundary

| Option | Description | Selected |
|--------|-------------|----------|
| Extend `live_stratum_runtime.rs` | Add ASIC dispatch and nonce feedback inside the existing Phase 25 socket shell | ✓ |
| New parallel production runtime module | Create a separate firmware runtime and re-orchestrate socket + ASIC there | |
| Route live socket mining through `controlled_mining_runtime.rs` | Reuse controlled path as the live production source of truth | |

**User's choice:** Extend `live_stratum_runtime.rs` as the sole live production bridge (recommended default).
**Notes:** Phase 25 already owns socket I/O, safe stop, prerequisites, and submit classification. Phase 24 production types are implemented but currently wired only through the controlled runtime.

## Production Command Dispatch

| Option | Description | Selected |
|--------|-------------|----------|
| Reuse guarded mining-loop dispatch into ASIC adapter | Translate live runtime work actions to `maybe_production_command` and existing status publishers | ✓ |
| Inline raw BM1366 frame construction in firmware | Build production frames directly in the live runtime module | |
| Keep live runtime socket-only until Phase 28 | Defer ASIC dispatch to a later promotion phase | |

**User's choice:** Reuse guarded mining-loop dispatch through the ASIC adapter with production-only command types (recommended default).
**Notes:** Diagnostic work must remain unreachable from the live socket path.

## Nonce Observation Feedback

| Option | Description | Selected |
|--------|-------------|----------|
| Stamp `ProductionNonceObservation` at firmware boundary | Preserve `PoolSessionGeneration` from the dispatch/read attempt before correlation | ✓ |
| Correlate directly from parsed ASIC bytes only | Infer session identity from nonce result alone | |
| Submit without correlation when hardware returns a nonce | Treat any nonce as share-ready | |

**User's choice:** Stamp observations at the firmware boundary and correlate through `ProductionWorkRegistry` before submit (recommended default).
**Notes:** Accepted/rejected classification still requires live pool response plus matching submit intent.

## Phase 27 Evidence Mode

| Option | Description | Selected |
|--------|-------------|----------|
| Distinct compile-time mode/ack pair | Add Phase 27-specific opt-in analogous to Phase 25 | ✓ |
| Reuse Phase 25 mode for bridge evidence | Extend Phase 25 mode semantics to cover ASIC bridge proof | |
| Runtime-only script flag without firmware mode | Gate evidence solely in shell scripts | |

**User's choice:** Distinct compile-time mode/ack pair with fail-closed default (recommended default).
**Notes:** Missing or mismatched mode/ack must not silently enable bridge behavior.

## Hardware Evidence And Redaction

| Option | Description | Selected |
|--------|-------------|----------|
| Repo-owned Phase 27 wrapper with blocked/hardware modes | Mirror Phase 25 detector-first evidence workflow for bridge proof | ✓ |
| Manual hardware log capture only | Rely on ad hoc monitor output without wrapper validation | |
| Promote share-outcome checklist rows from implementation alone | Advance STR-09 without detector-gated artifacts | |

**User's choice:** Repo-owned wrapper with blocked/hardware modes, allow-manifest validation, and exact non-claims (recommended default).
**Notes:** Committed evidence may record accepted, rejected, or blocked_safe_prerequisite categories only.

## Claude's Discretion

Module names, helper boundaries, evidence filenames, timeout budgets, mode constant names, fixture shapes, and plan count.

## Deferred Ideas

- Phase 28 checklist promotion from Phase 27 artifacts.
- Full active safety, OTA/recovery, non-205 boards, Stratum v2, display/input, BAP, and unbounded stress mining.
