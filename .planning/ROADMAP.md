# Roadmap: Bitaxe Rust Firmware

## Overview

v1.1 turns the shipped Ultra 205 v1.0 controlled no-share mining foundation into a trusted, bounded Stratum v1 production mining path. The milestone starts at Phase 22 to continue v1.0's completed Phase 1-21 sequence and remains scoped to Ultra 205 BM1366 only: exact-claim evidence governance, prerequisite safety gates, redacted operator evidence, BM1366 production work, real Stratum v1 socket/share behavior, runtime telemetry, safe stop, and parity promotion for only the claims proven by v1.1 artifacts.

## Milestones

- [x] **v1.0 Ultra 205 Parity** - Phases 1-21 shipped 2026-07-04; full archive: `.planning/milestones/v1.0-ROADMAP.md`.
- [ ] **v1.1 Ultra 205 Trusted Production Mining** - Phases 22-26 planned for bounded Ultra 205 BM1366 Stratum v1 production mining.

## Phases

**Phase Numbering:**
- Integer phases (22-26): planned milestone work continuing from v1.0.
- Decimal phases (22.1, 22.2): urgent insertions if needed between planned phases.

- [x] **Phase 22: Claim Ladder And Safety Preconditions** - Operators get exact v1.1 claim boundaries and prerequisite safety gates before production work can start. (completed 2026-07-04)
- [ ] **Phase 23: Redacted Operator Evidence Workflow** - Operators can run the repo-owned mining evidence flow without committing secrets or unsupported claims.
- [ ] **Phase 24: BM1366 Production Work Path** - The firmware separates diagnostic ASIC behavior from trusted pool-derived BM1366 work and result handling.
- [ ] **Phase 25: Live Stratum Runtime And Safe Stop** - Ultra 205 can run a real watchdog-responsive Stratum v1 mining session and stop safely.
- [ ] **Phase 26: Telemetry And Parity Closure** - Runtime API, WebSocket, counters, and parity checklist updates reflect only proven v1.1 mining events.

## Phase Details

### Phase 22: Claim Ladder And Safety Preconditions
**Goal**: Ultra 205 operators can tell exactly which v1.1 production-mining claims are allowed, and firmware can fail closed before BM1366 work dispatch when prerequisite safety evidence is missing or unsafe.
**Depends on**: Phase 21
**Requirements**: EVD-06, SAFE-10, SAFE-11
**Success Criteria** (what must be TRUE):
  1. Operator can distinguish v1.0 controlled no-share evidence from v1.1 live production-mining claim tiers.
  2. Operator can see that production mining requires fresh or explicitly bounded power, thermal, fan, voltage, and safety observations before BM1366 work dispatch.
  3. Operator receives a specific blocker reason when safety prerequisites are stale, unavailable, unsafe, ambiguous, or undocumented.
  4. Parity materials preserve explicit non-claims for full active voltage, fan, thermal, self-test, and fault-stimulus closure.
**Plans**: 3 plans
Plans:
- [x] 22-01-PLAN.md — Create the operator claim ladder and parity overclaim guard.
- [x] 22-02-PLAN.md — Add typed production-mining preconditions and exact blocker propagation.
- [x] 22-03-PLAN.md — Close Phase 22 evidence, checklist, and validation artifacts.

### Phase 23: Redacted Operator Evidence Workflow
**Goal**: Ultra 205 operators can use a repo-owned, detector-gated flow for bounded production mining evidence while local pool credentials and sensitive runtime values remain runtime-only inputs.
**Depends on**: Phase 22
**Requirements**: EVD-07, STR-10, REL-09, CFG-07, EVD-09
**Success Criteria** (what must be TRUE):
  1. Operator can run a documented flow for detect, package or flash, local credential input, bounded production mining, telemetry capture, safe stop, redaction, and evidence review.
  2. Committed v1.1 evidence has one redacted evidence root containing package, detector, board-info, command, log, API, WebSocket, share-outcome, redaction-review, safe-stop, and conclusion artifacts.
  3. Evidence records pool credential use only as category labels, never raw pool endpoints, ports, users, workers, owner addresses, passwords, or tokens.
  4. Redaction review covers retained logs, command summaries, API captures, WebSocket captures, NVS/settings values, Stratum fields, share payloads, device URLs, IP addresses, MAC addresses, Wi-Fi values, and pool secrets before commit.
  5. Operator-visible logs and evidence redact pool URLs, ports, workers, owner addresses, passwords, targets, extranonces, share payloads, and socket errors.
**Plans**: 4 plans
Plans:
- [x] 23-01-PLAN.md — Define the Phase 23 evidence-root contract and required slot artifacts.
- [x] 23-02-PLAN.md — Add the operator evidence-root validator and deterministic redaction checks.
- [x] 23-03-PLAN.md — Create the just-reachable detector-gated operator workflow and synthetic tests.
- [x] 23-04-PLAN.md — Integrate final redaction review, release docs, checklist rows, and validation closure.

