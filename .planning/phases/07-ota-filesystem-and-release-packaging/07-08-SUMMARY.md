---
phase: 07-ota-filesystem-and-release-packaging
plan: 08
subsystem: release-operator-evidence-docs
tags: [release-docs, ota, recovery, evidence, ultra-205]
requires:
  - phase: 07-04
    provides: SPIFFS/static/recovery firmware surfaces and Rust-owned assets.
  - phase: 07-05
    provides: Ultra 205 package artifacts and manifest v2.
  - phase: 07-06
    provides: release-gate, license inventory, and provenance manifest.
  - phase: 07-07
    provides: firmware OTA runtime, boot validation adapter, and explicit OTAWWW gap.
provides:
  - Complete Ultra 205 operator guide for package, flash, monitor, OTA, recovery, rollback, erase, failed update, and interruption procedures.
  - Phase 7 evidence rollup separating package, compile, live hardware, compliance, and REL-03 gap conclusions.
  - Manual Ultra 205 OTA/recovery hardware-smoke evidence template for Plan 07-09.
affects: [07-09, release-packaging, ota-recovery, parity-evidence]
tech-stack:
  added: []
  patterns:
    - Evidence documents distinguish package/compile proof from live hardware verification.
    - Operator docs include exact commands and public route/response text for risky update flows.
key-files:
  created:
    - docs/parity/evidence/phase-07-ultra-205-ota-hardware-smoke.md
  modified:
    - docs/release/ultra-205.md
    - docs/parity/evidence/phase-07-ota-filesystem-release.md
key-decisions:
  - "Keep OTAWWW as an explicit REL-03 release gap with the required UI-SPEC copy and public response `Wrong API input`."
  - "Do not elevate live OTA, rollback, recovery, erase, failed update, or interrupted-update behavior above `not run - hardware verification pending` without Ultra 205 evidence."
  - "Use the hardware-smoke document as a capture template only; it does not mark checklist rows verified by existing on disk."
patterns-established:
  - "Release docs should cite exact `just` commands, manifest paths, artifact names, routes, and public responses."
  - "Manual evidence templates should default every hardware-dependent conclusion to pending until filled with command/log observations."
requirements-completed: [REL-03, REL-04, REL-05, REL-07, REL-08]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 7-2026-06-28T13-30-15
generated_at: 2026-06-28T18:01:22Z
duration: 6m28s
completed: 2026-06-28
---

# Phase 07 Plan 08: Release Operator Evidence Docs Summary

**Ultra 205 operator guide, Phase 7 evidence rollup, and hardware-smoke template that keep release/update claims below live hardware proof.**

## Performance

- **Duration:** 6m28s
- **Started:** 2026-06-28T17:54:54Z
- **Completed:** 2026-06-28T18:01:22Z
- **Tasks:** 3
- **Files modified:** 3

## Accomplishments

- Replaced the Ultra 205 release guide with actionable build/package/flash/monitor, firmware OTA, OTAWWW gap, recovery, large erase, failed update, interrupted-update, rollback, and evidence-gate procedures.
- Updated the Phase 7 evidence rollup with separate package, manifest v2, partition/SPIFFS/static/recovery compile, firmware OTA compile, boot validation compile, release-gate/license/provenance, live hardware, and REL-03 gap conclusions.
- Added a manual Ultra 205 OTA/recovery hardware-smoke template with board, port, commit, package, OTA, invalid-image, rollback, static, recovery, OTAWWW, erase, interrupted-update, and final conclusion fields.

## Task Commits

Each task was committed atomically:

1. **Task 1: Complete Ultra 205 release operator guide** - `887a3e4` (docs)
2. **Task 2: Update Phase 7 evidence rollup** - `c705c06` (docs)
3. **Task 3: Add hardware OTA/recovery smoke evidence template** - `6f40964` (docs)

## Files Created/Modified

- `docs/release/ultra-205.md` - Complete operator guide with exact commands, artifact names, OTA/recovery routes, UI-SPEC gap copy, and hardware evidence requirements.
- `docs/parity/evidence/phase-07-ota-filesystem-release.md` - Phase 7 rollup that separates package/compile/compliance evidence from live hardware conclusions.
- `docs/parity/evidence/phase-07-ultra-205-ota-hardware-smoke.md` - Manual evidence capture template for Plan 07-09.

## Decisions Made

- OTAWWW remains a deliberate REL-03 gap for this release candidate and uses the exact required operator copy plus public response `Wrong API input`.
- Package and compile evidence are documented as real evidence, but live firmware OTA, rollback, recovery, large erase, failed update, and interrupted update stay `not run - hardware verification pending`.
- The hardware-smoke document is a template with pending conclusions; it is not evidence sufficient to mark checklist rows verified.

## Verification

- `node /Users/peterryszkiewicz/.codex/get-shit-done/bin/gsd-tools.cjs verify lifecycle 07 --require-plans` - passed before execution.
- Task 1 fixed-string acceptance loop for commands, artifacts, routes, response text, gap copy, and recovery/rollback headings - passed.
- Task 2 fixed-string acceptance loop for manifest v2, partition/SPIFFS/static/recovery, firmware OTA, boot validation, release-gate, REL-03, summary references, and pending hardware status - passed.
- Task 3 fixed-string acceptance loop for Ultra 205 smoke template fields and pending hardware conclusion - passed.
- Before each task commit: `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features` - passed.
- Whole-plan fixed-string acceptance loops for all three task files - passed.
- `just package` - passed and produced `bitaxe-ultra205.elf`, `esp-miner.bin`, `www.bin`, `otadata-initial.bin`, `bitaxe-ultra205-factory.bin`, and `bitaxe-ultra205-package.json`.
- `just parity` - passed with `validation_errors: none`.
- `git diff --check` - passed.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None during implementation. Sync/rebase was intentionally not attempted because the worktree already contained orchestrator-owned planning edits and the execution prompt instructed not to disturb them.

## Known Stubs

None. Stub scan hit only the required UI-SPEC sentence containing "not available"; that sentence is intentional release-gap copy, not placeholder content. The hardware-smoke template defaults hardware-dependent conclusions to `not run - hardware verification pending` by design.

## Threat Flags

None. This plan added documentation and evidence capture surfaces only; it did not introduce new runtime endpoints, auth paths, file access patterns, schema changes, or hardware-control behavior beyond planned route references.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Plan 07-09 can use `docs/parity/evidence/phase-07-ultra-205-ota-hardware-smoke.md` as the manual capture template for connected Ultra 205 OTA/recovery validation. The Phase 7 rollup and operator guide now prevent package-only evidence from being mistaken for live OTA, rollback, recovery, or interrupted-update verification.

## Self-Check: PASSED

- Confirmed summary, operator guide, evidence rollup, and hardware-smoke template files exist.
- Confirmed task commits `887a3e4`, `c705c06`, and `6f40964` exist in git history.

***
*Phase: 07-ota-filesystem-and-release-packaging*
*Completed: 2026-06-28*
