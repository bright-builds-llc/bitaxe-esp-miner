# Requirements: Bitaxe Rust Firmware v1.2

**Defined:** 2026-07-13
**Core Value:** A Bitaxe owner can build, flash, run, configure, monitor, and update Rust firmware on real Bitaxe hardware with the same observable behavior they expect from upstream ESP-Miner.

## v1.2 Requirements

Requirements for Ultra 205 Operator-Ready Runtime. Each requirement maps to exactly one roadmap phase.

### Read-Only Observation

- [x] **OBS-01**: An Ultra 205 operator can distinguish every power and thermal observation as `fresh`, `stale`, `unavailable`, or `fault`; compatibility numeric zero values never imply freshness.
- [x] **OBS-02**: An Ultra 205 operator retains the existing startup-display behavior while display, INA260, and EMC2101 access use one bounded, serialized, firmware-owned I2C0 lifecycle with no v1.2 actuator or control-register writes.
- [x] **OBS-03**: An Ultra 205 operator can observe INA260 current, bus voltage, and power values only when a producer-owned sequence and monotonic acquisition time prove a fresh read.
- [x] **OBS-04**: An Ultra 205 operator can observe EMC2101 temperature and independently available tachometer data through read-only transactions, with missing or invalid data represented explicitly.
- [x] **OBS-05**: An Ultra 205 operator can still use the API and unaffected observations when one sensor read fails, and the last affected value becomes stale or failed without being refreshed by a request.
- [x] **OBS-06**: An Ultra 205 operator sees system-info, live-WebSocket, retained-log, and evidence projections derived from the same boot session and monotonic operator-snapshot revision.

### Confirmed Configuration

- [x] **CFG-08**: An Ultra 205 operator can identify `hostname` as the complete v1.2 settings PATCH allowlist; every other field remains unsupported for v1.2 promotion.
- [x] **CFG-09**: An Ultra 205 operator receives a stable generic error for an invalid known `hostname`, with no NVS write, commit, live-state replacement, or partial change.
- [x] **CFG-10**: An Ultra 205 operator receives PATCH success only after the hostname write commits, the firmware performs an actual NVS reload, and typed reconciliation matches the requested non-secret value.
- [x] **CFG-11**: An Ultra 205 operator sees the storage-confirmed hostname in the immediate API readback and coherent operator snapshot after a successful PATCH.
- [ ] **CFG-12**: An Ultra 205 operator sees the same storage-confirmed hostname after one phase-approved normal reboot and reacquisition of the same detector-gated board.
- [x] **CFG-13**: An Ultra 205 operator can send unknown or unsupported settings fields without unintended writes or hardware effects, following the existing compatibility behavior while exposing no secret values.

### Truthful System Identity

- [x] **SYS-01**: An Ultra 205 operator can inspect the running firmware's semantic version and embedded source commit without host-checkout substitution.
- [x] **SYS-02**: An Ultra 205 operator can correlate the running firmware with its pinned reference commit and flashed package identity.
- [x] **SYS-03**: An Ultra 205 operator can inspect truthful ESP-IDF, AxeOS/static-asset, board `205`, BM1366, and running-partition identity.
- [x] **SYS-04**: An Ultra 205 operator can inspect decoded reset reason, uptime, and heap-health facts from the running firmware.
- [x] **SYS-05**: An Ultra 205 operator sees an explicit unavailable state for any identity or runtime fact the firmware cannot prove; live surfaces never substitute fixtures or synthetic placeholders.

### Passive Runtime Health

- [x] **HLT-01**: An Ultra 205 operator can inspect passive self-test lifecycle state as idle, blocked, running, passed, failed, canceled, or unavailable without v1.2 starting a hardware self-test submode.
- [x] **HLT-02**: An Ultra 205 operator can inspect supervisor availability, the latest bounded checkpoint category, checkpoint sequence, and checkpoint age through the coherent operator snapshot.
- [x] **HLT-03**: An Ultra 205 operator can distinguish pure supervisor/checkpoint visibility from actual ESP task-watchdog participation; unproved task-watchdog configuration is reported unavailable.
- [x] **HLT-04**: An Ultra 205 operator sees stalled or over-age checkpoints as stale or unhealthy rather than continuing to receive a healthy status from a one-time startup marker.

### Correlated Hardware Evidence

- [ ] **EVD-10**: A v1.2 hardware run stops before target, credential, flash, reset, monitor, or evidence-promotion work unless `just detect-ultra205` finds exactly one board `205` candidate and board-info succeeds.
- [ ] **EVD-11**: A v1.2 evidence root binds one exact source commit, reference commit, package manifest and digest, board category, boot session, target-lock provenance, and bounded capture chronology.
- [ ] **EVD-12**: A v1.2 evidence root correlates read-only sensor acquisitions with system-info, live-WebSocket, and retained-log projections from the same operator-snapshot revisions.
- [ ] **EVD-13**: A v1.2 evidence root correlates pre-PATCH, committed-and-reloaded, and post-reboot hostname observations without recording credentials, network identities, raw targets, or other secret values.
- [ ] **EVD-14**: A v1.2 evidence root passes complete inventory, redaction, lifecycle cleanup, no-actuation, reference-cleanliness, and current-head validation before atomic promotion.
- [ ] **EVD-15**: Phase 35 promotes only explicitly allowlisted operator-runtime parity rows supported by eligible evidence and records deterministic non-promotion for active control, self-test effects, watchdog intervention, mining, credentials, other boards, and every other excluded claim.

