# Project Research Summary

**Project:** Bitaxe Rust Firmware — v1.2 Ultra 205 Operator-Ready Runtime
**Domain:** Read-only embedded runtime observation, durable configuration, provenance, and evidence-backed operator health
**Researched:** 2026-07-13
**Confidence:** HIGH

## Executive Summary

v1.2 should turn the existing Ultra 205 firmware foundations into one truthful normal-runtime operator experience. The milestone is not a new framework, a mining retry, or an active hardware-control release. It composes the repository's existing Rust ESP-IDF services, pure safety/configuration/API models, AxeOS-compatible routes, and evidence tooling around four operator outcomes: fresh read-only power and thermal observations, storage-confirmed settings, truthful firmware/runtime identity, and passive health visibility.

The recommended architecture has one long-lived owner for I2C0, one serialized NVS settings owner, and one revisioned operator snapshot read by HTTP, WebSocket, logs, and evidence. Pure Rust types distinguish never-observed, fresh, stale, failed, and unsafe states; imperative firmware adapters own clocks, I/O, storage, and publication. Settings success means validation, write, commit, actual reload, reconciliation, and later bounded reboot continuity—not an optimistic cache update.

The main risk is false confidence from individually plausible but uncorrelated facts: a zero presented as telemetry, a PATCH acknowledged before durable reload, a version filled from the host checkout, a health marker mistaken for continuing liveness, or artifacts combined across sessions. v1.2 should prevent those errors through typed claim boundaries, single-owner lifecycles, same-revision public projection, and one detector-gated evidence chain. Active fan, voltage, reset, power sequencing, ASIC control, self-test effects, fault injection, mining diagnostics, direct UART, and pin manipulation remain explicitly excluded.

## Key Findings

### Recommended Stack

Keep the current stack and dependency pins described in [STACK.md](STACK.md). The milestone needs composition and lifecycle ownership, not new infrastructure. ESP-IDF `v5.5.4`, `esp-idf-svc 0.52.1`, the existing blocking I2C driver, NVS adapters, pure workspace crates, Bazel, `just`, and `espflash` already cover the required boundaries. Use existing register-level INA260 and EMC2101 adapters after separating read semantics from any mutating initialization path.

**Core technologies:**

- **ESP-IDF Rust `std`:** firmware services, blocking I2C, NVS, platform facts, logging, and FreeRTOS-backed tasks—already pinned and proven in this repository.
- **Existing pure crates:** acquisition/freshness, settings reconciliation, health projection, AxeOS-compatible DTOs, and evidence admission—extend their types instead of moving policy into firmware.
- **Bazel + `just` + existing parity tooling:** canonical build, package, detector, bounded capture, redaction, correlation, and exact-claim validation—add a v1.2 profile rather than a parallel runner.

No new framework, sensor crate, storage layer, metrics service, async executor, HAL, or build-provenance dependency is recommended.

### Expected Features

The feature boundary in [FEATURES.md](FEATURES.md) is one bounded operator-readiness chain, not a collection of independent demos.

**Must have (table stakes):**

- One firmware-owned read-only I2C lifecycle for the display handoff, INA260, and EMC2101-class observations.
- Producer-owned sample sequence and monotonic time with explicit `fresh`, `stale`, `unavailable`, and `fault` projection; a numeric zero is never an availability state.
- One coherent API/WebSocket/log snapshot so public surfaces describe the same device revision.
- A published, narrow, non-secret, non-actuating settings allowlist with atomic validation, NVS commit, actual reload, reconciliation, immediate readback, and bounded reboot durability.
- Truthful firmware, source/reference, ESP-IDF, static-asset, board/ASIC, partition, reset, uptime, heap, and package identity, or an explicit unavailable state.
- Passive self-test lifecycle and watchdog/supervisor checkpoint visibility without running hardware self-test, fault injection, or load/intervention experiments.
- A detector-gated, redacted, same-session evidence profile that promotes only the exact operator-runtime rows it proves.

**Should have after the core chain is stable:**

- Additional non-secret, non-actuating settings with explicit reload and reboot contracts.
- Longer bounded read-only observation runs and support summaries derived from the same snapshot.
- Bounded host-side history after firmware latest-state ownership and memory limits are established.