### Phase 24: BM1366 Production Work Path
**Goal**: The functional core models trusted BM1366 production work decisions while the firmware shell initializes Ultra 205 BM1366 hardware, dispatches pool-derived work, parses live results, and fails closed on unsafe or uncorrelated ASIC behavior.
**Depends on**: Phase 23
**Requirements**: ASIC-09, ASIC-10, ASIC-11, ASIC-12
**Success Criteria** (what must be TRUE):
  1. Operator can see diagnostic chip/work modes reported separately from trusted BM1366 production initialization and work-result modes.
  2. Firmware dispatches BM1366 work derived from the active pool job and tracks job, extranonce, and difficulty context.
  3. Firmware invalidates stale BM1366 work on clean-jobs or reconnect before another share claim can be recorded.
  4. Firmware maps live BM1366 nonce or result observations back to active pool work before any share submission claim is recorded.
  5. Firmware fails closed on BM1366 initialization, UART, reset, timeout, malformed result, or job-correlation failures without leaking raw frames in committed evidence.
**Plans**: 4 plans
Plans:
- [x] 24-01-PLAN.md — Create BM1366 production ASIC primitives and redaction-safe blocker taxonomy.
- [x] 24-02-PLAN.md — Add a session-generation production active-work registry.
- [ ] 24-03-PLAN.md — Correlate BM1366 results and wire guarded production dispatch/status.
- [ ] 24-04-PLAN.md — Close Phase 24 evidence, checklist rows, and validation metadata.

### Phase 25: Live Stratum Runtime And Safe Stop
**Goal**: A watchdog-responsive firmware shell drives a pure Stratum v1 production runtime with real TCP socket I/O, live ASIC-derived submit behavior, deterministic fake-pool coverage, and bounded safe-stop postconditions.
**Depends on**: Phase 24
**Requirements**: STR-08, STR-09, STR-11, SAFE-12, SAFE-13
**Success Criteria** (what must be TRUE):
  1. Operator can observe real Stratum v1 connect, subscribe, authorize, difficulty/extranonce, notify, submit, response classification, reconnect, and safe-stop lifecycle events.
  2. Tests prove deterministic fake-pool or fixture behavior for subscribe, authorize, notify, clean-jobs, submit response, reconnect, fallback, and error classification.
  3. Operator can observe at least one real pool response to a live ASIC-derived `mining.submit` classified as accepted or rejected, or an explicit safe-prerequisite blocker.
  4. Operator can stop production mining into a bounded safe state with socket activity stopped, work queues drained or invalidated, mining disabled, hardware control disabled, and post-stop API/WebSocket state updated.
  5. Operator can verify watchdog responsiveness under bounded socket, ASIC, API, WebSocket, and evidence-capture load.
**Plans**: TBD

### Phase 26: Telemetry And Parity Closure
**Goal**: API, WebSocket, statistics, scoreboard, and parity checklist projections are derived from the same v1.1 runtime events and promote only exact claims proven by redacted Ultra 205 artifacts.
**Depends on**: Phase 25
**Requirements**: API-11, API-12, API-13, EVD-08
**Success Criteria** (what must be TRUE):
  1. Operator can query `/api/system/info`, `/api/system/statistics`, and `/api/system/scoreboard` for mining state, counters, hashrate inputs, share outcomes, and post-stop state derived from v1.1 runtime events.
  2. Operator can observe `/api/ws` and `/api/ws/live` stream redacted, session-correlated mining telemetry without stale active-mining state after stop.
  3. Statistics, scoreboard, and share counters do not advance unless corresponding runtime events and parsed pool responses exist.
  4. Parity checklist updates promote only exact v1.1 claims proven by artifacts and preserve explicit non-claims for deferred surfaces.
**Plans**: TBD

## Progress

**Execution Order:** Phase 22 -> Phase 23 -> Phase 24 -> Phase 25 -> Phase 26

| Phase | Milestone | Plans Complete | Status | Completed |
|-------|-----------|----------------|--------|-----------|
| 22. Claim Ladder And Safety Preconditions | v1.1 | 3/3 | Complete    | 2026-07-04 |
| 23. Redacted Operator Evidence Workflow | v1.1 | 4/4 | Complete    | 2026-07-04 |
| 24. BM1366 Production Work Path | v1.1 | 2/4 | In Progress|  |
| 25. Live Stratum Runtime And Safe Stop | v1.1 | 0/TBD | Not started | - |
| 26. Telemetry And Parity Closure | v1.1 | 0/TBD | Not started | - |

## Coverage

All 21 v1.1 requirements are mapped exactly once across Phases 22-26. Non-205 boards, non-BM1366 ASIC families, full active voltage/fan/thermal/fault/self-test closure, OTAWWW/recovery destructive or fault-injection evidence, runtime display/input/BAP, Stratum v2, and unbounded stress mining remain deferred.
