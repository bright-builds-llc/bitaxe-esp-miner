# Roadmap: Bitaxe Rust Firmware

## Overview

V1 delivers Ultra 205 BM1366 device-user parity in deliberate layers: first a reproducible Rust ESP-IDF foundation with safe boot/log only, then typed configuration, BM1366 hardware behavior, Stratum v1 mining, AxeOS-compatible administration surfaces, safety controllers, OTA/release flows, and an evidence-backed Ultra 205 release gate. This supersedes the earlier Gamma 601-first roadmap per ADR-0014. The original V1 requirements naturally formed eight delivery boundaries; the first v1.0 milestone audit added five gap-closure phases to turn conservative completion into stronger live release-parity evidence, the next audit added three evidence-completion phases for active safety, trusted ASIC/mining, and current-commit release parity, and the current audit adds five targeted evidence-flow phases for live HTTP/API/static, OTA/rollback, recovery/OTAWWW, active safety telemetry, and live mining/soak proof.

## Phases

**Phase Numbering:**

- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [x] **Phase 1: Foundation And Ultra 205 Boot/Log** - Establish the monorepo, reference guardrails, automation graph, package/flash/monitor workflow, parity tooling, and safe Ultra 205 boot logs with mining and hardware control disabled.
- [x] **Phase 2: Ultra 205 Config And NVS Model** - Deliver typed Ultra 205 defaults, scoped board/ASIC identities, NVS semantics, validation, persistence, and reference-derived config fixtures. (completed 2026-06-26)
- [x] **Phase 3: BM1366 ASIC Protocol And Safe Initialization** - Deliver pure BM1366 protocol logic plus a narrow UART adapter and gated Ultra 205 initialization evidence. (completed 2026-06-27)
- [x] **Phase 4: Stratum V1 And First Mining Loop** - Deliver deterministic Stratum v1 behavior, fake pool coverage, work queue integration, and first evidence-backed Ultra 205 mining loop. (completed 2026-06-27)
- [x] **Phase 5: AxeOS API, Logs, And Telemetry** - Deliver upstream-compatible API models, handlers, logs, WebSocket telemetry, commands, static asset compatibility, and API comparison evidence. (completed 2026-06-27)
- [x] **Phase 6: Safety Controllers And Self-Test** - Deliver Ultra 205 power, voltage, thermal, fan, fault, display/input, watchdog, and self-test parity with hardware evidence gates. (completed 2026-06-28)
- [x] **Phase 7: OTA, Filesystem, And Release Packaging** - Deliver partition/filesystem behavior, OTA/recovery flows, static asset updates, release artifacts, license inventory, and safe operator docs. (completed 2026-06-28)
- [x] **Phase 8: Parity Evidence And Ultra 205 Release Gate** - Close V1 evidence governance so Ultra 205 parity claims are supported and deferred scope remains unverified or out of scope. (completed 2026-06-29)
- [x] **Phase 9: Flash-Monitor Evidence Wrapper Hardening** - Close the audit gap where repository `flash-monitor` evidence capture fell back to raw `espflash monitor --non-interactive`. (completed 2026-06-29)
- [x] **Phase 10: Route Manifest And API Compare Unification** - Close the audit gap where Phase 7 route manifests were not the source consumed by firmware route reporting and compare tooling. (completed 2026-06-29)
- [x] **Phase 11: Safety Controller Hardware Regression Evidence** - Close the audit gap for Ultra 205 voltage, power, thermal, fan, self-test, display/input, watchdog, and safety-critical hardware evidence. (completed 2026-06-29)
- [x] **Phase 12: ASIC And Mining Hardware Evidence** - Close the audit gap for BM1366 initialization, work/result handling, first mining loop, and controlled mining soak evidence. (completed 2026-06-30)
- [x] **Phase 13: Final Ultra 205 Release Evidence** - Close the audit gap for final-commit package/flash/boot identity plus live HTTP, static, recovery, OTA, rollback, erase, and interrupted-update evidence. (completed 2026-06-30)
- [x] **Phase 14: Safety Hardware Evidence Completion** - Close the current audit gap for active Ultra 205 safety-control, runtime telemetry, self-test submode, watchdog/load, and display/input evidence. (completed 2026-07-01)
- [x] **Phase 15: BM1366 Mining Evidence Completion** - Close the current audit gap for trusted BM1366 initialization, work/result handling, and controlled mining smoke/soak evidence. (completed 2026-07-01)
- [x] **Phase 16: Current Commit Release Evidence Completion** - Close the current audit gap for same-commit package, flash, serial boot, live HTTP/static/recovery/OTA, rollback, erase, failed-update, and interrupted-update evidence. (completed 2026-07-01)
- [x] **Phase 17: Live HTTP API And Static Evidence** - Close the current audit gap for explicit-`DEVICE_URL` live HTTP, static asset, recovery page, API route, and WebSocket evidence. (completed 2026-07-02)
- [ ] **Phase 18: Firmware OTA And Rollback Evidence** - Close the current audit gap for valid firmware OTA, invalid OTA rejection, reboot identity, rollback, and boot-validation evidence.
- [ ] **Phase 19: Recovery Regression And OTAWWW Evidence** - Close the current audit gaps for recovery fault-injection regressions and OTAWWW/static update behavior.
- [ ] **Phase 20: Active Safety Hardware Telemetry Evidence** - Close the current audit gap for active Ultra 205 safety hardware behavior and live telemetry evidence.
- [ ] **Phase 21: Live Mining And Soak Evidence** - Close the current audit gap for live production mining, accepted/rejected share behavior, watchdog responsiveness, and bounded soak evidence.

## Phase Details

### Phase 1: Foundation And Ultra 205 Boot/Log

**Goal**: A developer can build, package, flash, and monitor a safe Ultra 205 Rust firmware image that boots and logs identity/status while mining and hardware control remain disabled.
**Depends on**: Nothing (first phase)
**Requirements**: FND-01, FND-02, FND-03, FND-04, FND-05, FND-06, FND-07, FND-08, FND-09, FND-10, FND-11
**Success Criteria** (what must be TRUE):

