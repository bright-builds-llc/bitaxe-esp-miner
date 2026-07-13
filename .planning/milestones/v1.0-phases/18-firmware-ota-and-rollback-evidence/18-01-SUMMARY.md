---
phase: 18-firmware-ota-and-rollback-evidence
plan: "01"
subsystem: firmware-ota-evidence
tags: [phase-18, firmware-ota, target-lock, rollback, boot-validation, redaction]
requires:
  - phase: 13-final-ultra-205-release-evidence
    provides: Firmware OTA smoke helper with invalid-image and post-OTA marker tests
  - phase: 17-live-http-api-and-static-evidence
    provides: Trusted USB flash-monitor target provenance and redacted target-lock pattern
provides:
  - Phase 18 firmware OTA evidence wrapper with explicit target provenance
  - Fake-backed shell regression tests for target validation, trusted flash evidence, invalid rejection, valid OTA, and missing boot-validation markers
  - Phase 18 evidence artifact contract
  - Pending Phase 18 redaction review gate
affects: [phase-18, firmware-ota, release-evidence, rollback, boot-validation, redaction]
tech-stack:
  added: []
  patterns:
    - Thin Phase 18 wrapper around the existing Phase 13 OTA smoke helper
    - Origin-only DEVICE_URL validation with trusted board 205 flash-monitor extraction
    - Separate evidence fields for invalid rejection, valid OTA, boot validation, rollback, and non-claims
key-files:
  created:
    - scripts/phase18-firmware-ota-evidence.sh
    - scripts/phase18-firmware-ota-evidence-test.sh
    - docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/evidence-contract.md
    - docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/redaction-review.md
  modified:
    - scripts/BUILD.bazel
key-decisions:
  - "Phase 18 delegates OTA upload and post-OTA marker enforcement to the existing Phase 13 helper instead of duplicating OTA behavior."
  - "The wrapper accepts only an explicit origin-only DEVICE_URL or trusted board 205 flash-monitor evidence and records network_scan: disabled."
  - "Invalid image rejection, valid OTA, boot validation, rollback, and non-claims are recorded as distinct evidence classes."
  - "Phase 18 redaction starts pending until live target, OTA, serial, and recovery artifacts exist."
patterns-established:
  - "Phase 18 target-lock creation can run independently with --target-lock-only."
  - "Trusted flash evidence target extraction requires command_kind flash-monitor, board 205, trusted_output true, and exactly one origin-only device_url marker."
requirements-completed: [REL-02, REL-08, REL-07, EVD-05]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 18-2026-07-03T14-06-29
generated_at: 2026-07-03T15:08:17Z
duration: 11m46s
completed: 2026-07-03
---

# Phase 18 Plan 01: Firmware OTA Evidence Wrapper Summary

**Phase-owned firmware OTA evidence wrapper with explicit target provenance, fake-backed regression coverage, and a pending artifact/redaction contract**

## Performance

- **Duration:** 11m46s
- **Started:** 2026-07-03T14:56:31Z
- **Completed:** 2026-07-03T15:08:17Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments

- Added `scripts/phase18-firmware-ota-evidence.sh`, a Phase 18 wrapper that validates direct origin-only targets or trusted board `205` flash-monitor evidence, writes `target-lock.json`, and delegates OTA behavior to `scripts/phase13-firmware-ota-smoke.sh`.
- Added `scripts/phase18-firmware-ota-evidence-test.sh` and Bazel wiring for fake-backed tests covering missing target, invalid direct target, trusted/untrusted flash evidence, invalid rejection non-rollback wording, valid OTA markers, and missing boot-validation failure.
- Created the Phase 18 evidence contract and pending redaction review under `docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/`.

## Task Commits

Each task was committed atomically:

1. **Task 1: Add Phase 18 OTA evidence wrapper and tests** - `238a596` (feat)
2. **Task 2: Create Phase 18 evidence contract and pending redaction review** - `34ee19a` (docs)

**Plan metadata:** pending final metadata commit after SUMMARY and state updates.

## Files Created/Modified

