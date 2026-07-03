# Phase 20: Active Safety Hardware Telemetry Evidence - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md - this log preserves the alternatives considered.

**Date:** 2026-07-03T20:50:01.088Z
**Phase:** 20-active-safety-hardware-telemetry-evidence
**Mode:** Yolo
**Areas discussed:** Gated evidence runbook, Active safety surface coverage, Live telemetry correlation, Checklist redaction and verification

## Gated Evidence Runbook

| Option | Description | Selected |
| --- | --- | --- |
| Reuse Phase 14 allow manifests | Extend the existing board/package/safe-state gate and surface-scoped manifests. | yes |
| Create a new broad Phase 20 runner | One broad script for every safety surface. | no |
| Manual-only evidence | Rely on manually written evidence without machine gates. | no |

**User's choice:** Auto-selected recommended default: reuse and tighten the Phase 14 allow-manifest pattern.
**Notes:** This preserves existing safety governance and avoids weakening detector, board-info, package identity, recovery, and redaction requirements.

## Active Safety Surface Coverage

| Option | Description | Selected |
| --- | --- | --- |
| Surface-scoped active probes | Independent packs for voltage/power, fan/thermal, self-test, load, display/input, failure paths, and telemetry. | yes |
| Treat read-only observations as active control proof | Promote rows from sensor or route observations alone. | no |
| Skip blocked evidence | Leave unsupported surfaces unchanged when prerequisites are absent. | no |

**User's choice:** Auto-selected recommended default: surface-scoped probes with blocked evidence for missing prerequisites.
**Notes:** Read-only observations may support exact telemetry subclaims, but active control and fault behavior require hardware-regression evidence.

## Live Telemetry Correlation

| Option | Description | Selected |
| --- | --- | --- |
| Explicit target and correlated proof | Require explicit `DEVICE_URL`, API body or WebSocket frames, and related hardware or serial observations. | yes |
| Route presence only | Treat HTTP/WebSocket route presence as live telemetry proof. | no |
| Network discovery | Infer target URL from scans, serial logs, mDNS, ARP, or router state. | no |

**User's choice:** Auto-selected recommended default: explicit target and correlated proof.
**Notes:** This carries forward Phase 17 target-lock rules and Phase 14 live telemetry blockers.

## Checklist Redaction And Verification

| Option | Description | Selected |
| --- | --- | --- |
| Exact-claim promotion | Promote only rows supported by evidence class, redaction review, parity validation, and lifecycle verification. | yes |
| Broad row promotion | Let one narrow artifact verify an entire mixed safety row. | no |
| Commit before clean verification | Commit/push even if redaction, parity, reference, lifecycle, or verification gates fail. | no |

**User's choice:** Auto-selected recommended default: exact-claim promotion and strict verification gate.
**Notes:** The wrapper-level commit/push gate remains conditional on `20-VERIFICATION.md` status `passed` plus lifecycle validation.

## the agent's Discretion

- Choose exact plan count, helper names, evidence layout, manifest fields, and whether to extend Phase 14 helpers or add Phase 20-specific wrappers.
- Keep `reference/esp-miner` read-only, use repo-owned ESP/esp-rs tooling, preserve functional core plus imperative shell, and avoid standalone body `---` separators.

## Deferred Ideas

- Live mining and soak evidence belongs to Phase 21.
- OTAWWW, recovery regression, and destructive release evidence remain outside Phase 20.
- Full LVGL display carousel and broad input routing parity remain deferred unless a safe bounded route is proved during planning.
