---
gsd_state_version: 1.0
milestone: v1.1
milestone_name: milestone
status: verifying
stopped_at: Completed 26-04-PLAN.md
last_updated: "2026-07-05T04:36:23.709Z"
last_activity: 2026-07-05
progress:
  total_phases: 5
  completed_phases: 5
  total_plans: 18
  completed_plans: 18
  percent: 100
---

# Project State

## Project Reference

See: `.planning/PROJECT.md` (updated 2026-07-04)

**Core value:** A Bitaxe owner can build, flash, run, configure, monitor, and update Rust firmware on real Bitaxe hardware with the same observable behavior they expect from upstream ESP-Miner.

**Current focus:** Phase 26 — Telemetry And Parity Closure

## Current Position

Phase: 26 (Telemetry And Parity Closure) — EXECUTING
Plan: 4 of 4
Status: Phase complete — ready for verification
Last activity: 2026-07-05

Progress: [██████████] 100%

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
- [Phase 22]: Kept SAFE-10 and SAFE-11 at implemented with unit/workflow evidence because Phase 22 produced no detector-gated hardware proof for live prerequisite behavior.
- [Phase 22]: Promoted EVD-06 to verified using workflow evidence from the claim ladder, parity guard, and Phase 22 closure summary.
- [Phase 22]: Recorded only redaction-safe reason categories and explicit non-claims in committed evidence.
- [Phase 23]: Added a repo-owned redacted operator evidence workflow with required slots, validator checks, `just phase23-evidence`, and deterministic redaction review.
- [Phase 23]: Kept `CFG-07` below verified because the parity guard requires hardware evidence before promoting safety-critical runtime credential handling.
- [Phase 23]: Preserved exact non-claims for Phase 24 BM1366 production work, Phase 25 live Stratum/share behavior, and Phase 26 telemetry closure.
- [Phase 24]: Production BM1366 work uses distinct command and payload types instead of diagnostic work names.
- [Phase 24]: Production ASIC failures render stable redaction-safe category labels only.
- [Phase 24]: Production BM1366 work is bound to PoolSessionGeneration before dispatch.
- [Phase 24]: Clean-jobs, reconnect, authorization reset, and session replacement clear queued, active, and valid-job state.
- [Phase 24]: Raw-bearing production work registry surfaces render redacted category labels instead of raw job, extranonce, target, or payload details.
- [Phase 24]: BM1366 nonce observations must carry PoolSessionGeneration because parsed ASIC results have no pool-session identity.
- [Phase 24]: The guarded mining loop emits production BM1366 commands and submit intents instead of diagnostic commands or direct share submissions.
- [Phase 24]: Firmware production logs publish stable ASIC status labels and defer accepted/rejected pool-response classification to Phase 25.
- [Phase 24]: Phase 24 checklist rows stay implemented with unit,workflow evidence only; no hardware promotion branch was added.
- [Phase 24]: Phase 24 evidence preserves Phase 25 ownership of live socket and share-response outcomes.
- [Phase 24]: Phase 24 evidence preserves Phase 26 ownership of API, WebSocket, statistics, and scoreboard promotion.
- [Phase 25]: Pure live Stratum lifecycle, submit response classification, and fake-pool behavior stay in crates/bitaxe-stratum without socket or credential-file ownership.
- [Phase 25]: Accepted/rejected share classification requires SubmitIntent plus matching request id and typed StratumResponse; fake-pool outcomes remain deterministic STR-11 evidence only.
- [Phase 25]: Kept Phase 25 live Stratum startup behind a distinct compile-time mode and acknowledgment pair so Phase 21 controlled evidence cannot start the socket path.
- [Phase 25]: Evaluated typed Phase 22 production-mining preconditions before NVS pool settings access or TcpStream construction.
- [Phase 25]: Used a named Phase 25 snapshot helper for safe-stop refresh without adding Phase 26 statistics or scoreboard semantics.
- [Phase 25]: Recorded Phase 25 committed evidence as blocked-safe-prerequisite rather than accepted/rejected live share proof because no detector-gated live pool response artifact was produced.
- [Phase 25]: Allowed Phase 25 mining-allow manifests only through the repo-owned wrapper command surface while preserving raw Stratum, unsafe hardware-control, erase, rollback, and stale-target rejection.
- [Phase 25]: Promoted STR-11 to verified from deterministic unit coverage, while keeping STR-08, STR-09, SAFE-12, and SAFE-13 at implemented/workflow scope without hardware overclaiming.
- [Phase 26]: Kept Phase 26 telemetry projection as a pure stratum v1 core module exported through Rust and Bazel.
- [Phase 26]: Accepted and rejected projection counters advance only for current-generation SubmitClassification Accepted or Rejected events.
- [Phase 26]: Projection sample markers drain at most once per runtime boundary to prevent request-time statistics fabrication.
- [Phase 26]: Plan 26-02 derives API snapshot mining state and live telemetry JSON from RuntimeTelemetryProjection.
- [Phase 26]: Plan 26-02 materializes statistics rows only from explicit RuntimeProjectionSampleMarker values.
- [Phase 26]: Plan 26-02 keeps scoreboard output empty without parsed-response-backed and redaction-allowed share outcome material.
- [Phase 26]: RuntimeTelemetryProjection is stored beside command-visible mining state so firmware producers and API consumers share one telemetry source of truth.
- [Phase 26]: Only the projected statistics helper drains pending sample markers; system-info and live-WebSocket reads do not consume statistics samples.
- [Phase 26]: Scoreboard output remains empty until parsed-response-backed and redaction-allowed share outcome material exists.
- [Phase 26]: Kept Phase 26 closure as evidence and guardrail work without adding a promotion manifest.
- [Phase 26]: Validated Phase 26 verified-row claims from checklist identity fields and exact evidence tokens.
- [Phase 26]: Kept accepted/rejected live-share proof and detector-gated hardware telemetry as explicit non-claims.

### Pending Todos

None yet.

### Blockers/Concerns

- Live accepted/rejected share feasibility remains evidence-dependent until a detector-gated Ultra 205 run observes a parsed pool response to live ASIC-derived work.
- Hardware phases must follow the Ultra 205 detector gate, redaction rules, safe-stop evidence requirements, and exact non-claim governance.

## Performance Metrics

| Phase | Plan | Duration | Tasks | Files | Recorded |
| --- | --- | --- | --- | --- | --- |
| 25 | 03 | 7min | 3 | 14 | 2026-07-05 |
| Phase 26 P01 | 4min | 2 tasks | 3 files |
| Phase 26 P02 | 4min | 2 tasks | 3 files |
| Phase 26 P03 | 7min | 2 tasks | 3 files |
| Phase 26 P04 | 6min | 3 tasks | 9 files |

## Session Continuity

Last session: 2026-07-05T04:36:23.707Z
Stopped at: Completed 26-04-PLAN.md
Resume file: None

## Next Step

Run `/gsd-verify-work 25` to verify Phase 25 closure before Phase 26 telemetry planning.
