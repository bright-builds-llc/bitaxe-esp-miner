---
phase: 01-foundation-and-gamma-601-boot-log
plan: "07"
subsystem: parity
tags: [rust, bazel, parity, provenance, evidence, gamma-601]

requires:
  - phase: 01-foundation-and-gamma-601-boot-log
    provides: Reference guard, Rust workspace, Bazel graph, safe boot/log firmware, and package/tool stubs from Plans 01-06
provides:
  - Bazel-visible parity report CLI at //tools/parity:report
  - Checklist parser and report validation that rejects false verified parity claims
  - Phase 1 checklist implementation pointers without unsupported verified hardware claims
affects: [parity, evidence, provenance, workflow, future packaging, future flash]

tech-stack:
  added: [clap, serde, serde_json, camino, anyhow]
  patterns:
    - Thin CLI shell around pure checklist parsing and validation logic
    - Trusted parity output starts with the reference cleanliness guard
    - Checklist rows distinguish implementation status from verification evidence

key-files:
  created:
    - tools/parity/BUILD.bazel
  modified:
    - Cargo.lock
    - MODULE.bazel.lock
    - tools/parity/Cargo.toml
    - tools/parity/src/main.rs
    - docs/parity/checklist.md

key-decisions:
  - "Run //scripts:verify_reference_clean before checklist parsing, reference commit lookup, or report output."
  - "Keep implemented checklist rows at Evidence = pending until command or hardware evidence is recorded."
  - "Keep Justfile and //firmware/bitaxe:firmware_image rows not-started because those artifacts do not exist yet."
  - "Honor AGENTS.md Rust pre-commit rules by recording the TDD RED failure but committing only the passing implementation."

patterns-established:
  - "Parity report JSON includes reference_commit, rows, and validation_errors."
  - "Safety-critical verified rows require hardware-smoke or hardware-regression evidence."
  - "Bazel host tool targets depend on crate_universe labels and include guard tools as runfiles/data."

requirements-completed: [FND-03, FND-10, FND-11]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 1-2026-06-21T00-30-20
generated_at: 2026-06-21T03:33:03Z

duration: 11min
completed: 2026-06-21
---

# Phase 01 Plan 07: Parity Reporting Summary

**Guarded parity report CLI with reference provenance, checklist evidence output, and false-verification rejection**

## Performance

- **Duration:** 11 min
- **Started:** 2026-06-21T03:21:57Z
- **Completed:** 2026-06-21T03:33:03Z
- **Tasks:** 2
- **Files modified:** 6

## Accomplishments

- Replaced the parity tool stub with a real `report` subcommand that parses `docs/parity/checklist.md`, runs the reference guard first, reads the pinned reference commit after the guard passes, and emits text or JSON.
- Added validation for `verified` rows with pending evidence and for safety-critical verified rows without `hardware-smoke` or `hardware-regression`.
- Updated Phase 1 parity rows to point at the current implementation surfaces while preserving evidence-backed `verified` semantics.

## Task Commits

1. **Task 1: Implement parity report CLI** - `fa983ed` (feat)
2. **Task 2: Update Phase 1 checklist pointers without false verification** - `73a6551` (docs)

## Files Created/Modified

- `tools/parity/BUILD.bazel` - Public Bazel `report` binary and `tests` target with reference guard data dependency.
- `tools/parity/src/main.rs` - CLI, report environment, Markdown table parser, JSON/text renderers, validation rules, and unit tests.
- `tools/parity/Cargo.toml` - Parity CLI dependencies from workspace pins.
- `Cargo.lock` - Locked transitive dependencies for the parity CLI.
- `MODULE.bazel.lock` - Updated crate_universe resolution for the parity CLI dependency graph.
- `docs/parity/checklist.md` - Phase 1 implementation pointers and status updates with evidence left pending.

## Decisions Made