1. Developer can verify the pinned `reference/esp-miner` submodule is present, clean, and protected by workflows that fail on missing, dirty, or unpinned reference state.
1. Developer can run `just build`, `just test`, `just package`, `just verify-reference`, and `just parity` and see Bazel-backed outputs for the firmware skeleton, pure crates, package manifest, parity report, and reference guard.
1. Developer can use `just flash`, `just monitor`, and `just flash-monitor` for `board=205` with port discovery, clear ambiguous-port errors, build-before-flash behavior, and the underlying flashing command printed.
1. Bitaxe owner can observe Ultra 205 boot logs showing firmware identity, platform status, reset reason, partition/image identity, selected board/ASIC target, and an explicit safe no-mining/no-control state.
1. Developer can inspect parity and provenance tooling output showing checklist status, evidence gaps, implementation pointers, reference breadcrumbs, package metadata, and license guardrails without treating implementation as verification.

**Plans**: 9 plans
Plans:

- [x] 01-01-PLAN.md - Pin the upstream reference submodule and add the Bazel-visible reference guard.
- [x] 01-02-PLAN.md - Create the Rust workspace root, ESP toolchain pins, and Bazel crate mirror contract.
- [x] 01-03-PLAN.md - Create foundational pure crates for boot/log identity, safe state, config selection, and test support.
- [x] 01-04-PLAN.md - Create deferred pure crate contracts for ASIC, Stratum, and API surfaces.
- [x] 01-05-PLAN.md - Create compile-only firmware and host tool package contracts.
- [x] 01-06-PLAN.md - Build the safe ESP-IDF boot/log firmware target.
- [x] 01-07-PLAN.md - Wire parity/provenance reporting without false verification.
- [x] 01-08-PLAN.md - Generate the package manifest/default flash image and implement safe flash/monitor tooling.
- [x] 01-09-PLAN.md - Wire Just commands and record hardware-smoke evidence.
  **Verification expectations**: Completed for the safe-state slice in `docs/parity/evidence/ultra-205-pivot-safe-state-smoke-2026-06-26.md`: `just build`, `just test`, `just package`, `just parity`, Ultra 205 dry-runs, deferred `board=601` rejection, and live Ultra 205 flash/monitor boot evidence. Confirm the package manifest records image paths, offsets when applicable, checksums, tool versions, firmware commit, and reference commit. Review the diff to confirm `reference/esp-miner` is not modified.
  **Research flags**: Mostly standard setup work. The Bazel wrapper around Cargo/ESP-IDF and the flashable image manifest may need an implementation spike.

### Phase 2: Ultra 205 Config And NVS Model

**Goal**: Users and firmware can rely on upstream-compatible Ultra 205 settings, defaults, validation, persistence, and scoped board identity.
**Depends on**: Phase 1
**Requirements**: CFG-01, CFG-02, CFG-03, CFG-04, CFG-05, CFG-06
**Success Criteria** (what must be TRUE):

1. User-visible Ultra 205 defaults match the reference for device model, board version, ASIC model, frequency, voltage, pool defaults, fan defaults, and self-test defaults.
1. User-facing settings changes persist and reload across reboot with upstream-compatible key names, default handling, missing-key behavior, and migration behavior.
1. Invalid settings for frequencies, millivolts, temperatures, fan duty, hostnames, ports, and pool credentials are rejected with typed validation and upstream-compatible observable errors.
1. Developer can inspect typed board, device, and ASIC identifiers that include non-205 upstream entries while keeping them unverified or deferred.
1. Developer can run reference-derived golden fixtures for Ultra 205 defaults, NVS schemas, and representative valid and invalid settings updates.

**Plans**: 4 plans
Plans:

- [x] 02-01-PLAN.md - Establish Phase 2 fixtures, Ultra 205 defaults, and scoped board/ASIC catalog.
- [x] 02-02-PLAN.md - Implement pure NVS schema, key constraints, migrations, defaults, and corrupt-float fallback.
- [x] 02-03-PLAN.md - Implement typed validation and pure settings update decisions.
- [x] 02-04-PLAN.md - Implement pure persistence reload semantics and record parity evidence.
**Verification expectations**: Unit and golden fixture coverage for config defaults, NVS schemas, validation ranges, and persistence semantics. Reboot reload smoke should be added once the firmware storage adapter exists. Parity checklist rows must record reference breadcrumbs and fixture evidence.
**Research flags**: Standard Rust domain modeling. Use the pinned reference tree and golden fixtures; do targeted research only if upstream config extraction or NVS migration behavior is ambiguous.

### Phase 3: BM1366 ASIC Protocol And Safe Initialization

**Goal**: Firmware can communicate with BM1366 through typed pure protocol logic and a narrow UART adapter, with live Ultra 205 initialization guarded and fail-closed.
**Depends on**: Phase 2
**Requirements**: ASIC-01, ASIC-02, ASIC-03, ASIC-04, ASIC-05, ASIC-06, ASIC-07, ASIC-08
**Success Criteria** (what must be TRUE):

1. Developer can run BM1366 packet, register, CRC, work-encoding, result-parsing, nonce, domain, and error-case fixtures and see upstream-compatible outputs.
1. Firmware can reset, preflight, and stage BM1366 initialization on Ultra 205 only when required board, power, thermal, and config gates pass.
1. Unsafe or incomplete ASIC initialization conditions fail closed with visible logs/status instead of enabling mining or hardware control.
1. Firmware translates typed ASIC commands and observations through a narrow UART adapter without leaking raw protocol details into user-facing control logic.
1. Developer can inspect reference breadcrumbs and parity checklist rows for BM1366 behavior, including hardware-smoke evidence before release parity is claimed.

**Plans**: 5 plans
Plans:

- [x] 03-01-PLAN.md - Establish BM1366 protocol module contracts, CRC, packet framing, register codecs, and provenance fixtures.
- [x] 03-02-PLAN.md - Implement BM1366 diagnostic work encoding, job ID semantics, result parsing, and nonce/register fault handling.
- [x] 03-03-PLAN.md - Add active BM1366 dispatch, semantic command/observation types, and fake UART transcript coverage.
- [x] 03-04-PLAN.md - Implement fail-closed staged init planning and pure frequency/voltage transition decisions.
- [x] 03-05-PLAN.md - Add the narrow firmware UART/reset/status adapter, evidence records, checklist updates, and human-gated chip-detect smoke review.
**Verification expectations**: Pure unit and golden tests for BM1366 codecs, fake UART adapter tests, staged hardware-smoke evidence for reset/init/work-send/result-receive, and explicit unverified status for frequency/voltage transitions until hardware evidence exists.
**Research flags**: Needs phase research for BM1366 sequencing, timing, reset behavior, voltage dependencies, UART behavior, and hardware evidence planning.

