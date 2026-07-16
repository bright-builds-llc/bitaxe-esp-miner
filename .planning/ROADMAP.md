# Roadmap: Bitaxe Rust Firmware

## Milestones

- [x] **v1.0 Ultra 205 Parity** — Phases 1–21, shipped 2026-07-04; [archive](./milestones/v1.0-ROADMAP.md).
- [x] **v1.1 Ultra 205 Trusted Production Mining** — Phases 22–30, including inserted Phases 28.1 and 28.1.1–28.1.1.7, shipped 2026-07-13 with 18/21 requirements satisfied and three accepted unresolved gaps; [archive](./milestones/v1.1-ROADMAP.md).
- 🚧 **v1.2 Ultra 205 Operator-Ready Runtime** — Phases 31–35 for truthful read-only telemetry, confirmed hostname persistence, runtime identity and passive health, and exact detector-gated evidence.

## Overview

v1.2 makes one Ultra 205 observably configurable and trustworthy during normal operator use. The milestone builds a typed claim contract before hardware integration, establishes one firmware-owned read-only I2C0 lifecycle, confirms a narrow non-secret setting through NVS reload and reboot, composes device truth into one revisioned operator snapshot, and admits only exact claims supported by one detector-gated hardware evidence chain.

The milestone is observation-only. It prohibits active fan, voltage, reset, power, fault-injection, ASIC-control, and self-test effects; mining and all Phase 28.1.1 lineage work; credential access or promotion; direct UART and pin manipulation; OTA and recovery work; other boards; and broad parity or production-readiness promotion. Phase 35 may promote only explicitly allowlisted operator-runtime rows supported by eligible v1.2 evidence.

## Phases

**Phase numbering:** Integer phases continue after completed Phase 30. Decimal phases are reserved for urgent insertions and are not part of the initial v1.2 roadmap.

- [x] **Phase 31: Operator Claim and Telemetry Contract** — Define truthful observation, settings, health, and promotion states before effectful integration. (completed 2026-07-13)
- [x] **Phase 32: Shared I2C and Read-Only Sensor Acquisition** — Establish one bounded I2C0 owner for startup display handoff and read-only INA260/EMC2101 observations. (completed 2026-07-14)
- [x] **Phase 33: Confirmed Settings Durability** — Make hostname PATCH success mean committed, reloaded, reconciled, and immediately visible storage truth, with a fail-closed classifier ready for later reboot qualification. (completed 2026-07-15 on the remapped software boundary; CFG-12 remains pending for Phase 35)
- [ ] **Phase 34: Provenance, Runtime Health, and Coherent Operator Snapshot** — Publish identity, runtime facts, passive health, settings, and telemetry from one boot session and revisioned snapshot. (9/9 plans implemented; gaps found at 9/10 requirements)
- [ ] **Phase 35: Detector-Gated Correlated Evidence and Exact Parity Promotion** — Prove the completed chain on one Ultra 205 and promote only specifically supported rows.

## Phase Details

### Phase 31: Operator Claim and Telemetry Contract

**Goal:** Operators and downstream code can distinguish truthful observation, availability, health, settings, and parity states before firmware effects or evidence can mint unsupported claims.

**Depends on:** Phase 30

**Requirements:** OBS-01, CFG-08

**Success Criteria** (what must be TRUE):

1. Operator-facing power and thermal facts represent `fresh`, `stale`, `unavailable`, or `fault` independently from compatibility numeric values, so zero never implies freshness.
1. Producer-owned sequence and monotonic acquisition semantics prevent API reads, projections, or consumers from refreshing an observation.
1. The v1.2 settings contract names `hostname` as the complete PATCH allowlist and rejects every broader field from v1.2 promotion.
1. Typed capability and admission boundaries cannot represent active control, self-test effects, mining or Phase 28.1.1 work, credentials, direct-UART/pin work, OTA, other-board, or broad-promotion claims as eligible v1.2 outcomes.

**Plans:** 3/3 plans complete

### Phase 32: Shared I2C and Read-Only Sensor Acquisition

