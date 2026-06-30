# Phase 13: Final Ultra 205 Release Evidence - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md; this log preserves the alternatives considered.

**Date:** 2026-06-30T14:53:46.141Z
**Phase:** 13-final-ultra-205-release-evidence
**Mode:** Yolo
**Areas discussed:** Package identity, hardware and network gates, HTTP/static/recovery evidence, OTA and rollback evidence, destructive recovery, checklist and redaction

## Package Identity

| Option | Description | Selected |
| --- | --- | --- |
| Manifest-first release gate | Run `just package` and manifest-backed `release-gate` before trusting hardware evidence. | Yes |
| Hardware-first evidence | Flash first, then reconcile package identity after the fact. | No |
| Manual prose summary | Summarize artifact identity without requiring manifest-backed checks. | No |

**User's choice:** Yolo selected manifest-first release gate.
**Notes:** Final evidence must not mix package artifacts and live observations from different commits.

## Hardware And Network Gates

| Option | Description | Selected |
| --- | --- | --- |
| Strict detector plus explicit `DEVICE_URL` | Require `just detect-ultra205` and a reachable device URL before live probes. | Yes |
| USB-only evidence | Capture serial proof but leave HTTP/OTA unattempted. | No |
| Opportunistic network scan | Infer device address from local network scanning. | No |

**User's choice:** Yolo selected strict detector plus explicit `DEVICE_URL`.
**Notes:** If `DEVICE_URL` is missing or unreachable, record a blocker and keep affected rows below `verified`.

## HTTP, Static, And Recovery Smoke

| Option | Description | Selected |
| --- | --- | --- |
| Exact release smoke set | Probe `/`, `/assets/app.css.gz`, missing static redirect, `/recovery`, API/WebSocket/update route coexistence. | Yes |
| Minimal homepage smoke | Probe only `/` and infer the rest from route registration. | No |
| Route-manifest proof only | Use Phase 10 route tooling without live HTTP. | No |

**User's choice:** Yolo selected exact release smoke set.
**Notes:** Phase 10 manifest evidence is useful but does not prove live route behavior.

## OTA And Rollback Evidence

| Option | Description | Selected |
| --- | --- | --- |
| Valid and invalid firmware OTA plus boot validation | Upload `esp-miner.bin`, reject invalid image, capture reboot, partition, identity, and rollback/boot-validation logs. | Yes |
| Valid OTA only | Treat successful upload response as sufficient. | No |
| Defer all OTA | Leave OTA rows implemented only. | No |

**User's choice:** Yolo selected valid and invalid firmware OTA plus boot validation.
**Notes:** Invalid upload rejection is not rollback proof; rollback requires captured post-update boot state.

## Destructive Recovery

| Option | Description | Selected |
| --- | --- | --- |
| Phase-gated runbooks only | Run rollback, large erase, failed-update, and interrupted-update checks only after exact recovery procedure is documented. | Yes |
| Manual bench judgment | Allow operator judgment at runtime. | No |
| Skip destructive checks silently | Avoid destructive evidence and leave docs unchanged. | No |

**User's choice:** Yolo selected phase-gated runbooks only.
**Notes:** Missing recovery prerequisites should produce pending evidence, not ad hoc raw flash or erase commands.

## Checklist, Release Docs, And Redaction

| Option | Description | Selected |
| --- | --- | --- |
| Conservative evidence matrix | Update checklist and docs only after exact evidence exists, with redaction review. | Yes |
| Promote broad release rows | Treat final package plus partial live checks as enough for broad verified parity. | No |
| Docs-only closure | Close the phase through prose without new command evidence. | No |

**User's choice:** Yolo selected conservative evidence matrix.
**Notes:** OTAWWW remains an explicit REL-03 gap unless whole-`www` interrupted-update hardware-regression evidence exists.

## the agent's Discretion

- Exact plan count and wave split.
- Whether HTTP/OTA probes are implemented as shell, Rust, or another repo-owned helper.
- Evidence JSON schema details and release-summary layout.
- Targeted parity guard additions if existing guards cannot express Phase 13 evidence semantics.

## Deferred Ideas

- Non-205 board release evidence.
- Full OTAWWW whole-`www` update parity without interruption/recovery evidence.
- Unbounded mining, voltage, fan, or destructive stress.
- Angular AxeOS rewrite or all-board release matrix.