### Phase 4: Stratum V1 And First Mining Loop

**Goal**: Ultra 205 can mine through an upstream-compatible Stratum v1 loop using safe ASIC work dispatch and evidence-backed result reporting.
**Depends on**: Phase 3
**Requirements**: STR-01, STR-02, STR-03, STR-04, STR-05, STR-06, STR-07
**Success Criteria** (what must be TRUE):

1. Developer can run deterministic fake pool scenarios covering subscribe, authorize, notify, set-difficulty, submit, fallback, reconnect, and error logging behavior.
1. Firmware can construct jobs, decode coinbase and extranonce data, queue work, dispatch BM1366 work, parse results, and submit shares without bypassing safety gates.
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

**Goal**: Ultra 205 power, thermal, fan, peripheral, watchdog, and self-test behavior protects hardware while preserving upstream-visible status.
**Depends on**: Phase 5
**Requirements**: SAFE-01, SAFE-02, SAFE-03, SAFE-04, SAFE-05, SAFE-06, SAFE-07, SAFE-08, SAFE-09
**Success Criteria** (what must be TRUE):

1. User sees voltage, current, temperature, fan duty, RPM, power, and thermal telemetry where Ultra 205 hardware exposes them, using upstream-compatible units and failure reporting.
1. Unsafe voltage, fan, power, thermal, ASIC, or control-path failures enter safe states and expose user-visible status instead of continuing unsafe mining or hardware effects.
1. Developer can run pure PID, thermal-control, fan-control, and bounded power-decision tests before hardware effects are enabled.
1. User can run the self-test lifecycle for factory flags, start, pass, fail, restart, cancel, and result reporting with upstream-compatible behavior.
1. Display and input status surfaces needed for normal Ultra 205 administration work, or are explicitly documented as V1 parity gaps with evidence.

**Plans**: 10 plans
Plans:

- [x] 06-01-PLAN.md - Create the pure safety crate and shared status/evidence/effect contracts.
- [x] 06-02-PLAN.md - Create the safety crate feature module graph.
- [x] 06-03-PLAN.md - Implement power, voltage, current, and observe-only DS4432U/INA260 decisions.
- [x] 06-04-PLAN.md - Implement thermal, fan, PID, overheat, and fault decisions.
- [x] 06-05-PLAN.md - Implement self-test lifecycle and watchdog-friendly step supervision.
- [x] 06-06-PLAN.md - Wire power, thermal, and safety evidence tokens into ASIC and mining gates.
- [x] 06-07-PLAN.md - Replace zeroed API telemetry with explicit safety telemetry status.
- [x] 06-08-PLAN.md - Add firmware observe-only safety adapters and runtime snapshot integration.
- [x] 06-09-PLAN.md - Add firmware safety supervisor and display/input runtime gap status.
- [x] 06-10-PLAN.md - Enforce parity evidence gates and record Phase 6 checklist/evidence status.
**Verification expectations**: Pure unit tests for safety decisions, hardware-smoke and hardware-regression evidence for voltage/fan/thermal/power paths, self-test smoke, watchdog/load responsiveness checks, and parity checklist enforcement that safety-critical rows cannot be `verified` without hardware evidence.
**Research flags**: Needs deeper research and hardware planning for DS4432U, INA260, EMC2101, fan/PID behavior, fault paths, self-test sequencing, and soak protocol.

### Phase 7: OTA, Filesystem, And Release Packaging

**Goal**: An Ultra 205 owner can build, package, factory-flash, boot, and inspect release artifacts through upstream-compatible image, filesystem, OTA-capable, and packaging flows, with live network OTA/recovery verification deferred to Phase 8.
**Depends on**: Phase 6
**Requirements**: REL-01, REL-02, REL-03, REL-04, REL-05, REL-06, REL-07
**Success Criteria** (what must be TRUE):

1. User can package and flash named Ultra 205 artifacts with checksums, image metadata, source/reference commits, installation notes, and manifest entries.
1. User can factory-flash the merged image and observe on-device serial evidence for the expected partition layout, SPIFFS `www` mount, PSRAM availability, boot validation entry, safe startup, display startup, and HTTP route registration.
1. Firmware OTA, recovery, static route, and rollback code paths are implemented, host-tested, packaged, documented, and protected by parity guards, while live network requests and fault-injection recovery are explicitly deferred to Phase 8 before release parity is claimed.
1. OTAWWW or static-asset update behavior is implemented or explicitly reported as a V1 parity gap with evidence and owner.
1. Developer can review dependency license inventory, reference provenance manifest, and build/flash/monitor/OTA/recovery docs before public release.

**Plans**: 9 plans
Plans:

- [x] 07-01-PLAN.md - Create pure update/static route contracts and tests.
- [x] 07-02-PLAN.md - Define manifest v2 and partition validation contracts.
- [x] 07-03-PLAN.md - Create release docs, license, provenance, and evidence contracts.
- [x] 07-04-PLAN.md - Implement SPIFFS mount, static serving, and recovery page behavior.
- [x] 07-05-PLAN.md - Expand release artifact generation and manifest v2 packaging.
- [x] 07-06-PLAN.md - Implement license/provenance release-gate validation.
- [x] 07-07-PLAN.md - Implement firmware OTA, rollback validation, and OTAWWW gap behavior.
- [x] 07-08-PLAN.md - Complete operator docs and Phase 7 evidence records.
- [x] 07-09-PLAN.md - Add parity gates and final hardware verification checkpoint.
**UI hint**: yes
**Verification expectations**: Package manifest checks, factory flash/install serial smoke, route registration, partition/SPIFFS boot evidence, release license inventory, release-gate validation, parity guard validation, and operator documentation review. Live HTTP static/recovery checks, valid/invalid OTA upload, rollback, large erase, failed update, and interrupted-update evidence are deferred to Phase 8.
**Research flags**: Needs phase research for ESP-IDF OTA/partition details, rollback/recovery testing, large erase behavior, static asset packaging, image manifests, and release compliance.

