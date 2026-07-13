---
phase: 01-foundation-and-gamma-601-boot-log
plan: "01"
subsystem: foundation
tags: [bazel, bzlmod, submodule, reference-guard, provenance]
requires: []
provides:
  - "Pinned upstream ESP-Miner reference submodule at c1915b0a63bfabebdb95a515cedfee05146c1d50"
  - "Bazel-visible reference cleanliness guard and tests"
  - "Minimal Bazel/Bzlmod root for Phase 1 follow-on targets"
affects: [foundation, parity, provenance, packaging, flash]
tech-stack:
  added: [bazel-9.1.1, rules_rust-0.70.0, rules_shell-0.8.0]
  patterns:
    - "Bazel targets expose repo-owned shell workflow checks"
    - "Reference guard fails closed on missing, dirty, uninitialized, or mismatched upstream state"
key-files:
  created:
    - .gitmodules
    - .bazelversion
    - .gitignore
    - BUILD.bazel
    - MODULE.bazel
    - MODULE.bazel.lock
    - scripts/BUILD.bazel
    - scripts/verify-reference-clean.sh
    - scripts/verify-reference-clean-test.sh
  modified:
    - reference/esp-miner
key-decisions:
  - "Pin reference/esp-miner to c1915b0a63bfabebdb95a515cedfee05146c1d50 and initialize nested upstream submodules for recursive cleanliness."
  - "Use rules_shell 0.8.0 for Bazel-visible shell targets because Bazel 9.1.1 did not expose native sh_binary/sh_test in this workspace."
  - "Track MODULE.bazel.lock and ignore bazel-* output trees so Bzlmod resolution is reproducible without committing generated build output."
patterns-established:
  - "Reference guard uses EXPECTED_REFERENCE_COMMIT and REFERENCE_DIR env overrides for isolated tests while defaulting to the pinned project reference."
  - "Bazel shell targets live under //scripts and are backed by checked-in, rerunnable Bash scripts."
requirements-completed: [FND-01, FND-02, FND-03, FND-10]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 1-2026-06-21T00-30-20
generated_at: 2026-06-21T02:06:33Z
duration: 10 min
completed: 2026-06-21
---

# Phase 01 Plan 01: Reference Trust Root Summary

**Pinned ESP-Miner reference submodule with a Bazel-visible fail-closed cleanliness guard**

## Performance

- **Duration:** 10 min
- **Started:** 2026-06-21T01:56:29Z
- **Completed:** 2026-06-21T02:06:33Z
- **Tasks:** 2
- **Files modified:** 10

## Accomplishments

- Added `reference/esp-miner` as a pinned upstream submodule at `c1915b0a63bfabebdb95a515cedfee05146c1d50`.
- Added `scripts/verify-reference-clean.sh` to fail on missing, dirty, uninitialized, or mismatched reference state and print the pinned commit on success.
- Added Bazel 9.1.1 Bzlmod root files and public `//scripts:verify_reference_clean` / `//scripts:verify_reference_clean_test` targets.

## Task Commits

Each task was committed atomically:

1. **Task 1: Pin the upstream ESP-Miner reference submodule** - `a8fba58` (feat)
2. **Task 2 RED: Reference guard behavior tests** - `79671a8` (test)
3. **Task 2 GREEN: Bazel-visible reference guard** - `ca6b2bb` (feat)

## Files Created/Modified

- `.gitmodules` - Defines the upstream ESP-Miner submodule at `reference/esp-miner`.
- `reference/esp-miner` - Parent repo submodule pointer pinned to the accepted upstream commit.
- `.bazelversion` - Pins Bazelisk/Bazel execution to `9.1.1`.
- `MODULE.bazel` and `MODULE.bazel.lock` - Establish the Bzlmod root, `rules_rust`, and `rules_shell` dependencies.
- `BUILD.bazel` - Root Bazel package anchor.
- `.gitignore` - Ignores generated `bazel-*` output trees.
- `scripts/verify-reference-clean.sh` - Fail-closed reference cleanliness guard.
- `scripts/verify-reference-clean-test.sh` - Temporary-repo behavior tests for guard outcomes.
- `scripts/BUILD.bazel` - Public shell binary and shell test targets.

