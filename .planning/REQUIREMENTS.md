# Requirements: Bitaxe Rust Firmware

**Defined:** 2026-06-20
**Core Value:** A Bitaxe owner can build, flash, run, configure, monitor, and update Rust firmware on real Bitaxe hardware with the same observable behavior they expect from upstream ESP-Miner.

## v1 Requirements

V1 targets device-user parity for the Bitaxe Ultra 205 with BM1366 ASIC. Other upstream boards, including Gamma 601 with BM1370, remain in parity scope, but they are deferred and not hardware-verified V1 targets unless explicit evidence is added.

### Foundation And Workflow

- [x] **FND-01**: The repo includes upstream ESP-Miner as a pinned git submodule at `reference/esp-miner`.
- [x] **FND-02**: Normal project workflows fail when `reference/esp-miner` is missing, unpinned, or locally modified.
- [x] **FND-03**: Bazel/Bzlmod is the canonical automation graph for build, test, package, flash-shaped, parity, and release-shaped workflows.
- [x] **FND-04**: The Rust workspace pins the ESP-IDF Rust toolchain, ESP-IDF version, Rust target, firmware metadata, and dependency versions needed for Ultra 205 firmware builds.
- [x] **FND-05**: The monorepo contains the planned pure Rust crates for core state, config, ASIC, Stratum, API, and test support.
- [x] **FND-06**: The ESP-IDF Rust firmware app can boot on Ultra 205 and log firmware identity, platform status, reset reason, partition/image identity, and selected board/ASIC target while mining and hardware control remain disabled.
- [x] **FND-07**: `just build`, `just test`, `just package`, `just flash`, `just monitor`, `just flash-monitor`, `just verify-reference`, and `just parity` are available and route through Bazel or repo-owned scripts represented in the automation graph.
- [x] **FND-08**: USB flashing ergonomics support `board=205`, optional `port=...`, likely-port discovery, clear ambiguous-port errors, build-before-flash by default, and printing the underlying flashing command.
- [x] **FND-09**: Firmware packaging records image paths, offsets when applicable, checksums, tool versions, firmware commit, and reference commit in a machine-readable manifest.
- [x] **FND-10**: Provenance and license guardrails keep original project work MIT-first where possible while marking upstream-derived GPL-compatible expression explicitly.
- [x] **FND-11**: Parity tooling reports checklist status, evidence gaps, implementation pointers, and reference breadcrumbs without treating implementation alone as verification.

### Config And NVS

- [x] **CFG-01**: Ultra 205 defaults match the reference config for device model, board version, ASIC model, ASIC frequency, ASIC voltage, pool defaults, fan defaults, and self-test defaults.
- [x] **CFG-02**: Board, device, and ASIC identifiers are represented as typed Rust domain values, including non-205 upstream boards as scoped but not hardware-verified entries.
- [x] **CFG-03**: NVS key names, default values, missing-key behavior, and migration behavior match upstream observable behavior for V1 settings.
- [x] **CFG-04**: Runtime settings use typed validation for ranges and units such as frequency, millivolts, temperatures, fan duty, hostnames, ports, and pool credentials.
- [x] **CFG-05**: Settings changed through user-facing surfaces persist and reload across reboot with upstream-compatible semantics.
- [x] **CFG-06**: Reference-derived golden fixtures cover Ultra 205 defaults, NVS schemas, and representative valid/invalid settings updates.

### BM1366 ASIC And Mining Hardware

