# Roadmap: Bitaxe Rust Firmware

## Overview

v1.1 turns the shipped Ultra 205 v1.0 controlled no-share mining foundation into a trusted, bounded Stratum v1 production mining path. The milestone starts at Phase 22 to continue v1.0's completed Phase 1-21 sequence and remains scoped to Ultra 205 BM1366 only: exact-claim evidence governance, prerequisite safety gates, redacted operator evidence, BM1366 production work, real Stratum v1 socket/share behavior, runtime telemetry, safe stop, and parity promotion for only the claims proven by v1.1 artifacts.

## Milestones

- [x] **v1.0 Ultra 205 Parity** - Phases 1-21 shipped 2026-07-04; full archive: `.planning/milestones/v1.0-ROADMAP.md`.
- [ ] **v1.1 Ultra 205 Trusted Production Mining** - Phases 22-30 for bounded Ultra 205 BM1366 Stratum v1 production mining, hardware evidence promotion, and audit gap closure.

## Phases

**Phase Numbering:**
- Integer phases (22-28): planned milestone work continuing from v1.0.
- Decimal phases (22.1, 22.2): urgent insertions if needed between planned phases.

- [x] **Phase 22: Claim Ladder And Safety Preconditions** - Operators get exact v1.1 claim boundaries and prerequisite safety gates before production work can start. (completed 2026-07-04)
- [x] **Phase 23: Redacted Operator Evidence Workflow** - Operators can run the repo-owned mining evidence flow without committing secrets or unsupported claims. (completed 2026-07-04)
- [x] **Phase 24: BM1366 Production Work Path** - The firmware separates diagnostic ASIC behavior from trusted pool-derived BM1366 work and result handling. (completed 2026-07-05)
- [x] **Phase 25: Live Stratum Runtime And Safe Stop** - Ultra 205 can run a real watchdog-responsive Stratum v1 mining session and stop safely. (completed 2026-07-05)
- [x] **Phase 26: Telemetry And Parity Closure** - Runtime API, WebSocket, counters, and parity checklist updates reflect only proven v1.1 mining events. (completed 2026-07-05)
- [x] **Phase 27: Live Hardware ASIC And Stratum Bridge** - Live firmware wires Phase 24 BM1366 production dispatch and nonce correlation into the Phase 25 socket loop and produces detector-gated share-outcome evidence. (completed 2026-07-05)
- [x] **Phase 28: Hardware Evidence And Checklist Promotion** - Redacted hardware evidence promotes only exact verified checklist rows supported by Phase 27 artifacts. (completed 2026-07-06)
- [ ] **Phase 29: Evidence Workflow Automation Closure** - Phase 25/27/28 evidence wrappers auto-validate operator evidence roots and close the redact-validate-promote flow without manual consolidation steps.
- [ ] **Phase 30: Live Share Outcome And Verified Promotion** - Promote STR-09/CFG-07 to `verified` using Phase 28.1.1.2 (or later) share-outcome evidence; close Nyquist metadata (no duplicate wire-diff work).

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
- [x] 24-03-PLAN.md — Correlate BM1366 results and wire guarded production dispatch/status.
- [x] 24-04-PLAN.md — Close Phase 24 evidence, checklist rows, and validation metadata.

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
**Plans**: 3 plans
Plans:
- [x] 25-01-PLAN.md — Create the pure live Stratum runtime, submit classifier, and deterministic fake-pool coverage.
- [x] 25-02-PLAN.md — Wire the firmware live socket adapter with safe-stop and watchdog postconditions.
- [x] 25-03-PLAN.md — Add detector-gated evidence tooling and exact parity closure.

### Phase 26: Telemetry And Parity Closure
**Goal**: API, WebSocket, statistics, scoreboard, and parity checklist projections are derived from the same v1.1 runtime events and promote only exact claims proven by redacted Ultra 205 artifacts.
**Depends on**: Phase 25
**Requirements**: API-11, API-12, API-13, EVD-08
**Success Criteria** (what must be TRUE):
  1. Operator can query `/api/system/info`, `/api/system/statistics`, and `/api/system/scoreboard` for mining state, counters, hashrate inputs, share outcomes, and post-stop state derived from v1.1 runtime events.
  2. Operator can observe `/api/ws` and `/api/ws/live` stream redacted, session-correlated mining telemetry without stale active-mining state after stop.
  3. Statistics, scoreboard, and share counters do not advance unless corresponding runtime events and parsed pool responses exist.
  4. Parity checklist updates promote only exact v1.1 claims proven by artifacts and preserve explicit non-claims for deferred surfaces.
