---
gsd_state_version: 1.0
milestone: v1.0
milestone_name: milestone
status: executing
stopped_at: Completed 02-ultra-205-config-and-nvs-model-01-PLAN.md
last_updated: "2026-06-26T16:46:01.996Z"
last_activity: 2026-06-26
progress:
  total_phases: 8
  completed_phases: 1
  total_plans: 13
  completed_plans: 10
  percent: 77
---

______________________________________________________________________

## gsd_state_version: 1.0 milestone: v1.0 milestone_name: milestone status: complete stopped_at: Ultra 205 safe-state boot/flash verified last_updated: "2026-06-26T13:37:28.000Z" last_activity: 2026-06-26 progress: total_phases: 8 completed_phases: 1 total_plans: 9 completed_plans: 9 percent: 100

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-06-26)

**Core value:** A Bitaxe owner can build, flash, run, configure, monitor, and update Rust firmware on real Bitaxe hardware with the same observable behavior they expect from upstream ESP-Miner.
**Current focus:** Phase 02 — ultra-205-config-and-nvs-model

## Current Position

Phase: 02 (ultra-205-config-and-nvs-model) — EXECUTING
Plan: 2 of 4
Status: Ready to execute
Last activity: 2026-06-26

Progress: [██████████] 100%

## Performance Metrics

**Velocity:**

- Total plans completed: 9
- Average duration: 10 min
- Total execution time: 1.5 hours

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
| --- | --- | --- | --- |
| 1. Foundation And Ultra 205 Boot/Log | 9/9 | 1.5h | 10 min |

**Recent Trend:**

- Last 5 plans: 01-05 (8 min), 01-06 (16 min), 01-07 (11 min), 01-08 (20 min), 01-09 (10 min)
- Trend: Phase 1 plan execution complete; post-pivot Ultra 205 hardware smoke passed after ADR-0014.

*Updated after each plan completion*
| Phase 01 P03 | 7 min | 2 tasks | 11 files |
| Phase 01-foundation-and-gamma-601-boot-log P04 | 4 min | 2 tasks | 11 files |
| Phase 01-foundation-and-gamma-601-boot-log P05 | 8 min | 2 tasks | 11 files |
| Phase 01-foundation-and-gamma-601-boot-log P06 | 16 | 2 tasks | 11 files |
| Phase 01-foundation-and-gamma-601-boot-log P07 | 11 | 2 tasks | 6 files |
| Phase 01-foundation-and-gamma-601-boot-log P08 | 20 | 2 tasks | 11 files |
| Phase 01-foundation-and-gamma-601-boot-log P09 | 10 | 3 tasks | 4 files |
| Phase 02-ultra-205-config-and-nvs-model P01 | 14 min | 2 tasks | 15 files |

## Accumulated Context

### Decisions

Decisions are logged in PROJECT.md Key Decisions table.
Recent decisions affecting current work:

