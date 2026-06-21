# Roadmap: Bitaxe Rust Firmware

## Overview

V1 delivers Gamma 601 BM1370 device-user parity in deliberate layers: first a reproducible Rust ESP-IDF foundation with safe boot/log only, then typed configuration, BM1370 hardware behavior, Stratum v1 mining, AxeOS-compatible administration surfaces, safety controllers, OTA/release flows, and an evidence-backed Gamma 601 release gate. The config uses coarse granularity, but the V1 requirements naturally form eight delivery boundaries; merging them would blur safety-critical evidence and release readiness.

## Phases

**Phase Numbering:**

- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [ ] **Phase 1: Foundation And Gamma 601 Boot/Log** - Establish the monorepo, reference guardrails, automation graph, package/flash/monitor workflow, parity tooling, and safe Gamma 601 boot logs with mining and hardware control disabled.
- [ ] **Phase 2: Gamma 601 Config And NVS Model** - Deliver typed Gamma 601 defaults, scoped board/ASIC identities, NVS semantics, validation, persistence, and reference-derived config fixtures.
- [ ] **Phase 3: BM1370 ASIC Protocol And Safe Initialization** - Deliver pure BM1370 protocol logic plus a narrow UART adapter and gated Gamma 601 initialization evidence.
- [ ] **Phase 4: Stratum V1 And First Mining Loop** - Deliver deterministic Stratum v1 behavior, fake pool coverage, work queue integration, and first evidence-backed Gamma 601 mining loop.
- [ ] **Phase 5: AxeOS API, Logs, And Telemetry** - Deliver upstream-compatible API models, handlers, logs, WebSocket telemetry, commands, static asset compatibility, and API comparison evidence.
- [ ] **Phase 6: Safety Controllers And Self-Test** - Deliver Gamma 601 power, voltage, thermal, fan, fault, display/input, watchdog, and self-test parity with hardware evidence gates.
- [ ] **Phase 7: OTA, Filesystem, And Release Packaging** - Deliver partition/filesystem behavior, OTA/recovery flows, static asset updates, release artifacts, license inventory, and safe operator docs.
- [ ] **Phase 8: Parity Evidence And Gamma 601 Release Gate** - Close V1 evidence governance so Gamma 601 parity claims are supported and deferred scope remains unverified or out of scope.

## Phase Details

### Phase 1: Foundation And Gamma 601 Boot/Log

**Goal**: A developer can build, package, flash, and monitor a safe Gamma 601 Rust firmware image that boots and logs identity/status while mining and hardware control remain disabled.
**Depends on**: Nothing (first phase)
**Requirements**: FND-01, FND-02, FND-03, FND-04, FND-05, FND-06, FND-07, FND-08, FND-09, FND-10, FND-11
**Success Criteria** (what must be TRUE):

1. Developer can verify the pinned `reference/esp-miner` submodule is present, clean, and protected by workflows that fail on missing, dirty, or unpinned reference state.
1. Developer can run `just build`, `just test`, `just package`, `just verify-reference`, and `just parity` and see Bazel-backed outputs for the firmware skeleton, pure crates, package manifest, parity report, and reference guard.
1. Developer can use `just flash`, `just monitor`, and `just flash-monitor` for `board=601` with port discovery, clear ambiguous-port errors, build-before-flash behavior, and the underlying flashing command printed.
1. Bitaxe owner can observe Gamma 601 boot logs showing firmware identity, platform status, reset reason, partition/image identity, selected board/ASIC target, and an explicit safe no-mining/no-control state.
1. Developer can inspect parity and provenance tooling output showing checklist status, evidence gaps, implementation pointers, reference breadcrumbs, package metadata, and license guardrails without treating implementation as verification.

