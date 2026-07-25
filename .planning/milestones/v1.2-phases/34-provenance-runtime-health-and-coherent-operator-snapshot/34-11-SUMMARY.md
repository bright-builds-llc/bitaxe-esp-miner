---
phase: 34-provenance-runtime-health-and-coherent-operator-snapshot
plan: "11"
subsystem: firmware-package-admission
tags: [esp32-s3, descriptor-layout, segment-overlap, package-admission, fail-closed]
requires:
  - phase: 34-10-executable-and-elf-identity-admission
    provides: Typed ESP32-S3 image parsing and immutable OTA/factory identity admission
provides:
  - Typed non-empty DROM segment-0 descriptor invariant
  - Direct destination and ESP32-S3 D/IRAM alias disjointness
  - OTA, factory, and parsed CLI pre-effect regression proof for both layout invariants
affects: [phase-34-review, phase-34-verification, phase-35-correlated-evidence]
tech-stack:
  added: []
  patterns: [validated-segment-layout, half-open-range-disjointness, alias-normalization, pre-effect-sentinels]
key-files:
  created: []
  modified:
    - tools/flash/src/esp32s3_image.rs
    - tools/flash/src/package_admission.rs
    - tools/flash/src/main.rs
    - tools/parity/src/phase34_source_guard.rs
key-decisions:
  - "Only a non-empty DROM segment 0 can become the descriptor-bearing validated layout, and descriptor fields remain anchored at payload offset zero."
  - "Direct destination intersections are classified before D/IRAM physical aliases, with IRAM normalized by pinned SOC_I_D_OFFSET 0x006f0000."
  - "Zero-length segments remain range-free and exact numeric or physical adjacency remains valid."
patterns-established:
  - "Typed layout construction: descriptor segment -> checked non-empty ranges -> direct disjointness -> D/IRAM alias disjointness -> entry and descriptor consumers."
  - "Boundary proof: independently resealed malformed OTA and factory images -> exact category-only error -> zero parsed external-effect sentinels."
requirements-completed: []
generated_by: gsd-execute-plan
lifecycle_mode: interactive
phase_lifecycle_id: 34-2026-07-15T03-26-15
generated_at: 2026-07-17T13:34:00Z
duration: 32m
completed: 2026-07-17
---

# Phase 34 Plan 11: Descriptor Placement and Segment Disjointness Summary

**ESP32-S3 package admission now constructs an image only from a segment-0 DROM descriptor and non-overlapping direct and D/IRAM destination ranges, before any parsed flash effect.**

## Performance

- **Duration:** 32m
- **Started:** 2026-07-17T13:02:14Z
- **Completed:** 2026-07-17T13:34:00Z
- **Tasks:** 2
- **Implementation commits:** 2
- **Files:** 4

## Accomplishments

- Replaced the copyable image count/entry summary with an owned `ValidatedSegmentLayout` containing parsed segments and their already-checked non-empty destination ranges.
- Required application segment 0 to be non-empty, descriptor-sized DROM while keeping descriptor magic and fields anchored at payload offset zero.
- Added fixed typed categories for empty/non-DROM descriptor segments, direct destination overlap, and ESP32-S3 D/IRAM alias overlap without exposing addresses or bytes.
- Rejected partial, containment, and identical direct intersections before alias analysis; normalized IRAM through pinned `SOC_I_D_OFFSET = 0x006f0000`.
- Preserved exact direct and alias adjacency, range-free zero-length segments, mapped zero-length congruence, and all Plan 34-10 header, range, entry, descriptor, checksum, and digest policy.
- Proved each new rejection through independently resealed OTA and factory images, including factory-only failures paired with same-length valid OTA controls.
- Exercised all three malformed layout classes through real parsed dry-run and non-dry flash handlers and proved zero port listing, credential/NVS, snapshot, command, execution, and observed-flash effects.
- Strengthened Phase 34 source guards around constructor ordering, validated-layout consumers, package coverage, and parsed pre-effect sentinels.

## Task Commits

1. **Task 1: Make descriptor placement and destination disjointness typed parser invariants** - `d4b675cc`
2. **Task 2: Prove both layout invariants through package and parsed pre-effect boundaries** - `550e850c`

## Files Created/Modified

- `tools/flash/src/esp32s3_image.rs` - Typed layout constructor, descriptor-segment gate, half-open direct overlap checks, D/IRAM alias normalization, stable errors, and focused parser regressions.
- `tools/flash/src/package_admission.rs` - Fully resealed OTA/factory rejection fixtures plus adjacency and zero-length package positives.
- `tools/flash/src/main.rs` - Parsed dry/non-dry layout regressions and focused port, credential, NVS, snapshot, command, and flash effect recorders.
- `tools/parity/src/phase34_source_guard.rs` - Constructor ordering, validated consumer, package boundary, and parsed pre-effect source guards.

