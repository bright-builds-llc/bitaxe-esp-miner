---
gsd_state_version: 1.0
milestone: v1.1
milestone_name: milestone
status: executing
stopped_at: Completed 22-02-PLAN.md
last_updated: "2026-07-04T20:37:45.149Z"
last_activity: 2026-07-04
progress:
  total_phases: 5
  completed_phases: 0
  total_plans: 3
  completed_plans: 2
  percent: 67
---

# Project State

## Project Reference

See: `.planning/PROJECT.md` (updated 2026-07-04)

**Core value:** A Bitaxe owner can build, flash, run, configure, monitor, and update Rust firmware on real Bitaxe hardware with the same observable behavior they expect from upstream ESP-Miner.

**Current focus:** Phase 22 — Claim Ladder And Safety Preconditions

## Current Position

Phase: 22 (Claim Ladder And Safety Preconditions) — EXECUTING
Plan: 3 of 3
Status: Ready to execute
Last activity: 2026-07-04

Progress: [███░░░░░░░] 33%

## Active Artifacts

- Current project brief: `.planning/PROJECT.md`
- Current roadmap: `.planning/ROADMAP.md`
- Milestone summary: `.planning/MILESTONES.md`
- Current requirements: `.planning/REQUIREMENTS.md`
- Archived roadmap: `.planning/milestones/v1.0-ROADMAP.md`
- Archived requirements: `.planning/milestones/v1.0-REQUIREMENTS.md`
- Archived audit: `.planning/milestones/v1.0-MILESTONE-AUDIT.md`

## Current Milestone Scope

v1.1 targets trusted Ultra 205 Stratum v1 production mining. The milestone should move beyond controlled no-share evidence by proving real socket I/O, trusted BM1366 initialization/work/result behavior, at least one real accepted or rejected share outcome, live mining telemetry, watchdog responsiveness, and safe stop under prerequisite safety gates.

Explicit v1.1 deferrals: non-205 boards, other ASIC families, Stratum v2, OTAWWW, rollback/recovery fault injection, runtime display/input, LVGL-like UI flow, and BAP.

## Accepted Tech Debt

- Nyquist validation remains partial for older phases 01, 02, 03, 04, 07, 08, 09, 10, 17, and 18.
- Several parity checklist surfaces intentionally remain below `verified` as exact non-claims. These include deferred non-205 boards, accepted/rejected live-share behavior, active voltage/fan/fault/self-test/load controls, whole-`www` OTAWWW update behavior, rollback/boot-validation, destructive recovery cases, unbounded stress, and broader production mining behavior.

## Accumulated Context

### Decisions

- v1.1 phase numbering continues from v1.0, so planned work starts at Phase 22.
- v1.1 is limited to Ultra 205 BM1366 Stratum v1 trusted production mining.
- Full active voltage, fan, thermal, fault, recovery, self-test, non-205 boards, OTAWWW/recovery fault injection, runtime display/input/BAP, Stratum v2, and unbounded stress mining remain deferred.
- Phase goals preserve the functional-core/imperative-shell split for BM1366 work, Stratum runtime, safety decisions, and API projections.
- [Phase 22]: Kept claim ladder validation as a test-enforced parity helper without adding a CLI subcommand.
- [Phase 22]: Declared the claim ladder Markdown as a Bazel compile-time fixture for include_str validation.
- [Phase 22]: Separated controlled no-share evidence text from accepted/rejected share terms to avoid overclaim ambiguity.
- [Phase 22]: Modeled production mining prerequisites as typed Fresh, Bounded, or Blocked inputs instead of accepting shell-owned readiness strings.
- [Phase 22]: Kept existing power, thermal, safety, hardware ack, and ASIC initialization checks after typed production precondition decisions as defense in depth.
- [Phase 22]: Preserved controlled-runtime default blocker behavior by making controlled gate builders pass an explicit typed Ready decision.

### Pending Todos

None yet.

### Blockers/Concerns

- Live accepted/rejected share feasibility remains evidence-dependent until a detector-gated Ultra 205 run observes a parsed pool response to live ASIC-derived work.
- Hardware phases must follow the Ultra 205 detector gate, redaction rules, safe-stop evidence requirements, and exact non-claim governance.

## Session Continuity

Last session: 2026-07-04T20:37:45.147Z
Stopped at: Completed 22-02-PLAN.md
Resume file: None

## Next Step

Run `/gsd-execute-phase 22` to continue with `22-02-PLAN.md`.