**Plans**: 9 plans
Plans:
- [x] 01-01-PLAN.md - Pin the upstream reference submodule and add the Bazel-visible reference guard.
- [x] 01-02-PLAN.md - Create the Rust workspace root, ESP toolchain pins, and Bazel crate mirror contract.
- [x] 01-03-PLAN.md - Create foundational pure crates for Gamma 601 identity, safe state, config selection, and test support.
- [x] 01-04-PLAN.md - Create deferred pure crate contracts for ASIC, Stratum, and API surfaces.
- [ ] 01-05-PLAN.md - Create compile-only firmware and host tool package contracts.
- [ ] 01-06-PLAN.md - Build the safe Gamma 601 ESP-IDF boot/log firmware target.
- [ ] 01-07-PLAN.md - Wire parity/provenance reporting without false verification.
- [ ] 01-08-PLAN.md - Generate the package manifest/default flash image and implement safe flash/monitor tooling.
- [ ] 01-09-PLAN.md - Wire Just commands and record Gamma 601 hardware-smoke evidence.
**Verification expectations**: Run `just verify-reference`, `just build`, `just test`, `just package`, `just parity`, and a Gamma 601 flash/monitor smoke where hardware is available. Confirm the package manifest records image paths, offsets when applicable, checksums, tool versions, firmware commit, and reference commit. Review the diff to confirm `reference/esp-miner` is not modified.
**Research flags**: Mostly standard setup work. The Bazel wrapper around Cargo/ESP-IDF and the flashable image manifest may need an implementation spike.

### Phase 2: Gamma 601 Config And NVS Model

**Goal**: Users and firmware can rely on upstream-compatible Gamma 601 settings, defaults, validation, persistence, and scoped board identity.
**Depends on**: Phase 1
**Requirements**: CFG-01, CFG-02, CFG-03, CFG-04, CFG-05, CFG-06
**Success Criteria** (what must be TRUE):

1. User-visible Gamma 601 defaults match the reference for device model, board version, ASIC model, frequency, voltage, pool defaults, fan defaults, and self-test defaults.
1. User-facing settings changes persist and reload across reboot with upstream-compatible key names, default handling, missing-key behavior, and migration behavior.
1. Invalid settings for frequencies, millivolts, temperatures, fan duty, hostnames, ports, and pool credentials are rejected with typed validation and upstream-compatible observable errors.
1. Developer can inspect typed board, device, and ASIC identifiers that include non-601 upstream entries while keeping them unverified or deferred.
1. Developer can run reference-derived golden fixtures for Gamma 601 defaults, NVS schemas, and representative valid and invalid settings updates.

**Plans**: TBD
**Verification expectations**: Unit and golden fixture coverage for config defaults, NVS schemas, validation ranges, and persistence semantics. Reboot reload smoke should be added once the firmware storage adapter exists. Parity checklist rows must record reference breadcrumbs and fixture evidence.
**Research flags**: Standard Rust domain modeling. Use the pinned reference tree and golden fixtures; do targeted research only if upstream config extraction or NVS migration behavior is ambiguous.

### Phase 3: BM1370 ASIC Protocol And Safe Initialization

**Goal**: Firmware can communicate with BM1370 through typed pure protocol logic and a narrow UART adapter, with live Gamma 601 initialization guarded and fail-closed.
**Depends on**: Phase 2
**Requirements**: ASIC-01, ASIC-02, ASIC-03, ASIC-04, ASIC-05, ASIC-06, ASIC-07, ASIC-08
**Success Criteria** (what must be TRUE):

1. Developer can run BM1370 packet, register, CRC, work-encoding, result-parsing, nonce, domain, and error-case fixtures and see upstream-compatible outputs.
1. Firmware can reset, preflight, and stage BM1370 initialization on Gamma 601 only when required board, power, thermal, and config gates pass.
1. Unsafe or incomplete ASIC initialization conditions fail closed with visible logs/status instead of enabling mining or hardware control.
1. Firmware translates typed ASIC commands and observations through a narrow UART adapter without leaking raw protocol details into user-facing control logic.
1. Developer can inspect reference breadcrumbs and parity checklist rows for BM1370 behavior, including hardware-smoke evidence before release parity is claimed.

**Plans**: TBD
**Verification expectations**: Pure unit and golden tests for BM1370 codecs, fake UART adapter tests, staged hardware-smoke evidence for reset/init/work-send/result-receive, and explicit unverified status for frequency/voltage transitions until hardware evidence exists.
**Research flags**: Needs phase research for BM1370 sequencing, timing, reset behavior, voltage dependencies, UART behavior, and hardware evidence planning.

### Phase 4: Stratum V1 And First Mining Loop

