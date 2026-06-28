---
phase: 07-ota-filesystem-and-release-packaging
plan: 02
title: Package Manifest and Partition Contracts
generated_by: gsd-execute-plan
executor_model: gpt-5-codex
lifecycle_mode: yolo
phase_lifecycle_id: 7-2026-06-28T13-30-15
status: complete
started_at: 2026-06-28T15:30:38Z
completed_at: 2026-06-28T15:51:40Z
duration: 21m02s
requirements_completed:
  - REL-01
  - REL-04
  - REL-06
subsystem: release-package-contracts
tags:
  - rust
  - xtask
  - release-manifest
  - partitions
  - ota
dependency_graph:
  requires:
    - 07-01
  provides:
    - manifest-v2-contract
    - ultra205-partition-contract
    - validate-package-cli
  affects:
    - tools/xtask
    - tools/flash
    - firmware/bitaxe
tech_stack:
  added:
    - esp-idf-part 0.6.0
  patterns:
    - typed serde manifest contracts
    - typed partition CSV contract validation
    - xtask validation command
key_files:
  created:
    - tools/xtask/src/package_manifest.rs
    - tools/xtask/src/partition_contract.rs
    - firmware/bitaxe/partitions-ultra205.csv
  modified:
    - tools/xtask/src/main.rs
    - tools/xtask/Cargo.toml
    - tools/xtask/BUILD.bazel
    - tools/flash/src/main.rs
    - firmware/bitaxe/BUILD.bazel
    - Cargo.lock
    - MODULE.bazel.lock
decisions:
  - Keep existing package generation on the current v1 manifest while defining and validating the v2 release package contract for later packaging work.
  - Validate the checked-in Ultra 205 partition CSV before package manifest generation so release packaging fails on partition drift.
  - Keep flash compatibility anchored on top-level default_flash_image so tools/flash can read v2 manifests without adopting the full release schema.
metrics:
  tasks_completed: 3
  task_commits: 3
  files_created: 3
  files_modified: 8
---

# Phase 07 Plan 02: Package Manifest and Partition Contracts Summary

Package manifest v2, Ultra 205 partition layout, and package validation CLI contracts are implemented with focused Rust tests and Bazel coverage.

## Tasks Completed

| Task | Name | Commit | Files |
| ---- | ---- | ------ | ----- |
| 1 | Define package manifest v2 schema | 199ae9e | tools/xtask/src/package_manifest.rs, tools/xtask/src/main.rs, tools/xtask/BUILD.bazel, tools/flash/src/main.rs |
| 2 | Add Ultra 205 partition contract | f247202 | firmware/bitaxe/partitions-ultra205.csv, tools/xtask/src/partition_contract.rs, tools/xtask/src/main.rs, tools/xtask/Cargo.toml, tools/xtask/BUILD.bazel, firmware/bitaxe/BUILD.bazel, Cargo.lock, MODULE.bazel.lock |
| 3 | Add package validation CLI contract | c4822b2 | tools/xtask/src/main.rs, tools/xtask/src/package_manifest.rs |

## What Changed

- Added typed manifest v2 structs for release metadata, artifacts, image metadata, install notes, license inventory, and provenance manifest.
- Kept `tools/flash` compatible with manifest v2 by validating the existing `default_flash_image` field without requiring the full release schema.
- Added `firmware/bitaxe/partitions-ultra205.csv` with the required `www`, `ota_0`, `ota_1`, and `otadata` offsets for the Ultra 205 release image layout.
- Added an xtask partition contract validator and wired it into package generation so partition drift fails before manifest output.
- Added `xtask validate-package --manifest ... --partition-table ...` to validate manifest v2 required artifacts, metadata, SHA-256 values, factory offset, and partition layout together.

## Verification

- `cargo test -p xtask --all-features package_manifest`
- `cargo test -p xtask --all-features partition_contract`
- `cargo test -p xtask --all-features validate_package`
- `cargo test -p bitaxe-flash --all-features manifest`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo build --all-targets --all-features`
- `cargo test --all-features`
- `bazel test //tools/xtask:tests`
- `git diff --check`

All verification commands passed.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Exported the partition CSV for Bazel test data**
- **Found during:** Task 2
- **Issue:** `bazel test //tools/xtask:tests` could not reliably access the checked-in Ultra 205 partition CSV from the sandbox unless the file was exported and listed as test data.
- **Fix:** Added `exports_files(["partitions-ultra205.csv"])` to `firmware/bitaxe/BUILD.bazel` and listed the exported file in `tools/xtask/BUILD.bazel` test data.
- **Files modified:** firmware/bitaxe/BUILD.bazel, tools/xtask/BUILD.bazel
- **Commit:** f247202

**2. [Rule 3 - Blocking] Wired partition validation into package generation**
- **Found during:** Task 2
- **Issue:** The new validator would otherwise be unused in non-test code, which failed the Rust warnings-as-errors gate and would allow package generation without partition drift protection.
- **Fix:** `run_package_firmware` now validates `firmware/bitaxe/partitions-ultra205.csv` before writing the package manifest.
- **Files modified:** tools/xtask/src/main.rs
- **Commit:** f247202

### Process Adjustments

- The plan asked for TDD red commits, but the repo-level Rust pre-commit rule requires format, clippy, build, and tests before every commit. Red tests were run and recorded as failing evidence, then the passing implementation was committed atomically per task.

## Known Stubs

None. The current `package-firmware` command still emits the existing manifest format by design; manifest v2 is defined and validated as the contract for later package expansion work in this phase.

## Threat Flags

None. The added local CLI validation reads package manifests and partition CSV files at the planned release-packaging boundary and does not introduce network, auth, or device-control surfaces.

## State Notes

- `.planning/STATE.md`, `.planning/config.json`, and Phase 7 planning artifacts had pre-existing orchestrator-owned uncommitted edits before execution. They were not staged during task commits.
- Workflow state commands advanced `.planning/STATE.md` to plan 3, recalculated progress, recorded metrics, recorded decisions, and updated the session pointer. `STATE.md` remains unstaged in the final metadata commit because it includes pre-existing orchestrator-owned edits.
- `.planning/ROADMAP.md` and `.planning/REQUIREMENTS.md` were clean before the workflow updates and are included in the final metadata commit.

## Self-Check: PASSED

- Created files exist: `.planning/phases/07-ota-filesystem-and-release-packaging/07-02-SUMMARY.md`, `tools/xtask/src/package_manifest.rs`, `tools/xtask/src/partition_contract.rs`, `firmware/bitaxe/partitions-ultra205.csv`.
- Task commits exist: `199ae9e`, `f247202`, `c4822b2`.