- [x] **ASIC-01**: BM1366 packet, register, and CRC codecs are implemented as pure Rust logic with reference-derived fixtures.
- [x] **ASIC-02**: BM1366 work encoding and result parsing match upstream behavior for job payloads, nonces, domains, and error cases.
- [x] **ASIC-03**: ASIC model dispatch supports BM1366 as the V1 active path and represents other upstream ASIC families as deferred or not-yet-verified paths.
- [x] **ASIC-04**: The firmware contains a narrow UART adapter boundary that translates typed ASIC commands and observations between pure Rust logic and ESP-IDF serial I/O.
- [x] **ASIC-05**: Ultra 205 BM1366 reset, preflight, and staged initialization fail closed unless required board, power, thermal, and config gates pass.
- [x] **ASIC-06**: Frequency and voltage transition decisions are range-checked in pure Rust and require explicit hardware evidence before being marked verified.
- [x] **ASIC-07**: BM1366 initialization, work-send, and result-receive behavior have hardware-smoke evidence before release parity is claimed.
- [x] **ASIC-08**: ASIC modules and tricky behavior boundaries include reference breadcrumbs pointing to the pinned upstream implementation and parity checklist rows.

### Stratum And Mining Loop

- [x] **STR-01**: Stratum v1 message parsing and serialization match upstream-compatible request and response behavior.
- [x] **STR-02**: Subscribe, authorize, notify, set-difficulty, and submit flows work against a deterministic fake pool harness.
- [x] **STR-03**: Mining job construction, coinbase decoding, extranonce handling, and work queue integration match reference-observable behavior.
- [x] **STR-04**: Pool socket lifecycle, fallback pool behavior, reconnect behavior, and error logging match upstream user-visible behavior.
- [x] **STR-05**: Accepted shares, rejected shares, share difficulty, hashrate inputs, and pool result counters update consistently across mining, API, and telemetry surfaces.
- [x] **STR-06**: The first Ultra 205 mining loop connects config, Stratum v1, BM1366 work dispatch, result parsing, and global state without bypassing safety gates.
- [x] **STR-07**: Mining parity has hardware-smoke and soak criteria that record command, board, port, firmware commit, reference commit, logs, observed result, and conclusion.

### AxeOS API, Logs, And Telemetry

- [x] **API-01**: Rust API models are compatible with the upstream OpenAPI schema for V1 user-facing routes.
- [x] **API-02**: System info and settings responses expose upstream-compatible fields, names, units, defaults, and encoding.
- [x] **API-03**: Settings PATCH behavior validates, persists, rejects, reloads, and reports errors with upstream-compatible observable semantics.
- [x] **API-04**: ASIC, statistics, scoreboard, and mining-state endpoints report values derived from the Rust runtime state model.
- [x] **API-05**: Log buffer, log download, and log retention behavior support the user-facing API and WebSocket surfaces.
- [x] **API-06**: `/api/ws` streams log events in a client-compatible format.
- [x] **API-07**: `/api/ws/live` streams live telemetry with upstream-compatible payload shape, cadence, and state transitions.
- [x] **API-08**: Pause, resume, restart, identify, and related command routes preserve user-visible behavior and safe failure modes.
- [x] **API-09**: Static AxeOS assets and recovery page behavior remain compatible enough for device administration without requiring an Angular rewrite in V1.
- [x] **API-10**: API compare fixtures prove Rust responses match the upstream schema or captured upstream responses for representative success and error cases.

### Safety, Power, Thermal, Self-Test, And Peripherals

- [ ] **SAFE-01**: Ultra 205 voltage and power-control surfaces use bounded typed decisions and fail closed on invalid configuration, communication failure, or unsafe readings.
- [ ] **SAFE-02**: Thermal sensor and fan control surfaces expose upstream-compatible readings, fan duty behavior, RPM behavior, and failure reporting.
- [ ] **SAFE-03**: PID and thermal-control decisions are covered by pure unit tests before hardware effects are enabled.
- [ ] **SAFE-04**: Overheat, fan, power, thermal, and ASIC fault paths enter safe states and expose user-visible status compatible with upstream behavior.
- [ ] **SAFE-05**: Self-test lifecycle behavior covers factory flags, start, pass, fail, restart, cancel, and user-visible result reporting.
- [ ] **SAFE-06**: Display and input status surfaces needed for normal Ultra 205 administration are preserved or explicitly documented as deferred gaps.
- [ ] **SAFE-07**: Power, current, voltage, fan, and temperature telemetry are captured where Ultra 205 hardware exposes them.
- [ ] **SAFE-08**: Safety-critical surfaces cannot be marked `verified` without `hardware-smoke` or `hardware-regression` evidence.
- [ ] **SAFE-09**: Mining, control, API, and telemetry tasks avoid watchdog starvation and preserve observable responsiveness under load.