**Goal**: Gamma 601 can mine through an upstream-compatible Stratum v1 loop using safe ASIC work dispatch and evidence-backed result reporting.
**Depends on**: Phase 3
**Requirements**: STR-01, STR-02, STR-03, STR-04, STR-05, STR-06, STR-07
**Success Criteria** (what must be TRUE):

1. Developer can run deterministic fake pool scenarios covering subscribe, authorize, notify, set-difficulty, submit, fallback, reconnect, and error logging behavior.
1. Firmware can construct jobs, decode coinbase and extranonce data, queue work, dispatch BM1370 work, parse results, and submit shares without bypassing safety gates.
1. User-facing mining, API, and telemetry surfaces report accepted shares, rejected shares, share difficulty, hashrate inputs, pool result counters, and pool lifecycle status consistently.
1. Developer can review mining hardware-smoke and soak evidence recording command, board, port, firmware commit, reference commit, logs, observed result, and conclusion before mining parity is claimed.

**Plans**: TBD
**Verification expectations**: Stratum parser/serializer fixtures, deterministic fake pool tests, work queue integration tests, public or controlled pool smoke only after safe ASIC gates pass, and soak evidence for accepted/rejected shares and reconnect behavior.
**Research flags**: Needs deeper research for Stratum edge cases, fake pool design, reconnect/fallback behavior, watchdog/yielding expectations, and long-running soak criteria.

### Phase 5: AxeOS API, Logs, And Telemetry

**Goal**: Users, API clients, and existing AxeOS assets can administer and observe Rust firmware through upstream-compatible API, log, and telemetry surfaces.
**Depends on**: Phase 4
**Requirements**: API-01, API-02, API-03, API-04, API-05, API-06, API-07, API-08, API-09, API-10
**Success Criteria** (what must be TRUE):

1. API client receives upstream-compatible system info, settings, ASIC, statistics, scoreboard, and mining-state responses with matching fields, names, units, defaults, and encoding.
1. User can PATCH settings and see validation, persistence, reload, rejection, and error behavior match upstream-compatible observable semantics.
1. User can download logs and connect to `/api/ws` and `/api/ws/live` with compatible log payloads, telemetry payloads, cadence, and state transitions.
1. User can pause, resume, restart, identify, and use related command routes with safe visible success and failure behavior.
1. Existing static AxeOS assets and recovery page behavior can administer V1 surfaces without requiring an Angular rewrite, backed by schema or captured-response comparison fixtures.

**Plans**: TBD
**UI hint**: yes
**Verification expectations**: OpenAPI compatibility checks, captured upstream response comparison fixtures, HTTP handler tests, WebSocket cadence checks, settings PATCH persistence/reload checks, and real firmware API smoke for representative success and error cases.
**Research flags**: Standard HTTP and model work, but route-by-route upstream capture and API compare fixture design should be planned carefully.

### Phase 6: Safety Controllers And Self-Test

**Goal**: Gamma 601 power, thermal, fan, peripheral, watchdog, and self-test behavior protects hardware while preserving upstream-visible status.
**Depends on**: Phase 5
**Requirements**: SAFE-01, SAFE-02, SAFE-03, SAFE-04, SAFE-05, SAFE-06, SAFE-07, SAFE-08, SAFE-09
**Success Criteria** (what must be TRUE):

1. User sees voltage, current, temperature, fan duty, RPM, power, and thermal telemetry where Gamma 601 hardware exposes them, using upstream-compatible units and failure reporting.
1. Unsafe voltage, fan, power, thermal, ASIC, or control-path failures enter safe states and expose user-visible status instead of continuing unsafe mining or hardware effects.
1. Developer can run pure PID, thermal-control, fan-control, and bounded power-decision tests before hardware effects are enabled.
1. User can run the self-test lifecycle for factory flags, start, pass, fail, restart, cancel, and result reporting with upstream-compatible behavior.
1. Display and input status surfaces needed for normal Gamma 601 administration work, or are explicitly documented as V1 parity gaps with evidence.

