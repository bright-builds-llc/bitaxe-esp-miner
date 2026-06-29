---
gsd_state_version: 1.0
milestone: v1.0
milestone_name: milestone
status: executing
stopped_at: Completed 09-01-PLAN.md
last_updated: "2026-06-29T14:11:59.436Z"
last_activity: 2026-06-29
progress:
  total_phases: 13
  completed_phases: 8
  total_plans: 54
  completed_plans: 53
  percent: 98
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-06-26)

**Core value:** A Bitaxe owner can build, flash, run, configure, monitor, and update Rust firmware on real Bitaxe hardware with the same observable behavior they expect from upstream ESP-Miner.
**Current focus:** Phase 09 — flash-monitor-evidence-wrapper-hardening

## Current Position

Phase: 09 (flash-monitor-evidence-wrapper-hardening) — EXECUTING
Plan: 2 of 2
Status: Ready to execute
Last activity: 2026-06-29

Progress: [██████████] 100%

## Performance Metrics

**Velocity:**

- Total plans completed: 42
- Average duration: 10 min
- Total execution time: 1.5 hours

**By Phase:**
| Phase | Plans | Total | Avg/Plan |
| --- | --- | --- | --- |
| 1. Foundation And Ultra 205 Boot/Log | 9/9 | 1.5h | 10 min |
| 2 | 4 | - | - |
| 03 | 5 | - | - |
| 04 | 4 | - | - |
| 05 | 7 | - | - |
| 07 | 9 | - | - |
| 08 | 4 | - | - |

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
| Phase 05-axeos-api-logs-and-telemetry P01 | 11 min | 2 tasks | 9 files |
| Phase 05-axeos-api-logs-and-telemetry P02 | 10 min | 2 tasks | 4 files |
| Phase 05-axeos-api-logs-and-telemetry P03 | 12 min | 2 tasks | 11 files |
| Phase 05-axeos-api-logs-and-telemetry P04 | 9 min | 2 tasks | 6 files |
| Phase 05-axeos-api-logs-and-telemetry P06 | 8 min | 2 tasks | 4 files |
| Phase 05-axeos-api-logs-and-telemetry P05 | 38 | 2 tasks | 14 files |
| Phase 05-axeos-api-logs-and-telemetry P07 | 16m57s | 2 tasks | 10 files |
| Phase 06-safety-controllers-and-self-test P01 | 9 min | 2 tasks | 9 files |
| Phase 06 P02 | 4 min | 1 tasks | 6 files |
| Phase 06 P03 | 8 min | 2 tasks | 4 files |
| Phase 06 P04 | 18 min | 2 tasks | 5 files |
| Phase 06 P05 | 20 min | 2 tasks | 4 files |
| Phase 06 P06 | 28 min | 1 tasks | 8 files |
| Phase 06 P07 | 23 min | 2 tasks | 9 files |
| Phase 06 P08 | 7 min | 1 tasks | 9 files |
| Phase 06 P09 | 5 min | 1 tasks | 4 files |
| Phase 06 P10 | 6 min | 2 tasks | 5 files |
| Phase 07-ota-filesystem-and-release-packaging P01 | 11 min | 3 tasks | 5 files |
| Phase 07-ota-filesystem-and-release-packaging P02 | 21m02s | 3 tasks | 11 files |
| Phase 07-ota-filesystem-and-release-packaging P03 | 11m21s | 3 tasks | 8 files |
| Phase 07-ota-filesystem-and-release-packaging P04 | 19m49s | 3 tasks | 11 files |
| Phase 07-ota-filesystem-and-release-packaging P06 | 10m17s | 2 tasks | 5 files |
| Phase 07-ota-filesystem-and-release-packaging P05 | 20min | 3 tasks | 7 files |
| Phase 07-ota-filesystem-and-release-packaging P07 | 18min | 3 tasks | 6 files |
| Phase 07-ota-filesystem-and-release-packaging P08 | 6m28s | 3 tasks | 3 files |
| Phase 07-ota-filesystem-and-release-packaging P09 | 12m 8s | 3 tasks | 3 files |
| Phase 08-parity-evidence-and-ultra-205-release-gate P04 | 112min | 3 tasks | 7 files |
| Phase 09-flash-monitor-evidence-wrapper-hardening P01 | 11 min | 2 tasks | 1 files |

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
- [Quick 260628-kri]: ESP-IDF is a first-class contributor dependency through pinned `esp-idf-sys`; `just doctor` checks local readiness and `just bootstrap-esp` is the explicit opt-in installer for ESP Rust tooling.
- [Quick 260628-l4b]: Factory image assembly now uses ESP tooling: `espflash save-image --merge --skip-padding` creates the base image and managed `esptool.py merge_bin` adds `www.bin` at `0x410000` and `otadata-initial.bin` at `0xf10000`; the production `xtask overlay-factory-payloads` byte overlay command was removed.
- [Quick 260628-lgg]: Agents have standing repo permission to autonomously use a connected Ultra 205 for phase-gated hardware verification after `just detect-ultra205` finds exactly one ESP32-S3 USB candidate and `espflash board-info` succeeds; ambiguous or missing hardware stays pending.
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
- [Phase 05-axeos-api-logs-and-telemetry]: AxeOS wire DTOs are handwritten and independent from internal runtime/domain structs. — This preserves upstream field names, casing, units, and mixed encodings while allowing config, Stratum, ASIC, and platform structs to evolve behind ApiSnapshot.
- [Phase 05-axeos-api-logs-and-telemetry]: Safe Ultra 205 API fixtures use zeroed Phase 6-owned hardware telemetry. — Plan 05-01 proves field names and encodings without claiming live voltage, fan, thermal, power, or actual-frequency evidence before the safety-controller phase.
- [Phase 05-axeos-api-logs-and-telemetry]: Keep bitaxe-config as the only settings validation authority; bitaxe-api only parses JSON, ignores unknown fields, and maps public errors.
- [Phase 05-axeos-api-logs-and-telemetry]: Require write, commit, and reload completion before the settings route can produce an empty public success response.
- [Phase 05-axeos-api-logs-and-telemetry]: Represent hostname live apply as a best-effort post-persistence effect, not as a validation or persistence prerequisite.
- [Phase 05-axeos-api-logs-and-telemetry]: System, ASIC, mining, statistics, and scoreboard mappers stay pure and do not introduce ESP-IDF, HTTP, NVS, file, or hardware effects.
- [Phase 05-axeos-api-logs-and-telemetry]: Mining-visible share counters, rejected reasons, pool difficulty, fallback state, hashrate, and blocked status derive from MiningRuntimeState.
- [Phase 05-axeos-api-logs-and-telemetry]: Statistics and scoreboard empty states are explicit compatible response shapes, not fake historical mining data.
- [Phase 05-axeos-api-logs-and-telemetry]: Keep retained log buffering and raw log stream cursor behavior pure in bitaxe-api; ESP log hooks, mutexes, notifications, and WebSocket sends remain firmware adapter work.
- [Phase 05-axeos-api-logs-and-telemetry]: Start raw log WebSocket streams at the current absolute log end and reset that cursor while no clients are active to avoid retained-history replay.
- [Phase 05-axeos-api-logs-and-telemetry]: Model live telemetry as full update envelopes on connect, diff-only cadence frames after baseline, and baseline clearing while no live clients are active.
- [Phase 05-axeos-api-logs-and-telemetry]: Command planners return response JSON separately from typed effects so firmware route code can send the public response before executing restart, display, or state mutations.
- [Phase 05-axeos-api-logs-and-telemetry]: Pause and resume only plan MiningActivityStatus updates; resume derives Active versus SafeBlocked from the existing WorkSubmissionGate and never sets work submission readiness.
- [Phase 05-axeos-api-logs-and-telemetry]: Identify is represented as a typed on/off display effect with the upstream 30000 ms duration, while restart is represented only as an after-response effect.
- [Phase 05-axeos-api-logs-and-telemetry]: Block-found dismiss preserves blockFound, clears showNewBlock, and remains deterministic across repeated dismiss requests.
- [Phase 05-axeos-api-logs-and-telemetry]: Use raw ESP-IDF WebSocket handler registration for /api/ws and /api/ws/live while keeping unsafe calls behind small firmware helpers.
- [Phase 05-axeos-api-logs-and-telemetry]: Use raw ESP-IDF NVS calls in the firmware settings adapter so writes do not auto-commit before the pure executor commit step.
- [Phase 05-axeos-api-logs-and-telemetry]: Apply hostname changes best-effort through ESP-IDF netif handles after successful settings persistence; unavailable netifs are logged, not exposed publicly.
- [Phase 05-axeos-api-logs-and-telemetry]: Keep OTA/OTAWWW fail-closed in Phase 05; static assets and recovery-page parity remain Phase 07 scope.
- [Phase 05-axeos-api-logs-and-telemetry]: 05-07: Keep OpenAPI checks narrow and dependency-free rather than adding a deprecated YAML/OpenAPI parser dependency.
- [Phase 05-axeos-api-logs-and-telemetry]: 05-07: Use bitaxe-api's exported phase05_routes() as the Rust route manifest source of truth for API compare checks.
- [Phase 05-axeos-api-logs-and-telemetry]: 05-07: Record AxeOS static route compatibility separately from Phase 7-owned recovery/static packaging and OTA/OTAWWW success.
- [Phase 06-safety-controllers-and-self-test]: Use a focused pure bitaxe-safety crate for Phase 6 contracts before firmware hardware effects are touched.
- [Phase 06-safety-controllers-and-self-test]: Only hardware-smoke and hardware-regression evidence satisfy safety-critical hardware verification.
- [Phase 06-safety-controllers-and-self-test]: Fail-closed safety plans explicitly hold reset low, disable ASIC enable, suppress voltage writes, block work submission, and publish visible status.
- [Phase 07-ota-filesystem-and-release-packaging]: Preserve phase05_routes() for Phase 5 API compare while adding phase07_routes() for release/static/update ownership.
- [Phase 07-ota-filesystem-and-release-packaging]: Keep OTAWWW fail-closed as an explicit REL-03 gap until interruption and recovery evidence exists.
- [Phase 07-ota-filesystem-and-release-packaging]: Reject static path traversal in the pure resolver before any firmware file adapter can open a path.
- [Phase 07-ota-filesystem-and-release-packaging]: Keep existing package generation on the current v1 manifest while defining and validating the v2 release package contract for later packaging work.
- [Phase 07-ota-filesystem-and-release-packaging]: Validate the checked-in Ultra 205 partition CSV before package manifest generation so release packaging fails on partition drift.
- [Phase 07-ota-filesystem-and-release-packaging]: Keep flash compatibility anchored on top-level default_flash_image so tools/flash can read v2 manifests without adopting the full release schema.
- [Phase 07-ota-filesystem-and-release-packaging]: Keep cargo-about scoped to Cargo dependencies and require separate non-Cargo inventory for Bazel, ESP-IDF, tools, static assets, and artifacts.
- [Phase 07-ota-filesystem-and-release-packaging]: Preserve OTAWWW as an explicit REL-03 evidence gap until D-16 recovery/interruption evidence exists.
- [Phase 07-ota-filesystem-and-release-packaging]: Initialize Phase 7 evidence with not-run/live-hardware-pending conclusions instead of release parity claims.
- [Phase 07-ota-filesystem-and-release-packaging]: 07-04: Serve static HTTP paths through bitaxe-api::resolve_static_request before opening SPIFFS files.
- [Phase 07-ota-filesystem-and-release-packaging]: 07-04: Keep /recovery explicitly registered and register the static wildcard after API, OTA, and websocket routes.
- [Phase 07-ota-filesystem-and-release-packaging]: 07-04: Use Rust-owned minimal fallback and recovery assets instead of copying upstream AxeOS or recovery HTML.
- [Phase 07-ota-filesystem-and-release-packaging]: 07-04: Use an ESP-IDF-resolvable relative partition CSV path because custom partition filenames are resolved from the generated CMake project.
- [Phase 07-ota-filesystem-and-release-packaging]: 07-06: Keep release-gate validation in tools/parity with filesystem access isolated to the CLI adapter.
- [Phase 07-ota-filesystem-and-release-packaging]: 07-06: Require non-Cargo license/provenance sections so docs/release/cargo-about.html cannot satisfy REL-05 alone.
- [Phase 07-ota-filesystem-and-release-packaging]: 07-06: Record source commit provenance as the release-time git command while pinning the reference commit explicitly.
- [Phase 07-ota-filesystem-and-release-packaging]: Keep bitaxe-ultra205.elf as default_flash_image while manifest v2 lists loose OTA, SPIFFS, otadata, partition, and factory artifacts.
- [Phase 07-ota-filesystem-and-release-packaging]: Use Unavailable for the checked-in CSV partition-table offset and reserve 0x8000 for a future binary partition-table artifact.
- [Phase 07-ota-filesystem-and-release-packaging]: Declare .git/HEAD and .git/refs/heads/main as package action inputs so manifest source_commit refreshes when the main branch advances.
- [Phase 07-ota-filesystem-and-release-packaging]: Validate pending OTA images only after startup diagnostics and keep rollback evidence below verified until hardware logs exist.
- [Phase 07-ota-filesystem-and-release-packaging]: Stream firmware uploads directly from httpd_req_recv into ESP-IDF OTA APIs instead of buffering images in RAM.
- [Phase 07-ota-filesystem-and-release-packaging]: Keep OTAWWW fail-closed as an explicit REL-03 gap because interruption/recovery hardware evidence is not part of this plan.
- [Phase 07-ota-filesystem-and-release-packaging]: 07-08: Keep OTAWWW as an explicit REL-03 release gap with required UI-SPEC copy and public response `Wrong API input`.
- [Phase 07-ota-filesystem-and-release-packaging]: 07-08: Keep live OTA, rollback, recovery, erase, failed update, and interrupted-update conclusions at `not run - hardware verification pending` until Ultra 205 evidence exists.
- [Phase 07-ota-filesystem-and-release-packaging]: 07-08: Treat the hardware-smoke document as a capture template only; it does not verify checklist rows without command and log evidence.
- [Phase 07-ota-filesystem-and-release-packaging]: Keep release and OTA checklist rows below verified unless the evidence class satisfies the parity guard.
- [Phase 07-ota-filesystem-and-release-packaging]: Keep OTAWWW as explicit REL-03 gap until interrupted-update hardware regression evidence exists.
- [Phase 07-ota-filesystem-and-release-packaging]: Treat the Task 3 no-port checkpoint as not run - hardware verification pending, with no flash, OTA, monitor, erase, or rollback hardware commands run.
- [Phase 07-ota-filesystem-and-release-packaging]: Do not commit the failing TDD RED state because the repo Rust pre-commit rule requires passing checks before every commit.
- [Phase 08-parity-evidence-and-ultra-205-release-gate]: No Phase 8 checklist row was promoted to verified without live evidence; FS-001, OTA-001, REL-001, REL-002, and REL-003 remain implemented.
- [Phase 08-parity-evidence-and-ultra-205-release-gate]: OTA-002 remains deferred with public response Wrong API input because whole-www hardware-regression and interrupted-update evidence were not recorded.
- [Phase 08-parity-evidence-and-ultra-205-release-gate]: Release artifacts are GPL-risk-reviewed release artifacts and publication waits for final release approval.
- [Phase 09-flash-monitor-evidence-wrapper-hardening]: flash-monitor --evidence-dir now uses espflash monitor --chip esp32s3 --port <port> --non-interactive while ordinary monitor remains interactive.
- [Phase 09-flash-monitor-evidence-wrapper-hardening]: Evidence is trusted only when all seven serial-scope Ultra 205 boot markers are present.
- [Phase 09-flash-monitor-evidence-wrapper-hardening]: Monitor timeouts are accepted only after trusted output; untrusted timeout or failed monitor exits write JSON and fail visibly.
- [Phase 09-flash-monitor-evidence-wrapper-hardening]: TDD RED failures were run but not committed because AGENTS.md requires passing Rust checks before every commit.

