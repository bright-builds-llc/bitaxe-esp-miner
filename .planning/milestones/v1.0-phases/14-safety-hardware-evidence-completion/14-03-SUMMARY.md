---
phase: 14-safety-hardware-evidence-completion
plan: "03"
subsystem: safety-hardware-evidence
tags: [evidence, wrappers, hardware-smoke, safety, power, thermal]
requires:
  - phase: 14-01
    provides: typed safety-allow manifest gate and CLI command shape
  - phase: 14-02
    provides: component-pack evidence contract and redaction template
provides:
  - Phase 14 power and voltage evidence wrapper
  - Phase 14 thermal and fan evidence wrapper
  - Pending power, voltage, thermal, and fan evidence records
affects: [phase-14-evidence-wrappers, parity-checklist-promotion, safety-hardware-evidence]
tech-stack:
  added: []
  patterns:
    - allow-manifest gated evidence wrappers
    - explicit pending status for missing active-control evidence
key-files:
  created:
    - scripts/phase14-power-voltage.sh
    - scripts/phase14-power-voltage-test.sh
    - scripts/phase14-thermal-fan.sh
    - scripts/phase14-thermal-fan-test.sh
    - docs/parity/evidence/phase-14-safety-hardware-evidence-completion/power-telemetry.md
    - docs/parity/evidence/phase-14-safety-hardware-evidence-completion/voltage-control.md
    - docs/parity/evidence/phase-14-safety-hardware-evidence-completion/thermal-fan.md
    - docs/parity/evidence/phase-14-safety-hardware-evidence-completion/power-telemetry/allow-power-telemetry.json
    - docs/parity/evidence/phase-14-safety-hardware-evidence-completion/power-telemetry/power-voltage.log
    - docs/parity/evidence/phase-14-safety-hardware-evidence-completion/thermal-fan/allow-thermal-fan.json
    - docs/parity/evidence/phase-14-safety-hardware-evidence-completion/thermal-fan/thermal-fan.log
  modified:
    - scripts/BUILD.bazel
key-decisions:
  - "Kept power/current telemetry separate from voltage actuation claims."
  - "Kept thermal/fan observations separate from active fan duty claims."
  - "Recorded active voltage and fan duty as pending because no production-safe bounded hardware-regression route exists in this plan."
patterns-established:
  - "Evidence wrappers run the `safety-allow` manifest gate before recording trusted output."
  - "Missing serial observations produce explicit pending status lines instead of broad success claims."
requirements-completed: [SAFE-01, SAFE-02, SAFE-03, SAFE-04, SAFE-07, SAFE-08, EVD-05]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 14-2026-06-30T23-56-34
generated_at: 2026-07-01T01:31:12Z
duration: 13 min
completed: 2026-07-01
---

# Phase 14 Plan 03: Power Voltage And Thermal Fan Evidence Summary

**Allow-gated wrappers now record narrow power, voltage, thermal, and fan conclusions without promoting active-control parity.**

## Performance

- **Duration:** 13 min
- **Started:** 2026-07-01T01:18:33Z
- **Completed:** 2026-07-01T01:31:12Z
- **Tasks:** 2
- **Files modified:** 12

## Accomplishments

- Added `scripts/phase14-power-voltage.sh` and `scripts/phase14-thermal-fan.sh` with `set -euo pipefail`, allow-manifest validation, pending evidence behavior, and no destructive voltage, fan, flash, rollback, interrupted-update, or raw I2C command paths.
- Added script tests and Bazel targets proving missing manifests, failed validators, missing serial evidence, power telemetry pending status, voltage control pending status, thermal pending status, and fan duty pending status.
- Ran `just detect-ultra205` successfully for board `205` on `/dev/cu.usbmodem1101`, then used `just package` output to bind allow manifests to the package identity and current source/reference commits.
- Generated component evidence for `power-telemetry`, `voltage-control`, and `thermal-fan`, with `PWR-006`, `PWR-003`, `PWR-005`, `THR-001`, `THR-002`, and `THR-003` recorded as pending or read-only-observation only.

