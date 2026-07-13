# Requirements: Bitaxe Rust Firmware v1.1

**Defined:** 2026-07-04
**Milestone:** v1.1 Ultra 205 Trusted Production Mining
**Core Value:** A Bitaxe owner can build, flash, run, configure, monitor, and update Rust firmware on real Bitaxe hardware with the same observable behavior they expect from upstream ESP-Miner.

## v1.1 Requirements

Requirements for the current milestone. Each maps to roadmap phases and must be verified before promotion.

### Evidence Governance

- [x] **EVD-06**: An Ultra 205 operator can distinguish v1.0 controlled no-share evidence from v1.1 live production mining claims through a documented claim ladder.
- [x] **EVD-07**: Committed v1.1 evidence records a single redacted evidence root with package, detector, board-info, command, log, API, WebSocket, share-outcome, redaction-review, safe-stop, and conclusion artifacts.
- [x] **EVD-08**: Committed parity checklist updates promote only exact claims proven by v1.1 artifacts and preserve explicit non-claims for deferred surfaces.

### Mining Prerequisite Safety

- [x] **SAFE-10**: Ultra 205 production mining requires fresh or explicitly bounded power, thermal, fan, voltage, and safety observations before BM1366 work dispatch is enabled.
- [x] **SAFE-11**: Ultra 205 production mining fails closed with user-visible blocker reasons when safety prerequisites are stale, unavailable, unsafe, ambiguous, or undocumented.
- [x] **SAFE-12**: Ultra 205 production mining can stop in a bounded safe state with socket activity stopped, work queues drained or invalidated, mining disabled, hardware control disabled, and post-stop API/WebSocket state updated.
- [x] **SAFE-13**: Ultra 205 production mining preserves watchdog responsiveness under bounded socket, ASIC, API, WebSocket, and evidence-capture load.

### Stratum v1 Production Runtime

- [x] **STR-08**: Ultra 205 production mining uses a real Stratum v1 TCP socket lifecycle for connect, subscribe, authorize, difficulty/extranonce, notify, submit, response classification, reconnect, and safe stop.
- [x] **STR-09**: Ultra 205 production mining classifies at least one real pool response to a live ASIC-derived `mining.submit` as accepted, rejected, or explicitly blocked by a safe prerequisite.
- [x] **STR-10**: Ultra 205 production mining redacts pool URLs, ports, workers, owner addresses, passwords, targets, extranonces, share payloads, and socket errors in committed logs and evidence.
- [x] **STR-11**: Ultra 205 production mining has deterministic fake-pool or fixture tests for subscribe, authorize, notify, clean-jobs, submit response, reconnect, fallback, and error classification behavior.

### BM1366 Production Work Path

- [x] **ASIC-09**: Ultra 205 production mining separates BM1366 diagnostic chip/work modes from trusted production initialization and work-result modes.
- [x] **ASIC-10**: Ultra 205 production mining dispatches BM1366 work derived from the active pool job, tracks job/extranonce/difficulty context, and invalidates stale work on clean-jobs or reconnect.
- [x] **ASIC-11**: Ultra 205 production mining maps live BM1366 nonce or result observations back to active pool work before any share submission claim is recorded.
- [x] **ASIC-12**: Ultra 205 production mining fails closed on BM1366 initialization, UART, reset, timeout, malformed result, or job-correlation failures without leaking raw frames in committed evidence.

### Operator Workflow And Secret Handling

- [x] **REL-09**: An Ultra 205 operator can run a documented repo-owned flow for detect, package or flash, local credential input, bounded production mining, telemetry capture, safe stop, redaction, and evidence review.
- [x] **CFG-07**: Local pool credentials are accepted only as runtime inputs and committed evidence records category labels instead of raw pool endpoints, ports, users, workers, owner addresses, passwords, or tokens.
- [x] **EVD-09**: Redaction tests or review gates cover retained logs, command summaries, API captures, WebSocket captures, NVS/settings values, Stratum fields, share payloads, device URLs, IP addresses, MAC addresses, Wi-Fi values, and pool secrets before evidence is committed.

### Mining Telemetry And API Projection

- [x] **API-11**: `/api/system/info`, `/api/system/statistics`, and `/api/system/scoreboard` expose mining state, counters, hashrate inputs, share outcomes, and post-stop state derived from the same v1.1 runtime events.
- [x] **API-12**: `/api/ws` and `/api/ws/live` stream redacted, session-correlated mining telemetry during bounded production mining without stale active-mining state after stop.
- [x] **API-13**: Statistics, scoreboard, and share counters do not advance without corresponding runtime events and parsed pool responses.

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
| EVD-07 | Phase 23, 29 | Complete |
| EVD-08 | Phase 26, 29 | Complete |
| SAFE-10 | Phase 28 | Complete |
| SAFE-11 | Phase 28 | Complete |
| SAFE-12 | Phase 28 | Complete |
| SAFE-13 | Phase 28 | Complete |
| STR-08 | Phase 27 | Complete |
| STR-09 | Phase 27, 30 | Pending (gap closure) |
| STR-10 | Phase 23 | Complete |
| STR-11 | Phase 25 | Complete |
| ASIC-09 | Phase 28 | Complete |
| ASIC-10 | Phase 27 | Complete |
| ASIC-11 | Phase 27, 30 | Pending (gap closure) |
| ASIC-12 | Phase 28 | Complete |
| REL-09 | Phase 23, 29 | Complete |
| CFG-07 | Phase 28, 30 | Pending (gap closure) |
| EVD-09 | Phase 23, 29 | Complete |
| API-11 | Phase 26 | Complete |
| API-12 | Phase 26 | Complete |
| API-13 | Phase 26 | Complete |

**Coverage:**
- v1.1 requirements: 21 total
- Mapped to phases: 21
- Complete: 18
- Pending (gap closure): 3
- Unmapped: 0

**Phase distribution:**
- Phase 22: EVD-06
- Phase 23: EVD-07, STR-10, REL-09, EVD-09
- Phase 24: (baseline ASIC work — superseded for hardware promotion by Phases 27-28)
- Phase 25: STR-11
- Phase 26: EVD-08, API-11, API-12, API-13
- Phase 27: STR-08, STR-09, ASIC-10, ASIC-11
- Phase 28: SAFE-10, SAFE-11, SAFE-12, SAFE-13, CFG-07, ASIC-09, ASIC-12
- Phase 29: EVD-07, EVD-08, EVD-09, REL-09 (gap closure — workflow automation)
- Phase 30: STR-09, CFG-07, ASIC-11 (gap closure — verified promotion)

*Requirements defined: 2026-07-04*
*Last updated: 2026-07-13 after Phase 29 evidence workflow automation closure*