**Goal:** An Ultra 205 operator receives attributable, bounded, read-only power and thermal observations from one firmware-owned I2C0 lifecycle without disrupting startup display behavior.

**Depends on:** Phase 31

**Requirements:** OBS-02, OBS-03, OBS-04, OBS-05

**Success Criteria** (what must be TRUE):

1. Display startup, INA260, and EMC2101 access share one serialized, long-lived I2C0 owner, while the existing startup display remains observable.
1. INA260 current, bus voltage, and power publish only after a successful producer-owned read with a new sequence and monotonic acquisition time.
1. EMC2101 temperature and independently available tachometer facts use read-only transactions and explicitly represent missing or invalid data.
1. One sensor failure does not block the API or unaffected observations, and the failed observation becomes stale or faulted without request-driven refresh.
1. The phase performs no fan/configuration-register, voltage, reset, power, ASIC, fault-stimulus, self-test, mining, credential, direct-UART/pin, OTA, or other-board effect.

**Plans:** 3/3 plans complete

### Phase 33: Confirmed Settings Durability

**Goal:** An Ultra 205 operator receives hostname PATCH success only when firmware storage truth is committed, reloaded, reconciled, and immediately published, while a fail-closed evidence classifier is ready for later exact-current-package reboot qualification.

**Depends on:** Phase 32

**Requirements:** CFG-09, CFG-10, CFG-11, CFG-13

**Success Criteria** (what must be TRUE):

1. Invalid known hostname input fails generically and atomically with no NVS write, commit, live-state replacement, partial change, or hardware effect.
1. Successful hostname PATCH occurs only after serialized write and commit, actual NVS reload, and typed reconciliation against the requested non-secret value.
1. Immediate API readback and the coherent operator snapshot expose the storage-confirmed hostname rather than an optimistic overlay.
1. A fail-closed evidence classifier rejects extra resets, identity or origin ambiguity, stale source/package provenance, ownership or cleanup failures, and unredacted output before later hardware qualification.
1. Unknown and unsupported fields preserve existing compatibility behavior without writes, secrets, credentials, target changes, actuation, raw reset/power operations, mining, direct-UART/pin work, or broader settings promotion.

**Plans:** 3/3 plans complete

**Verification:** Passed on the 8/8 remapped software boundary. The sole `a630455` run remains credible historical non-promotional proof for that exact package only; CFG-12 remains pending for Phase 35 and does not qualify current firmware.

No additional Phase 33 hardware attempt is permitted.

### Phase 34: Provenance, Runtime Health, and Coherent Operator Snapshot

**Goal:** An Ultra 205 operator can inspect one internally coherent snapshot of read-only telemetry, confirmed settings, truthful identity, and passive runtime health across system-info, WebSocket, retained logs, and evidence projections.

**Depends on:** Phase 33

**Requirements:** OBS-06, SYS-01, SYS-02, SYS-03, SYS-04, SYS-05, HLT-01, HLT-02, HLT-03, HLT-04

**Success Criteria** (what must be TRUE):

1. System-info, live-WebSocket, retained-log, and evidence projections bind to the same opaque boot session and monotonic operator-snapshot revision.
1. The running device truthfully reports embedded semantic version and source commit, pinned reference and package identity, ESP-IDF/static-asset identity, board `205`, BM1366, running partition, reset reason, uptime, and heap facts—or an explicit unavailable state.
1. Passive self-test state reports only idle, blocked, running, passed, failed, canceled, or unavailable without starting a hardware self-test submode.
1. Supervisor availability, checkpoint category, sequence, and age become stale or unhealthy when progress stalls, and remain explicitly distinct from unproved ESP task-watchdog participation.
1. No fixture or host-checkout substitution, synthetic placeholder, active watchdog intervention, load/fault experiment, hardware actuation, mining or Phase 28.1.1 work, credential access, direct-UART/pin work, OTA, other-board evidence, or broad promotion occurs.

**Plans:** 9/9 plans complete

