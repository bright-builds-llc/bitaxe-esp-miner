---
phase: 20-active-safety-hardware-telemetry-evidence
plan: "04"
subsystem: parity-evidence
tags: [failure-paths, self-test, watchdog, display-input, safety-allow, hardware-evidence]

requires:
  - phase: 20-active-safety-hardware-telemetry-evidence
    provides: Plan 20-01 evidence contract and safety allow context.
  - phase: 20-active-safety-hardware-telemetry-evidence
    provides: Plan 20-02 package identity, detector-gated safe-baseline evidence, and redacted serial log.
  - phase: 20-active-safety-hardware-telemetry-evidence
    provides: Plan 20-03 active safety telemetry boundaries and non-claims.
provides:
  - Phase-owned failure-path wrapper and Bazel test for blocked or future allow-gated fault-stimulus evidence.
  - Self-test/watchdog/load evidence pack with exact startup breadcrumbs and below-verified boundaries.
  - Runtime display/input evidence pack with startup display breadcrumb and runtime-gap boundary.
  - Failure-path evidence pack documenting missing stimuli, expected faults, recovery paths, projections, and final safe-state markers.
affects: [phase-20, parity-evidence, active-safety, failure-paths, runtime-display-input]

tech-stack:
  added: []
  patterns:
    - Phase 20 safety evidence wrappers validate an allow manifest before writing machine-searchable status logs.
    - Unsupported-pending evidence records precise blocked fields instead of promoting unobserved hardware behavior.
    - Startup hardware-smoke markers are separated from runtime display/input, load, watchdog recovery, and fault-stimulus claims.

key-files:
  created:
    - scripts/phase20-failure-paths.sh
    - scripts/phase20-failure-paths-test.sh
    - docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/self-test-watchdog-load.md
    - docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/self-test-watchdog-load/allow-self-test-watchdog-load.json
    - docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/self-test-watchdog-load/self-test-watchdog-load.log
    - docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/runtime-display-input.md
    - docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/runtime-display-input/allow-display-input.json
    - docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/runtime-display-input/display-input.log
    - docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/failure-paths.md
    - docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/failure-paths/allow-failure-paths.json
    - docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/failure-paths/failure-paths.log
  modified:
    - scripts/BUILD.bazel

key-decisions:
  - "The Phase 20 failure-path wrapper is blocked-only in this plan and does not introduce live fault stimulus, raw hardware commands, flashing, stress, curl, or I2C paths."
  - "Self-test hardware submodes, bounded load, watchdog recovery, runtime display/input, and failure-path behavior remain independently below verified unless a future plan supplies bounded hardware-regression evidence."
  - "Startup SSD1306 and watchdog supervisor markers are breadcrumbs only, not runtime display/input or load/watchdog recovery proof."

patterns-established:
  - "Failure-path wrapper logs include `failure_paths_status`, `fault_stimulus_status`, `expected_fault_status`, API/WebSocket projection fields, final safe-state requirement, checklist rows, and explicit non-claims."
  - "Plan 20 evidence ledgers preserve checklist rows separately from claim tiers so later plans can promote only the rows they actually prove."

requirements-completed: [SAFE-04, SAFE-05, SAFE-06, SAFE-08, SAFE-09, EVD-05]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 20-2026-07-03T20-48-00
generated_at: 2026-07-03T22:58:09Z

duration: 16 min
completed: 2026-07-03
---

# Phase 20 Plan 04: Self-Test, Display/Input, and Failure-Path Evidence Summary

**Failure-path allow wrapper plus exact self-test/watchdog/load, display/input, and fault-stimulus evidence boundaries**

## Performance

- **Duration:** 16 min
- **Started:** 2026-07-03T22:42:46Z
- **Completed:** 2026-07-03T22:58:09Z
- **Tasks:** 3
- **Files modified:** 12

## Accomplishments

- Added `scripts/phase20-failure-paths.sh`, a phase-owned wrapper that validates `surface: "failure-paths"` allow manifests before producing blocked failure-path evidence.
- Added `scripts/phase20-failure-paths-test.sh` and Bazel registration for missing-manifest, failed-allow, and valid unsupported-pending wrapper behavior.
- Recorded self-test/watchdog/load evidence with `load_stress_status`, `self_test_hardware_status`, and `SAFE-09` kept below verified.
- Recorded runtime display/input evidence with startup display breadcrumbs only and `display_input_status=runtime_gap reason=hardware_evidence_pending startup_only=true`.
- Recorded failure-path evidence showing `failure_paths_status: blocked - no production-safe fault stimulus route`, no fault stimulus, no expected fault observation, no API/WebSocket projection, and no final safe-state marker.

