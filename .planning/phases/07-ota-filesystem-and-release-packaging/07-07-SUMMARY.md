---
phase: 07-ota-filesystem-and-release-packaging
plan: 07
subsystem: firmware-ota-runtime
tags: [rust, esp-idf, ota, rollback, otawww, firmware-http, release-evidence]

requires:
  - phase: 07-01
    provides: Pure update planner and OTA route decision model
  - phase: 07-04
    provides: Firmware HTTP route shell and static filesystem recovery context
  - phase: 07-05
    provides: App OTA and SPIFFS package artifact context
provides:
  - Rollback-capable ESP-IDF boot validation adapter with retained startup logs
  - Guarded streamed firmware OTA upload/apply route for `/api/system/OTA`
  - Explicit fail-closed OTAWWW runtime gap route and REL-03 evidence
affects: [firmware-http-api, ota-runtime, rollback-evidence, otawww-gap, phase-07-release]

tech-stack:
  added: []
  patterns:
    - Thin ESP-IDF OTA and rollback adapters behind pure update-planner decisions
    - Update route handlers preserve access/AP-mode gates before any partition side effects
    - Runtime gap routes log explicit owner and reason instead of sharing generic unsupported handlers

key-files:
  created:
    - firmware/bitaxe/src/boot_validation.rs
    - firmware/bitaxe/src/ota_update.rs
  modified:
    - firmware/bitaxe/src/http_api.rs
    - firmware/bitaxe/src/main.rs
    - firmware/bitaxe/sdkconfig.defaults
    - docs/parity/evidence/phase-07-ota-filesystem-release.md

key-decisions:
  - "Validate pending OTA images only after startup diagnostics and keep rollback evidence below verified until hardware logs exist."
  - "Stream firmware uploads directly from `httpd_req_recv` into ESP-IDF OTA APIs instead of buffering images in RAM."
  - "Keep OTAWWW fail-closed as an explicit REL-03 gap because interruption/recovery hardware evidence is not part of this plan."

patterns-established:
  - "Firmware update routes should call `plan_update_request` before OTA or partition side effects."
  - "OTA runtime statuses are emitted as retained firmware logs until a later plan adds a public status store."

requirements-completed: [REL-02, REL-03, REL-08]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 7-2026-06-28T13-30-15
generated_at: 2026-06-28T17:48:08Z

duration: 18min
completed: 2026-06-28
---

# Phase 07 Plan 07: OTA Runtime Update Summary

**Firmware OTA now streams through ESP-IDF with access gates, activation, reboot scheduling, and rollback boot validation while OTAWWW stays an explicit REL-03 runtime gap.**

## Performance

- **Duration:** 18 min
- **Started:** 2026-06-28T17:30:25Z
- **Completed:** 2026-06-28T17:48:08Z
- **Tasks:** 3
- **Files modified:** 6

## Accomplishments

- Added `boot_validation.rs` around ESP-IDF OTA state APIs so pending images are marked valid after startup diagnostics or marked invalid with rollback/reboot after diagnostic failure.
- Enabled rollback support in `sdkconfig.defaults` with `CONFIG_BOOTLOADER_APP_ROLLBACK_ENABLE=y`.
- Added `ota_update.rs` so `/api/system/OTA` streams request chunks directly into `esp_ota_write`, validates with `esp_ota_end`, activates with `esp_ota_set_boot_partition`, and aborts on protocol/write failures.
- Replaced the firmware OTA fail-closed route with a guarded handler that preserves access and AP-mode rejection behavior, reports upstream-visible status text, returns `Firmware update complete, rebooting now!`, and schedules `esp_restart()` after response delivery.
- Replaced the OTAWWW generic unsupported handler with an explicit Phase 7 gap route that preserves gates, returns 400 `Wrong API input`, and logs `otawww_update=gap reason=interruption_evidence_missing owner=phase-07-release`.
- Updated Phase 7 evidence to keep rollback and OTAWWW interruption/recovery below verified parity until hardware logs exist.

## Task Commits

Each task was committed atomically:

1. **Task 1: Add ESP-IDF rollback boot validation** - `96e2f02` (feat)
2. **Task 2: Replace firmware OTA fail-closed handler with guarded streaming adapter** - `6290179` (feat)
3. **Task 3: Wire OTAWWW explicit gap route and evidence** - `3c6eca4` (feat)

## Files Created/Modified