**Defer to later milestones:**

- OTA, rollback, interrupted-update recovery, and OTAWWW.
- Active fan, voltage, reset, power sequencing, thermal/power fault stimulus, active self-test, and ASIC control.
- BM1366 nonce/result/share work, all Phase 28.1.1 descendants, and mining credential evidence.
- Other boards/ASICs, display/input parity, BAP, Stratum v2, and an AxeOS frontend rewrite.

### Architecture Approach

Use the functional-core/imperative-shell design in [ARCHITECTURE.md](ARCHITECTURE.md). One I2C task serializes bounded read transactions and publishes acquisition outcomes. One NVS service serializes mutation and publishes only storage-observed settings. A coherent `OperatorRuntimeSnapshot` combines sensor, confirmed settings, provenance, and passive health state under an opaque boot session and monotonic revision. API and WebSocket adapters clone and project that state without performing I/O. Host evidence tooling stages artifacts and validates package, board, session, ordering, revisions, redaction, and non-claims before atomic promotion.

**Major components:**

1. **Typed claim and observation core** — separates acquisition, freshness, safety classification, compatible wire values, and parity eligibility.
1. **Single-owner firmware services** — own I2C0 and NVS for their complete lifetimes and publish immutable outcomes without holding state locks across I/O.
1. **Coherent operator snapshot and projections** — provides one revisioned source for system info, WebSocket, retained logs, provenance, and passive runtime health.
1. **v1.2 evidence profile** — proves one exact-head, detector-gated, redacted session and preserves every excluded control/mining claim.

### Critical Pitfalls

The full failure catalogue is in [PITFALLS.md](PITFALLS.md). The roadmap should address these risks explicitly:

1. **Competing I2C owners** — construct I2C0 once, serialize complete read transactions, and publish outcomes rather than sharing the driver with request handlers.
1. **Fabricated freshness** — only the acquisition producer advances sample time/sequence; API reads cannot refresh stale or absent data.
1. **Observation promoted as control proof** — make the v1.2 capability and evidence schema unable to issue or promote actuator, fault, ASIC, self-test, or mining effects.
1. **Optimistic settings success** — acknowledge only after commit, actual reload, typed reconciliation, and confirmed publication; prove reboot continuity separately.
1. **Mixed identity or runtime truth** — embed build identity, publish one coherent snapshot, and reject evidence whose package, board, session, boot, or revisions do not correlate.
1. **Archived diagnostic re-entry** — reject any Phase 28.1.1, nonce, share, external-UART, pin, or mining path during roadmap, planning, execution, verification, and evidence review.

## Implications for Roadmap

Roadmap numbering should continue after completed Phase 30. The five-phase structure below follows the actual dependency graph while keeping every v1.2 hardware path read-only and non-actuating.

### Phase 31: Operator Claim and Telemetry Contract

**Rationale:** Define what a device observation, public value, health statement, and parity claim mean before firmware code can accidentally mint unsupported truth.
**Delivers:** Typed acquisition/freshness/failure states, sample age/sequence semantics, unsafe-value separation, compatible additive API projection, the v1.2 settings allowlist boundary, and compile-time/runtime guards excluding active control and archived mining paths.
**Addresses:** Explicit telemetry states, coherent public semantics, passive health vocabulary, exact non-claims.
**Avoids:** Request-time freshness, sentinel-zero status, and promotion of sensor reads into safety/control claims.

### Phase 32: Shared I2C and Read-Only Sensor Acquisition

**Rationale:** Fresh operator telemetry cannot exist until normal boot has one durable I2C0 lifecycle and bounded attribution for each device transaction.
**Delivers:** A single `operator_i2c_runtime`, startup-display handoff through the same owner, bounded INA260 and EMC2101 read semantics, signed INA260 current decoding, independent sensor failure isolation, periodic progress, and revisioned acquisition publication.
**Uses:** Existing ESP-IDF blocking I2C and repository-owned register adapters; no new driver stack.
**Implements:** Single effect owner/latest-state readers and acquisition-before-safety patterns.
**Excludes:** EMC2101 configuration/fan writes, DS4432U writes, reset/power sequencing, ASIC operations, fault stimulus, and any diagnostic/mining mode.

### Phase 33: Confirmed Settings Durability

