---
phase: 03-bm1366-asic-protocol-and-safe-initialization
plan: "04"
subsystem: asic
tags: [rust, bm1366, init-plan, frequency, voltage, safety-gates, fixtures]

requires:
  - phase: 03-bm1366-asic-protocol-and-safe-initialization
    provides: Active BM1366 dispatch, semantic adapter actions, observations, and fail-closed transcript contracts from Plan 03-03
provides:
  - Fail-closed BM1366 staged initialization planning
  - Chip-detect-only staging with no mining or production work submission
  - Board, config, power, thermal, and safety preflight evidence tokens
  - Pure Ultra 205 BM1366 frequency and voltage transition decisions
  - Reference-derived init/frequency/voltage fixture metadata
affects: [phase-03, phase-04, firmware-uart-adapter, safety-evidence, parity-checklist]

tech-stack:
  added: []
  patterns:
    - "Pure staged decisions emit semantic adapter actions and fail closed before firmware effects"
    - "Hardware-sensitive frequency and voltage transitions carry explicit unverified evidence status"
    - "Task fixtures keep provenance metadata separate from MIT Rust source"

key-files:
  created:
    - crates/bitaxe-asic/src/bm1366/init_plan.rs
    - crates/bitaxe-asic/src/bm1366/frequency_voltage.rs
    - crates/bitaxe-asic/fixtures/bm1366/init-plan-cases.json
  modified:
    - crates/bitaxe-asic/BUILD.bazel
    - crates/bitaxe-asic/src/bm1366.rs

key-decisions:
  - "Run TDD RED failures but avoid committing failing RED states because AGENTS.md requires passing Rust checks before every commit."
  - "Use Phase 2 Ultra 205 BM1366 catalog/default facts as init preflight gates instead of duplicating board identity in firmware."
  - "Keep voltage transitions as pure data only and mark both frequency and voltage effects below verified until Ultra 205 hardware evidence exists."
  - "Use an independently written pure PLL search for BM1366 frequency command data while preserving MissingHardwareEvidence status."

patterns-established:
  - "Bm1366Preflight carries optional evidence tokens and returns PreflightMissing plus HoldResetLow when gates are absent."
  - "Bm1366FrequencyPlan and Bm1366VoltagePlan parse through AsicFrequencyMhz/CoreVoltageMv and expose parity/evidence status."

requirements-completed: [ASIC-05, ASIC-06, ASIC-08]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 3-2026-06-26T22-40-40
generated_at: 2026-06-27T00:55:28Z

duration: 14 min
completed: 2026-06-27
---

# Phase 03 Plan 04: BM1366 Safe Init And Transition Decisions Summary

**Fail-closed BM1366 init planning with explicit preflight evidence and unverified pure frequency/voltage decisions**

## Performance

- **Duration:** 14 min
- **Started:** 2026-06-27T00:41:36Z
- **Completed:** 2026-06-27T00:55:28Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments

- Added `Bm1366InitPlan` with chip-detect-only and full-init paths gated by board, config, power, thermal, and safety evidence.
- Added fail-closed `PreflightMissing` outcomes with `HoldResetLow` before register init, frequency/nonce setup, max baud, or initialized status when evidence is missing.
- Added pure `Bm1366FrequencyPlan` and `Bm1366VoltagePlan` decisions that reuse Phase 2 validation and preserve `MissingHardwareEvidence`/`ImplementedNotVerified`.
- Added init, frequency, voltage, rejection, and evidence-status fixture metadata with pinned reference commit and checklist IDs.

## Task Commits

Each task was committed atomically:

1. **Task 1: Add Bm1366InitPlan with fail-closed preflight gates** - `15989b3` (feat)
2. **Task 2: Add pure frequency and voltage transition decisions** - `5804c55` (feat)

_Note: TDD RED failures were run and recorded before implementation, but failing intermediate states were not committed because the repo Rust pre-commit rule requires passing format, clippy, build, and tests before every commit._

## Files Created/Modified