### Phase 8: Parity Evidence And Ultra 205 Release Gate

**Goal**: Ultra 205 V1 parity claims are evidence-backed, scoped, and ready for release without expanding into deferred boards, protocols, accessories, or UI rewrites.
**Depends on**: Phase 7
**Requirements**: REL-08, EVD-01, EVD-02, EVD-03, EVD-04, EVD-05
**Success Criteria** (what must be TRUE):

1. Developer can open `docs/parity/checklist.md` and see every V1 parity surface record observable behavior, reference breadcrumb, Rust implementation pointer when known, status, evidence, and notes.
1. Developer can confirm `verified` means evidence-backed parity, and safety-critical rows require hardware-smoke or hardware-regression evidence before verification.
1. Developer can confirm non-205 boards and ASICs remain unverified or deferred unless each has its own evidence set.
1. Developer can inspect Rust modules that port reference behavior and find module-level or behavior-level breadcrumbs without line-by-line translation comments.
1. Release readiness is derived from unit, golden, API-compare, hardware-smoke, and hardware-regression or soak evidence instead of implementation status alone.
1. User can reach the flashed Ultra 205 over HTTP and capture live `/`, `/assets/app.css.gz`, missing static redirect, `/recovery`, valid firmware OTA, invalid OTA rejection, OTAWWW gap response, rollback/boot validation, large erase recovery, failed update recovery, and interrupted-update recovery evidence before release parity is claimed.

**Plans**: 4 plans
Plans:

- [x] 08-01-PLAN.md - Record package, detector, factory flash, serial boot, and blocked live HTTP evidence.
- [x] 08-02-PLAN.md - Record blocked live static, OTA, OTAWWW, and release-gate evidence.
- [x] 08-03-PLAN.md - Record blocked rollback, recovery, failed-update, large erase, and destructive evidence.
- [x] 08-04-PLAN.md - Close final checklist, release summary, provenance, license, breadcrumb audit, and release gates.
**Verification expectations**: Full parity checklist audit, evidence type coverage review, non-205 deferred-scope review, reference breadcrumb audit, live Ultra 205 HTTP/OTA/recovery hardware evidence, release-readiness summary, and final confirmation that V2-only scope remains out of V1.
**Research flags**: No V1 expansion research. Each future board, ASIC family, Stratum v2, BAP, or Angular UI replacement needs its own later research and roadmap entry before work starts.

### Phase 9: Flash-Monitor Evidence Wrapper Hardening

**Goal**: A developer can capture Ultra 205 flash-monitor evidence through the repo wrapper without falling back to raw `espflash` commands.
**Depends on**: Phase 8
**Requirements**: FND-07, FND-08, REL-07, EVD-05
**Gap Closure**: Closes the v1.0 audit gap where `just flash-monitor ... evidence-dir=...` failed at monitor startup and evidence used raw `espflash monitor --non-interactive` instead.
**Success Criteria** (what must be TRUE):

1. `tools/flash` supports a first-class noninteractive monitor/evidence path for `flash-monitor` that does not depend on an interactive input reader.
1. `just flash-monitor board=205 port=... evidence-dir=...` records the selected board, port, source commit, reference commit, package manifest, exact commands, monitor log, observed behavior, and conclusion.
1. The wrapper prints clear recovery guidance for monitor startup failures and still fails visibly when evidence capture cannot be trusted.
1. Fresh Ultra 205 wrapper evidence replaces the fallback-only evidence path without changing `reference/esp-miner`.

**Plans**: 2 plans
Plans:

- [x] 09-01-PLAN.md - Harden the flash wrapper with noninteractive evidence capture, bounded monitor timeout, trusted marker classification, enriched JSON, and recovery guidance.
- [x] 09-02-PLAN.md - Capture fresh Ultra 205 wrapper evidence and refresh workflow/release docs without promoting later release rows.
**Verification expectations**: Unit tests for command construction and evidence path behavior, `just detect-ultra205` before hardware use, live wrapper-based `just flash-monitor board=205 port=... evidence-dir=...` evidence when exactly one Ultra 205 is detected, and parity evidence updates that remove the raw-monitor fallback as the only proof.
**Research flags**: Standard host-tool and ESP serial workflow work. Do not introduce a custom flashing backend unless `espflash` cannot support the documented evidence path.

### Phase 10: Route Manifest And API Compare Unification

**Goal**: Firmware route reporting and API/static/OTA compare tooling consume the same Phase 7 route manifest so route drift is caught before release evidence.
**Depends on**: Phase 9
**Requirements**: API-09, API-10, REL-01, REL-02, REL-03, EVD-01
**Gap Closure**: Closes the v1.0 audit gap where `phase07_routes()` existed but was not the manifest consumed by firmware route-count logging or API compare tooling.
**Success Criteria** (what must be TRUE):

1. The Phase 7 route manifest is the single source for firmware route-count/reporting behavior, including static, recovery, OTA, and OTAWWW gap routes.
1. API/static/OTA comparison tooling consumes the same manifest and fails when a required route is missing, downgraded, or incorrectly classified as verified.
1. Unit and fixture tests prove route reporting, compare tooling, and parity evidence stay aligned when route ownership changes.
1. `docs/parity/checklist.md` records the unified manifest evidence without claiming live HTTP or OTA behavior before Phase 13.

**Plans**: 3 plans
Plans:

- [x] 10-01-PLAN.md - Add Phase 7 route reporting helpers and switch firmware startup logging to manifest-derived counts.
- [x] 10-02-PLAN.md - Refactor API compare route policy to use the Phase 7 typed manifest and add route-drift regressions.
- [x] 10-03-PLAN.md - Record checklist/evidence claim boundaries and run final Phase 10 verification.
**Verification expectations**: Cargo tests for route manifest consumers, API compare fixture regression tests, `just parity`, and a diff review confirming route reporting and compare tooling cannot drift independently.
**Research flags**: Standard Rust domain and tooling work. Keep route classification in pure data and leave firmware handlers as thin adapters.

