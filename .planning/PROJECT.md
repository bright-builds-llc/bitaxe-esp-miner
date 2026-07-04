# Bitaxe Rust Firmware

## What This Is

Bitaxe Rust Firmware is a Rust ESP-IDF firmware monorepo for Bitaxe ESP-Miner. It builds a maintainable Rust implementation of the Bitaxe ESP32 miner firmware with device-user parity against upstream `bitaxeorg/ESP-Miner`, while keeping the upstream C code as a pinned read-only reference implementation.

The project is for Bitaxe owners and firmware contributors who need firmware that can be built, flashed, configured, monitored, updated, and audited with the same observable behavior they expect from upstream ESP-Miner.

## Core Value

A Bitaxe owner can build, flash, run, configure, monitor, and update Rust firmware on real Bitaxe hardware with the same observable behavior they expect from upstream ESP-Miner.

## Current State

v1.0 Ultra 205 Parity shipped on 2026-07-04. The milestone completed 21 phases, 116 plans, 226 tasks, 64/64 v1 requirements, and a final milestone audit with no requirement, integration, or flow gaps.

The shipped v1.0 evidence set proves the Ultra 205 BM1366 path through exact-claim evidence governance: Rust ESP-IDF foundation, package/flash/monitor workflows, typed config/NVS behavior, BM1366 protocol and diagnostic hardware evidence, Stratum v1 mining runtime, AxeOS-compatible API/static/WebSocket surfaces, safety controllers, release artifacts, OTA/recovery boundaries, redaction review, and controlled no-share live mining/soak closure.

The active planning surface is intentionally reset for the next milestone. Historical v1.0 roadmap, requirements, and audit records live under `.planning/milestones/`.

## Next Milestone Goals

Not yet defined. Start the next milestone with `/gsd-new-milestone` so the next requirement set is discovered from current project goals instead of inherited from v1.0.

Likely candidate themes, if prioritized during the next milestone discussion:

- Close selected exact-claim parity rows that remain below `verified`, such as accepted/rejected live share behavior, active voltage/fan/fault/self-test/load behavior, OTAWWW whole-`www` update behavior, rollback/boot-validation, destructive recovery cases, and broader production mining soak criteria.
- Add dedicated hardware evidence paths for non-205 boards or ASICs instead of inheriting Ultra 205 evidence.
- Reassess platform evolution such as ESP-IDF 6 or UI replacement only after the next milestone requirements justify it.

## Requirements

### Validated

- Rust firmware monorepo with Bazel as canonical automation graph and `just` as the ergonomic command surface - v1.0.
- Pinned read-only upstream ESP-Miner reference implementation at `reference/esp-miner` - v1.0.
- Device-user parity as the canonical rewrite scope with parity evidence in `docs/parity/checklist.md` - v1.0.
- ESP-IDF Rust bindings as the first production firmware stack - v1.0.
- Bitaxe Ultra 205 BM1366 as the first hardware bring-up, USB flashing, smoke, and acceptance target - v1.0.
- ESP-IDF firmware app plus pure Rust crates for core logic, ASIC behavior, Stratum behavior, config, API models, safety decisions, release tooling, and test support - v1.0.
- Provenance and license guardrails for MIT-first original work and explicit GPL-3.0 upstream-derived materials - v1.0.
- Ultra 205 release-parity evidence governance with conservative exact non-claims for surfaces that lack required hardware evidence - v1.0.

### Active

No active requirements are defined for the next milestone yet.

### Out of Scope

- Rewriting the Angular AxeOS UI in v1.0 - v1.0 targets API and static asset compatibility, not a new frontend.
- Bare-metal `no_std` firmware as the first production stack - ESP-IDF Rust is the practical stack for matching upstream device-user behavior.
- Modifying upstream ESP-Miner files inside `reference/esp-miner` - that tree remains read-only reference evidence.
- Claiming boards other than Ultra 205 are hardware-verified before each board has its own evidence set.
- Marking parity items verified without exact evidence - release readiness is evidence-backed, not implementation-only.
- Treating controlled no-share mining evidence as proof of accepted/rejected production share behavior.
- Treating package/static asset presence as proof of whole-`www` OTAWWW update parity without update and recovery evidence.

## Context

The accepted brief lives at `docs/project/gsd-new-project-brief.md`, with supporting decision docs under `docs/project/`, ADRs under `docs/adr/`, parity checklist under `docs/parity/checklist.md`, release docs under `docs/release/`, and provenance policy in `PROVENANCE.md`.

The first verified hardware target is Bitaxe Ultra 205 with BM1366 ASIC. This superseded the earlier Gamma 601-first direction through ADR-0014. Gamma 601 with BM1370 and other upstream boards remain in parity scope but require separate evidence before any verified hardware claim.

