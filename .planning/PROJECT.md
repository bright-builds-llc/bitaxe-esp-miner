# Bitaxe Rust Firmware

## What This Is

Bitaxe Rust Firmware is a new Rust firmware monorepo for Bitaxe ESP-Miner. It builds a Rust implementation of the Bitaxe ESP32 miner firmware with device-user parity against upstream `bitaxeorg/ESP-Miner`, while keeping the upstream C code as a pinned read-only reference implementation.

The project is for Bitaxe owners and firmware contributors who need a maintainable Rust firmware that can be built, flashed, configured, monitored, and updated with the same observable behavior as upstream ESP-Miner.

## Core Value

A Bitaxe owner can build, flash, run, configure, monitor, and update Rust firmware on real Bitaxe hardware with the same observable behavior they expect from upstream ESP-Miner.

## Requirements

### Validated

<!-- Shipped and confirmed valuable. -->

(None yet - ship to validate)

### Active

<!-- Current scope. Building toward these. -->

- [ ] Establish a Rust firmware monorepo with Bazel as the canonical automation graph and `just` as the ergonomic command surface.
- [ ] Include upstream ESP-Miner as a pinned read-only reference implementation at `reference/esp-miner`.
- [ ] Preserve device-user parity as the canonical rewrite scope and maintain parity evidence in `docs/parity/checklist.md`.
- [ ] Use ESP-IDF Rust bindings for the first production firmware stack.
- [ ] Prioritize Bitaxe Ultra 205 BM1366 for first hardware bring-up, USB flashing, smoke tests, and early acceptance.
- [ ] Split the implementation into an ESP-IDF firmware app plus pure Rust crates for core logic, ASIC behavior, Stratum behavior, config, API models, and test support.
- [ ] Provide provenance and license guardrails so original work stays MIT-first where possible while GPL-3.0 upstream-derived work is handled explicitly.
- [ ] Deliver the first milestone as project foundation plus a minimal Ultra 205 boot/log firmware path, not full mining parity.

### Out of Scope

<!-- Explicit boundaries. Includes reasoning to prevent re-adding. -->

- Rewriting the Angular AxeOS UI in the initial project scope - the first Rust firmware scope targets API and static asset compatibility.
- Full mining parity in the first milestone - the first milestone exists to establish foundation, build/package/flash flow, and Ultra 205 boot logging.
- Bare-metal `no_std` firmware as the first production stack - ESP-IDF Rust is the practical stack for matching upstream device-user behavior.
- Modifying upstream ESP-Miner files inside `reference/esp-miner` - that tree is read-only reference evidence.
- Claiming boards other than the current Ultra 205 target are hardware-verified before evidence exists - all upstream boards remain in parity scope, but verification requires recorded evidence.
- Marking parity items verified without explicit evidence - release readiness must be derived from audit evidence, not implementation status alone.

## Context

The repository currently contains Bright Builds rules and a prerequisite documentation packet for GSD New Project. The accepted brief lives at `docs/project/gsd-new-project-brief.md`, with supporting decision docs under `docs/project/`, ADRs under `docs/adr/`, parity seed checklist under `docs/parity/checklist.md`, and provenance policy in `PROVENANCE.md`.

Device-user parity is the canonical definition of "full parity." The Rust firmware must match observable behavior that a Bitaxe user, administrator, mining pool, API client, or flashing tool relies on from upstream ESP-Miner. This includes mining behavior, supported Bitaxe board configs, ASIC support, Stratum v1/v2 behavior, AxeOS HTTP/WebSocket API compatibility, NVS/settings behavior, OTA/update flow, self-test, display/input, power/thermal/fan control, logging/statistics, image packaging, and USB flashing ergonomics.

The first hardware target is Bitaxe Ultra 205 with BM1366 ASIC. This supersedes the earlier Gamma 601-first decision because the available verified hardware is the Ultra 205. Upstream breadcrumbs for the first target include `reference/esp-miner/config-205.cvs`, `reference/esp-miner/main/device_config.h`, `reference/esp-miner/components/asic/bm1366.c`, and `reference/esp-miner/components/asic/include/bm1366.h`. Gamma 601 with BM1370 remains in parity scope but is deferred until it has its own evidence set.

The accepted seed layout separates hardware-bound firmware from testable Rust logic:

- `reference/esp-miner` for the read-only upstream submodule.
- `firmware/bitaxe` for the ESP-IDF Rust firmware app.
- `crates/bitaxe-core` for pure shared firmware/domain logic.
- `crates/bitaxe-asic` for ASIC protocol and model support.
- `crates/bitaxe-stratum` for Stratum v1/v2 protocol logic.
- `crates/bitaxe-config` for board, device, and NVS config models.
- `crates/bitaxe-api` for AxeOS API models and OpenAPI compatibility.
- `crates/bitaxe-test-support` for fixtures, golden data, and hardware test helpers.
- `tools/flash` and `tools/parity` for workflow support.
- `scripts/verify-reference-clean.sh` for reference guard verification.