## Task Commits

Each task was committed atomically:

1. **Task 1: Add a failure-path evidence wrapper and tests** - `6957bed` (feat)
2. **Task 2: Record self-test/watchdog/load and runtime display/input evidence** - `446a94d` (docs)
3. **Task 3: Record failure-path blocked evidence** - `31743c5` (docs)

## Files Created/Modified

- `scripts/phase20-failure-paths.sh` - Writes blocked failure-path evidence after `safety-allow` validation and contains no live fault-stimulus command path.
- `scripts/phase20-failure-paths-test.sh` - Tests missing manifest, failed allow validation, and unsupported-pending output boundaries.
- `scripts/BUILD.bazel` - Registers `phase20_failure_paths` and `phase20_failure_paths_test`.
- `docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/self-test-watchdog-load.md` - Ledgers supported watchdog startup/yield breadcrumbs and below-verified self-test/load/watchdog recovery boundaries.
- `docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/self-test-watchdog-load/allow-self-test-watchdog-load.json` - Allows unsupported-pending deferred self-test/watchdog/load evidence for `SELF-001` and `SAFE-09`.
- `docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/self-test-watchdog-load/self-test-watchdog-load.log` - Captures wrapper output for watchdog supervisor breadcrumbs and blocked load/self-test routes.
- `docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/runtime-display-input.md` - Ledgers startup display observation and runtime display/input gap boundaries.
- `docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/runtime-display-input/allow-display-input.json` - Allows read-only display/input smoke evidence for `IO-001`, `UI-001`, `UI-002`, and `UI-003`.
- `docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/runtime-display-input/display-input.log` - Captures startup marker and runtime-gap output.
- `docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/failure-paths.md` - Ledgers each missing stimulus, expected fault, abort condition, restore path, projection, final safe-state marker, checklist row, and non-claim.
- `docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/failure-paths/allow-failure-paths.json` - Allows unsupported-pending deferred failure-path evidence for `PWR-001`, `PWR-002`, `THR-001`, `THR-002`, `SELF-001`, and `SAFE-04`.
- `docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/failure-paths/failure-paths.log` - Captures default no-stimulus wrapper output and exact blocked failure-path statuses.

## Decisions Made

- The failure-path wrapper stays strictly blocked by default; it validates allow-manifest context but does not run fault injection or hardware-control commands.
- Plan 20-04 preserves independent claim boundaries: self-test, load, watchdog recovery, runtime display/input, and failure paths can be promoted later only with their own bounded evidence.
- Plan 20-04 reuses Phase 14 wrapper behavior and Plan 20-02 safe-baseline evidence where useful, but does not turn startup breadcrumbs into runtime parity claims.

## Deviations from Plan

### Auto-Fixed Issues

**1. [Rule 1 - Bug] Fixed shell test helper option handling**
- **Found during:** Task 1 verification
- **Issue:** The new shell test helper used `grep -Fq "$needle"`, so a required assertion containing `--surface failure-paths` was parsed as a grep option and caused the Bazel test to fail.
- **Fix:** Added `--` before the grep pattern in the helper.
- **Files modified:** `scripts/phase20-failure-paths-test.sh`
- **Verification:** `bazel test //scripts:phase20_failure_paths_test` passed.
- **Commit:** `6957bed`

### Process Adjustments

**1. TDD RED not committed separately**
- **Found during:** Task 1 RED step
- **Issue:** The failing RED test correctly failed because `scripts/phase20-failure-paths.sh` did not exist, but repo instructions require full Rust pre-commit verification before any commit.
- **Fix:** Kept the RED result as execution evidence and committed only after GREEN implementation and required verification passed.
- **Files modified:** `scripts/phase20-failure-paths-test.sh`, `scripts/phase20-failure-paths.sh`, `scripts/BUILD.bazel`
- **Verification:** RED failed with missing wrapper; GREEN passed with Bazel and full Rust pre-commit checks.
- **Commit:** `6957bed`