- Kept trusted parity report generation fail-closed: dirty, missing, or mismatched reference state blocks report output instead of using `Unavailable`.
- Treated `implemented` as code-present only; evidence stayed `pending` until command output or hardware evidence is recorded.
- Left `Justfile` and `//firmware/bitaxe:firmware_image` rows as `not-started` because those artifacts are not present yet.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Recorded TDD RED without committing a failing state**
- **Found during:** Task 1 (Implement parity report CLI)
- **Issue:** The plan requested a TDD RED commit, but AGENTS.md requires Rust pre-commit checks to pass before commits.
- **Fix:** Added the RED tests, ran `cargo test -p bitaxe-parity` and captured the expected compile failure, then committed only after the implementation passed verification.
- **Files modified:** `tools/parity/src/main.rs`, `tools/parity/Cargo.toml`
- **Verification:** Initial RED run failed on missing report types/helpers; final `cargo test -p bitaxe-parity` passed.
- **Committed in:** `fa983ed`

**2. [Rule 3 - Blocking] Added required lockfile updates for parity dependencies**
- **Found during:** Task 1 (Implement parity report CLI)
- **Issue:** The plan file list did not name `Cargo.lock` or `MODULE.bazel.lock`, but adding CLI dependencies and Bazel crate labels requires both locks to stay reproducible.
- **Fix:** Let Cargo and Bazel update the locks, then included them in the task commit.
- **Files modified:** `Cargo.lock`, `MODULE.bazel.lock`
- **Verification:** `cargo test -p bitaxe-parity`, `bazel test //tools/parity:tests`, and `bazel run //tools/parity:report -- report --checklist docs/parity/checklist.md --fail-on-invalid-verified` passed.
- **Committed in:** `fa983ed`

***

**Total deviations:** 2 auto-fixed (2 blocking)
**Impact on plan:** Both were workflow/correctness requirements. No feature scope was added beyond the planned parity report and checklist ledger.

## Issues Encountered

- `cargo clippy --all-targets --all-features -- -D warnings` still fails on the pre-existing firmware host-target limitation documented in Plan 06: `esp-idf-sys` rejects host target `aarch64-apple-darwin`. The established scoped checks passed: host workspace checks excluding `bitaxe-firmware`, plus `cargo check -p bitaxe-firmware --target xtensa-esp32s3-espidf`.

## Verification

Passed:

- `cargo test -p bitaxe-parity`
- `bazel test //tools/parity:tests`
- `bazel query 'deps(//tools/parity:report)' | grep '//scripts:verify_reference_clean'`
- `bazel run //tools/parity:report -- report --checklist docs/parity/checklist.md --fail-on-invalid-verified`
- Task 1 acceptance greps for `fail-on-invalid-verified`, `hardware-smoke`, `hardware-regression`, `reference_commit`, `verify_reference_clean`, and `name = "report"`.
- Task 2 acceptance greps for `scripts/verify-reference-clean.sh`, `//firmware/bitaxe:firmware_image`, `tools/flash`, and `tools/parity`.
- `cargo fmt --all`
- `cargo clippy --workspace --exclude bitaxe-firmware --all-targets --all-features -- -D warnings`
- `cargo build --workspace --exclude bitaxe-firmware --all-targets --all-features`
- `cargo test --workspace --exclude bitaxe-firmware --all-features`
- `source "$HOME/export-esp.sh" && cargo check -p bitaxe-firmware --target xtensa-esp32s3-espidf`

## User Setup Required

None.

## Next Phase Readiness

- Plan 08 can build on the parity report guard pattern for package manifest provenance.
- Later hardware plans still need actual Gamma 601 `hardware-smoke` or `hardware-regression` evidence before any ASIC, voltage, fan, thermal, power, or mining parity row can be marked `verified`.

***
*Phase: 01-foundation-and-gamma-601-boot-log*
*Completed: 2026-06-21*

## Self-Check: PASSED

- Found created/modified files: `tools/parity/BUILD.bazel`, `tools/parity/src/main.rs`, `tools/parity/Cargo.toml`, `docs/parity/checklist.md`, `Cargo.lock`, `MODULE.bazel.lock`, and this summary.
- Found task commits: `fa983ed` and `73a6551`.
