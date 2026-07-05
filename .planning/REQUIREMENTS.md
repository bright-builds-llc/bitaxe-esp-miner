# Requirements: Bitaxe Rust Firmware v1.1

**Defined:** 2026-07-04
**Milestone:** v1.1 Ultra 205 Trusted Production Mining
**Core Value:** A Bitaxe owner can build, flash, run, configure, monitor, and update Rust firmware on real Bitaxe hardware with the same observable behavior they expect from upstream ESP-Miner.

## v1.1 Requirements

Requirements for the current milestone. Each maps to roadmap phases and must be verified before promotion.

### Evidence Governance

- [x] **EVD-06**: An Ultra 205 operator can distinguish v1.0 controlled no-share evidence from v1.1 live production mining claims through a documented claim ladder.
- [ ] **EVD-07**: Committed v1.1 evidence records a single redacted evidence root with package, detector, board-info, command, log, API, WebSocket, share-outcome, redaction-review, safe-stop, and conclusion artifacts.
- [ ] **EVD-08**: Committed parity checklist updates promote only exact claims proven by v1.1 artifacts and preserve explicit non-claims for deferred surfaces.

### Mining Prerequisite Safety

- [x] **SAFE-10**: Ultra 205 production mining requires fresh or explicitly bounded power, thermal, fan, voltage, and safety observations before BM1366 work dispatch is enabled.
- [x] **SAFE-11**: Ultra 205 production mining fails closed with user-visible blocker reasons when safety prerequisites are stale, unavailable, unsafe, ambiguous, or undocumented.
- [ ] **SAFE-12**: Ultra 205 production mining can stop in a bounded safe state with socket activity stopped, work queues drained or invalidated, mining disabled, hardware control disabled, and post-stop API/WebSocket state updated.
- [ ] **SAFE-13**: Ultra 205 production mining preserves watchdog responsiveness under bounded socket, ASIC, API, WebSocket, and evidence-capture load.

### Stratum v1 Production Runtime

- [ ] **STR-08**: Ultra 205 production mining uses a real Stratum v1 TCP socket lifecycle for connect, subscribe, authorize, difficulty/extranonce, notify, submit, response classification, reconnect, and safe stop.
- [ ] **STR-09**: Ultra 205 production mining classifies at least one real pool response to a live ASIC-derived `mining.submit` as accepted, rejected, or explicitly blocked by a safe prerequisite.
- [ ] **STR-10**: Ultra 205 production mining redacts pool URLs, ports, workers, owner addresses, passwords, targets, extranonces, share payloads, and socket errors in committed logs and evidence.
- [ ] **STR-11**: Ultra 205 production mining has deterministic fake-pool or fixture tests for subscribe, authorize, notify, clean-jobs, submit response, reconnect, fallback, and error classification behavior.

### BM1366 Production Work Path

- [x] **ASIC-09**: Ultra 205 production mining separates BM1366 diagnostic chip/work modes from trusted production initialization and work-result modes.
- [x] **ASIC-10**: Ultra 205 production mining dispatches BM1366 work derived from the active pool job, tracks job/extranonce/difficulty context, and invalidates stale work on clean-jobs or reconnect.
- [ ] **ASIC-11**: Ultra 205 production mining maps live BM1366 nonce or result observations back to active pool work before any share submission claim is recorded.
- [x] **ASIC-12**: Ultra 205 production mining fails closed on BM1366 initialization, UART, reset, timeout, malformed result, or job-correlation failures without leaking raw frames in committed evidence.

### Operator Workflow And Secret Handling

- [ ] **REL-09**: An Ultra 205 operator can run a documented repo-owned flow for detect, package or flash, local credential input, bounded production mining, telemetry capture, safe stop, redaction, and evidence review.
- [ ] **CFG-07**: Local pool credentials are accepted only as runtime inputs and committed evidence records category labels instead of raw pool endpoints, ports, users, workers, owner addresses, passwords, or tokens.
- [ ] **EVD-09**: Redaction tests or review gates cover retained logs, command summaries, API captures, WebSocket captures, NVS/settings values, Stratum fields, share payloads, device URLs, IP addresses, MAC addresses, Wi-Fi values, and pool secrets before evidence is committed.

### Mining Telemetry And API Projection

