---
phase: 01-foundation-and-gamma-601-boot-log
plan: "09"
subsystem: workflow-evidence
tags: [just, bazel, espflash, parity, evidence, gamma-601]

requires:
  - phase: 01-foundation-and-gamma-601-boot-log
    provides: Bazel-visible firmware, package, flash, parity, and reference guard targets
provides:
  - Thin Just command surface for Phase 1 build, test, package, flash, monitor, verify-reference, and parity workflows
  - Gamma 601 boot/log evidence record with explicit missing hardware-smoke conclusion
  - Updated parity checklist workflow evidence without false boot/log or safety-critical verification
affects: [developer-command-surface, hardware-smoke, parity-evidence, phase-1-verification]

tech-stack:
  added: [Justfile]
  patterns:
    - Just recipes remain thin wrappers over Bazel-visible targets
    - Hardware smoke absence is recorded as evidence state, not treated as pass/fail parity proof

key-files:
  created:
    - Justfile
    - docs/parity/evidence/phase-01-gamma-601-boot-log.md
    - .planning/phases/01-foundation-and-gamma-601-boot-log/01-09-SUMMARY.md
  modified:
    - docs/parity/checklist.md

key-decisions:
  - "Use `workflow` evidence for command-surface rows proved by Plan 09 Just/Bazel command output."
  - "Record missing Gamma 601 hardware-smoke evidence when `espflash list-ports` succeeds but reports no serial ports."
  - "Keep boot/log and safety-critical hardware rows below `verified` until captured Gamma 601 logs exist."

patterns-established:
  - "Checkpoint hardware verification in yolo mode still runs the real CLI precheck and records missing evidence instead of auto-claiming parity."
  - "Just forwards flash arguments to typed Rust tooling; it does not call `cargo`, `espflash`, or shell snippets directly."

requirements-completed: [FND-06, FND-07, FND-08, FND-09, FND-11]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 1-2026-06-21T00-30-20
generated_at: 2026-06-21T04:11:01Z

duration: 10min
completed: 2026-06-21
---

# Phase 01 Plan 09: Command Surface And Hardware Smoke Evidence Summary

**Bazel-backed Just workflows with explicit missing Gamma 601 hardware-smoke evidence**

## Performance

- **Duration:** 10 min
- **Started:** 2026-06-21T04:01:29Z
- **Completed:** 2026-06-21T04:11:01Z
- **Tasks:** 3
- **Files modified:** 4

## Accomplishments

- Added the required Phase 1 `Justfile` recipes: `build`, `test`, `package`, `flash`, `monitor`, `flash-monitor`, `verify-reference`, and `parity`.
- Verified package and parity paths route through `//scripts:verify_reference_clean` before trusted output.
- Recorded the Gamma 601 smoke checkpoint as missing hardware evidence because `espflash list-ports` succeeded and reported no known serial ports.
- Updated parity workflow rows with Plan 09 command evidence while keeping boot/log, ASIC, voltage, fan, thermal, power, and mining rows below `verified`.

## Task Commits

1. **Task 1: Create the required Just command surface** - `7657b54` (feat)
2. **Task 2: Prepare hardware-smoke evidence record** - `5c625c4` (docs)
3. **Task 3: Verify Gamma 601 flash-monitor smoke** - `239d395` (docs)

## Files Created/Modified

- `Justfile` - Thin public command surface over Bazel targets.
- `docs/parity/evidence/phase-01-gamma-601-boot-log.md` - Hardware smoke record with command template, required log patterns, manifest default ELF, and missing-evidence conclusion.
- `docs/parity/checklist.md` - Workflow evidence rows updated; boot/log smoke rows remain pending.

## Decisions Made

- Added `workflow` as a checklist evidence type because Phase 1 command-surface verification is command-output evidence, not unit/golden/API/hardware evidence.
- Treated yolo checkpoint automation as a real hardware precheck, not a verification bypass: no visible port means missing evidence, no flash-monitor attempt.
- Left `WF-005` at `implemented` rather than `verified` because the flash workflow is wired but live Gamma 601 flash-monitor evidence is absent.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Added workflow evidence taxonomy**
- **Found during:** Task 3 (Verify Gamma 601 flash-monitor smoke)
- **Issue:** The checklist needed to record passed command-surface evidence without misusing unit/golden/hardware evidence types.
- **Fix:** Added `workflow` as an evidence type and used it only for command/build/package/report rows.
- **Files modified:** `docs/parity/checklist.md`
- **Verification:** `just parity` passed with `validation_errors: none`; safety-critical rows remained unverified.
- **Committed in:** `239d395`

***

**Total deviations:** 1 auto-fixed (1 missing critical)
**Impact on plan:** The deviation clarified evidence semantics without expanding product scope or marking hardware behavior verified.

## Verification

Passed:

- `just --summary`
- `JUST_UNSTABLE=1 just --fmt --check`
- `just verify-reference`
- `bazel query 'deps(//firmware/bitaxe:firmware_image)' | grep '//scripts:verify_reference_clean'`
- `bazel query 'deps(//tools/parity:report)' | grep '//scripts:verify_reference_clean'`
- `just build`
- `just test`
- `just package`
- `just parity`
- `espflash list-ports` exited 0 with `No known serial ports found.`
- `cargo fmt --all`
- `cargo clippy --workspace --exclude bitaxe-firmware --all-targets --all-features -- -D warnings`
- `cargo build --workspace --exclude bitaxe-firmware --all-targets --all-features`
- `cargo test --workspace --exclude bitaxe-firmware --all-features`
- `source "$HOME/export-esp.sh" && cargo clippy -p bitaxe-firmware --target xtensa-esp32s3-espidf -- -D warnings`
- `source "$HOME/export-esp.sh" && cargo build -p bitaxe-firmware --target xtensa-esp32s3-espidf`

Known local limitation:

- The literal `cargo clippy --all-targets --all-features -- -D warnings` still fails for the pre-existing firmware host-target limitation documented in Plans 06-08: `esp-idf-sys` rejects host target `aarch64-apple-darwin`. Verification used the established host-workspace plus explicit ESP32-S3 firmware-target split.

## Hardware Smoke

`espflash list-ports` returned successfully with:

```text
No known serial ports found.
```

Conclusion: missing hardware-smoke evidence - no Gamma 601 serial port visible.

No `just flash-monitor` run was attempted, and no boot/log rows were marked `verified`.

## Known Stubs

None. The stub scan found only the plan-required `firmware_commit=Unavailable` fallback text in the evidence requirements; it does not feed UI rendering or mark evidence verified.

## Issues Encountered

- The yolo checkpoint could not complete live hardware smoke because no compatible serial port was visible. This is recorded as missing evidence, not as a passed smoke test.
- The repo still has the documented Cargo all-target host/firmware limitation; scoped host checks plus explicit firmware-target checks passed.

## User Setup Required

None for command verification. Live smoke still requires connecting a Gamma 601 over USB and rerunning `just flash-monitor board=601 port=<port> evidence-dir=docs/parity/evidence/phase-01-gamma-601-boot-log`.

## Next Phase Readiness

Phase 1 command, package, parity, and missing-hardware evidence paths are complete. Phase 2 can proceed with configuration/NVS modeling, while hardware smoke remains explicit missing evidence until a Gamma 601 serial port is available.

***
*Phase: 01-foundation-and-gamma-601-boot-log*
*Completed: 2026-06-21*

## Self-Check: PASSED

- Found created files: `Justfile`, `docs/parity/evidence/phase-01-gamma-601-boot-log.md`, and this summary.
- Found task commits: `7657b54`, `5c625c4`, and `239d395`.