**Plans**: 4 plans
Plans:
- [x] 26-01-PLAN.md — Create the shared runtime-event projection and sample-marker contract.
- [x] 26-02-PLAN.md — Add pure API/WebSocket projection view builders.
- [x] 26-03-PLAN.md — Wire Phase 25 runtime producers and firmware consumers into the projection.
- [x] 26-04-PLAN.md — Close Phase 26 evidence, checklist guardrails, and validation metadata.

### Phase 27: Live Hardware ASIC And Stratum Bridge
**Goal**: Ultra 205 live production firmware bridges Phase 24 BM1366 production dispatch and nonce correlation into the Phase 25 Stratum socket loop and records detector-gated redacted share-outcome evidence.
**Depends on**: Phase 26
**Requirements**: STR-08, STR-09, ASIC-10, ASIC-11
**Gap Closure**: Closes audit integration gap Phase 24 → Phase 25 and live-hardware ASIC/Stratum tech debt.
**Success Criteria** (what must be TRUE):
  1. Live firmware dispatches `Bm1366ProductionCommand` from pool-derived work inside the Phase 25 socket runtime path.
  2. Live firmware feeds `ProductionNonceObservation` back into runtime correlation and submit classification.
  3. Detector-gated hardware run records redacted share-outcome evidence as accepted, rejected, or an explicit safe-prerequisite blocker.
  4. Committed evidence preserves exact non-claims when hardware prerequisites block live share proof.
**Plans**: 4 plans
Plans:
- [x] 27-01-PLAN.md — Wire live firmware production command dispatch into `live_stratum_runtime`.
- [x] 27-02-PLAN.md — Bridge ASIC nonce observations into production correlation and submit intent emission.
- [x] 27-03-PLAN.md — Add detector-gated hardware evidence wrapper and redacted share-outcome artifacts.
- [x] 27-04-PLAN.md — Close Phase 27 evidence, tests, and validation metadata.

### Phase 28: Hardware Evidence And Checklist Promotion
**Goal**: Parity checklist rows promote to `verified` only where Phase 27 redacted hardware evidence supports exact claims; all other surfaces remain conservative non-claims.
**Depends on**: Phase 27
**Requirements**: SAFE-10, SAFE-11, SAFE-12, SAFE-13, CFG-07, ASIC-09, ASIC-12
**Gap Closure**: Closes audit tech debt for checklist rows below `verified` where hardware evidence is required.
**Success Criteria** (what must be TRUE):
  1. Operator evidence root consolidates Phase 27 hardware artifacts with required redaction review.
  2. Checklist rows for SAFE-10, SAFE-11, STR-08, STR-09, SAFE-12, SAFE-13, CFG-07, and ASIC-09 through ASIC-12 promote only to statuses supported by committed evidence.
  3. `just parity` rejects overbroad verified promotion without matching evidence artifacts.
  4. Explicit non-claims remain for deferred active safety, OTAWWW/recovery, non-205 boards, Stratum v2, UI/BAP, and unbounded stress.
**Plans**: 3 plans
Plans:
- [x] 28-01-PLAN.md — Consolidate Phase 27 hardware evidence into the operator evidence root contract.
- [x] 28-02-PLAN.md — Promote conservative checklist rows from Phase 27 evidence only.
- [x] 28-03-PLAN.md — Close Phase 28 verification, parity guardrails, and validation metadata.

### Phase 29: Evidence Workflow Automation Closure
**Goal**: Ultra 205 operators can run Phase 25, Phase 27, and Phase 28 evidence workflows end-to-end with automated `operator-evidence` validation and no manual consolidation gap between partial and full evidence roots.
**Depends on**: Phase 28.1
**Requirements**: EVD-07, EVD-08, EVD-09, REL-09
**Gap Closure**: Closes audit integration gaps Phase 23 → Phase 25/27 and Phase 27 → Phase 28; closes partial Redact → validate → parity promote flow.
**Success Criteria** (what must be TRUE):
  1. Phase 25 and Phase 27 evidence wrappers invoke `parity operator-evidence --require-redaction-passed` at workflow end.
  2. Operator can run `just phase28-evidence` to consolidate Phase 27 artifacts into a full Phase 23 slot inventory and pass operator-evidence validation.
  3. REL-09 operator flow documentation and regression tests cover the automated validate step for Phase 25, Phase 27, and Phase 28 roots.
  4. `just parity` and existing redaction guards continue to reject overbroad promotion without matching evidence artifacts.