### OTA, Filesystem, And Release Packaging

- [ ] **REL-01**: Partition layout, filesystem layout, SPIFFS/static assets, and recovery assets support the same user-facing flash and administration flows expected from upstream.
- [ ] **REL-02**: Firmware OTA route behavior accepts, rejects, applies, logs, and recovers from updates with upstream-compatible observable behavior.
- [ ] **REL-03**: OTAWWW or static-asset update behavior is implemented or explicitly reported as a V1 parity gap with evidence and owner.
- [ ] **REL-04**: Release packaging produces named artifacts with checksums, manifests, image metadata, installation notes, and source/reference commit identifiers.
- [ ] **REL-05**: Release preparation includes dependency license inventory, reference provenance manifest, and explicit review of GPL-derived materials.
- [ ] **REL-06**: Flashable image production is reachable through `just package` and `just flash board=205` without requiring manual artifact discovery.
- [ ] **REL-07**: Build, flash, monitor, OTA, and recovery documentation is sufficient for a developer with a connected Ultra 205 to operate the firmware safely.
- [ ] **REL-08**: Rollback, recovery, large erase, failed update, and interrupted update cases have verification evidence before release parity is claimed.

### Evidence And Governance

- [ ] **EVD-01**: Each V1 parity surface in `docs/parity/checklist.md` records observable behavior, reference breadcrumb, Rust implementation pointer when known, status, evidence, and notes.
- [ ] **EVD-02**: `verified` means evidence-backed parity, not only implemented code.
- [ ] **EVD-03**: Non-205 boards and ASICs stay unverified or deferred until each board or ASIC has its own evidence set.
- [ ] **EVD-04**: Rust modules that port reference behavior include module-level or behavior-level breadcrumbs without line-by-line translation comments.
- [ ] **EVD-05**: Verification layers include unit tests, golden fixtures, API comparison, hardware smoke tests, and hardware regression or soak evidence where appropriate.

## v2 Requirements

Deferred to future releases. Tracked but not in the current roadmap.

### Additional Boards And ASICs

- **V2-BOARD-01**: Gamma 601 with BM1370 receives a dedicated bring-up path, parity evidence set, and hardware verification.
- **V2-BOARD-02**: Additional upstream board families such as Gamma Duo, Gamma Turbo, Max, Ultra, Hex, and Supra receive one-board-at-a-time verification.
- **V2-ASIC-01**: BM1370, BM1368, BM1397, and later ASIC families reach verified parity with per-ASIC fixtures and hardware evidence.
- **V2-FACTORY-01**: All-board factory image matrix and release automation are produced only after each board has evidence.

### Protocol And Accessory Expansion

- **V2-STR-01**: Stratum v2 reaches full parity when it becomes an explicit acceptance target.
- **V2-BAP-01**: BAP accessory protocol completeness reaches parity when accessory behavior is prioritized.
- **V2-CUSTOM-01**: Custom board configuration flows and advanced tuning beyond upstream-compatible V1 ranges are supported.

### Platform Evolution

- **V2-IDF-01**: ESP-IDF 6 is reassessed after released Rust ESP-IDF crates support it fully and the Ultra 205 baseline is stable.
- **V2-UI-01**: A Rust-owned replacement for Angular AxeOS UI may be considered after API and asset compatibility are stable.

## Out of Scope

Explicitly excluded. Documented to prevent scope creep.