### Phase 11: Safety Controller Hardware Regression Evidence

**Goal**: Ultra 205 safety-critical hardware surfaces have documented hardware-regression evidence before they are promoted beyond implemented or safe-unavailable status.
**Depends on**: Phase 10
**Requirements**: SAFE-01, SAFE-02, SAFE-03, SAFE-04, SAFE-05, SAFE-06, SAFE-07, SAFE-08, SAFE-09, EVD-05
**Gap Closure**: Closes the v1.0 audit gap for voltage, fan, thermal, power, self-test, display/input, watchdog, and other safety-critical hardware evidence.
**Success Criteria** (what must be TRUE):

1. The phase plan documents the recovery path before any destructive, fault-injection, or hardware-actuation verification is run.
1. Ultra 205 power, voltage, current, thermal, fan duty, RPM, and failure-path evidence records exact commands, board, port, source commit, reference commit, logs, observed behavior, and conclusion.
1. Self-test, display/input status, watchdog, and load responsiveness evidence is captured or explicitly remains below verified with owner and follow-up.
1. Parity tooling continues to reject safety-critical `verified` rows without hardware-smoke or hardware-regression evidence.

**Plans**: 3 plans
Plans:

- [x] 11-01-PLAN.md - Document the Phase 11 runbook, recovery protocol, evidence-pack contract, and redaction gate.
- [x] 11-02-PLAN.md - Require hardware-regression evidence before active safety-control rows can be marked verified.
- [x] 11-03-PLAN.md - Capture detector-gated Ultra 205 safe boot evidence and update checklist boundaries.
**Verification expectations**: Existing pure safety tests stay green, `just detect-ultra205` gates live hardware work, hardware-regression evidence is recorded only for board `205`, destructive/fault-injection steps follow the documented recovery path, and `just parity` rejects any overclaim.
**Research flags**: Requires hardware planning for DS4432U, INA260, EMC2101, fan/PID behavior, self-test, display/input, and failure-path recovery. Do not run ad hoc voltage, fan, mining stress, erase, or fault injection outside the phase plan.

### Phase 12: ASIC And Mining Hardware Evidence

**Goal**: BM1366 initialization, work/result handling, and the first Ultra 205 mining loop have safety-gated hardware-smoke and soak evidence.
**Depends on**: Phase 11
**Requirements**: ASIC-07, STR-06, STR-07, EVD-05
**Gap Closure**: Closes the v1.0 audit gap for live BM1366 chip-detect/init, work-send/result-receive, and real mining loop evidence.
**Success Criteria** (what must be TRUE):

1. BM1366 chip-detect and staged initialization run only after board, power, thermal, config, and safety evidence gates pass.
1. Work-send and result-receive hardware-smoke evidence records commands, board, port, package manifest, source/reference commits, logs, observed behavior, and conclusion.
1. The first mining-loop evidence covers pool lifecycle, accepted/rejected shares or a documented controlled no-share condition, hashrate inputs, API/telemetry status, and watchdog responsiveness.
1. Mining parity rows remain below verified until the captured evidence satisfies the hardware-smoke or soak criteria.

**Plans**: TBD
**Verification expectations**: Pure BM1366 and Stratum tests stay green, live hardware work is gated by `just detect-ultra205`, controlled pool or fake-pool conditions are documented, smoke/soak evidence is captured without secrets, and `just parity` validates checklist semantics.
**Research flags**: Requires hardware and pool-soak planning. Do not bypass safety gates or store pool credentials in evidence.

### Phase 13: Final Ultra 205 Release Evidence

**Goal**: The final V1 source commit has package, flash, boot, HTTP, static, recovery, OTA, rollback, erase, failed-update, and interrupted-update evidence for Ultra 205 release parity.
**Depends on**: Phase 12
**Requirements**: FND-06, API-09, REL-01, REL-02, REL-03, REL-04, REL-08, EVD-05
**Gap Closure**: Closes the v1.0 audit gaps for unreachable `DEVICE_URL`, missing final-commit package-to-hardware evidence identity, and live release-parity evidence.
**Success Criteria** (what must be TRUE):

1. The final package manifest, factory image, source commit, reference commit, install notes, and license/provenance artifacts are recorded before flash.
1. The same source commit is flashed to Ultra 205 and serial boot evidence records identity, partition/image status, PSRAM/SPIFFS state, route registration, and safe startup behavior.
1. A reachable `DEVICE_URL` provides live evidence for `/`, `/assets/app.css.gz`, missing static redirect, `/recovery`, valid firmware OTA, invalid OTA rejection, and OTAWWW gap behavior.
1. Rollback, boot validation, large erase recovery, failed update recovery, and interrupted-update recovery evidence is captured under a documented recovery procedure before release parity is claimed.

**Plans**: 6 plans
Plans:

- [x] 13-01-PLAN.md - Establish release-candidate package identity, release gate, and evidence scaffold.
- [x] 13-02-PLAN.md - Capture detector-gated Ultra 205 flash-monitor serial boot evidence.
- [x] 13-03-PLAN.md - Add and run HTTP/static/recovery smoke evidence helper.
- [x] 13-04-PLAN.md - Add and run firmware OTA and boot-validation evidence helpers.
- [x] 13-05-PLAN.md - Document and run recovery/destructive evidence only behind exact recovery gates.
- [x] 13-06-PLAN.md - Close checklist, release docs, redaction review, and final verification.
**Verification expectations**: `just package`, wrapper-based flash-monitor evidence from Phase 9, live HTTP/static/recovery/OTA checks through the unified route manifest from Phase 10, recovery procedures documented before destructive tests, release gate validation, `just parity`, and final audit rerun.
**Research flags**: Requires a reachable device network setup and phase-gated recovery instructions. Stop and record evidence pending if `DEVICE_URL` is unavailable.

### Phase 14: Safety Hardware Evidence Completion