## Decisions Made

- The nested upstream `components/libsecp256k1/libsecp256k1` submodule was initialized so recursive submodule checks can be strict without failing the clean pinned checkout.
- `rules_shell` was added as the explicit shell-rule provider for Bazel 9.1.1; `sh_binary` and `sh_test` were not available as unqualified native rules in this workspace.
- The Bzlmod lockfile is tracked for reproducible resolution, while generated Bazel output directories are ignored.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Initialized nested upstream submodule**
- **Found during:** Task 1 (Pin the upstream ESP-Miner reference submodule)
- **Issue:** `git submodule status --recursive reference/esp-miner` showed the upstream nested `libsecp256k1` submodule as uninitialized, which would make a recursive reference guard fail on a valid checkout.
- **Fix:** Ran recursive submodule initialization for `reference/esp-miner`.
- **Files modified:** `reference/esp-miner` worktree only; parent repo records the planned top-level submodule pointer.
- **Verification:** `git submodule status --recursive reference/esp-miner` reports both submodules without `-`, `+`, or `U` prefixes.
- **Committed in:** `a8fba58`

**2. [Rule 3 - Blocking] Added explicit Bazel shell rules**
- **Found during:** Task 2 (Add the Bazel-visible reference guard)
- **Issue:** `bazel test //scripts:verify_reference_clean_test` failed because `sh_binary` and `sh_test` were not defined in this Bazel 9.1.1 workspace.
- **Fix:** Added `rules_shell 0.8.0` to `MODULE.bazel`, tracked `MODULE.bazel.lock`, and loaded `sh_binary` / `sh_test` explicitly in `scripts/BUILD.bazel`.
- **Files modified:** `MODULE.bazel`, `MODULE.bazel.lock`, `scripts/BUILD.bazel`
- **Verification:** `bazel test //scripts:verify_reference_clean_test` passes.
- **Committed in:** `ca6b2bb`

**3. [Rule 3 - Blocking] Ignored generated Bazel output trees**
- **Found during:** Task 2 (Add the Bazel-visible reference guard)
- **Issue:** Bazel verification created untracked `bazel-*` output trees.
- **Fix:** Added a minimal `.gitignore` entry for `bazel-*`.
- **Files modified:** `.gitignore`
- **Verification:** `git status --short` no longer reports Bazel output trees.
- **Committed in:** `ca6b2bb`

**Total deviations:** 3 auto-fixed (1 missing critical, 2 blocking)
**Impact on plan:** All fixes were required to make the planned recursive guard and Bazel-visible targets work reliably. No scope beyond the reference trust root was added.

## Issues Encountered

- `bazel run //scripts:verify_reference_clean` initially executed outside the workspace root and could not find `reference/esp-miner`; the guard now resolves relative paths through `BUILD_WORKSPACE_DIRECTORY` while preserving direct shell execution.

## User Setup Required

None - no external service configuration required.

## Verification

Passed:

- `git submodule status --recursive reference/esp-miner`
- `bash scripts/verify-reference-clean-test.sh`
- `bazel test //scripts:verify_reference_clean_test`
- `bazel run //scripts:verify_reference_clean`
- `shfmt -d scripts/verify-reference-clean.sh scripts/verify-reference-clean-test.sh`

## Known Stubs

None.

## Threat Flags

None - the new reference and shell-script trust surfaces were already covered by the plan threat model.

## Next Phase Readiness

Ready for `01-02-PLAN.md`. The reference trust root is pinned and Bazel-visible, so later Cargo/Bazel workspace and parity work can depend on a clean upstream baseline. FND-03 and FND-10 completion here is limited to the reference-trust portion scoped by this plan.

*Phase: 01-foundation-and-gamma-601-boot-log*
*Completed: 2026-06-21*

## Self-Check: PASSED

- Verified created summary and key files exist on disk.
- Verified task commits exist: `a8fba58`, `79671a8`, `ca6b2bb`.
