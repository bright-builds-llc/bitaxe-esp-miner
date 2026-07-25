---
phase: 34-provenance-runtime-health-and-coherent-operator-snapshot
plan: "05"
subsystem: package-admission
tags: [provenance, esp-image, partition-table, factory-image, fail-closed-admission]
requires:
  - phase: 34-01-canonical-build-identity
    provides: Canonical manifest identity tuple and packaged application descriptor metadata
provides:
  - Exactly-one admission for every required package artifact kind
  - Bounded ESP application-image and application-descriptor validation
  - Embedded factory partition-table validation and bytewise factory-to-OTA binding
  - Pre-effect rejection of digest-rewritten factory application tampering
affects: [phase-34-gap-closure, phase-35-correlated-evidence]
tech-stack:
  added: []
  patterns: [functional-core-imperative-shell, bounded-binary-parsing, fail-closed-admission, effect-ordering]
key-files:
  created:
    - tools/flash/src/package_admission.rs
  modified:
    - tools/flash/src/main.rs
    - tools/xtask/src/package_manifest.rs
    - tools/parity/src/phase34_source_guard.rs
    - tools/flash/Cargo.toml
    - tools/flash/BUILD.bazel
    - Cargo.lock
    - MODULE.bazel.lock
key-decisions:
  - "Package identity admission rejects ambiguous required artifact kinds before selecting an image or touching ports, credentials, or flash effects."
  - "The selected merged factory image qualifies only when its embedded factory partition contains the exact structurally admitted OTA image bytes."
patterns-established:
  - "Admission order: manifest uniqueness -> artifact digests -> bounded OTA descriptor identity -> embedded partition layout -> exact factory/OTA bytes -> external effects."
requirements-completed: []
generated_by: gsd-execute-plan
lifecycle_mode: interactive
phase_lifecycle_id: 34-2026-07-15T03-26-15
generated_at: 2026-07-16T00:16:02Z
duration: 31min
completed: 2026-07-15
---

# Phase 34 Plan 05: Structural Package Admission Summary

**Factory flashing now admits exactly one required artifact set, parses the OTA application identity and embedded factory partition table within bounded structures, and proves that the selected factory application bytes exactly equal the admitted OTA image before any external effect.**

## Performance

- **Duration:** 31 min
- **Started:** 2026-07-15T23:45:12Z
- **Completed:** 2026-07-16T00:16:02Z
- **Tasks:** 2
- **Implementation commits:** 2
- **Files:** 10

## Accomplishments

- Replaced first-match artifact lookup in both package validation and flash admission with exactly-one validation that distinguishes missing from duplicate OTA and factory artifacts.
- Added a pure bounded parser for ESP application headers, segment ranges, application descriptor magic/version/ELF SHA-256, and source-commit bytes restricted to validated segment payloads.
- Parsed the merged factory image's embedded ESP partition table, required the canonical factory partition layout, and compared only the OTA-length factory application prefix byte-for-byte with the admitted OTA image.
- Added production-path regressions proving artifact ambiguity and digest-rewritten one-byte factory tampering fail before port discovery, credential access, command execution, or flash effects.
- Strengthened Phase 34 source guards to require digest-before-binding order and prohibit the former whole-file marker scans.

## Task Commits

1. **Task 1: Reject ambiguous required package artifacts before image selection** - `c3bd02c2`
2. **Task 2: Structurally bind the selected factory image to the admitted OTA identity** - `f1d6092c`

## Files Created/Modified

- `tools/flash/src/package_admission.rs` - Pure bounded ESP image, descriptor, embedded partition-table, and factory/OTA equality admission with stable redacted categories.
- `tools/flash/src/main.rs` - Exactly-one artifact validation, structural admission before effects, valid structured fixtures, and production ordering/tamper regressions.
- `tools/xtask/src/package_manifest.rs` - Exactly-one required artifact-kind validation with distinct missing and duplicate errors.
- `tools/parity/src/phase34_source_guard.rs` - Structural parser and fail-closed ordering guards that reject whole-file identity scans.
- `tools/flash/Cargo.toml`, `tools/flash/BUILD.bazel`, `Cargo.lock`, and `MODULE.bazel.lock` - Direct use of the already pinned `esp-idf-part` parser and synchronized Cargo/Bazel dependency metadata.

