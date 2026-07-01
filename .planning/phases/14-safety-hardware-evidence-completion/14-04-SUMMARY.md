---
phase: 14-safety-hardware-evidence-completion
plan: "04"
subsystem: safety-hardware-evidence
tags: [evidence, wrappers, hardware-smoke, safety, watchdog, display]
requires:
  - phase: 14-03
    provides: Phase 14 wrapper and evidence-pack pattern
provides:
  - Phase 14 self-test/watchdog/load evidence wrapper
  - Phase 14 display/input evidence wrapper
  - Current-commit serial evidence for watchdog and startup display markers
  - Pending self-test hardware, load stress, runtime display, and input conclusions
affects: [phase-14-evidence-wrappers, parity-checklist-promotion, safety-hardware-evidence]
tech-stack:
  added: []
  patterns:
    - allow-manifest gated startup marker parsing
    - explicit pending status for runtime or active surfaces without hardware-regression evidence
key-files:
  created:
    - scripts/phase14-self-test-watchdog-load.sh
    - scripts/phase14-self-test-watchdog-load-test.sh
    - scripts/phase14-display-input.sh
    - scripts/phase14-display-input-test.sh
    - docs/parity/evidence/phase-14-safety-hardware-evidence-completion/self-test-watchdog-load.md
    - docs/parity/evidence/phase-14-safety-hardware-evidence-completion/display-input.md
    - docs/parity/evidence/phase-14-safety-hardware-evidence-completion/self-test-watchdog-load/allow-self-test-watchdog-load.json
    - docs/parity/evidence/phase-14-safety-hardware-evidence-completion/self-test-watchdog-load/self-test-watchdog-load.log
    - docs/parity/evidence/phase-14-safety-hardware-evidence-completion/self-test-watchdog-load/current-serial/flash-monitor.log
    - docs/parity/evidence/phase-14-safety-hardware-evidence-completion/self-test-watchdog-load/current-serial/flash-command-evidence.json
    - docs/parity/evidence/phase-14-safety-hardware-evidence-completion/display-input/allow-display-input.json
    - docs/parity/evidence/phase-14-safety-hardware-evidence-completion/display-input/display-input.log
  modified:
    - scripts/BUILD.bazel
key-decisions:
  - "Captured a fresh current-commit flash-monitor serial log instead of relying on older retained marker evidence."
  - "Recorded watchdog supervisor startup/yield as a narrow observed subclaim only."
  - "Kept self-test hardware, load stress, runtime display, and input hardware behavior below verified."
patterns-established:
  - "Startup-only SSD1306 evidence is explicitly separated from runtime display/input parity."
  - "Self-test and load evidence wrappers never start self-test hardware, reboot, mine, or run stress work."
requirements-completed: [SAFE-05, SAFE-06, SAFE-08, SAFE-09, EVD-05]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 14-2026-06-30T23-56-34
generated_at: 2026-07-01T01:41:45Z
duration: 10 min
completed: 2026-07-01
---

# Phase 14 Plan 04: Self-Test Watchdog Load And Display Input Evidence Summary

**Fresh current-commit startup markers are captured, while active self-test, load, runtime display, and input parity remain pending.**

## Performance

- **Duration:** 10 min
- **Started:** 2026-07-01T01:31:12Z
- **Completed:** 2026-07-01T01:41:45Z
- **Tasks:** 2
- **Files modified:** 13

## Accomplishments

- Added `scripts/phase14-self-test-watchdog-load.sh` with allow-manifest validation and marker parsing for `safety_supervisor=started thread=bitaxe-safety-supervisor cadence_ms=100` and `safety_supervisor_step=yield reason=yield_interval_reached`.
- Added `scripts/phase14-display-input.sh` with allow-manifest validation and marker parsing for `display_status=startup_text_rendered model=SSD1306 size=128x32 address=0x3c` plus the runtime-gap marker.
- Added shell tests and Bazel targets for missing manifests, failed validators, marker-positive cases, and marker-missing cases.
- Ran `just flash-monitor board=205 port=/dev/cu.usbmodem1101 evidence-dir=docs/parity/evidence/phase-14-safety-hardware-evidence-completion/self-test-watchdog-load/current-serial capture-timeout-seconds=25` to capture trusted current-commit serial evidence.
- Generated evidence docs for `SELF-001`, `IO-001`, `UI-001`, `UI-002`, and `UI-003` with active/runtime claims still pending.