## Task Commits

Each task was committed atomically:

1. **Task 1: Add power/voltage and thermal/fan wrapper scripts with tests** - `ff9da3b` (`feat`)
2. **Task 2: Run power/voltage and thermal/fan wrappers or record pending evidence** - `865c959` (`docs`)

## Files Created/Modified

- `scripts/phase14-power-voltage.sh` - Gated power telemetry and voltage-control wrapper.
- `scripts/phase14-power-voltage-test.sh` - Shell tests for power/voltage wrapper pending and allow-gate behavior.
- `scripts/phase14-thermal-fan.sh` - Gated thermal and fan wrapper.
- `scripts/phase14-thermal-fan-test.sh` - Shell tests for thermal/fan wrapper pending and allow-gate behavior.
- `scripts/BUILD.bazel` - Registered the wrapper and wrapper-test targets.
- `docs/parity/evidence/phase-14-safety-hardware-evidence-completion/power-telemetry.md` - Records `PWR-006` as pending read-only telemetry evidence.
- `docs/parity/evidence/phase-14-safety-hardware-evidence-completion/voltage-control.md` - Records `PWR-003` and `PWR-005` below verified with no bounded voltage route.
- `docs/parity/evidence/phase-14-safety-hardware-evidence-completion/thermal-fan.md` - Records thermal and fan subclaims with active fan duty still pending.
- `docs/parity/evidence/phase-14-safety-hardware-evidence-completion/power-telemetry/power-voltage.log` - Raw wrapper output for the power/voltage run.
- `docs/parity/evidence/phase-14-safety-hardware-evidence-completion/thermal-fan/thermal-fan.log` - Raw wrapper output for the thermal/fan run.

## Decisions Made

- Did not promote any checklist row in this plan.
- Treated the unavailable serial telemetry route as a blocker for fresh read-only sensor evidence while still recording the exact allow-gated command, detector result, package identity, and pending conclusion.
- Left voltage-control and fan-duty effects below verified because Phase 14 does not yet contain a documented recovery path and bounded hardware-regression procedure for active actuator work.

## Verification

- `bash -n scripts/phase14-power-voltage.sh && bash -n scripts/phase14-power-voltage-test.sh && bash -n scripts/phase14-thermal-fan.sh && bash -n scripts/phase14-thermal-fan-test.sh` - passed.
- `bazel test //scripts:phase14_power_voltage_test //scripts:phase14_thermal_fan_test` - passed.
- Wrapper acceptance scans for required status strings and prohibited command paths - passed.
- `just detect-ultra205` - passed with one likely ESP32-S3 port, `/dev/cu.usbmodem1101`.
- `just package` - passed and generated `bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json`.
- Focused evidence file checks and scans - passed.
- `cargo test -p bitaxe-safety --all-features power` - passed.
- `cargo test -p bitaxe-safety --all-features thermal` - passed.
- `just parity` - passed with no invalid verified rows.
- `cargo fmt --all` - passed before each task commit.
- `cargo clippy --all-targets --all-features -- -D warnings` - passed before each task commit.
- `cargo build --all-targets --all-features` - passed before each task commit.
- `cargo test --all-features` - passed before each task commit.
- `git diff --check` - passed for touched files.

## Deviations from Plan

- No checklist rows were promoted, as required.
- Serial runtime observations were unavailable, so the evidence docs record pending conclusions rather than fresh sensor values.

## Issues Encountered

- The generated package identity changed after the wrapper-script commit. The allow manifests were created after the final package build so the package, source commit, and wrapper run stay aligned.

## User Setup Required

None.

## Next Phase Readiness

Plan 14-04 can use the same allow-gated evidence-pack pattern for self-test, watchdog, load, display, and input surfaces.

## Self-Check: PASSED

- Found created files and generated raw logs listed above.
- Found task commits: `ff9da3b` and `865c959`.
- Confirmed this summary uses only frontmatter opening and closing standalone delimiters.

*Phase: 14-safety-hardware-evidence-completion*
*Completed: 2026-07-01*