## Decisions Made

- Kept binary parsing inside one pure byte-slice boundary; port discovery, credential loading, process execution, and hardware remain outside and later in the imperative shell.
- Required the canonical Ultra 205 factory layout declared by the embedded table: one app/factory partition at offset `0x10000` with size `0x400000`.
- Compared only the OTA image length inside the factory partition so package padding remains outside identity semantics.
- Preserved full source-commit embedding as a requirement, but restricted its lookup to validated ESP segment payloads while treating descriptor version and ELF SHA-256 as exact fixed-field comparisons.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Synchronized the generated Bazel module lock after adding a direct parser dependency**

- **Found during:** Task 2 Bazel verification.
- **Issue:** `esp-idf-part` was already pinned in the workspace, but making it a direct `bitaxe-flash` dependency changed Cargo member metadata consumed by Bazel's crate-universe extension.
- **Fix:** Retained the automatically regenerated `MODULE.bazel.lock` substitutions for the Cargo hash, flash manifest hash, and direct dependency mapping.
- **Verification:** The lock diff is limited to three generated substitutions, and focused plus repository-wide Bazel tests pass.
- **Committed in:** `f1d6092c`

**Total deviations:** 1 required generated-lock synchronization.
**Impact on plan:** No new dependency version or package format was introduced; the existing pinned partition parser is now directly available to the flash admission boundary.

## Issues Encountered

- The planned red duplicate-artifact regressions initially showed the old first-match behavior accepting ambiguity; they passed after exactly-one validation replaced it.
- The planned digest-rewritten factory tamper regression initially reached later behavior because only the OTA sibling was inspected; it now fails with `factory_ota_image_mismatch` before ports, credentials, or execution.
- Early mandatory Rust runs produced long-lived child test processes after abbreviated tool output. They were allowed to drain, and every subsequent mandatory sequence was run in one tracked session and polled through final exit code 0.
- The canonical ESP32-S3 firmware build retained 14 pre-existing dead-code warnings; this plan introduced no new warning and the required host Clippy gate passed with warnings denied.

## Verification

- Before each task commit, the required Rust sequence passed in exact order: `cargo fmt --all`, all-target/all-feature Clippy with warnings denied, all-target/all-feature build, and all-feature tests.
- Focused Cargo tests passed for xtask package manifests, flash package admission, and the Phase 34 package/hardware admission source guard.
- Focused Bazel tests passed for xtask, flash, parity, and package-firmware behavior.
- Final `bazel test //...` passed all 58 repository test targets and rebuilt the pinned ESP-IDF `v5.5.4` firmware/package graph.
- `just build`, `just package`, `just verify-reference`, and `git diff --check` passed; the reference remained clean at `c1915b0a63bfabebdb95a515cedfee05146c1d50`.
- Full diff review found no hardware, USB, credential, network, UART/pin, OTA execution, Phase 35, archived-lineage, or secret-bearing diagnostic expansion.

## User Setup Required

None. No board detection, hardware access, credentials, network discovery, flash, reset, monitor, direct UART, pin manipulation, OTA, mining, or evidence-promotion command was used.

## Next Phase Readiness

- Plan 34-05's SYS-02 gap is implemented and regression-covered, but SYS-02 remains deliberately unmarked until Plans 34-06 and 34-07 also complete and Phase 34 verification is rerun once.
- Phase 34 verification, requirements, roadmap, and state were not edited by this plan, preserving the ordered gap-closure contract.

## Self-Check: PASSED

- Task commits `c3bd02c2` and `f1d6092c` exist and all focused plus repository-wide software gates are green.
- The Plan 05 summary exists, `requirements-completed` remains empty, and no Phase 34 lifecycle or requirement promotion occurred.
- Only the orchestrator-owned `.planning/STATE.md` modification remains outside this plan's commits, and no push occurred.

***

*Phase: 34-provenance-runtime-health-and-coherent-operator-snapshot*
*Completed: 2026-07-15*
