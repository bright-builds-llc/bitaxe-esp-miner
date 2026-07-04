# Phase 23: Redacted Operator Evidence Workflow - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in `23-CONTEXT.md`; this log preserves the alternatives considered.

**Date:** 2026-07-04T22:55:05.649Z
**Phase:** 23-redacted-operator-evidence-workflow
**Mode:** Yolo
**Areas discussed:** Operator Evidence Flow, Single Redacted Evidence Root, Redaction Contract, Reuse And Integration, Verification Gate

## Operator Evidence Flow

| Option | Description | Selected |
| --- | --- | --- |
| Unified repo-owned flow | Add a documented `just` or repo-owned script path that covers detect, package/flash, local credentials, bounded run, telemetry, safe-stop, redaction, and review. | yes |
| Documentation-only flow | Document the existing scattered Phase 17/21/22 commands without a new unified surface. | no |
| Direct ad hoc command examples | Give operators command fragments to adapt locally. | no |

**Selected answer:** Unified repo-owned flow.
**Rationale:** Phase 23 requirements require a repeatable operator path, and prior phases already warn against hidden local scripts or ad hoc hardware commands. The flow must remain detector-gated and repo-owned.

## Single Redacted Evidence Root

| Option | Description | Selected |
| --- | --- | --- |
| One Phase 23 evidence root with required slots | Define one committed root containing package, detector, board-info, command, log, API, WebSocket, share-outcome, redaction-review, safe-stop, and conclusion artifacts. | yes |
| Continue Phase 21 multi-pack layout | Reuse the Phase 21 ladder as-is and scatter artifacts across pack directories. | no |
| Only write final summary artifacts | Keep detailed captures local and commit only a conclusion. | no |

**Selected answer:** One Phase 23 evidence root with required slots.
**Rationale:** EVD-07 explicitly requires a single redacted evidence root. Slots that later phases own should be present as blocked, pending, or deferred non-claims rather than omitted or overclaimed.

## Redaction Contract

| Option | Description | Selected |
| --- | --- | --- |
| First-class redaction contract with tests/checks | Treat redaction as an implementation deliverable and add deterministic coverage for pool, target, extranonce, share payload, socket error, NVS/settings, device URL, IP, MAC, Wi-Fi, and token categories. | yes |
| Manual redaction review only | Rely on Markdown review checklists and manual scans after evidence is generated. | no |
| Wrapper-only sanitization | Redact only host-side wrapper output and leave firmware/operator-visible logs to later phases. | no |

**Selected answer:** First-class redaction contract with tests/checks.
**Rationale:** STR-10 and EVD-09 require coverage beyond existing Phase 21 tests. PITFALLS.md calls out target, extranonce, share payload, socket error, NVS/settings, and owner identity leakage as explicit blockers.

## Reuse And Integration

| Option | Description | Selected |
| --- | --- | --- |
| Extend existing Phase 17/21/22 patterns | Build on existing flash redaction, mining evidence wrappers, WebSocket capture, claim ladder, blocker reasons, and parity validators. | yes |
| Create a separate new evidence stack | Add a parallel toolchain independent of Phase 17/21/22 helpers. | no |
| Postpone integration until Phase 25 | Wait for real socket/share behavior before hardening evidence tooling. | no |

**Selected answer:** Extend existing Phase 17/21/22 patterns.
**Rationale:** The repo already has useful detector, redaction, capture, and validation surfaces. Phase 23 should unify and harden them before later phases introduce more sensitive production data.

## Verification Gate

| Option | Description | Selected |
| --- | --- | --- |
| Pass with proven workflow/redaction and explicit later-phase non-claims | Allow Phase 23 verification to pass when the evidence workflow, root contract, redaction checks, and docs are clean, while share and production-work slots remain exact later-phase non-claims. | yes |
| Require live accepted/rejected share proof now | Make Phase 23 run real production mining and observe a real pool share response. | no |
| Skip hardware and redaction validation until later | Produce only plans/docs and rely on Phase 25 to validate redaction. | no |

**Selected answer:** Pass with proven workflow/redaction and explicit later-phase non-claims.
**Rationale:** Phase 23 owns the operator evidence workflow and secret-handling surface. BM1366 production work and live share proof are explicitly scoped to later roadmap phases, but the evidence root must be ready to contain those later artifacts safely.

## Claude's Discretion

The implementation may choose exact file names, command names, artifact suffixes, script boundaries, validator names, and test fixture values. The selected defaults require detector gating, runtime-only credential handling, committed redacted artifacts only, exact non-claim language, and repo-native verification.

## Deferred Ideas

- Trusted BM1366 production work and result handling are Phase 24.
- Live Stratum socket lifecycle, accepted/rejected share outcome, fake-pool production tests, watchdog load, and runtime safe-stop proof are Phase 25.
- API/WebSocket/statistics/scoreboard promotion and final parity closure are Phase 26.
- Full active safety closure, OTA/recovery destructive evidence, non-205 boards, other ASIC families, display/input, BAP, Stratum v2, and unbounded stress remain future scope.