**Plans**: 0 plans

### Phase 30: Live Share Outcome And Verified Promotion
**Goal**: Promote STR-09/CFG-07 checklist rows to `verified` only where Phase 28.1.1.1 redacted hardware evidence supports accepted/rejected share outcomes; close Phase 28.1 Nyquist metadata. Does not re-do wire-byte diff work (owned by Phase 28.1.1.1).
**Depends on**: Phase 28.1.1.6 (or later phase with share-outcome evidence), Phase 29
**Requirements**: STR-09, CFG-07, ASIC-11
**Gap Closure**: Closes audit tech debt for STR-09/CFG-07 below `verified` after Phase 28.1.1.1 share-outcome evidence exists.
**Success Criteria** (what must be TRUE):
  1. Committed Phase 30 evidence cites Phase 28.1.1.1 share-outcome artifacts (accepted/rejected) with redaction review complete.
  2. STR-09 and CFG-07 checklist rows promote to `verified` only when committed evidence supports exact claims; parity validator rejects overbroad promotion.
  3. Phase 28.1 Nyquist `wave_0_complete` metadata is closed after promotion evidence is captured.
  4. Explicit non-claims remain for full active safety, OTAWWW/recovery, non-205 boards, Stratum v2, UI/BAP, and unbounded stress.
**Plans**: 0 plans

## Progress

**Execution Order:** Phase 22 -> Phase 23 -> Phase 24 -> Phase 25 -> Phase 26 -> Phase 27 -> Phase 28 -> Phase 28.1 -> Phase 28.1.1 -> Phase 28.1.1.1 -> Phase 28.1.1.2 -> Phase 28.1.1.3 -> Phase 28.1.1.4 -> Phase 28.1.1.5 -> Phase 28.1.1.6 -> Phase 29 -> Phase 30

| Phase | Milestone | Plans Complete | Status | Completed |
|-------|-----------|----------------|--------|-----------|
| 22. Claim Ladder And Safety Preconditions | v1.1 | 3/3 | Complete    | 2026-07-04 |
| 23. Redacted Operator Evidence Workflow | v1.1 | 4/4 | Complete    | 2026-07-04 |
| 24. BM1366 Production Work Path | v1.1 | 4/4 | Complete    | 2026-07-05 |
| 25. Live Stratum Runtime And Safe Stop | v1.1 | 3/3 | Complete    | 2026-07-05 |
| 26. Telemetry And Parity Closure | v1.1 | 4/4 | Complete    | 2026-07-05 |
| 27. Live Hardware ASIC And Stratum Bridge | v1.1 | 4/4 | Complete   | 2026-07-05 |
| 28. Hardware Evidence And Checklist Promotion | v1.1 | 3/3 | Complete   | 2026-07-06 |
| 28.1. Live Mining Blocker Fix (H4/W13 + Probes) | v1.1 | 5/5 | Complete    | 2026-07-07 |
| 28.1.1. BM1366 Nonce Production Wire Parity | v1.1 | 5/5 | Gaps Found | 2026-07-08 |
| 28.1.1.1. BM1366 Upstream Golden Comparator And Nonce-Production Gap Reconciliation | v1.1 | 5/5 | Gaps Found | 2026-07-08 |
| 28.1.1.2. BM1366 Result-Path And ASIC Side-Effect Nonce-Production Diagnosis | v1.1 | 4/4 | Gaps Found | 2026-07-09 |
| 28.1.1.3. BM1366 Result RX Acquisition Model Nonce-Production Diagnosis | v1.1 | 4/4 | Gaps Found | 2026-07-09 |
| 28.1.1.4. BM1366 ASIC Init-Content Sequencing Nonce-Production Diagnosis | v1.1 | 4/4 | Gaps Found | 2026-07-09 |
| 28.1.1.5. BM1366 Match Upstream Chip-Enumerate Before Init Nonce-Production Diagnosis | v1.1 | 4/4 | Gaps Found | 2026-07-09 |
| 28.1.1.6. BM1366 Version-Rolling Negotiation Nonce-Production Diagnosis | v1.1 | 0/4 | Not started | — |
| 29. Evidence Workflow Automation Closure | v1.1 | 0/3 | Not started | — |
| 30. Live Share Outcome And Verified Promotion | v1.1 | 0/4 | Not started | — |