**Goal**: Ultra 205 active safety-control and runtime telemetry surfaces have hardware-regression evidence, or remain explicitly below verified with bounded recovery instructions and no overclaim.
**Depends on**: Phase 13
**Requirements**: SAFE-01, SAFE-02, SAFE-03, SAFE-04, SAFE-05, SAFE-06, SAFE-07, SAFE-08, SAFE-09, EVD-05
**Gap Closure**: Closes the current v1.0 audit gap where safety controllers are wired and fail-closed, but active voltage, fan, thermal, self-test submode, runtime input/display, watchdog/load, and live telemetry evidence remains pending.
**Success Criteria** (what must be TRUE):

1. The phase plan documents recovery steps, explicit allow gates, stop conditions, redaction rules, and post-action safe-state checks before any active voltage, fan, thermal, self-test, or load/stress hardware verification runs.
1. Ultra 205 safety evidence records board `205`, selected port, source commit, reference commit, package manifest, exact commands, board-info, captured logs, observed readings or failures, and conclusion.
1. Active voltage/power, fan/thermal, self-test, watchdog/load, display/input, and live API/WebSocket telemetry claims either have hardware-regression evidence or remain below `verified` with an owner and blocker.
1. `just parity` continues to reject safety-critical verified rows without valid hardware-smoke or hardware-regression evidence.

**Plans**: 6 plans
Plans:

- [x] 14-01-PLAN.md - Add the machine-enforced Phase 14 safety allow-manifest gate.
- [x] 14-02-PLAN.md - Create the Phase 14 component evidence scaffold and redaction contract.
- [x] 14-03-PLAN.md - Add and run power, voltage, thermal, and fan evidence wrappers.
- [x] 14-04-PLAN.md - Add and run self-test, watchdog/load, display, and input evidence wrappers.
- [x] 14-05-PLAN.md - Add and run live API/WebSocket safety telemetry evidence wrapper.
- [x] 14-06-PLAN.md - Close final ledger, redaction, checklist, and validation status.
**Verification expectations**: `just detect-ultra205`, gated hardware-regression evidence for board `205`, pure safety tests, relevant firmware/package checks, checklist validation, redaction review, and final phase verification.
**Research flags**: Requires careful hardware recovery planning. Do not run ad hoc voltage, fan, mining stress, erase, rollback, or fault-injection commands outside the approved phase plan.

### Phase 15: BM1366 Mining Evidence Completion

**Goal**: BM1366 initialization, work-send, result-receive, and controlled Ultra 205 mining smoke/soak have trusted safety-gated hardware evidence.
**Depends on**: Phase 14
**Requirements**: ASIC-07, STR-06, STR-07, SAFE-09, EVD-05
**Gap Closure**: Closes the current v1.0 audit gap where Phase 12 restored trusted safe boot and fail-closed mining state, but chip-detect was wrapper-untrusted and work/result/mining smoke/soak evidence did not run.
**Success Criteria** (what must be TRUE):

1. BM1366 chip-detect and staged initialization evidence is captured through a trusted wrapper path after safety and board gates pass.
1. Work-send and result-receive evidence records commands, board, port, source commit, reference commit, package manifest, logs, observed ASIC behavior, and conclusion without exposing secrets.
1. Controlled mining smoke or bounded soak records pool lifecycle, accepted/rejected share or controlled no-share behavior, hashrate inputs, API/WebSocket telemetry, watchdog responsiveness, and redaction review.
1. Checklist rows for ASIC, Stratum, statistics, and mining remain below `verified` unless their hardware-smoke or soak evidence meets the documented criteria.

**Plans**: 5 plans
Plans:

- [x] 15-01-PLAN.md - Add the mining allow-manifest gate and Phase 15 evidence/redaction scaffold.
- [x] 15-02-PLAN.md - Build and run package-backed BM1366 chip-detect evidence without weakening wrapper trust.
- [x] 15-03-PLAN.md - Add and run typed bounded BM1366 work-send/result-receive diagnostics.
- [x] 15-04-PLAN.md - Add and run controlled mining smoke and bounded soak evidence with no-share fallback.
- [x] 15-05-PLAN.md - Close redaction, checklist, validation, and lifecycle verification.
**Verification expectations**: Pure BM1366 and Stratum tests stay green, `just detect-ultra205` gates live hardware, any pool credentials are redacted or avoided, `just parity` validates checklist semantics, and final evidence restores safe state.
**Research flags**: Requires controlled pool and recovery planning. Do not bypass safety gates or store pool credentials, private endpoints, Wi-Fi credentials, or NVS secret values in evidence.

### Phase 16: Current Commit Release Evidence Completion

**Goal**: The current release-candidate source commit has same-commit package, flash, serial boot, live HTTP/static/recovery/OTA, rollback, erase, failed-update, and interrupted-update evidence for Ultra 205 release parity.
**Depends on**: Phase 15
**Requirements**: FND-06, API-09, REL-01, REL-02, REL-03, REL-04, REL-07, REL-08, EVD-05
**Gap Closure**: Closes the current v1.0 audit gaps for stale package-to-hardware identity, missing explicit `DEVICE_URL`, blocked live HTTP/static/recovery/OTA evidence, and pending destructive/fault-injection evidence.
**Success Criteria** (what must be TRUE):

1. `just package` produces a manifest for the same source commit that is flashed and monitored on Ultra 205, with source commit, reference commit, image paths, checksums, and install notes recorded before flash.
1. `just flash-monitor board=205 port=... evidence-dir=...` records serial boot evidence for the same source commit, including partition/image state, PSRAM/SPIFFS, route registration, reset reason, and safe startup.
1. An explicit reachable `DEVICE_URL` for the just-flashed device captures live `/`, `/assets/app.css.gz`, missing static redirect, `/recovery`, valid firmware OTA, invalid OTA rejection, and OTAWWW gap behavior without network scanning.
1. Rollback, boot validation, large erase recovery, failed update recovery, and interrupted-update evidence runs only behind documented recovery gates and explicit allow flags, then restores and records safe state.
1. Release docs, parity checklist, requirements traceability, milestone audit, and redaction review are updated without promoting rows that still lack evidence.

**Plans**: 6 plans
Plans:

