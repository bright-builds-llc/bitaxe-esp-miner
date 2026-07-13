# Phase 11: Safety Controller Hardware Regression Evidence - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md - this log preserves the alternatives considered.

**Date:** 2026-06-29T20:24:23.587Z
**Phase:** 11-safety-controller-hardware-regression-evidence
**Mode:** Yolo
**Areas discussed:** Hardware Run And Recovery Protocol, Sensor And Actuator Evidence Coverage, Self-Test Display/Input Watchdog And Load Evidence, Checklist Promotion And Parity Guard Rules

## Hardware Run And Recovery Protocol

| Option | Description | Selected |
| --- | --- | --- |
| Strict phase-gated runbook | Gate every live run through `just detect-ultra205`, exact allowed commands, and a documented recovery path before actuation or fault/destructive work. | yes |
| Observe-only evidence fallback | Record safe-unavailable or read-only observations when detection passes but recovery or actuation prerequisites are incomplete. | yes |
| Scripted bounded regression harness | Use repeatable probes only when limits, redaction, recovery, and fail-closed behavior are explicit. | yes |
| Manual bench checklist with explicit recovery signoff | Use human-observed evidence for physical behavior that is safe but not yet automatable. | yes |

**User's choice:** Yolo auto-selected a strict phase-gated runbook with observe-only fallback, bounded regression where safe, and manual bench evidence only for physical observations.
**Notes:** No ad hoc erase, rollback, interrupted-update, voltage/fan/mining stress, raw writes, or raw monitor proof should run outside documented phase procedures.

## Sensor And Actuator Evidence Coverage

| Option | Description | Selected |
| --- | --- | --- |
| Tiered per-surface evidence matrix | Map PWR/THR/IO/UI/SELF rows to claim type, pass criteria, evidence artifact, and promotion status. | yes |
| Component-scoped evidence packs | Group DS4432U, INA260, thermal/fan/PID, self-test/watchdog, and display/input evidence independently. | yes |
| Black-box live smoke only | Use only broad flash/monitor smoke and leave safety rows below verified. | no |
| Automated HIL fault-injection bench | Add a full hardware-in-the-loop bench for fault injection and regression. | no |

**User's choice:** Yolo auto-selected a tiered matrix plus component-scoped packs. Black-box smoke is insufficient for broad promotion, and full HIL fault injection is not assumed for this phase.
**Notes:** Telemetry reads and actuator writes must be evidenced separately.

## Self-Test Display/Input Watchdog And Load Evidence

| Option | Description | Selected |
| --- | --- | --- |
| Narrow live evidence for ready routes | Capture real hardware proof only where existing firmware/API/log/WebSocket/serial routes can safely expose the state. | yes |
| Explicit gap ledger for unready surfaces | Keep unready runtime display/input or self-test paths below verified with owner and follow-up. | yes |
| Startup-only display evidence as bounded surrogate | Reuse startup SSD1306 evidence only as supporting evidence, not runtime display/input verification. | yes |
| Fault-injection or stress-style watchdog/load evidence | Run only if safe stimulus, stop conditions, and recovery path are documented first. | yes |

**User's choice:** Yolo auto-selected narrow live evidence for ready routes and explicit gap ledgers for unready surfaces.
**Notes:** Watchdog/load evidence must show observable responsiveness or liveness, not only the Phase 6 pure watchdog model.

## Checklist Promotion And Parity Guard Rules

| Option | Description | Selected |
| --- | --- | --- |
| Tiered promotion by claim type | Use smoke for narrow observations and regression for active-control or failure-path claims. | yes |
| Smoke-only promotion when template passes | Let one hardware-smoke template verify broad safety rows. | no |
| Regression-only promotion for safety-critical verified rows | Require regression for every safety-critical verified claim. | partial |
| Split broad rows into exact subclaims | Split or hold mixed rows when evidence cannot support the full row. | yes |

**User's choice:** Yolo auto-selected tiered promotion by claim type, with row splitting or below-verified status for mixed claims.
**Notes:** Existing parity guard behavior must remain: safety-critical `verified` rows require `hardware-smoke` or `hardware-regression`; Phase 11 may add stricter semantics for active-control claims if needed.

## the agent's Discretion

- Exact plan count, evidence schema, evidence filenames, helper names, and whether to implement a minimal hardware-regression CLI or keep evidence as structured docs.
- Exact checklist row split strategy if a broad row cannot be safely verified from a narrow observation.

## Deferred Ideas

- Full display/input parity, ASIC/mining hardware evidence, and final release HTTP/OTA/recovery evidence remain later phases.