## Coverage

All 21 v1.1 requirements are mapped across Phases 22-28 with gap-closure reinforcement in Phases 29-30. Non-205 boards, non-BM1366 ASIC families, full active voltage/fan/thermal/fault/self-test closure, OTAWWW/recovery destructive or fault-injection evidence, runtime display/input/BAP, Stratum v2, and unbounded stress mining remain deferred.

### Phase 28.1: live mining blocker fix (H4/W13 orchestration parity + discriminating hardware probes) (INSERTED)

**Goal:** The Ultra 205 produces post-dispatch BM1366 UART evidence (register-read proof and/or nonces) under upstream-parity default orchestration, or the blocker is conclusively isolated to hardware via discriminating probes and the approved upstream A/B fallback
**Requirements**: None formally mapped (inserted urgent phase); STR-09 and CFG-07 promotion prerequisites are context only — no checklist promotion is claimed by this phase.
**Depends on:** Phase 28
**Plans:** 5/5 plans complete

Plans:
- [x] 28.1-01-PLAN.md — Pure core: BridgeOrchestrator with J2c dispatch-priority regression tests, regenerate_work + extranonce2 counter, bm1366_job_interval_ms
- [x] 28.1-02-PLAN.md — Firmware defaults: W13 unconditional post-init retention and bridge pump delegation to BridgeOrchestrator (timeout=continue, keep mining)
- [x] 28.1-03-PLAN.md — Discriminating probes: post-init/post-dispatch register-read probe and INA260 power-delta marker with I2C bus retention
- [x] 28.1-04-PLAN.md — Flag disposition: retire obsolete investigation modes, add single_dispatch_bounded_read control lever, fail-closed default proof
- [x] 28.1-05-PLAN.md — Hardware evidence: parity-default success-ladder run, single-dispatch control run, conditional upstream A/B with documented recovery, blocker disposition

### Phase 28.1.1: BM1366 nonce production wire parity (INSERTED)

**Goal:** Fix `firmware-nonce-production` by diffing upstream golden UART bytes against Rust TX, correcting init/job wire divergences, and verifying `result_correlated` plus live accepted/rejected shares on Ultra 205.
**Depends on:** Phase 28.1
**Requirements:** STR-09, CFG-07 (blocker closure)
**Plans:** 5/5 plans executed; verification `gaps_found`

Plans:
- [x] 28.1.1-01-PLAN.md — Upstream SERIALTX_DEBUG golden wire capture + recovery
- [x] 28.1.1-02-PLAN.md — Rust TX trace + diff tooling + J3-equivalent capture
- [x] 28.1.1-03-PLAN.md — Dynamic init frame fixes + register-read parser + fixture tests
- [x] 28.1.1-04-PLAN.md — Job frame field parity fixes with per-hypothesis hardware runs
- [x] 28.1.1-05-PLAN.md — Version-rolling fallback + redacted evidence + verification

### Phase 28.1.1.1: BM1366 upstream golden comparator and nonce-production gap reconciliation (INSERTED)

**Goal:** Reconcile the Phase 28.1.1 `gaps_found` result by capturing upstream `SERIALTX_DEBUG`/`SERIALRX_DEBUG` golden job bytes, comparing them field-by-field against Rust init/job frames, patching only confirmed divergences, and rerunning detector-gated Ultra 205 hardware evidence until either nonce/share evidence exists or the blocker is narrowed below job construction.
**Requirements**: STR-09, CFG-07, ASIC-11 (blocker closure input only; Phase 30 owns checklist promotion)
**Depends on:** Phase 28.1.1
**Plans:** 5/5 plans executed; verification `gaps_found`

