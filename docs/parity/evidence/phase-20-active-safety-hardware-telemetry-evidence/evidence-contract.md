# Phase 20 Evidence Contract

## Status

contract_status: draft
phase: 20-active-safety-hardware-telemetry-evidence
board_scope: 205
source: .planning/phases/20-active-safety-hardware-telemetry-evidence/20-01-PLAN.md

## Purpose

Phase 20 evidence must support exact active-safety and live-telemetry claims
without hiding unsafe actions behind broad scripts or promoting unsupported
checklist rows. Every pack below is independently promotable or independently
blocked. A pack may cite only reviewed artifacts that satisfy its claim tier,
metadata, redaction, and evidence-class requirements.

Any new diagnostic route or active probe trigger must be compile-gated,
bounded, tested, non-production-exposed, documented in the relevant evidence
pack, and reviewed before citation.

## Common Evidence Metadata

All Phase 20 evidence packs must record these fields when the field applies to
the claim:

- board `205`
- selected detector port
- `just detect-ultra205` output or blocked reason
- `espflash board-info --chip esp32s3 --port <port> --non-interactive` output or blocked reason
- source commit
- reference commit
- package manifest identity when firmware was built or flashed
- exact command run
- allowed surface
- claim tier
- evidence class
- bounded inputs
- abort conditions
- recovery steps
- post-action safe-state markers
- evidence directory
- redaction reviewer
- affected checklist rows or subclaims
- non-claims

Active claim tiers must include all required abort conditions, recovery steps,
and final safe-state markers before execution. Blocked or deferred records may
use `unsupported-pending` plus `deferred` and must state the missing prerequisite.

## Evidence Packs

| Pack | Allowed claim tiers | Minimum metadata | Evidence class required | Non-claims |
| --- | --- | --- | --- | --- |
| `safe-baseline` | `safe-baseline`, `read-only-observation`, `safe-unavailable`, `unsupported-pending` | detector gate, board-info gate, board `205`, source commit, reference commit, package identity when flashed, exact safe-state markers, redaction reviewer | `hardware-smoke` for observed baseline, `deferred` for blocked baseline | Does not prove voltage writes, fan duty effects, fault recovery, self-test hardware, load stress, runtime display/input, mining, OTA, or live telemetry freshness. |
| `active-power-voltage` | `read-only-observation`, `bounded-actuation`, `safe-unavailable`, `unsupported-pending` | detector gate, board-info gate, package identity, power/voltage command, bounded setpoint or read-only input, abort conditions, recovery steps, final safe-state markers, checklist rows | `hardware-regression` for active DS4432U or power-control claims; `hardware-smoke` for exact read-only or unavailable telemetry; `deferred` for blocked evidence | Read-only or unavailable data does not verify DS4432U writes, ASIC power sequencing, fresh INA260 telemetry, or closed-loop control. |
| `active-thermal-fan` | `read-only-observation`, `bounded-actuation`, `fault-stimulus`, `safe-unavailable`, `unsupported-pending` | detector gate, board-info gate, package identity, thermal/fan command, bounded fan duty or sensor input, expected observation, abort conditions, recovery steps, final safe-state markers | `hardware-regression` for fan duty effects, RPM behavior, overheat/fan fault stimulus, or restart behavior; `hardware-smoke` for exact read-only or unavailable observations; `deferred` for blocked evidence | Thermal read shape does not verify fan duty effects, physical fan response, overheat stop/restart, or fan fault handling. |
| `self-test-watchdog-load` | `read-only-observation`, `self-test-hardware`, `load-stress`, `safe-unavailable`, `unsupported-pending` | detector gate, board-info gate, package identity, self-test submode or bounded workload, pass/fail/cancel criteria, production-mining gate, abort conditions, recovery steps, final safe-state markers | `hardware-regression` for self-test hardware submodes, bounded load, watchdog recovery, or load responsiveness; `hardware-smoke` for exact startup/yield or safe-unavailable observations; `deferred` for blocked evidence | Supervisor startup/yield logs do not prove load stress, watchdog recovery, self-test pass/fail/cancel behavior, or production mining behavior. |
| `runtime-display-input` | `read-only-observation`, `runtime-display-input`, `safe-unavailable`, `unsupported-pending` | detector gate, board-info gate, package identity, runtime display/input action, physical/log/API/WebSocket observation, abort conditions, recovery steps, final safe-state markers | `hardware-regression` for runtime display or input claims; `hardware-smoke` for exact startup or unavailable observations; `deferred` for blocked evidence | Startup SSD1306 text and runtime-gap markers do not verify runtime page flow, button routing, display configuration, or physical input parity. |
| `failure-paths` | `fault-stimulus`, `safe-unavailable`, `unsupported-pending` | detector gate, board-info gate, package identity, named stimulus, expected fault, abort condition, restore path, observed status, API/log/WebSocket projection when available, final safe-state marker, checklist rows | `hardware-regression` for any active fault stimulus or fault-path claim; `deferred` for blocked evidence | A blocked or unavailable fault route does not verify overheat, fan, power, thermal, ASIC, watchdog, self-test, or recovery behavior. |
| `live-api-websocket-telemetry` | `api-websocket-projection`, `read-only-observation`, `safe-unavailable`, `unsupported-pending` | detector gate or trusted target lock, explicit `DEVICE_URL` when used, package identity, route path, bounded capture duration, frame count, redaction reviewer, correlated hardware or safe-state observation | `hardware-smoke` for exact live route or safe-unavailable telemetry projection; `deferred` for blocked target/client evidence. Active safety-control projection still needs the active pack's `hardware-regression` evidence before promotion. | Route presence, no-upgrade responses, stale cached bodies, and uncorrelated frames do not prove telemetry freshness, cadence, active control behavior, or fault recovery. |
| `parity-redaction` | `parity-redaction`, `unsupported-pending` | artifact list, reviewer, raw-artifact policy, scan command, reviewed matches, retained identifiers, absent artifacts, citation decision | `workflow` for completed redaction review; `deferred` for pending review | A redaction review clears only named artifacts. It does not verify safety behavior, live telemetry, hardware effects, or checklist promotion by itself. |

## Citation Rules

- Active voltage, fan, fault, self-test, load, runtime input, and runtime display
  claims require `hardware-regression`.
- Narrow read-only, startup, safe-unavailable, or live transport observations may
  use `hardware-smoke` only for the exact subclaim proven.
- `unsupported-pending` plus `deferred` may document blocked evidence and must
  not be promoted as verified behavior.
- `just parity` must pass before any final checklist promotion cites Phase 20
  artifacts.
- `reference/esp-miner` remains read-only evidence and must not be modified.