| Feature | Reason |
| --- | --- |
| Line-by-line C translation | Device-user parity is the scope; preserving C structure is unnecessary unless it affects observable behavior. |
| Modifying files inside `reference/esp-miner` | The upstream submodule is behavioral evidence and must stay read-only. |
| Bare-metal `no_std` first production stack | ESP-IDF Rust is the accepted first stack because upstream behavior depends on ESP-IDF services. |
| Angular AxeOS rewrite in V1 | V1 targets API, assets, and administration compatibility, not frontend replacement. |
| Mining parity in the first milestone | First milestone is foundation plus safe Ultra 205 boot/log only. |
| Marking safety-critical hardware behavior verified without hardware evidence | Voltage, fan, thermal, power, and ASIC initialization need hardware proof. |
| Claiming all boards are hardware-verified from Ultra 205 evidence | Each board and ASIC needs its own evidence set. |
| Publishing MIT-only firmware images without license review | Upstream GPL-3.0 provenance and third-party dependency licenses must be reviewed first. |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
| --- | --- | --- |
| FND-01 | Phase 1 | Complete |
| FND-02 | Phase 1 | Complete |
| FND-03 | Phase 1 | Complete |
| FND-04 | Phase 1 | Complete |
| FND-05 | Phase 1 | Complete |
| FND-06 | Phase 1 | Complete |
| FND-07 | Phase 1 | Complete |
| FND-08 | Phase 1 | Complete |
| FND-09 | Phase 1 | Complete |
| FND-10 | Phase 1 | Complete |
| FND-11 | Phase 1 | Complete |
| CFG-01 | Phase 2 | Complete |
| CFG-02 | Phase 2 | Complete |
| CFG-03 | Phase 2 | Complete |
| CFG-04 | Phase 2 | Complete |
| CFG-05 | Phase 2 | Complete |
| CFG-06 | Phase 2 | Complete |
| ASIC-01 | Phase 3 | Complete |
| ASIC-02 | Phase 3 | Complete |
| ASIC-03 | Phase 3 | Complete |
| ASIC-04 | Phase 3 | Complete |
| ASIC-05 | Phase 3 | Complete |
| ASIC-06 | Phase 3 | Complete |
| ASIC-07 | Phase 3 | Complete |
| ASIC-08 | Phase 3 | Complete |
| STR-01 | Phase 4 | Complete |
| STR-02 | Phase 4 | Complete |
| STR-03 | Phase 4 | Complete |
| STR-04 | Phase 4 | Complete |
| STR-05 | Phase 4 | Complete |
| STR-06 | Phase 4 | Complete |
| STR-07 | Phase 4 | Complete |
| API-01 | Phase 5 | Complete |
| API-02 | Phase 5 | Complete |
| API-03 | Phase 5 | Complete |
| API-04 | Phase 5 | Complete |
| API-05 | Phase 5 | Complete |
| API-06 | Phase 5 | Complete |
| API-07 | Phase 5 | Complete |
| API-08 | Phase 5 | Complete |
| API-09 | Phase 5 | Complete |
| API-10 | Phase 5 | Complete |
| SAFE-01 | Phase 6 | Pending |
| SAFE-02 | Phase 6 | Pending |
| SAFE-03 | Phase 6 | Pending |
| SAFE-04 | Phase 6 | Pending |
| SAFE-05 | Phase 6 | Pending |
| SAFE-06 | Phase 6 | Pending |
| SAFE-07 | Phase 6 | Pending |
| SAFE-08 | Phase 6 | Pending |
| SAFE-09 | Phase 6 | Pending |
| REL-01 | Phase 7 | Pending |
| REL-02 | Phase 7 | Pending |
| REL-03 | Phase 7 | Pending |
| REL-04 | Phase 7 | Pending |
| REL-05 | Phase 7 | Pending |
| REL-06 | Phase 7 | Pending |
| REL-07 | Phase 7 | Pending |
| REL-08 | Phase 7 | Pending |
| EVD-01 | Phase 8 | Pending |
| EVD-02 | Phase 8 | Pending |
| EVD-03 | Phase 8 | Pending |
| EVD-04 | Phase 8 | Pending |
| EVD-05 | Phase 8 | Pending |

**Coverage:**

- v1 requirements: 64 total
- Mapped to phases: 64
- Unmapped: 0

______________________________________________________________________

*Requirements defined: 2026-06-20*\
*Last updated: 2026-06-20 after roadmap traceability mapping*
