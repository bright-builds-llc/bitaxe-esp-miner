---
phase: 33-confirmed-settings-durability
plan: 03
subsystem: firmware-evidence
tags: [esp32s3, nvs, rtc-noinit, hostname, reboot, hardware-evidence]
requires:
  - phase: 33-confirmed-settings-durability
    provides: Confirmed hostname persistence and immediate storage-truth publication from Plans 01 and 02
provides:
  - Detector-gated same-board hostname durability proof across one application restart
  - RTC-backed boot ordinal and typed boot/origin replay across native USB enumeration gaps
  - Redacted non-promotional evidence with complete cleanup and restoration
affects: [phase-34-operator-snapshot, phase-35-correlated-evidence]
tech-stack:
  added: []
  patterns: [rtc-noinit boot ordinal, typed replay classifier, protected raw and redacted shareable evidence]
key-files:
  created:
    - scripts/phase33-confirmed-settings-durability.sh
    - scripts/phase33-confirmed-settings-durability-test.sh
    - crates/bitaxe-api/src/boot_identity.rs
    - crates/bitaxe-api/src/phase33_evidence.rs
    - firmware/bitaxe/src/rtc_boot_ordinal.rs
    - docs/evidence/phase-33/hardware-summary.md
  modified:
    - firmware/bitaxe/src/boot_evidence.rs
    - firmware/bitaxe/src/http_api.rs
    - firmware/bitaxe/src/main.rs
    - firmware/bitaxe/src/wifi_adapter.rs
    - tools/parity/src/phase33_source_guard.rs
    - .planning/phases/33-confirmed-settings-durability/33-VALIDATION.md
key-decisions:
  - "Boot durability evidence uses an RTC no-init ordinal and boot-lifetime typed replay so native USB enumeration loss cannot erase the proof surface."
  - "The hardware proof consumes exactly one detector preflight and one application restart, then fails closed on any classifier, cleanup, or restoration error."
  - "Phase 33 evidence remains a narrow durability proof; Phase 35 still owns admission and parity promotion."
patterns-established:
  - "Reset-spanning proof: baseline session/ordinal A/N must become a distinct B/N+1 software-reset identity with one bound fresh origin."
  - "Evidence split: protected raw traces stay mode 0700/0600 under ignored storage while only denylist-clean categories, digests, counts, durations, and booleans are tracked."
requirements-completed: [CFG-09, CFG-10, CFG-11, CFG-12, CFG-13]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 33-2026-07-14T01-50-49
generated_at: 2026-07-15T00:49:40Z
duration: 26min
completed: 2026-07-15
---

# Phase 33 Plan 03: Confirmed Settings Durability Hardware Proof Summary

**The exact `a630455` package proved confirmed hostname durability across one normal Ultra 205 application restart with typed reset replay, same-board identity, complete cleanup, and restoration.**

## Performance

- **Duration:** 26 min
- **Started:** 2026-07-15T00:23:28Z
- **Completed:** 2026-07-15T00:49:40Z
- **Tasks:** 2
- **Files modified:** 29

## Accomplishments

- Added a fail-closed Phase 33 wrapper and simulation/source-guard suite that enforce one detector, exact-package flash, the complete passive ESP32-S3 monitor contract, one restart, redaction, cleanup, and recovery.
- Replaced lossy one-shot reboot evidence with an RTC-backed boot ordinal and typed boot/origin replay that survives the native USB enumeration gap.
- Passed the sole eligible fresh hardware attempt with A/N to B/N+1, software CPU reset, one fresh bound origin, stable physical identity, matching immediate/post-reboot hostname digests, cleanup, and restoration.
- Tracked only the redacted non-promotional summary; protected traces remain ignored and mode restricted.

## Task Commits

1. **Task 1: Build a fail-closed Phase 33 durability wrapper and simulation suite** - `cbd8e20`, qualified through fixes `946003d`, `eae411e`, `638a72e`, `3c61ec6`, and `a630455`.
2. **Task 2: Run software gates and the approved Ultra 205 durability proof** - `323e5e4`.

## Files Created/Modified

