# Phase 20 Active Safety Hardware Telemetry Evidence Summary

## Status

phase20_status: draft
redaction_status: pending
checklist_status: not-updated
source: .planning/phases/20-active-safety-hardware-telemetry-evidence/20-01-PLAN.md

## Purpose

This ledger is the draft Phase 20 exact-claim matrix. It starts all claims at
`pending` so downstream plans can add hardware, network, API, WebSocket, blocked,
or redaction evidence without overclaiming behavior before artifacts exist.

No Phase 20 checklist row has been promoted by this scaffold. Final promotion
requires reviewed artifacts, the evidence class required by
`evidence-contract.md`, completed redaction review, and `just parity`.

## Evidence Pack Ledger

| Pack | Status | Redaction | Checklist impact |
| --- | --- | --- | --- |
| `safe-baseline` | pending | pending | not-updated |
| `active-power-voltage` | pending | pending | not-updated |
| `active-thermal-fan` | pending | pending | not-updated |
| `self-test-watchdog-load` | pending | pending | not-updated |
| `runtime-display-input` | pending | pending | not-updated |
| `failure-paths` | pending | pending | not-updated |
| `live-api-websocket-telemetry` | pending | pending | not-updated |
| `parity-redaction` | pending | pending | not-updated |

## Exact-Claim Matrix

| Requirement | Draft Phase 20 claim status | Required evidence before promotion |
| --- | --- | --- |
| `SAFE-01` | pending | `hardware-regression` for active voltage or power-control effects; exact read-only power telemetry may use `hardware-smoke` only if fresh board evidence exists. |
| `SAFE-02` | pending | `hardware-regression` for fan duty, fan RPM response, overheat, or fan fault behavior; exact read-only thermal/fan observations may use `hardware-smoke`. |
| `SAFE-03` | pending | Pure PID and thermal-control tests remain prerequisite evidence; active fan behavior still requires hardware evidence. |
| `SAFE-04` | pending | `failure-paths` evidence must name stimulus, expected fault, abort, restore path, projection, and final safe-state marker; active fault claims require `hardware-regression`. |
| `SAFE-05` | pending | Self-test submode, pass/fail/cancel, restart, production-mining gate, and recovery behavior require `hardware-regression` or blocked evidence. |
| `SAFE-06` | pending | Runtime display/input claims require a real runtime route plus physical or log/API/WebSocket observation; startup-only evidence remains a narrow subclaim. |
| `SAFE-07` | pending | Power, current, voltage, fan, and temperature telemetry must be captured from board `205` and correlated with API/WebSocket projection where claimed. |
| `SAFE-08` | pending | Safety-critical verified rows must cite hardware evidence; active safety-control and `failure-paths` rows require `hardware-regression`. |
| `SAFE-09` | pending | Watchdog/load responsiveness requires bounded workload evidence, API/WebSocket responsiveness where claimed, and final safe-state markers. |
| `EVD-05` | pending | Final evidence must layer unit/workflow checks, hardware smoke or regression artifacts, redaction review, parity validation, and reference cleanliness. |

## Current Non-Claims

- This scaffold does not prove active voltage writes, fan duty effects, overheat
  behavior, fan fault behavior, ASIC fault behavior, self-test hardware submodes,
  bounded load stress, watchdog recovery, runtime display/input behavior, or
  live telemetry freshness.
- This scaffold does not cite detector, board-info, package, API body,
  WebSocket frame, command-output, or manual-observation artifacts.
- This scaffold does not update `docs/parity/checklist.md`.
- This scaffold does not modify `reference/esp-miner`.

## Next Evidence Inputs

Downstream plans may populate this ledger only after they create reviewed
evidence packs under this directory. Blocked evidence is acceptable when a safe
detector, target, route, stimulus, recovery path, or redaction prerequisite is
missing.