**Plans**: TBD
**Verification expectations**: Pure unit tests for safety decisions, hardware-smoke and hardware-regression evidence for voltage/fan/thermal/power paths, self-test smoke, watchdog/load responsiveness checks, and parity checklist enforcement that safety-critical rows cannot be `verified` without hardware evidence.
**Research flags**: Needs deeper research and hardware planning for TPS546/vcore control, EMC2101, fan/PID behavior, fault paths, self-test sequencing, and soak protocol.

### Phase 7: OTA, Filesystem, And Release Packaging

**Goal**: A Gamma 601 owner can install, update, recover, and inspect release artifacts through upstream-compatible image, filesystem, OTA, and packaging flows.
**Depends on**: Phase 6
**Requirements**: REL-01, REL-02, REL-03, REL-04, REL-05, REL-06, REL-07, REL-08
**Success Criteria** (what must be TRUE):

1. User can package and flash named Gamma 601 artifacts with checksums, image metadata, source/reference commits, installation notes, and manifest entries.
1. User can use partition layout, filesystem layout, SPIFFS/static assets, and recovery page behavior through the same administration and recovery flows expected from upstream.
1. User can perform OTA firmware update attempts that accept, reject, apply, log, roll back, and recover with upstream-compatible observable behavior.
1. OTAWWW or static-asset update behavior is implemented or explicitly reported as a V1 parity gap with evidence and owner.
1. Developer can review dependency license inventory, reference provenance manifest, and build/flash/monitor/OTA/recovery docs before public release.

**Plans**: TBD
**UI hint**: yes
**Verification expectations**: Package manifest checks, flash/install smoke, OTA success and failure tests, rollback/recovery/interrupted-update evidence, static asset/recovery checks, release license inventory, and operator documentation review.
**Research flags**: Needs phase research for ESP-IDF OTA/partition details, rollback/recovery testing, large erase behavior, static asset packaging, image manifests, and release compliance.

### Phase 8: Parity Evidence And Gamma 601 Release Gate

**Goal**: Gamma 601 V1 parity claims are evidence-backed, scoped, and ready for release without expanding into deferred boards, protocols, accessories, or UI rewrites.
**Depends on**: Phase 7
**Requirements**: EVD-01, EVD-02, EVD-03, EVD-04, EVD-05
**Success Criteria** (what must be TRUE):

1. Developer can open `docs/parity/checklist.md` and see every V1 parity surface record observable behavior, reference breadcrumb, Rust implementation pointer when known, status, evidence, and notes.
1. Developer can confirm `verified` means evidence-backed parity, and safety-critical rows require hardware-smoke or hardware-regression evidence before verification.
1. Developer can confirm non-601 boards and ASICs remain unverified or deferred unless each has its own evidence set.
1. Developer can inspect Rust modules that port reference behavior and find module-level or behavior-level breadcrumbs without line-by-line translation comments.
1. Release readiness is derived from unit, golden, API-compare, hardware-smoke, and hardware-regression or soak evidence instead of implementation status alone.

**Plans**: TBD
**Verification expectations**: Full parity checklist audit, evidence type coverage review, non-601 deferred-scope review, reference breadcrumb audit, release-readiness summary, and final confirmation that V2-only scope remains out of V1.
**Research flags**: No V1 expansion research. Each future board, ASIC family, Stratum v2, BAP, or Angular UI replacement needs its own later research and roadmap entry before work starts.

## Progress

**Execution Order:**
Phases execute in numeric order: 1 -> 2 -> 3 -> 4 -> 5 -> 6 -> 7 -> 8

| Phase | Plans Complete | Status | Completed |
| --- | --- | --- | --- |
| 1. Foundation And Gamma 601 Boot/Log | 2/9 | In Progress | - |
| 2. Gamma 601 Config And NVS Model | 0/TBD | Not started | - |
| 3. BM1370 ASIC Protocol And Safe Initialization | 0/TBD | Not started | - |
| 4. Stratum V1 And First Mining Loop | 0/TBD | Not started | - |
| 5. AxeOS API, Logs, And Telemetry | 0/TBD | Not started | - |
| 6. Safety Controllers And Self-Test | 0/TBD | Not started | - |
| 7. OTA, Filesystem, And Release Packaging | 0/TBD | Not started | - |
| 8. Parity Evidence And Gamma 601 Release Gate | 0/TBD | Not started | - |