- `crates/bitaxe-asic/src/bm1366/init_plan.rs` - Staged chip-detect/full-init decisions, preflight evidence tokens, fail-closed actions, and unit tests.
- `crates/bitaxe-asic/src/bm1366/frequency_voltage.rs` - Pure frequency/voltage transition decisions, PLL-derived frequency command data, evidence status, and unit tests.
- `crates/bitaxe-asic/fixtures/bm1366/init-plan-cases.json` - Reference-derived init and transition fixture metadata.
- `crates/bitaxe-asic/BUILD.bazel` - Adds new BM1366 source files to the Bazel library target.
- `crates/bitaxe-asic/src/bm1366.rs` - Exports `init_plan` and `frequency_voltage`.

## Decisions Made

- TDD RED failures are evidence only, not commits, because the stricter repo Rust commit rule takes precedence.
- Full init remains pure and fail-closed; firmware effects still require a future adapter and evidence plan.
- Frequency decisions may emit typed `SetFrequency` command data but remain `MissingHardwareEvidence`.
- Voltage decisions intentionally produce no adapter command in this phase.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Preserved Rust pre-commit requirements during TDD**
- **Found during:** Tasks 1 and 2
- **Issue:** The generic TDD flow allows failing RED commits, but repo-local Rust rules require `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features` before every commit.
- **Fix:** Ran RED tests to prove failure, then committed only passing task outcomes after the full Rust gate.
- **Files modified:** Task files only.
- **Verification:** Full Rust pre-commit sequence passed before both task commits.
- **Committed in:** `15989b3`, `5804c55`

**2. [Rule 1 - Bug] Fixed PLL candidate comparison overflow**
- **Found during:** Task 2
- **Issue:** The pure PLL search used `u32` cross-multiplication against the initial sentinel candidate and overflowed in debug tests.
- **Fix:** Widened comparison arithmetic to `u64` before selecting the best candidate.
- **Files modified:** `crates/bitaxe-asic/src/bm1366/frequency_voltage.rs`
- **Verification:** `cargo test -p bitaxe-asic frequency_voltage --all-features` passed, and the full Rust pre-commit sequence passed.
- **Committed in:** `5804c55`

---

**Total deviations:** 2 auto-fixed (1 missing critical, 1 bug)
**Impact on plan:** No scope change. Both adjustments preserved repo rules and correctness for the pure decision core.

## Issues Encountered

- Expected RED failures occurred for missing init-plan and frequency/voltage APIs.
- Task 2 initially hit a const `From` compile restriction and a redundant-cast Clippy warning; both were fixed before the task commit.

## User Setup Required

None - no external service configuration required.

## Known Stubs

None - source and fixture files touched by this plan have no placeholder or stub data that blocks the plan goal.

## Verification

- `cargo test -p bitaxe-asic init_plan --all-features` - passed, 6 tests.
- `cargo test -p bitaxe-asic frequency_voltage --all-features` - passed, 4 tests.
- `cargo test -p bitaxe-asic --all-features` - passed, 39 tests.
- `bazel test //crates/bitaxe-asic:tests` - passed.
- `rg -n "PreflightMissing|MissingHardwareEvidence|power_thermal_evidence_missing|safety_preflight_evidence_missing|SafetyPreflightEvidence" crates/bitaxe-asic/src crates/bitaxe-asic/fixtures` - passed.
- `git status --short reference/esp-miner` - clean, no output.
- `cargo fmt --all` - passed before each task commit.
- `cargo clippy --all-targets --all-features -- -D warnings` - passed before each task commit.
- `cargo build --all-targets --all-features` - passed before each task commit.
- `cargo test --all-features` - passed before each task commit.

## Next Phase Readiness

Ready for Plan 03-05 to add the narrow firmware UART/reset/status adapter, evidence records, checklist updates, and human-gated chip-detect smoke review. Safety-critical init/frequency/voltage behavior remains below verified until Ultra 205 hardware evidence exists.

---
*Phase: 03-bm1366-asic-protocol-and-safe-initialization*
*Completed: 2026-06-27*

## Self-Check: PASSED

- Created files verified on disk.
- Summary file verified on disk.
- Task commits verified in git history: `15989b3`, `5804c55`.
