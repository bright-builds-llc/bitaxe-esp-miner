---
gsd_state_version: 1.0
milestone: v1.0
milestone_name: milestone
status: planning
stopped_at: Phase 4 complete; Phase 5 ready for planning
last_updated: "2026-06-27T15:46:52.437Z"
last_activity: 2026-06-27
progress:
  total_phases: 8
  completed_phases: 4
  total_plans: 22
  completed_plans: 22
  percent: 100
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-06-26)

**Core value:** A Bitaxe owner can build, flash, run, configure, monitor, and update Rust firmware on real Bitaxe hardware with the same observable behavior they expect from upstream ESP-Miner.
**Current focus:** Phase 05 — axeos-api-logs-and-telemetry

## Current Position

Phase: 5
Plan: Not started
Status: Ready for Phase 5 planning
Last activity: 2026-06-27

Progress: [██████████] 100%

## Performance Metrics

**Velocity:**

- Total plans completed: 22
- Average duration: 10 min
- Total execution time: 1.5 hours

**By Phase:**
| Phase | Plans | Total | Avg/Plan |
| --- | --- | --- | --- |
| 1. Foundation And Ultra 205 Boot/Log | 9/9 | 1.5h | 10 min |
| 2 | 4 | - | - |
| 03 | 5 | - | - |
| 04 | 4 | - | - |

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
| Phase 02-ultra-205-config-and-nvs-model P02 | 13 | 2 tasks | 5 files |
| Phase 02-ultra-205-config-and-nvs-model P03 | 13 min | 2 tasks | 5 files |
| Phase 02-ultra-205-config-and-nvs-model P04 | 10 min | 2 tasks | 5 files |
| Phase 03-bm1366-asic-protocol-and-safe-initialization P01 | 11 min | 2 tasks | 11 files |
| Phase 03-bm1366-asic-protocol-and-safe-initialization P02 | 10 min | 2 tasks | 6 files |
| Phase 03-bm1366-asic-protocol-and-safe-initialization P03 | 16 min | 2 tasks | 12 files |
| Phase 03-bm1366-asic-protocol-and-safe-initialization P04 | 14 min | 2 tasks | 5 files |
| Phase 03-bm1366-asic-protocol-and-safe-initialization P05 | 5 | 3 tasks | 14 files |
| Phase 04-stratum-v1-and-first-mining-loop P01 | 10 min | 2 tasks | 10 files |
| Phase 04-stratum-v1-and-first-mining-loop P02 | 8 min | 2 tasks | 5 files |
| Phase 04-stratum-v1-and-first-mining-loop P03 | 10 min | 2 tasks | 9 files |
| Phase 04-stratum-v1-and-first-mining-loop P04 | 10 min | 3 tasks | 10 files |

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
- [Quick 260627-b0q]: Ultra 205 startup display debug text is startup-only SSD1306 output, not full LVGL/screen parity. Serial hardware smoke verified `display_status=startup_text_rendered`; user confirmed the OLED text is visible.
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
- [Phase 02-ultra-205-config-and-nvs-model]: NVS schema and migrations remain pure data/functions; ESP-IDF reads, writes, erases, and commits stay deferred to a future firmware adapter.
- [Phase 02-ultra-205-config-and-nvs-model]: Legacy keys asicfrequency, fanspeed, and fbSv2ChanType are represented explicitly so migration decisions preserve upstream compatibility.
- [Phase 02-ultra-205-config-and-nvs-model]: Corrupt FloatString values fall back to schema defaults, including Ultra 205 asicfrequency_f = 485.0.
- [Phase 02-ultra-205-config-and-nvs-model]: Keep frequency, voltage, fan, thermal, and settings validation as pure data checks with no ESP-IDF or hardware side effects.
- [Phase 02-ultra-205-config-and-nvs-model]: Use all_settings_schema() as the settings update authority so future API handlers do not duplicate validation or mapping logic.
- [Phase 02-ultra-205-config-and-nvs-model]: Preserve upstream legacy mirror writes for frequency and manual fan updates.
- [Phase 02-ultra-205-config-and-nvs-model]: Keep persistence in crates/bitaxe-config pure: no ESP-IDF NVS calls, HTTP handlers, flashing, mining, ASIC, voltage, fan, thermal, power, or hardware side effects.
- [Phase 02-ultra-205-config-and-nvs-model]: Use SettingsUpdateDecision and existing NvsWrite/NvsErase data as the adapter contract for future firmware storage work.
- [Phase 02-ultra-205-config-and-nvs-model]: Leave CFG-001 and CFG-005 implemented rather than verified where the checklist row would otherwise overclaim hardware-control or API route evidence.
- [Phase 03-bm1366-asic-protocol-and-safe-initialization]: Keep raw BM1366 frame construction inside bitaxe-asic through CommandFrame, JobFrame, and FrameBytes. — Firmware adapters should translate typed ASIC actions instead of constructing preambles, lengths, payload bytes, or CRCs.
- [Phase 03-bm1366-asic-protocol-and-safe-initialization]: Compute CRC16-FALSE bitwise instead of copying the upstream GPL CRC table. — This preserves MIT-first source posture while matching the pinned reference behavior.
- [Phase 03-bm1366-asic-protocol-and-safe-initialization]: Run TDD RED failures but avoid committing failing RED states under the Rust pre-commit rule. — AGENTS.md requires passing format, clippy, build, and tests before every commit.
- [Phase 03-bm1366-asic-protocol-and-safe-initialization]: Keep work construction explicitly diagnostic through diagnostic_job_frame.
- [Phase 03-bm1366-asic-protocol-and-safe-initialization]: Reject stale or unknown result job IDs through Bm1366ValidJobIds before producing nonce observations.
- [Phase 03-bm1366-asic-protocol-and-safe-initialization]: Add InvalidCoreId so nonce-derived core IDs outside the BM1366 normal-core range fail as typed protocol faults.
- [Phase 03-bm1366-asic-protocol-and-safe-initialization]: Only board version 205, family Ultra, ASIC model BM1366, count 1, and ActiveUltra205 scope return ActiveBm1366.
- [Phase 03-bm1366-asic-protocol-and-safe-initialization]: Firmware-facing ASIC behavior is expressed as Bm1366Command, Bm1366AdapterAction, Bm1366Observation, and AsicInitStatus while raw frames stay inside bitaxe-asic.
- [Phase 03-bm1366-asic-protocol-and-safe-initialization]: Fake UART transcripts fail closed on timeout, partial read, bad preamble, bad CRC, unknown register, invalid job ID, and chip-count mismatch.
- [Phase 03-bm1366-asic-protocol-and-safe-initialization]: Run TDD RED failures but avoid committing failing RED states because AGENTS.md requires passing Rust checks before every commit.
- [Phase 03-bm1366-asic-protocol-and-safe-initialization]: Use Phase 2 Ultra 205 BM1366 catalog/default facts as init preflight gates instead of duplicating board identity in firmware.
- [Phase 03-bm1366-asic-protocol-and-safe-initialization]: Keep voltage transitions as pure data only and mark both frequency and voltage effects below verified until Ultra 205 hardware evidence exists.
- [Phase 03-bm1366-asic-protocol-and-safe-initialization]: Use an independently written pure PLL search for BM1366 frequency command data while preserving MissingHardwareEvidence status.
- [Phase 03-bm1366-asic-protocol-and-safe-initialization]: Use the safe skip path for the human-gated checkpoint because no live Ultra 205 flashing/monitoring approval or port was provided.
- [Phase 03-bm1366-asic-protocol-and-safe-initialization]: Do not run flash, monitor, chip-detect, or port detection during Plan 03-05 checkpoint continuation.
- [Phase 03-bm1366-asic-protocol-and-safe-initialization]: Keep phase-03-ultra-205-bm1366-chip-detect.md concluded as not run - hardware verification pending.
- [Phase 03-bm1366-asic-protocol-and-safe-initialization]: Keep ASIC-002 through ASIC-007 below verified until board-named Ultra 205 chip-detect hardware-smoke evidence exists.
- [Phase 04-stratum-v1-and-first-mining-loop]: Stratum v1 parser rejects unknown methods, invalid hex fields, malformed params, malformed responses, and oversized extranonce2 lengths before mining state can consume pool data.
- [Phase 04-stratum-v1-and-first-mining-loop]: Protocol tests and fixtures use synthetic usernames/passwords only, keeping real pool credentials out of source and evidence.
- [Phase 04-stratum-v1-and-first-mining-loop]: TDD RED failures were run but not committed because AGENTS.md requires passing Rust verification before every commit.
- [Phase 04-stratum-v1-and-first-mining-loop]: Mining runtime state remains pure and telemetry-ready; no Phase 5 HTTP/WebSocket handlers, firmware sockets, live pool calls, or hardware side effects were introduced.
- [Phase 04-stratum-v1-and-first-mining-loop]: Fake-pool transcripts update state only from typed Stratum client/server messages and fail on unexpected client messages instead of advancing silently.
- [Phase 04-stratum-v1-and-first-mining-loop]: Timeout transitions represent fallback activation in the deterministic fake-pool harness, while disconnect and client.reconnect map to Reconnecting lifecycle state.
- [Phase 04-stratum-v1-and-first-mining-loop]: TDD RED failures were run but not committed because AGENTS.md requires passing Rust verification before every commit.
- [Phase 04-stratum-v1-and-first-mining-loop]: Stratum mining job construction produces typed Bm1366WorkFields and never constructs raw ASIC JobFrame or CommandFrame values.
- [Phase 04-stratum-v1-and-first-mining-loop]: Malformed hex and oversized extranonce2 lengths fail with StratumV1Error before pool data can become mining work.
- [Phase 04-stratum-v1-and-first-mining-loop]: Clean-jobs behavior is explicit through MiningWorkQueue::clear_jobs, which clears both queued work and Bm1366ValidJobIds.
- [Phase 04-stratum-v1-and-first-mining-loop]: TDD RED failures were run but not committed because AGENTS.md requires passing Rust verification before every commit.
- [Phase 04-stratum-v1-and-first-mining-loop]: Mining-loop work submission defaults to hardware_evidence_ack_missing and reaches Ready only when ASIC initialization, safety evidence, and hardware-evidence acknowledgement are all present.
- [Phase 04-stratum-v1-and-first-mining-loop]: Stratum v2 remains deferred by Phase 4 scope; Plan 04 records the Stratum v1 first-loop boundary only.
- [Phase 04-stratum-v1-and-first-mining-loop]: Phase 4 checklist rows advance pure Stratum v1, fake-pool, job, queue, and fail-closed coordination surfaces to implemented, while live hardware mining smoke and soak remain not run - hardware evidence pending.
- [Phase 04-stratum-v1-and-first-mining-loop]: Firmware publishes mining_loop_status=blocked reason=hardware_evidence_ack_missing work_submission=disabled while main.rs remains free of live pool sockets and BM1366 work submission.

### Pending Todos

None yet.

### Blockers/Concerns

- Hardware evidence: Mining, ASIC init, voltage, fan, thermal, power, and safety-critical verification still need Ultra 205 hardware-smoke or regression evidence before `verified` parity claims.
- Release scope: Non-205 boards and ASICs must remain unverified or deferred until each has its own evidence set.

### Quick Tasks Completed

| ID | Date | Task | Result | Evidence |
| --- | --- | --- | --- | --- |
| 260627-b0q | 2026-06-27 | Display startup debug text on Ultra 205 OLED | implemented, serial-smoked, user-visible on OLED | `.planning/quick/260627-b0q-display-startup-debug-text-on-ultra-205-/260627-b0q-SUMMARY.md`, `docs/parity/evidence/ultra-205-startup-display-debug-2026-06-27.md` |

## Session Continuity

Last session: 2026-06-27T15:06:59.728Z
Stopped at: Phase 4 complete; Phase 5 ready for planning
Resume file: None