**Rationale:** Configuration is operator-ready only when public success represents storage truth and survives the supported lifecycle.
**Delivers:** One serialized NVS owner, pure reconciliation of planned writes against the actual post-commit snapshot, explicit commit/reload/mismatch failures, a confirmed settings revision, boot-time projection, and a bounded reboot round-trip for safe non-secret settings.
**Addresses:** Published allowlist, atomic rejection, commit/readback/reload, immediate API confirmation, and reboot durability.
**Avoids:** Partial writes, optimistic cache overlays, secret-bearing evidence, and settings that disrupt target continuity or actuate hardware.

### Phase 34: Provenance, Runtime Health, and Coherent Operator Snapshot

**Rationale:** Sensors and settings must be composed with truthful identity and passive liveness before operator surfaces or evidence can make a system-level statement.
**Delivers:** Typed firmware/build/reference/package identity, decoded runtime facts, one revisioned `OperatorRuntimeSnapshot`, shared HTTP/WebSocket/log projection, passive self-test state, supervisor/checkpoint recency, and explicit distinction between model liveness and actual ESP task-watchdog participation.
**Addresses:** System information, build provenance, runtime health, self-test visibility, cross-surface correlation.
**Avoids:** Synthetic version placeholders, host-substituted runtime identity, mixed-generation API fields, one-time log markers as health proof, and active self-test or watchdog fault/load experiments.

### Phase 35: Detector-Gated Correlated Evidence and Exact Parity Promotion

**Rationale:** Hardware claims must follow a complete software contract and be proven as one current-session chain rather than assembled from separately plausible artifacts.
**Delivers:** A dedicated v1.2 operator evidence profile and bounded wrapper; exact source/reference/package binding; detector and board-info gate; read-only sensor/API/WebSocket correlation; confirmed settings reload and reboot continuity; provenance/health correlation; cleanup proof; redaction/inventory validation; and row-specific promotion or explicit non-promotion.
**Addresses:** The milestone's final operator-readiness claim on board `205`.
**Avoids:** Stale target reuse, network scanning, mixed sessions, fixture promotion, credential access, active hardware effects, mining evidence slots, and broad parity promotion.

### Phase Ordering Rationale

- Phase 31 makes illegal claim states unrepresentable before hardware, storage, or API integration begins.
- Phase 32 establishes the sole physical-bus producer needed by every telemetry consumer.
- Phase 33 closes storage truth independently of telemetry and avoids coupling persistence debugging to final hardware evidence.
- Phase 34 composes typed producers into the single operator view needed for trustworthy public and evidence projection.
- Phase 35 validates the finished dependency chain on one detector-gated Ultra 205 and promotes only evidence-supported rows.
- Hardware evidence in an earlier phase may be used narrowly to validate safe read-only acquisition or reboot mechanics, but final parity admission remains Phase 35 and must never cross into actuation or mining.

### Research Flags

Phases likely needing focused research during planning:

- **Phase 32:** Confirm read-only EMC2101 register behavior and display handoff without invoking mutating initialization; plan bounded bus timeout/cadence behavior before hardware use.
- **Phase 33:** Pin the exact safe settings allowlist, compatibility behavior for unknown fields, and the approved normal reboot path without exposing or changing credentials.
- **Phase 34:** Decide whether v1.2 reports actual ESP task-watchdog subscription or truthfully reports it unavailable; do not infer it from the pure supervisor.
- **Phase 35:** Define the evidence schema, target/session correlation keys, redaction inventory, parity-row allowlist, and exact stop/non-promotion outcomes before capture.

Phases with established patterns that can generally skip separate research:

- **Phase 31:** Pure typed-state and projection patterns are already well established in the repository and Bright Builds architecture guidance.
- **Phase 33 core persistence model:** Validation, commit, reload, and reconciliation patterns already exist; planning should focus on ownership and acceptance details.
- **Phase 34 provenance collection:** Existing build/package/runtime sources are known; the work is consolidation and truthful projection rather than technology selection.

## Confidence Assessment