- Roadmap: Keep the research-shaped eight-phase V1 structure despite coarse granularity because the requirements form distinct evidence and safety boundaries.
- Scope: V1 is Ultra 205 BM1366 device-user parity; Gamma 601, Stratum v2 completeness, BAP completeness, all-board release matrix, and Angular UI rewrite remain deferred or out of scope.
- Phase 1: Foundation includes safe Ultra 205 boot/log only, with mining and hardware control disabled.
- \[Quick 260626-bnt\]: ADR-0014 supersedes the Gamma 601-first decision. Ultra 205/BM1366 is the initial parity target; Gamma 601/BM1370 remains deferred until it has its own evidence set.
- \[Quick 260626-bnt\]: `Phase1BoardSelection::ultra_205()` now exposes the `config-205.cvs` identity/default values (`ultra`, `205`, `BM1366`, `485`, `1200`) while still avoiding NVS, Wi-Fi, mining, ASIC behavior, voltage, fan, thermal, and power side effects.
- \[Quick 260626-bnt\]: Post-pivot evidence in `docs/parity/evidence/ultra-205-pivot-safe-state-smoke-2026-06-26.md` verifies Ultra 205 package artifacts, `board=205` dry-runs, deferred `board=601` rejection, real flash-monitor, boot identity, PSRAM/platform logging, and safe-state disabled mining/work/control.
- \[Phase 01-foundation-and-gamma-601-boot-log\]: Track MODULE.bazel.lock and ignore bazel-\* output trees so Bzlmod resolution is reproducible without committing generated build output.
- \[Phase 01-foundation-and-gamma-601-boot-log\]: Use rules_shell 0.8.0 for Bazel-visible shell targets because Bazel 9.1.1 did not expose native sh_binary/sh_test in this workspace.
- \[Phase 01-foundation-and-gamma-601-boot-log\]: Pin reference/esp-miner to c1915b0a63bfabebdb95a515cedfee05146c1d50 and initialize nested upstream submodules for recursive cleanliness.
- \[Phase 01-foundation-and-gamma-601-boot-log\]: Keep root Cargo.toml virtual with members = [] until package directories are created. — Plans 03-05 add members as each package exists so Cargo commands never point at missing packages.
- \[Phase 01-foundation-and-gamma-601-boot-log\]: Wire crate_universe to Cargo.toml and future Cargo.lock without generating the lockfile in Plan 02. — Plan 03 owns first package creation and lockfile generation; Plan 02 only establishes the mirror contract.
- \[Phase 01-foundation-and-gamma-601-boot-log\]: Historical Gamma-first note superseded by ADR-0014; active Phase 1 domain values now represent Ultra 205, BM1366, and disabled mining/work/hardware-control state.
- \[Phase 01-foundation-and-gamma-601-boot-log\]: Keep Phase1BoardSelection identity-only; NVS, Wi-Fi, mining, ASIC behavior, voltage, fan, thermal, and power stay outside the pure config crate.
- \[Phase 01-foundation-and-gamma-601-boot-log\]: Honor the AGENTS.md Rust pre-commit rule by recording TDD RED failures without committing failing intermediate states.
- \[Phase 01-foundation-and-gamma-601-boot-log\]: Represent deferred ASIC, Stratum, and API surfaces as explicit single-variant enums instead of empty modules or active skeletons.
- \[Phase 01-foundation-and-gamma-601-boot-log\]: Keep Phase 1 deferred surface crates dependency-free and side-effect-free; later phases add behavior with evidence.
- \[Phase 01-foundation-and-gamma-601-boot-log\]: Honor AGENTS.md Rust pre-commit rule by recording TDD RED failures without committing failing intermediate states.
- \[Phase 01-foundation-and-gamma-601-boot-log\]: Keep firmware and host tool entrypoints empty until their owning implementation plans add behavior.
- \[Phase 01-foundation-and-gamma-601-boot-log\]: Do not add a firmware Bazel target in Plan 05 because Plan 06 owns the ESP-IDF firmware Bazel integration.
- \[Phase 01-foundation-and-gamma-601-boot-log\]: Keep Plan 05 host tools free of process execution, package generation, parity mutation, flashing, monitoring, and hardware-control behavior.
- \[Phase 01-foundation-and-gamma-601-boot-log\]: Use Cargo build-std for xtensa-esp32s3-espidf so plain target commands work with the checked-in esp rust-src component.
- \[Phase 01-foundation-and-gamma-601-boot-log\]: Export ESP_IDF_VERSION and related esp-idf-sys settings in the Bazel wrapper so Bazel cannot fall back to the crate default ESP-IDF v5.2.3.
- \[Phase 01-foundation-and-gamma-601-boot-log\]: Use heap_caps_get_total_size(MALLOC_CAP_SPIRAM) for PSRAM status because the direct esp_psram_is_initialized symbol did not link in this build.
- \[Phase 01-foundation-and-gamma-601-boot-log\]: Run //scripts:verify_reference_clean before checklist parsing, reference commit lookup, or report output.
- \[Phase 01-foundation-and-gamma-601-boot-log\]: Keep implemented checklist rows at Evidence = pending until command or hardware evidence is recorded.
- \[Phase 01-foundation-and-gamma-601-boot-log\]: Keep Justfile and //firmware/bitaxe:firmware_image rows not-started because those artifacts do not exist yet.
- \[Phase 01-foundation-and-gamma-601-boot-log\]: Honor AGENTS.md Rust pre-commit rules by recording TDD RED failures without committing failing intermediate states.
- \[Phase 01-foundation-and-gamma-601-boot-log\]: Run package and flash Bazel subprocesses from the real workspace path when Bazel actions or bazel run start in execroot/output directories.
- \[Phase 01-foundation-and-gamma-601-boot-log\]: Historical Gamma-first package note superseded by ADR-0014; active default flash image is `bitaxe-ultra205.elf`, with the factory bin as additional metadata.
- \[Phase 01-foundation-and-gamma-601-boot-log\]: Honor AGENTS.md Rust pre-commit rules by recording TDD RED failures without committing failing intermediate states.
- \[Phase 01-foundation-and-gamma-601-boot-log\]: Use workflow evidence for command-surface rows proved by Plan 09 Just/Bazel command output.
- \[Phase 01-foundation-and-gamma-601-boot-log\]: Historical Gamma-first hardware evidence note superseded by ADR-0014; active safe-state boot/log rows are verified by Ultra 205 hardware smoke, while safety-critical hardware-control rows remain below verified.
- \[Phase 01-foundation-and-gamma-601-boot-log\]: Historical missing Gamma 601 hardware-smoke evidence is preserved in Phase 1 artifacts and does not apply to Ultra 205 verification.
- [Phase 02-ultra-205-config-and-nvs-model]: Keep Phase1BoardSelection::ultra_205() as a compatibility shim while exposing Phase 2 defaults/catalog modules.
- [Phase 02-ultra-205-config-and-nvs-model]: Represent all non-205 upstream boards in the catalog as NotHardwareVerified so Ultra 205 evidence cannot be inherited.
- [Phase 02-ultra-205-config-and-nvs-model]: Treat reference-derived fixture files as GPL-risk source data with explicit provenance metadata.

### Pending Todos

None yet.

### Blockers/Concerns

- Hardware evidence: Mining, ASIC init, voltage, fan, thermal, power, and safety-critical verification still need Ultra 205 hardware-smoke or regression evidence before `verified` parity claims.
- Release scope: Non-205 boards and ASICs must remain unverified or deferred until each has its own evidence set.

## Session Continuity

Last session: 2026-06-26T16:46:01.994Z
Stopped at: Completed 02-ultra-205-config-and-nvs-model-01-PLAN.md
Resume file: None