- [x] 34-01 — Canonical build identity, LCD/API/log projection, manifest v3, and exact pre-hardware admission
- [x] 34-02 — Coherent boot-session and operator-snapshot revision
- [x] 34-03 — Remaining truthful platform identity and runtime facts
- [x] 34-04 — Passive runtime-health projection
- [x] 34-05 — Structurally bind the selected factory image to admitted package provenance
- [x] 34-06 — Preserve recurring supervisor checkpoints when duplicate yield logs are suppressed
- [x] 34-07 — Serialize snapshot identity, retention, and actual HTTP/WebSocket issuance
- [x] 34-08 — Bind complete admitted ESP32-S3 factory bytes immutably through child execution
- [x] 34-09 — Make concrete retained correlation transactional, fallible, and host-verified

**Verification:** Fresh review and independent verification after Plans 34-08 and 34-09 report `gaps_found` at 9/10 requirements. OBS-06 now passes through the exact atomic, fallible production retention path. SYS-02 remains pending because admission ignores executable entry/load-address fields and does not bind the packaged ELF artifact digest to `app_elf_sha256`; both defects were reproduced through the real dry-run admission path. Phase 35 stays blocked.

### Phase 35: Detector-Gated Correlated Evidence and Exact Parity Promotion

**Goal:** One bounded, detector-gated Ultra 205 evidence chain proves the eligible v1.2 operator-runtime outcomes and deterministically preserves every excluded claim as a non-promotion.

**Depends on:** Phase 34

**Requirements:** CFG-12, EVD-10, EVD-11, EVD-12, EVD-13, EVD-14, EVD-15

**Success Criteria** (what must be TRUE):

1. The evidence workflow stops before target, credential, flash, reset, monitor, or promotion work unless `just detect-ultra205` finds exactly one board `205` candidate and board-info succeeds.
1. One evidence root binds the exact source and reference commits, package manifest and digest, board category, target-lock provenance, boot session, bounded chronology, and correlated operator-snapshot revisions.
1. One final detector-gated exact-current-package run jointly closes CFG-12 and EVD-13 by correlating pre-PATCH, storage-confirmed immediate readback, and the matching storage-confirmed hostname after one approved normal reboot and same-board reacquisition, without recording credentials, network identities, raw targets, or secrets.
1. Inventory, redaction, lifecycle cleanup, no-actuation, reference-cleanliness, and current-head validation all pass before atomic evidence admission.
1. Only explicitly allowlisted operator-runtime parity rows supported by eligible evidence promote; active control, self-test effects, watchdog intervention, mining and Phase 28.1.1, credentials, direct UART/pins, OTA, other boards, and every other excluded or broad claim receive deterministic non-promotion.

**Plans:** TBD

## Dependency Order

```text
Phase 31 claim contract
  -> Phase 32 read-only acquisition
  -> Phase 33 confirmed storage truth
  -> Phase 34 coherent operator snapshot
  -> Phase 35 correlated evidence and exact promotion
```

The order is intentionally evidence-driven: typed claim boundaries precede I/O; the sole telemetry producer precedes settings and snapshot composition; final hardware admission follows the complete software contract. Narrow read-only hardware checks or the approved normal reboot may occur while planning the owning phase, but only Phase 35 may admit final v1.2 parity evidence.

## Progress

| Phase | Name | Requirements | Status |
| --- | --- | ---: | --- |
| 31 | Operator Claim and Telemetry Contract | 2 | Complete |
| 32 | Shared I2C and Read-Only Sensor Acquisition | 4 | Complete |
| 33 | Confirmed Settings Durability | 4 | Complete (8/8 software) |
| 34 | Provenance, Runtime Health, and Coherent Operator Snapshot | 10 | Verification pending (9/9 implemented) |
| 35 | Detector-Gated Correlated Evidence and Exact Parity Promotion | 7 | Not started |

**Overall:** 3/5 phases complete; 18/27 requirements complete.

## Coverage

- v1.2 requirements: 27
- Requirements mapped exactly once: 27
- Unmapped requirements: 0
- Duplicate mappings: 0

*Roadmap created: 2026-07-13*