- `firmware/bitaxe/src/boot_validation.rs` - Adds rollback-state inspection, valid-image marking, invalid-image rollback/reboot, and retained validation logs.
- `firmware/bitaxe/src/ota_update.rs` - Adds the streamed ESP-IDF OTA writer, activation path, abort handling, and status/result types.
- `firmware/bitaxe/src/http_api.rs` - Wires firmware OTA and OTAWWW route handlers through the Phase 7 update planner.
- `firmware/bitaxe/src/main.rs` - Calls boot validation after startup diagnostics and before serving normal runtime work.
- `firmware/bitaxe/sdkconfig.defaults` - Enables rollback-capable bootloader behavior.
- `docs/parity/evidence/phase-07-ota-filesystem-release.md` - Records host/compile evidence and hardware-pending parity status.

## Decisions Made

- Firmware OTA activation is allowed only after the existing private-network/origin and AP-mode update planner gates pass.
- Firmware images stream from `httpd_req_recv` into ESP-IDF OTA APIs using bounded chunks; the full upload is never buffered in RAM.
- Rollback validation happens after startup diagnostics so a failed diagnostic path can mark the pending app invalid before reboot.
- OTAWWW remains a deliberate runtime gap for V1 because whole-partition static update interruption/recovery evidence is not scheduled in this plan set.

## Verification

- `node /Users/peterryszkiewicz/.codex/get-shit-done/bin/gsd-tools.cjs verify lifecycle 07 --require-plans` passed before execution.
- Task 1 acceptance grep found rollback APIs, retained rollback logs, and `CONFIG_BOOTLOADER_APP_ROLLBACK_ENABLE`.
- Task 2 acceptance grep found ESP-IDF OTA APIs, abort handling, success response, validation error response, and reboot status.
- Task 3 acceptance grep found OTAWWW gap logging, explicit owner/reason, `Wrong API input`, `AxeOsStaticOtaWww`, and AP-mode rejection text.
- `cargo fmt --all` passed before implementation commits.
- `cargo clippy --all-targets --all-features -- -D warnings` passed before implementation commits.
- `cargo build --all-targets --all-features` passed before implementation commits.
- `cargo test --all-features` passed before implementation commits.
- `cargo test -p bitaxe-api --all-features update_plan` passed.
- `cargo build -p bitaxe-firmware --target xtensa-esp32s3-espidf` passed.
- `bazel build //firmware/bitaxe:firmware` passed and produced `bazel-bin/firmware/bitaxe/bitaxe-firmware.elf`.
- `just build` passed.
- `git diff --check` passed.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Fixed OTA result-shape compile failure**
- **Found during:** Task 2
- **Issue:** The first OTA adapter pass mixed `Result<FirmwareOtaApplyResult, FirmwareOtaApplyResult>` control flow with a direct `FirmwareOtaApplyResult`, causing the firmware build to fail.
- **Fix:** Matched the stream result explicitly and returned either `finish_ota(...)` or the protocol/write failure result.
- **Files modified:** `firmware/bitaxe/src/ota_update.rs`
- **Verification:** `cargo build -p bitaxe-firmware --target xtensa-esp32s3-espidf` passed.
- **Committed in:** `6290179`

### Process Adjustments

- The plan marked tasks as TDD. RED failure signals were captured before implementation, but separate RED commits were not created because repo-local pre-commit rules require the full Rust verification suite before every commit.

**Total deviations:** 1 auto-fixed, 1 process adjustment.
**Impact on plan:** The fix stayed inside the planned OTA adapter and did not change the public route contract.

## Known Stubs

None. Stub scan found only Rust formatting placeholders in retained log strings, not empty/mock data or TODO-style placeholders.

## Threat Flags

None. The new browser-upload-to-OTA, bootloader-to-app, and OTAWWW trust boundaries were the planned threat model scope; no unplanned network, auth, file-write, schema, or hardware-control surface was added.

## Issues Encountered

- The removed generic unsupported-update helper became dead code after firmware OTA and OTAWWW received route-specific handlers, so the unused helper was deleted during Task 3 cleanup.
- No hardware rollback, reboot, or interrupted OTAWWW evidence was produced; those surfaces remain explicitly below verified hardware parity.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Release packaging can now expose an app OTA artifact to a firmware runtime route with guarded streaming activation. OTAWWW whole-`www` partition update work remains owned by `phase-07-release` until interruption/recovery evidence is scheduled and captured on hardware.

## Self-Check

PASSED.

- Confirmed summary and newly created source files exist.
- Confirmed commits `96e2f02`, `6290179`, and `3c6eca4` exist.

***
*Phase: 07-ota-filesystem-and-release-packaging*
*Completed: 2026-06-28*
