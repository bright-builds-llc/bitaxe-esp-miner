---
phase: 31-operator-claim-and-telemetry-contract
plan: "02"
subsystem: api-firmware
tags: [rust, telemetry, observation, websocket, esp-idf]
requires:
  - phase: 31-01-observation-truth-core
    provides: Typed four-state observations with producer-owned stamps
provides:
  - Six independent system-info truth fields beside compatibility numerics
  - Pure read-only observation store with immutable producer metadata
  - Firmware consumer boundary that never performs request-time acquisition
affects: [32-read-only-i2c-producer, 34-operator-snapshot, axeos-api]
tech-stack:
  added: []
  patterns: [truth-only wire DTO, complete snapshot replacement, producer-side publication]
key-files:
  created:
    - crates/bitaxe-api/src/observation.rs
    - firmware/bitaxe/src/safety_adapter/observation_store.rs
  modified:
    - crates/bitaxe-api/src/snapshot.rs
    - crates/bitaxe-api/src/wire.rs
    - crates/bitaxe-api/src/telemetry.rs
    - firmware/bitaxe/src/runtime_snapshot.rs
    - firmware/bitaxe/src/safety_adapter/phase27_bring_up.rs
key-decisions:
  - "Observation truth serializes independently from AxeOS numeric compatibility values."
  - "Firmware consumers clone one complete stored snapshot; only producer completion may replace it."
  - "Phase 27 leaves fan RPM unavailable because it has no independently owned stamp in the retained legacy path."
patterns-established:
  - "Consumer immutability: HTTP, WebSocket, statistics, and serialization reads cannot advance observation metadata."
  - "Fact projection: extracting one value from an observation copies its original state and stamp exactly."
requirements-completed: [OBS-01]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 31-2026-07-13T19-47-51
generated_at: 2026-07-13T21:07:57Z
duration: 15 min
completed: 2026-07-13
---

# Phase 31 Plan 02: API and Firmware Consumer Boundary Summary

**AxeOS system-info and WebSocket views now expose six independently stamped truth objects while firmware request paths only clone stored observations.**

## Performance

- **Duration:** 15 min
- **Started:** 2026-07-13T20:52:03Z
- **Completed:** 2026-07-13T21:07:57Z
- **Tasks:** 2
- **Files modified:** 15

## Accomplishments

- Added stable `fresh`, `stale`, `unavailable`, and `fault` wire truth with typed reasons and untouched producer stamps.
- Preserved existing numeric compatibility fields and zero fallbacks while adding exactly six independent `*Status` fields.
- Replaced request-time safety collection with a `OnceLock<Mutex<ObservationStore>>` consumer boundary initialized to explicit unavailable truth.
- Moved retained Phase 27 publication to its producer-completion path and kept unstamped fan RPM unavailable.

## Task Commits

Each task was committed atomically:

1. **Task 1: Add independent API truth projections without changing compatibility numerics** — `abc8e04` (feat)
2. **Task 2: Make firmware API collection consume stored observations only** — `2de6def` (feat)

**Plan metadata:** this commit

## Files Created/Modified

- `crates/bitaxe-api/src/observation.rs` — Truth wire DTOs, pure store, metadata-preserving fact projection, and consumer immutability tests.
- `crates/bitaxe-api/src/snapshot.rs` — Independent truth fields and fresh-only numeric compatibility projection.
- `crates/bitaxe-api/src/wire.rs` — Six stable AxeOS system-info status fields.
- `crates/bitaxe-api/src/telemetry.rs` — Nested WebSocket truth-diff regression.
- `crates/bitaxe-api/fixtures/api/system-info-ultra205-safe.json` — Explicit unavailable truth beside unchanged safe numerics.
- `firmware/bitaxe/src/safety_adapter/observation_store.rs` — Read-only firmware store and producer-only replacement boundary.
- `firmware/bitaxe/src/runtime_snapshot.rs` — Stored observation consumption instead of safety collection.
- `firmware/bitaxe/src/safety_adapter/phase27_bring_up.rs` — One-time producer publication with preserved stamps.
- `firmware/bitaxe/src/safety_adapter/power.rs` and `thermal.rs` — Removed obsolete query-time report collectors.
- `firmware/bitaxe/src/asic_adapter.rs` and `asic_adapter/chip_detect_investigation.rs` — Updated Plan 01 typed-observation callsites required by the canonical firmware build.

## Decisions Made

- Numeric zero remains a compatibility value only; a fresh zero and an unavailable zero are distinguished exclusively by their truth object.
- A missing projected sub-fact becomes explicitly unavailable instead of manufacturing a value or stamp.
- The retained Phase 27 path publishes only facts with existing producer stamps. Fan RPM remains unavailable until Phase 32 owns its acquisition metadata.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Resolved stale system-info fixture path**

- **Found during:** Task 1
- **Issue:** The read-first path `crates/bitaxe-api/fixtures/system-info-safe-ultra205.json` does not exist.
- **Fix:** Used the canonical existing fixture at `crates/bitaxe-api/fixtures/api/system-info-ultra205-safe.json`.
- **Files modified:** `crates/bitaxe-api/fixtures/api/system-info-ultra205-safe.json`
- **Verification:** Cargo and Bazel API fixture round-trip tests passed.
- **Committed in:** `abc8e04`

**2. [Rule 3 - Blocking] Migrated remaining firmware callers to Plan 01 observation accessors**

- **Found during:** Task 2 canonical firmware build
- **Issue:** Firmware-only callsites still used removed public fields and pre-Plan-01 thermal names, so the ESP32-S3 build could not compile.
- **Fix:** Updated the narrow Phase 27 and chip-detect callsites to typed accessors and corrected reset mutability exposed by the build.
- **Files modified:** `firmware/bitaxe/src/asic_adapter.rs`, `firmware/bitaxe/src/asic_adapter/chip_detect_investigation.rs`, `firmware/bitaxe/src/safety_adapter/phase27_bring_up.rs`, `firmware/bitaxe/src/safety_adapter/thermal.rs`
- **Verification:** `just build` completed successfully for `xtensa-esp32s3-espidf`.
- **Committed in:** `2de6def`

**Total deviations:** 2 auto-fixed (2 blocking). **Impact:** Both were narrow prerequisites for the planned API/firmware migration; no hardware access, active effect, credential, reference, evidence, or archived-lineage scope was added.

## Issues Encountered

- The first source-field assertion also matched the pre-existing `wifiStatus` key; the test was narrowed to the exact six new telemetry fields.
- The first firmware build exposed four typed-accessor callsites, and a follow-up mutability correction was needed for two similar reset constructors. The final firmware build passed.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Ready for Plan 31-03 to add hostname-only authority and exact claim admission.
- Phase 32 can replace the complete observation snapshot from its sole read-only I2C producer without changing consumer behavior.
- Existing firmware build warnings are pre-existing dormant-code warnings; no new acquisition, hardware effect, or request-time freshness path was introduced.

## Self-Check: PASSED

- Created files exist: `crates/bitaxe-api/src/observation.rs` and `firmware/bitaxe/src/safety_adapter/observation_store.rs`.
- Task commits exist: `abc8e04`, `2de6def`.
- Focused/full Cargo, API Bazel, source-boundary, diff, and canonical ESP32-S3 firmware build verification passed.

***

*Phase: 31-operator-claim-and-telemetry-contract*
*Completed: 2026-07-13*