### Pending Todos

None yet.

### Blockers/Concerns

- Hardware evidence: Mining, ASIC init, voltage, fan, thermal, power, and safety-critical verification still need Ultra 205 hardware-smoke or regression evidence before `verified` parity claims.
- Release scope: Non-205 boards and ASICs must remain unverified or deferred until each has its own evidence set.

### Quick Tasks Completed

| ID | Date | Task | Result | Evidence |
| --- | --- | --- | --- | --- |
| 260627-b0q | 2026-06-27 | Display startup debug text on Ultra 205 OLED | implemented, serial-smoked, user-visible on OLED | `.planning/quick/260627-b0q-display-startup-debug-text-on-ultra-205-/260627-b0q-SUMMARY.md`, `docs/parity/evidence/ultra-205-startup-display-debug-2026-06-27.md` |
| 260628-kri | 2026-06-28 | Add ESP-IDF contributor dependency workflow | implemented, dependency doctor and bootstrap documented | `.planning/quick/260628-kri-add-esp-idf-contributor-dependency-workf/260628-kri-SUMMARY.md`, `scripts/esp-doctor-test.sh` |
| 260628-l4b | 2026-06-28 | Refactor factory image merging to ESP tooling | implemented, package path uses managed `esptool.py merge_bin` and validation remains in xtask | `.planning/quick/260628-l4b-refactor-factory-image-merging-to-esp-to/260628-l4b-SUMMARY.md`, `scripts/package-firmware-test.sh` |
| 260628-lgg | 2026-06-28 | Add autonomous Ultra 205 hardware verification rule | implemented, read-only detector found `port=/dev/cu.usbmodem1101` locally | `.planning/quick/260628-lgg-add-autonomous-ultra-205-hardware-verifi/260628-lgg-SUMMARY.md`, `scripts/detect-ultra205-test.sh` |

## Session Continuity

Last session: 2026-06-29T14:11:38.281Z
Stopped at: Completed 09-01-PLAN.md
Resume file: None