- `scripts/phase33-confirmed-settings-durability.sh` - Owns the detector, exact-package setup, confirmed PATCH, passive restart proof, cleanup, restoration, and redacted output lifecycle.
- `scripts/phase33-confirmed-settings-durability-test.sh` - Exercises success and every fail-closed evidence boundary without hardware.
- `crates/bitaxe-api/src/boot_identity.rs` - Models typed boot identity and RTC ordinal transitions.
- `crates/bitaxe-api/src/phase33_evidence.rs` - Classifies baseline, delivery, and post-restart evidence.
- `firmware/bitaxe/src/rtc_boot_ordinal.rs` - Retains validated boot ordinals in writable `.rtc_noinit` storage.
- `firmware/bitaxe/src/boot_evidence.rs` and `firmware/bitaxe/src/wifi_adapter.rs` - Replay boot identity and session-bound origin for the bounded evidence window.
- `docs/evidence/phase-33/hardware-summary.md` - Records the denylist-clean durability result without Phase 35 admission.
- `.planning/phases/33-confirmed-settings-durability/33-VALIDATION.md` - Marks the hardware gate and CFG-12 green from the fresh exact-package proof.

## Decisions Made

- Kept boot proof owned by a boot-lifetime producer and origin proof session-bound, rather than inferring missing early bytes from later service recovery.
- Required the reset-surviving ordinal to advance exactly once and the replay classifier to observe a distinct session with software CPU reset provenance.
- Preserved the original hostname through a confirmed restoration PATCH without a second restart.
- Kept Phase 35 admission and all parity promotion explicitly outside this plan.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Corrected macOS stable physical-identity derivation and producer draining**

- **Found during:** Task 1 hardware qualification preparation.
- **Issue:** macOS USB ancestry and producer behavior could make a valid same-board session appear ambiguous or incomplete.
- **Fix:** Corrected stable identity ancestry and drained the identity producer deterministically.
- **Files modified:** `scripts/serial-session-trace.sh` and its tests.
- **Verification:** Direct/Bazel serial-session and detector tests passed.
- **Committed in:** `946003d`, `eae411e`.

**2. [Rule 1 - Bug] Preserved response-before-restart under constrained firmware stack**

- **Found during:** Task 1 hardware qualification preparation.
- **Issue:** System-info stack pressure and restart response ordering could prevent the required response-before-effect proof.
- **Fix:** Reduced request stack usage and completed the HTTP restart response before scheduling the application reset.
- **Files modified:** firmware HTTP/runtime surfaces and source guards.
- **Verification:** Mandatory Rust gates, targeted Bazel guards, canonical firmware build, and the hardware response-before-effect gate passed.
- **Committed in:** `638a72e`, `3c61ec6`.

**3. [Rule 1 - Bug] Made reboot evidence survive the native USB enumeration gap**

- **Found during:** Task 1 hardware qualification.
- **Issue:** A prior fail-closed run recovered service but lost one-shot boot/origin lines while native USB was unavailable.
- **Fix:** Added validated RTC boot ordinals, boot-lifetime identity replay, bounded session-origin replay, and a typed A/N to B/N+1 classifier.
- **Files modified:** API evidence core, firmware replay surfaces, wrapper, simulations, Bazel wiring, and source guards.
- **Verification:** Ordered Rust gates, shell/Bazel simulations, canonical build/package/reference checks, writable `.rtc_noinit` ELF proof, and the sole fresh hardware attempt passed.
- **Committed in:** `a630455`.

**Total deviations:** 3 auto-fixed bugs.
**Impact on plan:** Each correction tightened the planned proof boundary; no hardware, credential, archived-lineage, or promotion scope was added.

## Issues Encountered

- The pre-existing package manifest referenced the pre-fix source. The canonical firmware and package were rebuilt at exact source `a630455`, all six artifact digests and the writable `.rtc_noinit` section were verified, and only then was the one-shot hardware wrapper started.
- The earlier native USB one-shot-marker failure remained recorded as non-promotional history; the fresh eligible attempt passed without retry.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Phase 33 is complete and CFG-12 now has bounded same-board hardware evidence.
- Phase 34 can compose the confirmed hostname with provenance, health, and coherent operator-snapshot revisions.
- Phase 35 remains the only owner of final correlated admission and parity promotion.

## Self-Check: PASSED

- The redacted hardware summary exists and passes the sensitive-output denylist.
- Protected recent directories/files have modes 0700/0600, and process/holder cleanup plus hostname restoration are recorded as complete.
- Exact-package manifest/reference/artifact checks, Phase 33 simulation/Bazel/parity tests, shell static checks, canonical build/package/reference gates, and `git diff --check` passed.

***

*Phase: 33-confirmed-settings-durability*
*Completed: 2026-07-15*
