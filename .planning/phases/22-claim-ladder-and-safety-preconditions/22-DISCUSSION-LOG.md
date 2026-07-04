# Phase 22: Claim Ladder And Safety Preconditions - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in `22-CONTEXT.md`; this log preserves the alternatives considered by the yolo recommendation pass.

**Date:** 2026-07-04T20:10:36.848Z
**Phase:** 22-Claim Ladder And Safety Preconditions
**Mode:** Yolo
**Lifecycle:** 22-2026-07-04T20-10-36
**Areas discussed:** Claim ladder, Safety prerequisite contract, Blocker reasons and fail-closed behavior, Evidence and checklist boundaries

## Claim Ladder

| Option | Description | Selected |
| --- | --- | --- |
| Explicit tier ladder | Define named claim tiers from v1.0 controlled no-share through later live production-share outcomes, with residual non-claims. | yes |
| Single readiness statement | Describe v1.1 readiness as one broad operator note. | no |
| Defer ladder to final parity closure | Wait until Phase 26 to define claim boundaries. | no |

**Chosen answer:** Explicit tier ladder.
**Notes:** This satisfies EVD-06 early and gives later phases a stable promotion vocabulary. The ladder must not let Phase 21 controlled no-share evidence imply accepted/rejected share proof.

## Safety Prerequisite Contract

| Option | Description | Selected |
| --- | --- | --- |
| Typed prerequisite model | Model power, thermal, fan, voltage, and safety prerequisites as typed domain inputs consumed before work dispatch. | yes |
| Script-only checklist | Keep prerequisite checks as shell-script conditions around hardware commands. | no |
| Hardware-only evidence later | Let later hardware phases discover prerequisite semantics during live runs. | no |

**Chosen answer:** Typed prerequisite model.
**Notes:** Existing `crates/bitaxe-safety` and `crates/bitaxe-stratum` gates already provide the right direction. Phase 22 should clarify freshness and boundedness rather than bypassing the pure gate.

## Blocker Reasons And Fail-Closed Behavior

| Option | Description | Selected |
| --- | --- | --- |
| Stable blocker taxonomy | Emit specific redaction-safe reason strings for missing, stale, unavailable, unsafe, ambiguous, or undocumented prerequisites. | yes |
| Generic blocked status | Emit only a broad blocked/not-ready status. | no |
| Operator-only prose | Document blockers without making them machine-checkable or visible in runtime state. | no |

**Chosen answer:** Stable blocker taxonomy.
**Notes:** Blocker reasons need to be visible to operators and tests while remaining safe to commit. Reuse existing reason strings where possible.

## Evidence And Checklist Boundaries

| Option | Description | Selected |
| --- | --- | --- |
| Conservative Phase 22 artifacts | Produce docs, pure tests, parity guards, and blocked evidence where appropriate; avoid new live claims without detector-gated proof. | yes |
| Promote implementation-only rows | Mark rows verified once code exists. | no |
| Run live mining in Phase 22 | Attempt live Stratum/share proof during claim-ladder work. | no |

**Chosen answer:** Conservative Phase 22 artifacts.
**Notes:** Phase 22 owns EVD-06, SAFE-10, and SAFE-11. Live socket behavior, production ASIC work, accepted/rejected share proof, and telemetry projection are intentionally later phases.

## Claude's Discretion

- Exact module names, documentation layout, artifact filenames, enum labels, and test grouping.
- Whether to implement the claim ladder in a new safety/core module or extend an existing crate, as long as the pure decision model remains clear.

## Deferred Ideas

- Redacted end-to-end operator evidence root: Phase 23.
- Trusted BM1366 production work path: Phase 24.
- Live Stratum runtime, accepted/rejected share outcome, safe stop, and watchdog under load: Phase 25.
- Telemetry, scoreboard, statistics, and parity closure: Phase 26.
- Full active safety closure, OTA/recovery, display/input, BAP, Stratum v2, non-205 boards, and unbounded stress mining: future milestones.