## Future Requirements

Deferred to later milestones and excluded from the v1.2 roadmap.

### Operator Runtime Expansion

- **OBS-07**: Operator can inspect a bounded historical telemetry series after latest-state ownership, cadence, and memory limits are proven.
- **CFG-14**: Operator can durably update additional non-secret, non-actuating settings after each field receives an independent reload, reboot, and target-continuity contract.

### Update and Recovery

- **REL-10**: Operator can complete valid OTA, boot validation, rollback, interrupted-update recovery, recovery-page upload, and OTAWWW flows with exact hardware evidence.

### Active Hardware Control

- **SAFE-14**: Operator can use evidence-backed fan, voltage, thermal protection, reset, power sequencing, and bounded fault handling under explicit recovery contracts.

### Mining Re-Entry

- **ASIC-13**: Operator can receive a correlated live BM1366 nonce/result only after a future milestone supplies genuinely new evidence, a discriminating hypothesis, and a hard stopping rule.
- **STR-12**: Operator can observe an eligible live ASIC-derived share outcome only after nonce/result correlation is independently proven.

### Broader Device Parity

- **FND-12**: Operator can use Gamma 601/BM1370 or another board after the Ultra 205 operator, update, safety, and mining journey is evidence-backed.
- **UI-01**: Operator can use runtime display/input, BAP, or a separately justified UI evolution after the firmware compatibility surfaces are stable.

## Out of Scope

Explicitly excluded from v1.2 to prevent scope and evidence drift.

| Feature | Reason |
| --- | --- |
| Active fan, voltage, reset, power sequencing, ASIC control, or fault injection | Safety-critical effects require a later milestone with recovery and hardware-regression evidence. |
| Hardware self-test execution or watchdog intervention/load experiments | v1.2 exposes passive state only and cannot manufacture stronger health proof. |
| BM1366 nonce/result/share diagnostics or any Phase 28.1.1 descendant | The archived lineage is terminal; re-entry requires genuinely new evidence and a separately approved milestone. |
| CFG-07 runtime-only credential promotion | v1.2 uses only non-secret hostname persistence and does not access or prove Wi-Fi/pool credential handling. |
| Direct UART, pins, pads, GPIO manipulation, probes, jumpers, or injected signals | Prohibited without fresh explicit authorization and outside the v1.2 observation path. |
| OTA, rollback, interrupted update, recovery, or OTAWWW | Reserved for the update-and-recovery milestone. |
| Non-205 boards or non-BM1366 ASIC families | Each device requires its own evidence after the Ultra 205 journey is credible. |
| Runtime display/input parity, BAP, Stratum v2, telemetry history, scoreboard/share statistics, or AxeOS UI rewrite | None is needed to prove v1.2 operator-ready observation, configuration, identity, and passive health. |
| Broad “production ready,” “active safety verified,” or “mining verified” language | v1.2 permits only row-specific claims supported by its exact evidence profile. |

## Traceability

Which phase covers each v1.2 requirement. Populated during roadmap creation.

| Requirement | Phase | Status |
| --- | --- | --- |
| OBS-01 | Phase 31 | Complete |
| OBS-02 | Phase 32 | Complete |
| OBS-03 | Phase 32 | Complete |
| OBS-04 | Phase 32 | Complete |
| OBS-05 | Phase 32 | Complete |
| OBS-06 | Phase 34 | Complete |
| CFG-08 | Phase 31 | Complete |
| CFG-09 | Phase 33 | Complete |
| CFG-10 | Phase 33 | Complete |
| CFG-11 | Phase 33 | Complete |
| CFG-12 | Phase 35 | Pending |
| CFG-13 | Phase 33 | Complete |
| SYS-01 | Phase 34 | Complete |
| SYS-02 | Phase 34 | Complete |
| SYS-03 | Phase 34 | Complete |
| SYS-04 | Phase 34 | Complete |
| SYS-05 | Phase 34 | Complete |
| HLT-01 | Phase 34 | Complete |
| HLT-02 | Phase 34 | Complete |
| HLT-03 | Phase 34 | Complete |
| HLT-04 | Phase 34 | Complete |
| EVD-10 | Phase 35 | Pending |
| EVD-11 | Phase 35 | Pending |
| EVD-12 | Phase 35 | Pending |
| EVD-13 | Phase 35 | Pending |
| EVD-14 | Phase 35 | Pending |
| EVD-15 | Phase 35 | Pending |

**Coverage:**

- v1.2 requirements: 27 total
- Mapped to phases: 27
- Unmapped: 0
- Duplicate mappings: 0

*Requirements defined: 2026-07-13*
*Last updated: 2026-07-17 after Phase 34 passed fresh review and independent verification; OBS-06 and SYS-02 are complete*