- [x] 16-01-PLAN.md - Add Wave 0 current-commit identity, HTTP route, recovery gate, and redaction prerequisites.
- [x] 16-02-PLAN.md - Capture current package, release-gate, detector, and serial boot evidence.
- [x] 16-03-PLAN.md - Capture explicit-DEVICE_URL HTTP, static, recovery, API/WebSocket, OTA route, and OTAWWW gap evidence.
- [x] 16-04-PLAN.md - Capture firmware OTA valid-upload, invalid-rejection, reboot identity, and boot-validation evidence.
- [x] 16-05-PLAN.md - Capture gated failed-update, large-erase, interrupted-update, restore, and recovery evidence.
- [x] 16-06-PLAN.md - Close final redaction, checklist, release docs, requirements traceability, validation, and lifecycle verification.
**Verification expectations**: `just package`, release gate, wrapper-based flash-monitor evidence, live HTTP/static/recovery/OTA checks, destructive/recovery checks only behind documented gates, `just parity`, `just verify-reference`, and final milestone audit rerun.
**Research flags**: Requires reachable device network setup and explicit recovery authorization. Stop and record evidence pending if `DEVICE_URL`, detector gate, board-info, or recovery prerequisites are unavailable.

### Phase 17: Live HTTP API And Static Evidence

**Goal**: The just-flashed Ultra 205 exposes live administration surfaces at an explicit `DEVICE_URL` with static asset, recovery page, API route, and WebSocket evidence.
**Depends on**: Phase 16
**Requirements**: API-09, REL-01, REL-07, EVD-05
**Gap Closure**: Closes the current v1.0 audit gap `live-http-static-recovery-api-websocket`, where implementation exists but the current evidence set lacks an explicit reachable `DEVICE_URL` and live HTTP/API/WebSocket captures.
**Success Criteria** (what must be TRUE):

1. Ultra 205 detector output, board `205`, selected port, source commit, reference commit, package manifest, and explicit reachable `DEVICE_URL` are recorded without network scanning or secrets.
1. Live evidence captures `/`, `/assets/app.css.gz`, representative missing static behavior, `/recovery`, API route coexistence, `/api/ws`, and `/api/ws/live` from the just-flashed device.
1. Evidence records exact commands, HTTP status and response summaries, relevant device logs, observed behavior, conclusion, and redaction review.
1. Release docs, parity checklist, and requirements traceability are updated without marking rows `verified` unless their evidence criteria are met.

**Plans**: 7 plans
Plans:

- [x] 17-01-PLAN.md - Add Wave 0 helper tests, Bazel wiring, WebSocket capture helper, and evidence/redaction scaffold.
- [x] 17-02-PLAN.md - Capture current package, release-gate, detector, and flash-monitor identity evidence.
- [x] 17-03-PLAN.md - Capture explicit-target HTTP/static/API route evidence or no-scan blocked evidence.
- [x] 17-04-PLAN.md - Capture bounded WebSocket frame evidence for `/api/ws/live` and `/api/ws` or pending/blocked evidence.
- [x] 17-05-PLAN.md - Close summary ledger, redaction sign-off, release docs, checklist, requirements traceability, and final verification.
- [x] 17-06-PLAN.md - Capture live HTTP/static/API route evidence from trusted USB flash-monitor target provenance.
- [x] 17-07-PLAN.md - Capture bounded live WebSocket frame evidence and final traceability updates.
**Verification expectations**: `just detect-ultra205`, same-commit package/flash evidence, explicit `DEVICE_URL` smoke, HTTP/API/WebSocket capture, redaction review, `just parity`, and `just verify-reference`.
**Research flags**: Requires reachable device network setup. Stop and record evidence pending if detector, board-info, port selection, or `DEVICE_URL` is unavailable.

### Phase 18: Firmware OTA And Rollback Evidence

**Goal**: Firmware OTA accepts valid images, rejects invalid images, preserves reboot identity, and records rollback or boot-validation behavior on Ultra 205.
**Depends on**: Phase 17
**Requirements**: REL-02, REL-08, REL-07, EVD-05
**Gap Closure**: Closes the current v1.0 audit gap `firmware-ota-rollback-boot-validation`, where live valid OTA, invalid OTA rejection, rollback, and boot-validation evidence remains missing.
**Success Criteria** (what must be TRUE):

1. Same-commit package artifacts and OTA image identity are recorded before upload, including source commit, reference commit, image path, checksum, board, port, and `DEVICE_URL`.
1. Valid firmware OTA evidence records upload behavior, HTTP response, reboot behavior, selected partition, flashed image identity, serial logs, and post-OTA safe state.
1. Invalid OTA rejection and rollback or boot-validation evidence run only behind documented gates and record the before/after partition and safety state.
1. Release docs, parity checklist, requirements traceability, and redaction review reflect the evidence without exposing secrets or overclaiming release parity.

**Plans**: 0 plans
Plans: Pending.
**Verification expectations**: `just package`, `just detect-ultra205`, explicit `DEVICE_URL` OTA upload checks, serial reboot capture, rollback/boot-validation checks only behind documented gates, `just parity`, and `just verify-reference`.
**Research flags**: Requires recovery instructions before rollback or boot-validation fault cases. Stop if OTA prerequisites, rollback gates, or restore instructions are missing.

### Phase 19: Recovery Regression And OTAWWW Evidence

**Goal**: Recovery regressions and OTAWWW/static update behavior have bounded evidence, or remain explicitly below verified with owner, blocker, and release documentation.
**Depends on**: Phase 18
**Requirements**: REL-03, REL-08, REL-07, API-09, EVD-05
**Gap Closure**: Closes the current v1.0 audit gaps `recovery-regression-fault-injection` and `otawww-static-update`, where destructive recovery cases and OTAWWW behavior are still pending or deferred.
**Success Criteria** (what must be TRUE):

1. The phase plan documents explicit allow flags, recovery path, stop conditions, expected restore state, and redaction rules before failed-update, erase, or interrupted-update tests run.
1. Failed-update, large-erase or factory restore, and interrupted-update evidence records exact commands, board, port, source commit, package identity, logs, observed behavior, restore action, and conclusion.
1. OTAWWW/static asset update behavior is either implemented and verified with live evidence or documented as an explicit V1 parity gap with owner, blocker, and operator impact.
1. Release docs, parity checklist, requirements traceability, and final evidence ledgers distinguish verified behavior from blocked, deferred, or below-verified behavior.