| Area | Confidence | Notes |
| --- | --- | --- |
| Stack | HIGH | Exact repository pins and existing adapters cover the milestone; no dependency addition is needed. |
| Features | HIGH | Active goals, parity gaps, AxeOS surfaces, and explicit deferrals are documented locally. |
| Architecture | HIGH | Current ownership conflicts and target components were traced in the repository; the recommended functional-core/imperative-shell pattern matches local standards. |
| Pitfalls | HIGH | Most failure modes are already evidenced by v1.0/v1.1 artifacts and current code boundaries; live shared-I2C timing remains the principal hardware uncertainty. |

**Overall confidence:** HIGH

### Gaps to Address

- **Safe settings allowlist:** Requirements should name the exact supported fields. `hostname` is the safest baseline; every additional key needs a non-secret, non-actuating, target-preserving reload/reboot contract.
- **EMC2101 read-only path:** Planning must prove temperature/tach reads do not require configuration writes. If safe observation cannot be established, report the affected fact unavailable.
- **Sensor cadence and staleness thresholds:** Select bounded values from existing models and hardware behavior; keep them configurable/testable in the pure core rather than hidden in request handlers.
- **Watchdog claim level:** Separate supervisor heartbeat from real ESP task-watchdog configuration. Absence of a narrow adapter and direct evidence means `Unavailable`, not implied protection.
- **Reboot continuity:** Use only a phase-approved normal reboot path with detector/target/session preservation; no raw reset, power sequencing, erase, or fault injection.
- **Final parity rows:** Requirements and Phase 35 must enumerate the exact checklist rows eligible for promotion and preserve active-control, mining, credentials, and other-board non-claims.

## Sources

### Primary (HIGH confidence)

- [.planning/PROJECT.md](../PROJECT.md) — active v1.2 goal, project constraints, terminal mining boundary, and read-only milestone decision.
- [.planning/research/STACK.md](STACK.md) — current pins, existing adapters, and recommended composition changes.
- [.planning/research/FEATURES.md](FEATURES.md) — operator table stakes, dependencies, launch boundary, and deferrals.
- [.planning/research/ARCHITECTURE.md](ARCHITECTURE.md) — component ownership, data flow, lifecycle boundaries, and build order.
- [.planning/research/PITFALLS.md](PITFALLS.md) — repository-specific false-claim, ownership, durability, identity, and evidence risks.
- [.planning/milestones/v1.1-MILESTONE-AUDIT.md](../milestones/v1.1-MILESTONE-AUDIT.md) and [.planning/RETROSPECTIVE.md](../RETROSPECTIVE.md) — accepted gaps, terminal unresolved history, exact non-claims, and bounded evidence lessons.
- [docs/parity/checklist.md](../../docs/parity/checklist.md) — current status and evidence boundaries for operator-runtime parity surfaces.
- Repository source under `firmware/bitaxe`, `crates/bitaxe-safety`, `crates/bitaxe-config`, `crates/bitaxe-api`, and `tools/parity` — current implementation and ownership boundaries.
- `AGENTS.md`, `AGENTS.bright-builds.md`, `standards/core/architecture.md`, and `standards/core/verification.md` — read-only hardware authorization, archive guards, functional-core/imperative-shell structure, and verification rules materially applied here.
- [ESP-IDF NVS documentation](https://docs.espressif.com/projects/esp-idf/en/v5.5.4/esp32s3/api-reference/storage/nvs_flash.html) — commit and namespace behavior.
- [ESP-IDF Task Watchdog documentation](https://docs.espressif.com/projects/esp-idf/en/v5.5/esp32s3/api-reference/system/wdts.html) — task/user subscription and configuration boundaries.
- [Texas Instruments INA260 datasheet](https://www.ti.com/lit/ds/symlink/ina260.pdf) and [Microchip EMC2101 datasheet](https://ww1.microchip.com/downloads/aemDocuments/documents/MSLD/ProductDocuments/DataSheets/EMC2101-Data-Sheet-DS20006703.pdf) — read register semantics and value encoding.

### Secondary (MEDIUM confidence)

- Existing committed phase evidence under `docs/parity/evidence/` — observed current behavior and evidence gaps; useful as history but not reusable as v1.2 live proof.
- Pinned read-only upstream sources under `reference/esp-miner` — behavioral breadcrumbs for device-user compatibility, not authority to copy implementation or promote Rust hardware claims.

*Research completed: 2026-07-13*
*Ready for roadmap: yes*