- `scripts/phase18-firmware-ota-evidence.sh` - Phase 18 wrapper for explicit target validation, redacted target-lock writing, and Phase 13 OTA helper delegation.
- `scripts/phase18-firmware-ota-evidence-test.sh` - Fake-backed shell regression coverage for target provenance, OTA claim boundaries, and boot-validation marker requirements.
- `scripts/BUILD.bazel` - Registers `phase18_firmware_ota_evidence` and `phase18_firmware_ota_evidence_test`.
- `docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/evidence-contract.md` - Lists Phase 18 artifact classes, required fields, and separate claim classes.
- `docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/redaction-review.md` - Starts the Phase 18 redaction gate as `redaction_status: pending`.

## Decisions Made

- Reused the Phase 13 firmware OTA smoke helper for upload, invalid rejection, response capture, and post-OTA marker enforcement so Phase 18 remains evidence orchestration, not a second OTA implementation.
- Kept target provenance fail-closed: direct targets must be origin-only, and USB-derived targets require trusted flash-monitor JSON for board `205` with exactly one raw `device_url` marker.
- Kept invalid rejection, valid OTA, boot validation, rollback, and non-claims as separate log and contract fields so invalid image rejection cannot become rollback proof.
- Left redaction review pending because no live Phase 18 target, OTA, serial, recovery, or response artifacts exist yet.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

- Initial wrapper editing introduced a duplicate shell `then` around the flash-evidence heredoc. `bash -n` caught it before commit, and the syntax check passed after repair.
- The test harness initially assumed the `target/phase18-firmware-ota-and-rollback-evidence-dev-raw` scratch parent existed. The test now creates that allowed raw-evidence parent before `mktemp`.
- `state advance-plan` did not infer the Phase 18 pointer because `STATE.md` still pointed at the last Phase 17 plan. After GSD progress/metrics/decision updates, `STATE.md` was narrowly corrected to Phase 18 Plan 2 of 4.

## Verification

- `bash -n scripts/phase18-firmware-ota-evidence.sh scripts/phase18-firmware-ota-evidence-test.sh`: passed.
- `scripts/phase18-firmware-ota-evidence-test.sh`: passed.
- `bazel test //scripts:phase18_firmware_ota_evidence_test //scripts:phase13_firmware_ota_smoke_test`: passed.
- Plan script rg checks for `phase18_firmware_ota_evidence`, `target-lock-only`, `device-url-from-flash-evidence`, `network_scan`, invalid rejection non-rollback wording, `ota_boot_validation`, and Phase 13 helper provenance: passed.
- Plan docs rg checks for `network_scan: disabled`, valid OTA, invalid rejection, boot validation, rollback, non-claim, required artifact names, `redaction_status: pending`, and secret categories: passed.
- Standalone body `---` separator check for Phase 18 evidence Markdown: passed.
- `git diff --check -- scripts/phase18-firmware-ota-evidence.sh scripts/phase18-firmware-ota-evidence-test.sh scripts/BUILD.bazel docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence`: passed.
- Rust pre-commit sequence before task commits: `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features`: passed.

## Known Stubs

None. Stub scan found only shell variable initializers and fake-command test fixture variables such as `port=""`, `target_lock_out=""`, `header_file=""`, and `url=""`; these are not UI-rendered placeholders or incomplete product behavior.

## Threat Flags

None. The new target-resolution and evidence-writing surface is covered by the plan threat model: explicit origin-only target validation, trusted board `205` flash evidence checks, `network_scan: disabled`, separate invalid/valid/boot-validation/rollback evidence fields, and pending redaction review. No new firmware endpoint, auth path, schema, or reference implementation change was introduced.

## User Setup Required

None for Plan 18-01. Later live Phase 18 OTA evidence still requires the planned package, detector, flash-monitor, explicit target, and redaction gates.

## Next Phase Readiness

Ready for Plan 18-02. The phase now has a wrapper and artifact contract for package/serial/target evidence capture without mutating historical Phase 13 or Phase 17 evidence paths.

## Self-Check: PASSED

- Created files exist: `phase18-firmware-ota-evidence.sh`, `phase18-firmware-ota-evidence-test.sh`, Phase 18 evidence contract, pending redaction review, and `18-01-SUMMARY.md`.
- Task commits exist: `238a596` and `34ee19a`.
- SUMMARY frontmatter delimiter check passed with exactly two standalone `---` delimiters.

*Phase: 18-firmware-ota-and-rollback-evidence*
*Completed: 2026-07-03*