**2. Generated self-test/display logs normalized before commit**
- **Found during:** Task 2 commit review
- **Issue:** `git diff --cached --check` flagged generated Bazel progress lines in self-test/watchdog/load and display/input logs for trailing whitespace.
- **Fix:** Mechanically trimmed trailing whitespace in generated log files without changing evidence fields or conclusions.
- **Files modified:** `self-test-watchdog-load/self-test-watchdog-load.log`, `runtime-display-input/display-input.log`
- **Verification:** `git diff --cached --check` passed.
- **Commit:** `446a94d`

**3. Generated failure-path log normalized before commit**
- **Found during:** Task 3 commit review
- **Issue:** `git diff --cached --check` flagged generated Bazel progress lines in `failure-paths.log` for trailing whitespace.
- **Fix:** Mechanically trimmed trailing whitespace in the generated log file without changing evidence fields or conclusions.
- **Files modified:** `failure-paths/failure-paths.log`
- **Verification:** `git diff --cached --check` passed.
- **Commit:** `31743c5`

**Total deviations:** 1 auto-fixed issue; 3 repo-rule/process adjustments.
**Impact on plan:** No scope change. All adjustments preserved the plan's no-active-fault-stimulus and exact-claim boundaries.

## Issues Encountered

None requiring blockers. Self-test hardware submodes, bounded load stress, watchdog recovery, runtime display/input, overheat stimulus, fan fault stimulus, power fault stimulus, thermal fault stimulus, ASIC fault stimulus, projections, recovery paths, and final safe-state markers remain intentionally below verified because this plan did not define a production-safe bounded route for them.

## Verification

- `node /Users/peterryszkiewicz/.codex/get-shit-done/bin/gsd-tools.cjs verify lifecycle 20 --require-plans --raw` returned `valid` before execution began and again after task commits.
- `bash -n scripts/phase20-failure-paths.sh scripts/phase20-failure-paths-test.sh` passed.
- `bazel test //scripts:phase20_failure_paths_test` passed.
- `bazel test //scripts:phase14_self_test_watchdog_load_test //scripts:phase14_display_input_test` passed.
- `bazel test //scripts:phase20_failure_paths_test //scripts:phase14_self_test_watchdog_load_test //scripts:phase14_display_input_test` passed.
- `cargo test -p bitaxe-safety --all-features self_test` passed.
- `cargo test -p bitaxe-safety --all-features watchdog` passed.
- `cargo test -p bitaxe-safety --all-features` passed and covered self-test, watchdog, and fault tests.
- Task acceptance searches passed for wrapper status fields, Bazel registration, no active command patterns, self-test/watchdog/load evidence fields, display/input evidence fields, failure-path evidence fields, checklist rows, claim tiers, evidence classes, and non-claims.
- Targeted redaction scanning found no secret, credential, private endpoint, IP, SSID, token, authorization, cookie, or MAC-address hazards in the new Plan 20-04 scripts and evidence files.
- `just test` passed.
- `just parity` passed with `validation_errors: none`.
- `just verify-reference` passed and printed reference commit `c1915b0a63bfabebdb95a515cedfee05146c1d50`.
- `git diff -- reference/esp-miner --exit-code` passed.
- Before each task commit: `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features` passed.

## Known Stubs

None. The `pending`, `not-run`, `blocked`, `deferred`, and `below verified` statuses in the evidence files are intentional claim-boundary statuses required by the plan; they do not prevent the plan goal.

## Auth Gates

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Ready for Plan 20-05. Failure-path, self-test, load, watchdog recovery, runtime display/input, and final safe-state promotion remain gated on future detector-gated hardware-regression evidence with documented inputs, abort conditions, recovery paths, projections, and final safe-state markers.

## Self-Check: PASSED

- Found `.planning/phases/20-active-safety-hardware-telemetry-evidence/20-04-SUMMARY.md`.
- Found all 12 Plan 20-04 script, Bazel, and evidence files.
- Found task commits `6957bed`, `446a94d`, and `31743c5`.
- Confirmed the summary uses only the opening and closing frontmatter delimiters.
- Stub scan found no implementation stubs; matches were limited to shell argument default initializers, and blocked/deferred evidence statuses are intentional claim-boundary statuses.
- Threat surface scan found no unplanned network endpoint, auth path, raw hardware command, or schema trust-boundary change.

*Phase: 20-active-safety-hardware-telemetry-evidence*
*Completed: 2026-07-03*