Plans:
- [x] 28.1.1.1-01-PLAN.md — Redaction-safe BM1366 UART/job-frame field comparator + fixture tests
- [x] 28.1.1.1-02-PLAN.md — Upstream golden capture helper + detector-gated Rust/upstream A/B
- [x] 28.1.1.1-03-PLAN.md — Source-work alignment + divergence disposition (job fields match; no patch)
- [x] 28.1.1.1-04-PLAN.md — Deterministic fake-pool source-work capture proving 11/11 job-field match
- [x] 28.1.1.1-05-PLAN.md — Below-job-byte sequence comparator + post_max_baud_delay_2000 A/B (still no Rust submit)

**Gaps found:** Job bytes match under aligned source work; below-job semantic events match 12/12; Rust still lacks `result_correlated` and share-submission evidence. Next diagnostic layer is result-path / ASIC side effects (not more job-byte patches).

### Phase 28.1.1.2: BM1366 result-path and ASIC side-effect nonce-production diagnosis (INSERTED)

**Goal:** Isolate and close the remaining `firmware-nonce-production` gap below job bytes by comparing result-path / ASIC runtime side effects against upstream (continuous RX / chip-id polling vs Rust result-read polling, enable/power/reset sequencing, and missing upstream runtime side effects), then patch only a confirmed divergence and rerun detector-gated Ultra 205 evidence until `result_correlated` plus fake-pool/live share submit appear, or the blocker is narrowed further with redacted evidence.
**Requirements**: STR-09, CFG-07, ASIC-11 (blocker closure input only; Phase 30 owns checklist promotion)
**Depends on:** Phase 28.1.1.1
**Plans:** 4/4 plans complete

Plans:
- [x] 28.1.1.2-01-PLAN.md — Wave 0 result-path comparator + D-05 fixture tests
- [x] 28.1.1.2-02-PLAN.md — Diagnostic wrapper + baseline compare + hardware A/B `match_upstream_register_read_poll`
- [x] 28.1.1.2-03-PLAN.md — Optional firmware patch only if A/B confirms divergence; else disposition
- [x] 28.1.1.2-04-PLAN.md — Final redacted evidence + verification/validation closure

**Gaps found:** Register-read-poll A/B regressed; disposition falsified default promotion; no `result_correlated` / fake-pool submit; next_hypothesis=`result_rx_acquisition_model`. Phase 30 promotion pending. Post-fix board-info re-run blocked.


### Phase 28.1.1.3: BM1366 result RX acquisition model nonce-production diagnosis (INSERTED)

**Goal:** Isolate whether Rust short-poll `try_read_exact` + partial-frame `clear_rx` discards BM1366 nonce bytes versus upstream long-block `SERIAL_rx`/`receive_work`, then patch only a confirmed RX-acquisition divergence and rerun detector-gated Ultra 205 fake-pool evidence until `result_correlated` plus share submit appear, or narrow the blocker further with redacted evidence.
**Requirements**: STR-09, CFG-07, ASIC-11 (blocker closure input only; Phase 30 owns checklist promotion)
**Depends on:** Phase 28.1.1.2
**Plans:** 4/4 plans executed

Plans:
- [x] 28.1.1.3-01-PLAN.md — Wave 0: RX acquisition comparator + tests (+ optional compact firmware counters)
- [x] 28.1.1.3-02-PLAN.md — Diagnostic wrapper + baseline metrics + hardware A/B `upstream_like_long_block_receive`
- [x] 28.1.1.3-03-PLAN.md — Optional default patch only if improved; else falsified disposition → next hypothesis asic_enable_power_sequencing
- [x] 28.1.1.3-04-PLAN.md — Final redacted evidence + VERIFICATION + VALIDATION/STATE/ROADMAP closure

**Gaps found:** Long-block RX A/B `ab_outcome: unchanged`; disposition `falsified_upstream_like_long_block_receive`; no `result_correlated` / fake-pool submit; `next_hypothesis=asic_enable_power_sequencing`. Phase 30 promotion pending.


### Phase 28.1.1.4: BM1366 ASIC init-content sequencing nonce-production diagnosis (INSERTED)

