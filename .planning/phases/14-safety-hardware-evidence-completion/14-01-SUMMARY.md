---
phase: 14-safety-hardware-evidence-completion
plan: "01"
subsystem: safety-hardware-evidence
tags: [rust, tools-parity, safety-allow, hardware-regression, clap, serde-json, camino]
requires:
  - phase: 13-final-ultra-205-release-evidence
    provides: package manifest provenance and detector-gated Ultra 205 evidence conventions
provides:
  - Typed Phase 14 safety allow-manifest parser and validator
  - `tools/parity` `safety-allow` CLI preflight gate
  - Bazel registration for the safety allow module
affects: [phase-14-active-safety-probes, hardware-regression-evidence, parity-checklist-promotion]
tech-stack:
  added: []
  patterns:
    - typed JSON manifest validation in tools/parity
    - thin CLI shell over pure validator functions
key-files:
  created:
    - tools/parity/src/safety_allow.rs
  modified:
    - tools/parity/src/main.rs
    - tools/parity/BUILD.bazel
key-decisions:
  - "Implemented the allow gate as a typed tools/parity validator with file I/O isolated to the loader and CLI dispatch."
  - "Kept the TDD RED failure as summary evidence rather than committing failing code, because repo rules require passing Rust checks before every commit."
  - "Used a custom Utf8PathBuf deserializer instead of enabling new dependency features."
patterns-established:
  - "Safety hardware probes must pass a machine-readable allow manifest before active evidence can support hardware-regression claims."
  - "Active claim tiers require recovery steps, abort conditions, safe-state markers, and hardware-regression evidence class."
requirements-completed: [SAFE-01, SAFE-04, SAFE-08, EVD-05]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 14-2026-06-30T23-56-34
generated_at: 2026-07-01T01:12:39Z
duration: 14m 27s
completed: 2026-07-01
---

# Phase 14 Plan 01: Safety Allow Manifest Gate Summary

**Typed Phase 14 safety allow-manifest gate for board-205 active hardware evidence.**

## Performance

- **Duration:** 14m 27s
- **Started:** 2026-07-01T00:58:12Z
- **Completed:** 2026-07-01T01:12:39Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments

- Added `SafetyAllowManifest` with all required Phase 14 fields and typed validation for board, detector, board-info, package identity, surface, claim tier, evidence class, allowed command, recovery, abort conditions, safe-state markers, redaction reviewer, and checklist rows.
- Added the `safety-allow` CLI path with `--manifest`, optional `--surface`, and optional `--allowed-command` filters.
- Registered the module in Bazel so `bazel run //tools/parity:report -- safety-allow --manifest ...` can be used by later Phase 14 probe wrappers.

## Task Commits

Each task was committed atomically in buildable states:

1. **Task 1/2: Safety allow validator and CLI dispatch** - `0083a85` (`feat`)
2. **Task 2: Bazel module registration** - `3777e58` (`chore`)

The TDD RED failure was observed and recorded below instead of committed, because AGENTS.md requires passing Rust checks before every commit.

## Files Created/Modified

- `tools/parity/src/safety_allow.rs` - Typed manifest loader, pure validator, CLI report renderer, and focused unit tests.
- `tools/parity/src/main.rs` - `SafetyAllow` subcommand arguments and dispatch.
- `tools/parity/BUILD.bazel` - Includes `src/safety_allow.rs` in the parity binary source list.

## Decisions Made

- Kept manifest validation in a pure module with a small loader boundary, matching the existing `tools/parity` shape.
- Required active claim tiers to use `hardware-regression`; mapped non-active allowed tiers to exact evidence classes (`hardware-smoke`, `deferred`, or `workflow`).
- Preserved the repo dependency surface by deserializing `Utf8PathBuf` locally instead of enabling a new `camino` feature.

## TDD Evidence

- **RED:** `cargo test -p bitaxe-parity --all-features safety_allow` initially produced the intended failing safety assertions for board `205`, detector port mismatch, board-info success, package identity, active-tier `hardware-regression`, and required recovery/safe-state fields.
- **GREEN:** `cargo test -p bitaxe-parity --all-features safety_allow` passed with 10 safety-allow tests.

## Verification

- `cargo fmt --all` - passed before each code commit.
- `cargo clippy --all-targets --all-features -- -D warnings` - passed before each code commit.
- `cargo build --all-targets --all-features` - passed before each code commit.
- `cargo test --all-features --quiet` - passed before each code commit.
- `cargo test -p bitaxe-parity --all-features safety_allow` - passed, 10 tests.
- `bazel test //tools/parity:tests --test_filter=safety_allow` - passed.
- `just parity` - passed with `validation_errors: none`.
- `git diff --check` - passed.
- Acceptance scans confirmed required manifest fields, required rejection strings, and Bazel source registration.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added local Utf8PathBuf deserialization**
- **Found during:** Task 1 (TDD RED safety allow tests)
- **Issue:** `camino::Utf8PathBuf` did not implement `serde::Deserialize` with the current dependency feature set, blocking compilation before the intended RED assertions could run.
- **Fix:** Added a local `deserialize_utf8_path_buf` helper for path fields in `SafetyAllowManifest`.
- **Files modified:** `tools/parity/src/safety_allow.rs`
- **Verification:** `cargo test -p bitaxe-parity --all-features safety_allow`, full Rust pre-commit gate, and Bazel filtered test passed.
- **Committed in:** `0083a85`

**Total deviations:** 1 auto-fixed Rule 3 issue.
**Impact on plan:** No scope expansion; the fix keeps the manifest gate typed without changing dependencies.

## Issues Encountered

- The repo-level Rust pre-commit rule prevented a failing RED commit. The RED result is documented in this summary, and both code commits were made only after the required Rust gate passed.

## Known Stubs

None. Stub scan hits were intentional test/status vocabulary (`unsupported-pending`, invalid redaction-reviewer values, and existing checklist pending-state tests), not runtime placeholders or unwired data paths.

## Threat Flags

None. The new file-reading CLI surface is the planned allow-manifest gate from 14-01 and does not add unplanned network, auth, schema, or hardware-control surface.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Later Phase 14 wrappers can now call `tools/parity:report safety-allow` before any active Ultra 205 safety probe. The gate rejects non-205 boards, detector mismatches, board-info failure, stale package identity, missing recovery/safe-state requirements, incomplete redaction review, and mismatched wrapper filters.

## Self-Check: PASSED

- Found created/modified files: `tools/parity/src/safety_allow.rs`, `tools/parity/src/main.rs`, `tools/parity/BUILD.bazel`, and this summary.
- Found task commits: `0083a85` and `3777e58`.
- Confirmed summary frontmatter uses only the opening and closing standalone delimiters.

*Phase: 14-safety-hardware-evidence-completion*
*Completed: 2026-07-01*