- [ ] **API-11**: `/api/system/info`, `/api/system/statistics`, and `/api/system/scoreboard` expose mining state, counters, hashrate inputs, share outcomes, and post-stop state derived from the same v1.1 runtime events.
- [ ] **API-12**: `/api/ws` and `/api/ws/live` stream redacted, session-correlated mining telemetry during bounded production mining without stale active-mining state after stop.
- [ ] **API-13**: Statistics, scoreboard, and share counters do not advance without corresponding runtime events and parsed pool responses.

## Future Requirements

Deferred to future milestones. Tracked but not in the current roadmap.

### Active Safety Hardware Closure

- **V2-SAFE-01**: Ultra 205 full active voltage, fan, thermal fault-stimulus, load, recovery, and self-test closure is verified with hardware evidence.

### Release And Recovery Trust

- **V2-REL-09**: OTAWWW, rollback, failed-update, large erase, interrupted-update, and destructive recovery cases are verified with phase-gated recovery plans.

### Boards And ASIC Families

- **V2-BOARD-03**: Non-205 boards receive dedicated trusted mining evidence instead of inheriting Ultra 205 claims.
- **V2-ASIC-02**: Non-BM1366 ASIC families receive dedicated trusted mining evidence instead of inheriting BM1366 claims.

### Protocols And User Interfaces

- **V2-STR-02**: Stratum v2 reaches trusted mining parity after Stratum v1 production mining is stable.
- **V2-UI-02**: Runtime display/input, LVGL-like UI flow, and BAP accessory behavior receive separate parity evidence.

### Soak And Stress

- **V2-MINE-01**: Unbounded or extended production soak and stress mining are verified after v1.1 bounded trusted mining closes safely.

## Out of Scope

Explicitly excluded from v1.1 to prevent scope creep.

| Feature | Reason |
|---------|--------|
| Non-205 boards | v1.1 targets Ultra 205 BM1366 only. Other boards require their own evidence. |
| Non-BM1366 ASIC families | v1.1 trusted mining must not inherit unsupported ASIC claims. |
| Full active voltage, fan, thermal fault-stimulus, load, recovery, and self-test closure | v1.1 needs prerequisite safety gates for mining, not full active safety parity. |
| OTAWWW, rollback, failed-update, large erase, interrupted-update, and destructive recovery cases | Release/update trust remains a separate later milestone. |
| Runtime display/input, LVGL-like UI flow, and BAP | Mining runtime correctness and evidence are higher priority for v1.1. |
| Stratum v2 | v1.1 focuses on trusted Stratum v1 production mining. |
| Unbounded production stress mining | v1.1 must remain bounded, detector-gated, and safe-stop capable. |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| EVD-06 | Phase 22 | Complete |
| EVD-07 | Phase 23 | Pending |
| EVD-08 | Phase 26 | Pending |
| SAFE-10 | Phase 22 | Complete |
| SAFE-11 | Phase 22 | Complete |
| SAFE-12 | Phase 25 | Pending |
| SAFE-13 | Phase 25 | Pending |
| STR-08 | Phase 25 | Pending |
| STR-09 | Phase 25 | Pending |
| STR-10 | Phase 23 | Pending |
| STR-11 | Phase 25 | Pending |
| ASIC-09 | Phase 24 | Complete |
| ASIC-10 | Phase 24 | Complete |
| ASIC-11 | Phase 24 | Pending |
| ASIC-12 | Phase 24 | Complete |
| REL-09 | Phase 23 | Pending |
| CFG-07 | Phase 23 | Pending |
| EVD-09 | Phase 23 | Pending |
| API-11 | Phase 26 | Pending |
| API-12 | Phase 26 | Pending |
| API-13 | Phase 26 | Pending |

**Coverage:**
- v1.1 requirements: 21 total
- Mapped to phases: 21
- Unmapped: 0

**Phase distribution:**
- Phase 22: EVD-06, SAFE-10, SAFE-11
- Phase 23: EVD-07, STR-10, REL-09, CFG-07, EVD-09
- Phase 24: ASIC-09, ASIC-10, ASIC-11, ASIC-12
- Phase 25: STR-08, STR-09, STR-11, SAFE-12, SAFE-13
- Phase 26: EVD-08, API-11, API-12, API-13

*Requirements defined: 2026-07-04*
*Last updated: 2026-07-04 after v1.1 roadmap creation*