**Goal:** Isolate the deferred `asic_enable_power_sequencing` gap as init/content sequencing that leaves the BM1366 UART-alive but not hashing (enable/voltage/reset markers already present). First lever: golden byte diff of dynamic mining-ready frames regs `0x14` / `0x08` / `0x10` vs upstream SERIALTX, using `asic_probe=power_delta` as fast feedback; patch only a confirmed divergence and rerun detector-gated Ultra 205 fake-pool evidence until `result_correlated` plus share submit appear, or narrow the blocker (e.g. chip-enumerate) with redacted evidence.
**Requirements**: STR-09, CFG-07, ASIC-11 (blocker closure input only; Phase 30 owns checklist promotion)
**Depends on:** Phase 28.1.1.3
**Plans:** 4/4 plans complete

Plans:
- [x] 28.1.1.4-01-PLAN.md — Wave 0 init-sequencing comparator + tests (+ optional compact summary)
- [x] 28.1.1.4-02-PLAN.md — Ticket-mask ASIC-256 pure-crate patch + fixture + diagnostic wrapper + ≥360s A/B
- [x] 28.1.1.4-03-PLAN.md — Promote default only if improved; else falsify → chip-enumerate next
- [x] 28.1.1.4-04-PLAN.md — Final evidence + VERIFICATION + VALIDATION/STATE/ROADMAP

**Gaps found:** Ticket-mask A/B `ab_outcome: unchanged`; disposition `falsified_ticket_mask_asic_difficulty_as_sole_blocker`; ASIC-256 `wire_parity_retained: true`; no `result_correlated` / fake-pool submit; `next_hypothesis=match_upstream_chip_enumerate_before_init`. Phase 30 promotion pending.

### Phase 28.1.1.5: BM1366 match upstream chip-enumerate before init nonce-production diagnosis (INSERTED)

**Goal:** Isolate whether Rust chip-detect / address-split sequencing before BM1366 init diverges from upstream `55 AA 52 05 00 00 0A` enumerate + `count_asic_chips` + chip-address assignment in a way that leaves cores UART-alive but idle. Patch only a confirmed enumerate/sequencing divergence; run detector-gated Ultra 205 fake-pool evidence until `result_correlated` plus share submit appear, or narrow the blocker with redacted evidence.
**Requirements**: STR-09, CFG-07, ASIC-11 (blocker closure input only; Phase 30 owns checklist promotion)
**Depends on:** Phase 28.1.1.4
**Plans:** 4/4 plans complete

Plans:
- [x] 28.1.1.5-01-PLAN.md — Wave 0 chip-enumerate comparator + fixture tests (`forced_ab_label: count_asic_chips_rx_loop_parity`)
- [x] 28.1.1.5-02-PLAN.md — RX-loop + counted-handoff patch + detector-gated ≥360s fake-pool A/B
- [x] 28.1.1.5-03-PLAN.md — Patch disposition: promote only if improved; else next_hypothesis `version_rolling_negotiation`
- [x] 28.1.1.5-04-PLAN.md — Final redacted evidence + VERIFICATION + VALIDATION/STATE/ROADMAP close

**Gaps found:** RX-loop A/B `ab_outcome: unchanged`; disposition `falsified_count_asic_chips_rx_loop_parity_as_sole_blocker`; `wire_parity_rx_loop_retained: true` + ASIC-256 ticket mask retained; no `result_correlated` / fake-pool submit; `next_hypothesis=version_rolling_negotiation`. Phase 30 promotion pending.

### Phase 28.1.1.6: BM1366 version-rolling negotiation nonce-production diagnosis (INSERTED)

**Goal:** Isolate whether incomplete Stratum version-rolling semantics—`mining.configure` response handling, negotiated mask application to work generation, and ASIC version-mask runtime alignment—leave BM1366 cores UART-alive but idle after enumerate wire parity. First lever: apply pool-negotiated version mask in work-field construction (`negotiated_version_mask_work_field_parity`); patch only a confirmed divergence and rerun detector-gated Ultra 205 fake-pool evidence until `result_correlated` plus share submit appear, or narrow the blocker with redacted evidence.
**Requirements**: STR-09, CFG-07, ASIC-11 (blocker closure input only; Phase 30 owns checklist promotion)
**Depends on:** Phase 28.1.1.5
**Plans:** 0 plans

Plans:
- [ ] TBD (run /gsd-plan-phase 28.1.1.6 to break down)
