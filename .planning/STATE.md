---
gsd_state_version: 1.0
milestone: v1.0
milestone_name: milestone
status: executing
stopped_at: Completed 01-foundation-and-gamma-601-boot-log-07-PLAN.md
last_updated: "2026-06-21T03:34:37.238Z"
last_activity: 2026-06-21
progress:
  total_phases: 8
  completed_phases: 0
  total_plans: 9
  completed_plans: 7
  percent: 78
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-06-20)

**Core value:** A Bitaxe owner can build, flash, run, configure, monitor, and update Rust firmware on real Bitaxe hardware with the same observable behavior they expect from upstream ESP-Miner.
**Current focus:** Phase 01 — Foundation And Gamma 601 Boot/Log

## Current Position

Phase: 01 (Foundation And Gamma 601 Boot/Log) — EXECUTING
Plan: 8 of 9
Status: Ready to execute
Last activity: 2026-06-21

Progress: [██████░░░░] 56%

## Performance Metrics

**Velocity:**

- Total plans completed: 5
- Average duration: 7 min
- Total execution time: 0.6 hours

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
| --- | --- | --- | --- |
| 1. Foundation And Gamma 601 Boot/Log | 5/9 | 0.6h | 7 min |

**Recent Trend:**

- Last 5 plans: 01-01 (10 min), 01-02 (4 min), 01-03 (7 min), 01-04 (4 min), 01-05 (8 min)
- Trend: Foundation setup progressing

*Updated after each plan completion*
| Phase 01 P03 | 7 min | 2 tasks | 11 files |
| Phase 01-foundation-and-gamma-601-boot-log P04 | 4 min | 2 tasks | 11 files |
| Phase 01-foundation-and-gamma-601-boot-log P05 | 8 min | 2 tasks | 11 files |
| Phase 01-foundation-and-gamma-601-boot-log P06 | 16 | 2 tasks | 11 files |
| Phase 01-foundation-and-gamma-601-boot-log P07 | 11 | 2 tasks | 6 files |

## Accumulated Context

### Decisions

Decisions are logged in PROJECT.md Key Decisions table.
Recent decisions affecting current work:

- Roadmap: Keep the research-shaped eight-phase V1 structure despite coarse granularity because the requirements form distinct evidence and safety boundaries.
- Scope: V1 is Gamma 601 BM1370 device-user parity; additional boards, Stratum v2 completeness, BAP completeness, all-board release matrix, and Angular UI rewrite remain deferred or out of scope.
- Phase 1: Foundation includes safe Gamma 601 boot/log only, with mining and hardware control disabled.
- [Phase 01-foundation-and-gamma-601-boot-log]: Track MODULE.bazel.lock and ignore bazel-* output trees so Bzlmod resolution is reproducible without committing generated build output.
- [Phase 01-foundation-and-gamma-601-boot-log]: Use rules_shell 0.8.0 for Bazel-visible shell targets because Bazel 9.1.1 did not expose native sh_binary/sh_test in this workspace.
- [Phase 01-foundation-and-gamma-601-boot-log]: Pin reference/esp-miner to c1915b0a63bfabebdb95a515cedfee05146c1d50 and initialize nested upstream submodules for recursive cleanliness.
- [Phase 01-foundation-and-gamma-601-boot-log]: Keep root Cargo.toml virtual with members = [] until package directories are created. — Plans 03-05 add members as each package exists so Cargo commands never point at missing packages.
- [Phase 01-foundation-and-gamma-601-boot-log]: Wire crate_universe to Cargo.toml and future Cargo.lock without generating the lockfile in Plan 02. — Plan 03 owns first package creation and lockfile generation; Plan 02 only establishes the mirror contract.
- [Phase 01-foundation-and-gamma-601-boot-log]: Represent Gamma 601, BM1370, and Phase 1 disabled mining/work/hardware-control state as pure domain values in bitaxe-core.
- [Phase 01-foundation-and-gamma-601-boot-log]: Keep Phase1BoardSelection identity-only; NVS, Wi-Fi, mining, ASIC behavior, voltage, fan, thermal, and power stay outside the pure config crate.
- [Phase 01-foundation-and-gamma-601-boot-log]: Honor the AGENTS.md Rust pre-commit rule by recording TDD RED failures without committing failing intermediate states.
- [Phase 01-foundation-and-gamma-601-boot-log]: Represent deferred ASIC, Stratum, and API surfaces as explicit single-variant enums instead of empty modules or active skeletons.
- [Phase 01-foundation-and-gamma-601-boot-log]: Keep Phase 1 deferred surface crates dependency-free and side-effect-free; later phases add behavior with evidence.
- [Phase 01-foundation-and-gamma-601-boot-log]: Honor AGENTS.md Rust pre-commit rule by recording TDD RED failures without committing failing intermediate states.
- [Phase 01-foundation-and-gamma-601-boot-log]: Keep firmware and host tool entrypoints empty until their owning implementation plans add behavior.
- [Phase 01-foundation-and-gamma-601-boot-log]: Do not add a firmware Bazel target in Plan 05 because Plan 06 owns the ESP-IDF firmware Bazel integration.
- [Phase 01-foundation-and-gamma-601-boot-log]: Keep Plan 05 host tools free of process execution, package generation, parity mutation, flashing, monitoring, and hardware-control behavior.
- [Phase 01-foundation-and-gamma-601-boot-log]: Use Cargo build-std for xtensa-esp32s3-espidf so plain target commands work with the checked-in esp rust-src component.
- [Phase 01-foundation-and-gamma-601-boot-log]: Export ESP_IDF_VERSION and related esp-idf-sys settings in the Bazel wrapper so Bazel cannot fall back to the crate default ESP-IDF v5.2.3.
- [Phase 01-foundation-and-gamma-601-boot-log]: Use heap_caps_get_total_size(MALLOC_CAP_SPIRAM) for PSRAM status because the direct esp_psram_is_initialized symbol did not link in this build.
- [Phase 01-foundation-and-gamma-601-boot-log]: Run //scripts:verify_reference_clean before checklist parsing, reference commit lookup, or report output.
- [Phase 01-foundation-and-gamma-601-boot-log]: Keep implemented checklist rows at Evidence = pending until command or hardware evidence is recorded.
- [Phase 01-foundation-and-gamma-601-boot-log]: Keep Justfile and //firmware/bitaxe:firmware_image rows not-started because those artifacts do not exist yet.
- [Phase 01-foundation-and-gamma-601-boot-log]: Honor AGENTS.md Rust pre-commit rules by recording TDD RED failures without committing failing intermediate states.

### Pending Todos

None yet.

### Blockers/Concerns

- Hardware evidence: Mining, ASIC init, voltage, fan, thermal, power, and safety-critical verification need Gamma 601 hardware-smoke or regression evidence before `verified` parity claims.
- Release scope: Non-601 boards and ASICs must remain unverified or deferred until each has its own evidence set.

## Session Continuity

Last session: 2026-06-21T03:34:37.236Z
Stopped at: Completed 01-foundation-and-gamma-601-boot-log-07-PLAN.md
Resume file: None