**Plans**: 0 plans
Plans: Pending.
**Verification expectations**: `just detect-ultra205`, gated recovery/fault-injection evidence, OTAWWW/static update check, restore proof, redaction review, `just parity`, and `just verify-reference`.
**Research flags**: Requires documented recovery procedures and explicit destructive-test authorization in the phase plan. Do not run ad hoc erase, rollback, or interrupted-update commands.

### Phase 20: Active Safety Hardware Telemetry Evidence

**Goal**: Active Ultra 205 safety-control behavior and live telemetry have hardware-regression evidence, or remain explicitly below verified with bounded recovery instructions and no overclaim.
**Depends on**: Phase 17
**Requirements**: SAFE-01, SAFE-02, SAFE-03, SAFE-04, SAFE-05, SAFE-06, SAFE-07, SAFE-08, SAFE-09, EVD-05
**Gap Closure**: Closes the current v1.0 audit gap `active-safety-hardware-and-live-telemetry`, where active voltage, fan, thermal, self-test, watchdog/load, display/input, and live telemetry evidence remains below verified.
**Success Criteria** (what must be TRUE):

1. The phase plan documents recovery steps, explicit allow gates, stop conditions, redaction rules, and post-action safe-state checks before active safety hardware verification runs.
1. Active voltage/power, fan/thermal, self-test, watchdog/load, display/input, and failure-path evidence records board `205`, selected port, source commit, reference commit, package manifest, exact commands, logs, observed readings, and conclusion.
1. Live API and WebSocket telemetry evidence is correlated with hardware observations and shows safe state before and after active checks.
1. `just parity` continues to reject safety-critical verified rows without valid hardware-smoke or hardware-regression evidence.

**Plans**: 0 plans
Plans: Pending.
**Verification expectations**: `just detect-ultra205`, active safety allow manifest, hardware-regression evidence, live telemetry capture, pure safety tests, redaction review, `just parity`, and `just verify-reference`.
**Research flags**: Requires careful hardware recovery planning. Do not run ad hoc voltage, fan, thermal, self-test, load, or stress commands outside the approved phase plan.

### Phase 21: Live Mining And Soak Evidence

**Goal**: Live Ultra 205 mining, share handling, watchdog responsiveness, and bounded soak behavior have trusted safety-gated evidence.
**Depends on**: Phase 20
**Requirements**: ASIC-07, STR-06, STR-07, SAFE-09, EVD-05
**Gap Closure**: Closes the current v1.0 audit gap `live-production-mining-soak`, where evidence remains diagnostic or no-share and does not yet prove live production mining or bounded soak behavior.
**Success Criteria** (what must be TRUE):

1. The phase plan documents controlled pool setup, credential redaction, recovery steps, safety gates, stop conditions, and post-run safe-state checks before live mining or soak runs.
1. BM1366 initialization, work-send, and result-receive evidence records exact commands, board, port, source commit, reference commit, package manifest, logs, observed ASIC behavior, and conclusion.
1. Controlled mining smoke or bounded soak records pool lifecycle, accepted/rejected share behavior or explicitly bounded no-share behavior, hashrate inputs, API/WebSocket telemetry, watchdog responsiveness, and redaction review.
1. Checklist rows for ASIC, Stratum, statistics, and mining remain below `verified` unless their hardware-smoke or soak evidence meets the documented criteria.

**Plans**: 0 plans
Plans: Pending.
**Verification expectations**: Pure BM1366 and Stratum tests, `just detect-ultra205`, safety-gated live mining smoke, bounded soak, live telemetry, no stored secrets, redaction review, `just parity`, and `just verify-reference`.
**Research flags**: Requires controlled pool and recovery planning. Do not store pool credentials, private endpoints, Wi-Fi credentials, or NVS secret values in evidence.

## Progress

**Execution Order:**
Phases execute in numeric order: 1 -> 2 -> 3 -> 4 -> 5 -> 6 -> 7 -> 8 -> 9 -> 10 -> 11 -> 12 -> 13 -> 14 -> 15 -> 16 -> 17 -> 18 -> 19 -> 20 -> 21

| Phase | Plans Complete | Status | Completed |
| --- | --- | --- | --- |
| 1. Foundation And Ultra 205 Boot/Log | 9/9 | Complete | 2026-06-21 |
| 2. Ultra 205 Config And NVS Model | 4/4 | Complete | 2026-06-26 |
| 3. BM1366 ASIC Protocol And Safe Initialization | 5/5 | Complete | 2026-06-27 |
| 4. Stratum V1 And First Mining Loop | 4/4 | Complete    | 2026-06-27 |
| 5. AxeOS API, Logs, And Telemetry | 7/7 | Complete | 2026-06-27 |
| 6. Safety Controllers And Self-Test | 10/10 | Complete | 2026-06-28 |
| 7. OTA, Filesystem, And Release Packaging | 9/9 | Complete | 2026-06-28 |
| 8. Parity Evidence And Ultra 205 Release Gate | 4/4 | Complete | 2026-06-29 |
| 9. Flash-Monitor Evidence Wrapper Hardening | 2/2 | Complete | 2026-06-29 |
| 10. Route Manifest And API Compare Unification | 3/3 | Complete    | 2026-06-29 |
| 11. Safety Controller Hardware Regression Evidence | 3/3 | Complete | 2026-06-29 |
| 12. ASIC And Mining Hardware Evidence | 4/4 | Complete | 2026-06-30 |
| 13. Final Ultra 205 Release Evidence | 6/6 | Complete    | 2026-06-30 |
| 14. Safety Hardware Evidence Completion | 6/6 | Complete    | 2026-07-01 |
| 15. BM1366 Mining Evidence Completion | 5/5 | Complete    | 2026-07-01 |
| 16. Current Commit Release Evidence Completion | 6/6 | Complete | 2026-07-01 |
| 17. Live HTTP API And Static Evidence | 7/7 | Complete   | 2026-07-03 |
| 18. Firmware OTA And Rollback Evidence | 0/0 | Pending | TBD |
| 19. Recovery Regression And OTAWWW Evidence | 0/0 | Pending | TBD |
| 20. Active Safety Hardware Telemetry Evidence | 0/0 | Pending | TBD |
| 21. Live Mining And Soak Evidence | 0/0 | Pending | TBD |
