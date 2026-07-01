# Phase 14: Safety Hardware Evidence Completion - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md. This log preserves the alternatives considered.

**Date:** 2026-06-30T23:56:34.602Z
**Phase:** 14-safety-hardware-evidence-completion
**Mode:** Yolo
**Areas discussed:** Recovery and allow gates, active safety telemetry/control evidence, self-test/watchdog/load/display/input scope, checklist promotion/redaction/final verification

## Recovery And Allow Gates

| Option | Description | Selected |
| --- | --- | --- |
| Runbook-only phase gate | Smallest change; good for pending or observe-only conclusions but weak against drift and broad overclaims. | |
| Machine-enforced preflight allow manifest | Fail-closed, testable, reuses `just detect-ultra205`, and binds board-info, package identity, recovery, redaction, and stop conditions. | yes |
| Surface-scoped bounded probe wrappers | Separates voltage, fan, thermal, self-test, watchdog/load, display/input risk and supports per-surface limits, aborts, safe-state checks, and exact evidence artifacts. | yes |
| Bench/HIL interlock harness | Strongest recovery posture for repeated stress/fault testing, but heavier than this phase needs unless repeated stress or fault injection is required. | |

**User's choice:** Yolo recommendation: machine-enforced preflight allow manifest plus surface-scoped bounded probe wrappers for active hardware-regression claims.

**Notes:** If recovery, stimulus, redaction, package identity, or post-action safe-state checks are missing, the phase should record pending evidence instead of running a workaround.

## Active Safety Telemetry And Control Evidence

| Option | Description | Selected |
| --- | --- | --- |
| Component-scoped evidence packs with claim tiers | Separates read-only observation, bounded actuation, API/WebSocket projection, and unsupported/pending claims while preserving current checklist shape. | yes |
| Split checklist rows into explicit subclaims | Improves machine-visible precision but churns established row IDs and traceability. | |
| Procedure-scoped hardware regression manifests | Binds active or cadence-sensitive probes to exact commands, bounds, aborts, recovery, raw artifacts, and promotion targets. | yes |
| Conservative observe-only completion | Safest when recovery is incomplete, but leaves active voltage, fan, overheat/fault, and live telemetry claims below `verified`. | |

**User's choice:** Yolo recommendation: component-scoped evidence packs plus procedure manifests for active actuation or live cadence probes.

**Notes:** INA260, thermal, fan RPM, and API/WebSocket observations can support only exact read-only or projected telemetry claims. DS4432U writes, fan duty effects, overheat/fault behavior, self-test hardware submodes, runtime display/input, and load/stress require bounded `hardware-regression`.

## Self-Test, Watchdog/Load, And Runtime Display/Input

| Option | Description | Selected |
| --- | --- | --- |
| Component-scoped evidence packs with bounded probes and explicit blockers | Extends Phase 11 pattern, supports narrow promotion, and keeps unsupported surfaces below `verified`. | yes |
| Add temporary diagnostic firmware routes | Could improve repeatability, but risks expanding production surface unless compile-gated and recovery-bounded. | |
| Physical bench stimulus protocol | Useful for physical input/display observations but less repeatable and harder to redact. | maybe |
| Conservative deferral plus stronger blockers only | Lowest risk but may under-deliver Phase 14 evidence completion. | |

**User's choice:** Yolo recommendation: run only detector-gated bounded probes where safe routes or physical stimulus exist; otherwise keep subclaims below `verified` with owner/blocker.

**Notes:** Startup-only SSD1306 evidence remains startup-only. Self-test hardware submodes, watchdog/load stress, and runtime display/input require concrete runtime observation and recovery planning.

## Checklist Promotion, Redaction, And Final Verification

| Option | Description | Selected |
| --- | --- | --- |
| Exact-claim promotion with existing guards | Aligns with ADR/checklist semantics and lets narrow read-only smoke claims move while active controls require `hardware-regression`. | yes |
| Promotion manifest plus parity validator extension | Stronger machine-checking, but adds schema/tooling maintenance. | maybe |
| Freeze safety rows unless fresh Phase 14 regression evidence exists | Very safe, but can understate valid read-only or safe-unavailable observations. | |
| Split broad checklist rows into narrower subclaim rows | Useful if broad rows block truthful promotion, but creates checklist churn. | maybe |

**User's choice:** Yolo recommendation: exact-claim promotion by default, with parity validator extensions only when needed for new manifest semantics.

**Notes:** Final gate should include redaction review, `just parity`, relevant Rust checks if code changed, `just test`, `just verify-reference`, diff review, GSD verification status `passed`, and lifecycle validation before wrapper commit/push.

## the agent's Discretion

- Exact plan split, evidence directory layout, manifest schema, probe command names, JSON field names, and parity/tooling placement.
- Whether temporary diagnostics are necessary; if used, they must be compile-gated or otherwise prevented from becoming accidental production surface.
- Whether checklist row splitting is needed; default is precise notes and existing row IDs.

## Deferred Ideas

- Phase 15 owns trusted BM1366 and mining evidence completion.
- Phase 16 owns current-commit release evidence completion.
- Full LVGL display parity, non-205 boards, all-board factory images, Stratum v2, BAP, and Angular UI replacement remain deferred.
