---
phase: 34-provenance-runtime-health-and-coherent-operator-snapshot
plan: "08"
subsystem: exact-package-admission
tags: [esp32-s3, package-admission, immutable-snapshot, flash, provenance]
requires:
  - phase: 34-05-structural-factory-admission
    provides: Structurally bound factory and OTA application identity
provides:
  - Complete supported ESP32-S3 application-envelope validation
  - Closed unique factory-artifact admission for non-dry-run flashing
  - Admission-owned immutable mode-0600 execution snapshot
affects: [phase-34-gap-closure, phase-35-correlated-evidence]
tech-stack:
  added: []
  patterns: [parse-complete-envelope, typed-admitted-artifact, owner-held-execution-snapshot]
key-files:
  created: []
  modified:
    - tools/flash/src/package_admission.rs
    - tools/flash/src/main.rs
    - tools/parity/src/phase34_source_guard.rs
key-decisions:
  - "Non-dry-run package flashing can construct only one admitted factory kind; filenames and arbitrary manifest artifact kinds never select the effect."
  - "The child receives a private snapshot of the validated bytes, while logs and errors use a bounded placeholder or stable category instead of its local path."
  - "The pure parser accepts only the concrete ESP32-S3 revision/header policy emitted by the pinned package path and consumes checksum, padding, optional digest, and EOF exactly."
patterns-established:
  - "Package admission: read each artifact once -> validate digest and structure -> own admitted factory bytes -> materialize private execution snapshot -> execute."
  - "Effect commands dispatch from a closed admitted kind and never from a basename or reopened package path."
requirements-completed: []
generated_by: gsd-execute-plan
lifecycle_mode: interactive
phase_lifecycle_id: 34-2026-07-15T03-26-15
generated_at: 2026-07-16T03:33:33Z
duration: 32min
completed: 2026-07-15
---

# Phase 34 Plan 08: Exact Immutable Package Admission Summary

**The complete supported ESP32-S3 factory application is now structurally admitted once, retained as owned bytes, and consumed by `espflash` only through a private immutable snapshot.**

## Performance

- **Duration:** 32 min
- **Started:** 2026-07-16T03:01:20Z
- **Completed:** 2026-07-16T03:33:33Z
- **Tasks:** 2
- **Implementation commits:** 2
- **Files:** 3

## Accomplishments

- Extended the pure application parser through the complete ESP32-S3 image envelope: closed chip/revision/header policy, checked segment bounds, seeded XOR checksum, zero alignment padding, declared appended SHA-256, and exact EOF.
- Added distinct stable redacted failures for foreign chips, unsupported headers, checksum corruption, nonzero padding, digest declaration mismatch, digest corruption/truncation, and trailing data.
- Replaced path/basename effect selection with closed `AdmittedFlashImage` and `AdmittedFactoryImage` values that permit only the unique validated `factory_merged_image` in non-dry-run package flows.
- Read each package artifact once during admission and carried the exact validated factory bytes forward without reopening or rehashing the package path.
- Materialized those bytes in an owner-held `NamedTempFile`, flushed and synced it, enforced mode 0600 on Unix, retained it through child return, redacted its displayed path and child errors, and proved cleanup on success and every tested failure.
- Added production-boundary regressions for explicit ELF/extra/alias/factory-like bypasses and a deterministic source-replacement race whose fake child still observes the original admitted bytes.

## Task Commits

1. **Task 1: Validate the complete supported ESP32-S3 application envelope** - `4d9857c3`
2. **Task 2: Carry admitted immutable factory bytes through the flash effect** - `d4ac3331`

## Files Created/Modified

- `tools/flash/src/package_admission.rs` - Complete checked ESP32-S3 envelope parser, stable failure categories, and mutation regressions.
- `tools/flash/src/main.rs` - Closed factory admission, single-read artifact validation, private execution snapshot ownership, redacted execution failures, and adversarial production-path tests.
- `tools/parity/src/phase34_source_guard.rs` - Guards for complete envelope checks, snapshot-before-port ordering, typed command dispatch, owner-held execution, and prohibited path-reopen helpers.

## Decisions Made

- Kept the accepted header policy deliberately narrow at the exact values emitted by the repository's pinned ESP-IDF/esptool package flow: ESP32-S3 chip ID 9, zero legacy/full minimum revision, full maximum revision 99, zero reserved field, and a Boolean hash declaration.
- Kept the existing inert developer-image path available only for manifest-free dry runs. It cannot construct the admitted factory kind or enter non-dry-run flashing.
- Stored the package-facing manifest and factory paths only for display/evidence compatibility; the real child command receives a separate private snapshot path represented as `<admitted-factory-snapshot>` in emitted output.
- Converted artifact validation from validate-then-reread to one read returning validated bytes, making the owned byte vector the sole authority after admission.

## Deviations from Plan

None. The existing `tempfile` and SHA-256 dependencies already covered the implementation, so no Cargo or Bazel dependency-file change was necessary.

## Issues Encountered

- Task 1 RED produced the eight intended distinct parser failures before the complete envelope implementation; the exact mandatory sequence was restarted after an earlier interrupted/buffered invocation and then passed cleanly.
- Task 2 RED isolated all four explicit-artifact bypasses and both immutable-execution gaps before production wiring.
- The first expanded source-guard run also matched a legitimate factory-filename check used only to reject an invalid manifest default. The guard was narrowed to prohibit filename dispatch specifically inside the effect command builder, preserving the required manifest validation.
- The ESP32-S3 firmware build retained 14 pre-existing dead-code warnings; host Clippy passed with warnings denied and this plan introduced no new firmware warning.

## Verification

- The exact pre-commit Rust sequence passed for both task commits, and the final plan-wide rerun passed in order: `cargo fmt --all`, all-target/all-feature Clippy with warnings denied, all-target/all-feature build, and all-feature tests.
- Focused Cargo verification passed 22 package-admission tests, 12 identity-admission tests, 5 admitted-execution tests, and the Phase 34 package/hardware admission source guard.
- Focused Bazel verification passed `//tools/flash:tests` and `//tools/parity:tests`.
- Repository-wide `bazel test //...` passed all 59 test targets; `just build`, `just package`, `just verify-reference`, and `git diff --check` also passed.
- The final source audit found no basename-based effect dispatch, arbitrary artifact selection, package-path reopen after admission, unchecked image-range arithmetic, incomplete trailer acceptance, or private snapshot path in emitted command/error diagnostics.
- No hardware, USB, serial, credentials, network access, OTA execution, direct UART/pins, mining, Phase 35, or archived Phase 28.1.1 operation was used.

## User Setup Required

None.

## Next Phase Readiness

- The SYS-02 production defect is implemented and regression-covered, but the requirement remains deliberately pending under the plan contract.
- Plan 34-09 is the only next implementation step. Fresh Phase 34 review and verification remain required after both Plan 34-08 and Plan 34-09 summaries exist; Phase 35 remains blocked until that authority passes.

## Self-Check: PASSED

- Implementation commits `4d9857c3` and `d4ac3331` exist and contain the two planned task changes.
- All focused and repository-wide software gates pass, the summary has matching lifecycle provenance and exactly two standalone frontmatter delimiters, and `SYS-02` remains pending.
- No push occurred and no Plan 34-09, hardware, credential, network, OTA, direct-UART/pin, Phase 35, or archived-lineage work was performed.

***

*Phase: 34-provenance-runtime-health-and-coherent-operator-snapshot*
*Completed: 2026-07-15*