## Task Commits

Each task was committed atomically:

1. **Task 1: Add self-test/watchdog/load and display/input wrapper scripts with tests** - `8d39c6b` (`feat`)
2. **Task 2: Run self-test/watchdog/load and display/input wrappers or record pending evidence** - `0b90e87` (`docs`)

## Files Created/Modified

- `scripts/phase14-self-test-watchdog-load.sh` - Gated marker parser for self-test, watchdog, and load evidence.
- `scripts/phase14-self-test-watchdog-load-test.sh` - Shell tests for watchdog marker and pending self-test/load behavior.
- `scripts/phase14-display-input.sh` - Gated marker parser for startup display and runtime display/input gap evidence.
- `scripts/phase14-display-input-test.sh` - Shell tests for startup display marker and runtime pending behavior.
- `scripts/BUILD.bazel` - Registered the wrapper and wrapper-test targets.
- `docs/parity/evidence/phase-14-safety-hardware-evidence-completion/self-test-watchdog-load.md` - Records watchdog supervisor markers and pending self-test/load claims.
- `docs/parity/evidence/phase-14-safety-hardware-evidence-completion/display-input.md` - Records startup-only SSD1306 marker and pending runtime display/input claims.
- `docs/parity/evidence/phase-14-safety-hardware-evidence-completion/self-test-watchdog-load/current-serial/flash-monitor.log` - Fresh current-commit serial evidence.
- `docs/parity/evidence/phase-14-safety-hardware-evidence-completion/self-test-watchdog-load/current-serial/flash-command-evidence.json` - Flash-monitor command and trusted-output classification.

## Decisions Made

- Did not promote any checklist row in this plan.
- Treated watchdog supervisor markers as narrow hardware-smoke evidence, not proof of bounded workload responsiveness.
- Treated startup-only SSD1306 output as startup display evidence only, not runtime display/input parity.
- Kept self-test hardware submodes below verified because no safe route proved exact submode, voltage/fan/ASIC interactions, cancel/pass/fail behavior, recovery, and production-mining gates.

## Verification

- `bash -n scripts/phase14-self-test-watchdog-load.sh && bash -n scripts/phase14-self-test-watchdog-load-test.sh && bash -n scripts/phase14-display-input.sh && bash -n scripts/phase14-display-input-test.sh` - passed.
- `bazel test //scripts:phase14_self_test_watchdog_load_test //scripts:phase14_display_input_test` - passed.
- Wrapper acceptance scans for required status strings and marker-positive/marker-missing cases - passed.
- `just detect-ultra205` - passed with one likely ESP32-S3 port, `/dev/cu.usbmodem1101`.
- `just package` - passed and generated `bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json`.
- `just flash-monitor board=205 port=/dev/cu.usbmodem1101 evidence-dir=docs/parity/evidence/phase-14-safety-hardware-evidence-completion/self-test-watchdog-load/current-serial capture-timeout-seconds=25` - passed with trusted current-commit output.
- Focused evidence file checks and scans - passed.
- `cargo test -p bitaxe-safety --all-features self_test` - passed.
- `cargo test -p bitaxe-safety --all-features watchdog` - passed.
- `just parity` - passed with no invalid verified rows.
- `cargo fmt --all` - passed before each task commit.
- `cargo clippy --all-targets --all-features -- -D warnings` - passed before each task commit.
- `cargo build --all-targets --all-features` - passed before each task commit.
- `cargo test --all-features` - passed before each task commit.
- `git diff --check` - passed for touched files.

## Deviations from Plan

- Captured fresh current-commit serial evidence before running wrappers to avoid citing older retained startup markers.

## Issues Encountered

- The new shell tests initially lacked executable bits, which Bazel requires for these `sh_test` targets. The mode was corrected before the Task 1 commit.

## User Setup Required

None.

## Next Phase Readiness

Plan 14-05 can use the same evidence-pack structure to evaluate live API and WebSocket safety telemetry without promoting mining, ASIC, or runtime-control claims.

## Self-Check: PASSED

- Found created files and generated raw logs listed above.
- Found task commits: `8d39c6b` and `0b90e87`.
- Confirmed this summary uses only frontmatter opening and closing standalone delimiters.

*Phase: 14-safety-hardware-evidence-completion*
*Completed: 2026-07-01*