## Current State

Phase 05 is complete: the repo now has AxeOS-compatible API route contracts, handwritten wire DTOs, settings PATCH planning and firmware persistence, retained log download/raw WebSocket behavior, live telemetry cadence planning, safe command routes, static AxeOS route usage evidence, and Phase 5 parity evidence. Live Ultra 205 HTTP/WebSocket smoke, SPIFFS/static packaging, recovery, OTA, and safety-controller hardware behavior remain later-phase work and are not claimed as verified.

## Constraints

- **Tech stack**: Use ESP-IDF Rust bindings for the first production firmware stack - upstream ESP-Miner depends heavily on ESP-IDF services such as Wi-Fi, HTTP serving, NVS, SPIFFS, OTA, FreeRTOS tasks, PSRAM-aware allocation, logging, partition images, and ESP flashing conventions.
- **Build orchestration**: Use Bazel as the canonical automation graph and `just` as the human command surface - local development and CI should route through the same graph where practical.
- **Reference implementation**: Keep upstream ESP-Miner pinned and read-only at `reference/esp-miner` - it is behavioral evidence, not a workspace for project changes.
- **Parity evidence**: Maintain a parity checklist with breadcrumbs, implementation pointers, statuses, and verification evidence - implemented code is not enough to claim parity.
- **Hardware priority**: Optimize first bring-up for Ultra 205 BM1366 - other upstream boards remain in scope but require their own evidence before verification claims.
- **Architecture**: Prefer functional core and imperative shell - pure logic belongs in testable crates, while ESP-IDF, FreeRTOS, Wi-Fi, NVS, SPIFFS, OTA, serial, GPIO, I2C, ADC, power, display, and task orchestration stay in firmware adapters.
- **Licensing**: Keep original work MIT-first where legally possible, but mark intentionally ported GPL-covered source expression as GPL-3.0-compatible and review distributed firmware artifacts before release.
- **Safety**: Hardware-control surfaces such as voltage, fan, thermal, power, and ASIC initialization require hardware evidence before verified parity.

## Key Decisions

<!-- Decisions that constrain future work. Add throughout project lifecycle. -->

| Decision | Rationale | Outcome |
| --- | --- | --- |
| Device-user parity defines rewrite scope. | Observable behavior matters more than preserving C internals. | - Pending |
| This repo is the new Rust firmware monorepo. | The Rust implementation needs independent architecture while upstream remains reference evidence. | - Pending |
| ESP-IDF Rust is the first production stack. | It preserves access to the platform services upstream already relies on. | - Pending |
| Bazel owns automation and `just` owns ergonomics. | Build, test, package, flash, and release workflows need a shared graph with convenient commands. | - Pending |
| Upstream ESP-Miner is read-only at `reference/esp-miner`. | Normal project work must not hide local patches inside the reference implementation. | - Pending |
| The parity checklist is audit evidence. | Release readiness should be proved by evidence, not task completion claims. | - Pending |
| Ultra 205 BM1366 is first hardware target. | The available connected hardware is an Ultra 205, and recent evidence confirms safe flash/boot workflows on that board. | Accepted in ADR-0014; safe-state flash/boot verified in `docs/parity/evidence/ultra-205-pivot-safe-state-smoke-2026-06-26.md`. |
| Gamma 601 BM1370 is deferred. | Gamma 601 remains in scope, but it should not block the first evidence-backed V1 path or inherit Ultra 205 verification. | Accepted in ADR-0014. |
| Reference breadcrumbs appear at module and behavior boundaries. | Breadcrumbs preserve provenance without forcing line-by-line translation comments. | - Pending |
| AxeOS scope is API and asset compatibility first. | Rewriting the Angular UI would expand the first project beyond firmware parity. | - Pending |
| USB flashing is a first-class `just` workflow. | A connected Bitaxe should be easy to build, flash, and monitor over USB. | - Pending |
| MIT-first original code with GPL guardrails. | The repo is MIT-licensed, but upstream ESP-Miner is GPL-3.0 and provenance must be explicit. | - Pending |

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `/gsd-transition`):

1. Requirements invalidated? Move to Out of Scope with reason.
1. Requirements validated? Move to Validated with phase reference.
1. New requirements emerged? Add to Active.
1. Decisions to log? Add to Key Decisions.
1. "What This Is" still accurate? Update if drifted.

**After each milestone** (via `/gsd-complete-milestone`):

1. Full review of all sections.
1. Core Value check - still the right priority?
1. Audit Out of Scope - reasons still valid?
1. Update Context with current state.

______________________________________________________________________

*Last updated: 2026-06-27 after Phase 05 AxeOS API, logs, and telemetry verification*