The seed layout separates hardware-bound firmware from testable Rust logic:

- `reference/esp-miner` for the read-only upstream submodule.
- `firmware/bitaxe` for the ESP-IDF Rust firmware app.
- `crates/bitaxe-core` for pure shared firmware/domain logic.
- `crates/bitaxe-asic` for ASIC protocol and model support.
- `crates/bitaxe-stratum` for Stratum v1/v2 protocol logic.
- `crates/bitaxe-config` for board, device, and NVS config models.
- `crates/bitaxe-api` for AxeOS API models and OpenAPI compatibility.
- `crates/bitaxe-safety` for power, thermal, fan, fault, self-test, and watchdog decisions.
- `crates/bitaxe-test-support` for fixtures, golden data, and hardware test helpers.
- `tools/flash` and `tools/parity` for workflow support.
- `scripts/` for reference, packaging, release, hardware-evidence, and redaction helpers.

## Constraints

- **Tech stack**: Use ESP-IDF Rust bindings for the production firmware stack because upstream ESP-Miner depends on ESP-IDF services such as Wi-Fi, HTTP serving, NVS, SPIFFS, OTA, FreeRTOS tasks, PSRAM-aware allocation, logging, partition images, and ESP flashing conventions.
- **Build orchestration**: Use Bazel as the canonical automation graph and `just` as the human command surface.
- **Reference implementation**: Keep upstream ESP-Miner pinned and read-only at `reference/esp-miner`.
- **Parity evidence**: Maintain a parity checklist with breadcrumbs, implementation pointers, statuses, and verification evidence. Implemented code alone is not a parity claim.
- **Hardware priority**: Ultra 205 BM1366 is the verified v1.0 target. Other boards remain in scope but need separate evidence.
- **Architecture**: Prefer functional core and imperative shell. Pure logic belongs in testable crates; ESP-IDF, FreeRTOS, Wi-Fi, NVS, SPIFFS, OTA, serial, GPIO, I2C, ADC, power, display, and task orchestration stay in firmware adapters.
- **Licensing**: Keep original work MIT-first where legally possible, mark intentionally ported GPL-covered source expression as GPL-3.0-compatible, and review distributed firmware artifacts before release.
- **Safety**: Hardware-control surfaces such as voltage, fan, thermal, power, and ASIC initialization require exact hardware evidence before verified parity claims.

## Key Decisions

| Decision | Rationale | Outcome |
| --- | --- | --- |
| Device-user parity defines rewrite scope. | Observable behavior matters more than preserving C internals. | Accepted in v1.0. |
| This repo is the Rust firmware monorepo. | The Rust implementation needs independent architecture while upstream remains reference evidence. | Accepted in v1.0. |
| ESP-IDF Rust is the production stack. | It preserves access to the platform services upstream already relies on. | Accepted in v1.0. |
| Bazel owns automation and `just` owns ergonomics. | Build, test, package, flash, and release workflows need a shared graph with convenient commands. | Accepted in v1.0. |
| Upstream ESP-Miner is read-only at `reference/esp-miner`. | Normal project work must not hide local patches inside the reference implementation. | Enforced by reference guardrails. |
| The parity checklist is audit evidence. | Release readiness should be proved by evidence, not task completion claims. | Enforced through `just parity` and milestone audit. |
| Ultra 205 BM1366 is first hardware target. | The available connected hardware is an Ultra 205, and evidence confirms safe flash/boot workflows on that board. | Accepted in ADR-0014 and validated through v1.0 evidence. |
| Gamma 601 BM1370 is deferred. | Gamma 601 remains in scope but should not block or inherit the first evidence-backed v1.0 path. | Accepted in ADR-0014; remains future scope. |
| Reference breadcrumbs appear at module and behavior boundaries. | Breadcrumbs preserve provenance without forcing line-by-line translation comments. | Adopted in v1.0. |
| AxeOS scope is API and asset compatibility first. | Rewriting Angular would expand the firmware parity milestone beyond its goal. | Preserved through v1.0. |
| USB flashing is a first-class `just` workflow. | A connected Bitaxe should be easy to build, flash, and monitor over USB. | Implemented and evidence-backed for Ultra 205. |
| MIT-first original code with GPL guardrails. | The repo is MIT-licensed, but upstream ESP-Miner is GPL-3.0 and provenance must be explicit. | Implemented through provenance and release gates. |

## Evolution

This document evolves at milestone boundaries. The next update should happen when `/gsd-new-milestone` defines the next active requirements and roadmap.

______________________________________________________________________

*Last updated: 2026-07-04 after v1.0 Ultra 205 Parity milestone completion*