## Decisions Made

- Kept the parser dependency-free and bounded the pairwise checks by the existing maximum of 16 segments.
- Classified direct numeric overlap before D/IRAM alias overlap so malformed layouts receive one deterministic category.
- Used checked subtraction for both endpoints of every IRAM interval before comparing it in DRAM coordinates.
- Omitted zero-length segments from destination ranges while preserving the existing DROM/IROM mapped-congruence rule.
- Kept `SYS-02` and Phase 34 pending because this plan supplies implementation and regression evidence only; fresh code review and independent verification retain completion authority.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Preserved the established truncated-descriptor category inside the new typed layout**

- **Found during:** Task 1 focused Bazel verification
- **Issue:** The first constructor ordering caused a short non-DROM segment 0 to report the new family category instead of the established `app_descriptor_truncated` category.
- **Fix:** Made descriptor-sized payload validation part of the descriptor-segment gate before family validation, so the validated layout cannot contain an undersized descriptor segment and prior error meaning remains stable.
- **Files modified:** `tools/flash/src/esp32s3_image.rs`
- **Commit:** `d4b675cc`

**2. [Rule 1 - Tooling] Corrected mechanical phase completion after plan accounting**

- **Found during:** Final state and roadmap accounting
- **Issue:** The generic roadmap progress helper marked Phase 34 complete solely because all 11 plans had summaries, despite the plan's explicit requirement that SYS-02 and Phase 34 remain pending fresh review and independent verification.
- **Fix:** Preserved Plan 34-11 as implemented and accounted while restoring the Phase 34 checkbox, state position, phase count, verification text, and SYS-02 truth to their pending-authority disposition.
- **Files modified:** `.planning/STATE.md`, `.planning/ROADMAP.md`
- **Commit:** Final metadata commit

## Issues Encountered

- The first Task 1 focused Bazel run exposed the descriptor-truncation ordering regression above; the full focused suite passed after the typed constructor was tightened.
- The first mandatory Task 1 Clippy run required the eager `bool::then_some` form. The exact four-command sequence was restarted from formatting and then passed.
- Long initial Cargo, Bazel, and firmware package commands exceeded their first output yield; each process was allowed to complete and its successful terminal result was captured.

## Verification

- The exact mandatory Rust sequence passed before both task commits and again plan-wide: `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features`.
- Focused Cargo suites passed for `esp32s3_image`, package admission, executable admission, identity admission, and Phase 34 source guards.
- Focused Bazel tests passed for `//tools/flash:tests` and `//tools/parity:tests`.
- Repository-wide `bazel test //...` passed all 60 test targets; `just build`, `just package`, `just verify-reference`, and `git diff --check` passed.
- Fresh manifest inspection proved schema version 3, exactly one firmware ELF, OTA image, and factory merged image, and `firmware_elf.sha256 == app_elf_sha256`.
- The exact software-only `bazel run //tools/flash -- flash --board 205 --port /dev/null --manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --dry-run` admission passed without executing the rendered command.
- The lifecycle audit resolved `34-2026-07-15T03-26-15` and reported the prior verification stale relative to Plan 34-11, the expected condition requiring fresh independent verification rather than phase promotion.
- No hardware, USB, serial device, credential, Wi-Fi/network discovery, flash/OTA execution, direct UART/pin work, Phase 35 work, archived-lineage phase operation, dependency, public API/schema, reference-tree, or generated `.embuild` change occurred.

## User Setup Required

None.

## Next Phase Readiness

- All eleven Phase 34 implementation plans now have summaries and the complete software regression surface passes.
- `SYS-02` remains deliberately pending. Fresh Phase 34 code review and independent goal verification are the only authorized next actions; Phase 35 remains blocked until those authorities pass.

## Self-Check: PASSED

- Implementation commits `d4b675cc` and `550e850c` exist and contain the two planned task changes.
- All four modified source files and this summary exist.
- The summary carries lifecycle `34-2026-07-15T03-26-15` and exactly two standalone frontmatter delimiters.
- All focused and repository-wide software gates pass, `SYS-02` and Phase 34 remain pending, and no push or prohibited external effect occurred.

***

*Phase: 34-provenance-runtime-health-and-coherent-operator-snapshot*
*Completed: 2026-07-17*
