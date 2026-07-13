---
phase: 13-final-ultra-205-release-evidence
plan: "04"
subsystem: release-evidence
tags: [ultra-205, firmware-ota, ota-smoke, boot-validation, device-url, redaction]
requires:
  - phase: 13-final-ultra-205-release-evidence
    provides: Plan 13-05 recovery runbook, bounded monitor helper, and recovery pending evidence
provides:
  - Repo-owned firmware OTA smoke helper with explicit DEVICE_URL gating
  - Bazel shell tests for missing URL, invalid rejection, valid OTA, and post-OTA marker requirements
  - Blocked firmware OTA evidence with matching Markdown and generated smoke-log status
  - Redaction review coverage for firmware OTA blocker artifacts
affects: [phase-13, release-evidence, firmware-ota, rollback, boot-validation, parity-checklist]
tech-stack:
  added: []
  patterns:
    - Explicit DEVICE_URL-only OTA helpers with no target discovery path
    - Invalid image rejection recorded separately from valid OTA and never treated as rollback proof
    - Valid OTA requires bounded post-reboot monitor markers before passed evidence
key-files:
  created:
    - scripts/phase13-firmware-ota-smoke.sh
    - scripts/phase13-firmware-ota-smoke-test.sh
    - docs/parity/evidence/phase-13-final-ultra-205-release-evidence/firmware-ota.md
    - docs/parity/evidence/phase-13-final-ultra-205-release-evidence/firmware-ota/firmware-ota-smoke.log
    - docs/parity/evidence/phase-13-final-ultra-205-release-evidence/firmware-ota/post-ota-monitor.log
  modified:
    - scripts/BUILD.bazel
    - docs/parity/evidence/phase-13-final-ultra-205-release-evidence/redaction-review.md
key-decisions:
  - "Firmware OTA helpers require an explicit DEVICE_URL and never scan or infer a target."
  - "Without DEVICE_URL, Plan 13-04 records blocked OTA evidence and does not upload esp-miner.bin."
  - "Invalid image rejection is separate failed-update evidence and is not rollback proof."
patterns-established:
  - "Firmware OTA passed evidence requires firmware_commit=, reference_commit=, and ota_boot_validation= markers after valid upload."
  - "Blocked OTA evidence must use the same firmware_ota_status line in Markdown and generated logs."
requirements-completed: [REL-02, REL-08, EVD-05]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 13-2026-06-30T14-53-46
generated_at: 2026-06-30T17:29:09Z
duration: 10 min
completed: 2026-06-30
---

# Phase 13 Plan 04: Firmware OTA Smoke Evidence Summary

**Firmware OTA smoke helper with manifest-backed upload guards and blocked evidence because DEVICE_URL is unavailable**

## Performance

- **Duration:** 10 min
- **Started:** 2026-06-30T17:19:03Z
- **Completed:** 2026-06-30T17:29:09Z
- **Tasks:** 2
- **Files modified:** 7

## Accomplishments

- Added `scripts/phase13-firmware-ota-smoke.sh`, which validates manifest `esp-miner.bin`, records invalid image rejection separately from valid OTA, and uses `phase13-monitor-capture.sh` for post-reboot evidence.
- Added `scripts/phase13-firmware-ota-smoke-test.sh` and Bazel targets covering missing `DEVICE_URL`, fake invalid rejection, fake valid OTA success, and missing post-OTA marker failure.
- Recorded blocked firmware OTA evidence because no real `DEVICE_URL` is available, with matching `firmware_ota_status: blocked - DEVICE_URL unavailable` in `firmware-ota.md`, `firmware-ota-smoke.log`, and `post-ota-monitor.log`.
- Updated Phase 13 redaction review to cover OTA request/response scope and the generated blocked OTA artifacts.

## Task Commits

Each task was committed atomically:

1. **Task 1: Implement firmware OTA smoke helper** - `4e7bd62` (feat)
2. **Task 2: Run firmware OTA smoke or record blocker** - `51023d3` (docs)

**Plan metadata:** committed separately after SUMMARY, STATE, ROADMAP, and REQUIREMENTS updates.

## Files Created/Modified

- `scripts/phase13-firmware-ota-smoke.sh` - Explicit firmware OTA smoke helper with manifest checksum validation, invalid upload logging, valid OTA upload, and post-OTA marker enforcement.
- `scripts/phase13-firmware-ota-smoke-test.sh` - Fake-curl and fake-monitor tests for blocked, invalid, valid, and missing-marker helper behavior.
- `scripts/BUILD.bazel` - Registers `phase13_firmware_ota_smoke` and `phase13_firmware_ota_smoke_test`.
- `docs/parity/evidence/phase-13-final-ultra-205-release-evidence/firmware-ota.md` - Blocked firmware OTA evidence summary.
- `docs/parity/evidence/phase-13-final-ultra-205-release-evidence/firmware-ota/firmware-ota-smoke.log` - Helper-generated missing `DEVICE_URL` blocker log.
- `docs/parity/evidence/phase-13-final-ultra-205-release-evidence/firmware-ota/post-ota-monitor.log` - Blocked post-OTA monitor placeholder.
- `docs/parity/evidence/phase-13-final-ultra-205-release-evidence/redaction-review.md` - Redaction review extended to Plan 13-04 OTA artifacts.

## Decisions Made

- Live valid OTA was skipped because `DEVICE_URL` is not present and Plan 13-03 already recorded that network evidence is blocked.
- The helper does not scan the network or infer device URL from serial output; it only uses an explicit `--device-url` or `DEVICE_URL`.
- Blocked OTA evidence keeps OTA, rollback, selected partition, and boot-validation claims below verified.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

- `DEVICE_URL` was not set. This is an evidence gate, not a task failure: the helper wrote `firmware_ota_status: blocked - DEVICE_URL unavailable`, no curl or valid OTA upload ran, and no destructive or fault-injection action ran.
- `mdformat --check` found wrapping drift in `firmware-ota.md`; `mdformat` normalized the Markdown and the check passed afterward.

## Verification

- Lifecycle validation before execution: `verify lifecycle 13 --expect-id 13-2026-06-30T14-53-46 --expect-mode yolo --require-plans --raw`: passed.
- Sync before implementation: `git fetch origin` and `git pull --rebase`: branch already up to date.
- Task 1 checks: `bash -n scripts/phase13-firmware-ota-smoke.sh`, `bash -n scripts/phase13-firmware-ota-smoke-test.sh`, `bazel test //scripts:phase13_firmware_ota_smoke_test`, `shfmt -l -d`, and acceptance grep checks: passed.
- Task 2 checks: helper blocked run, required evidence files present, matching `firmware_ota_status: blocked - DEVICE_URL unavailable`, plan grep verification, `mdformat --check`, and redaction grep checks: passed.
- Plan-level verification: `bash -n scripts/phase13-monitor-capture.sh && bash -n scripts/phase13-firmware-ota-smoke.sh`, `bazel test //scripts:phase13_monitor_capture_test //scripts:phase13_firmware_ota_smoke_test`, `just parity`, and `git diff -- reference/esp-miner --exit-code`: passed.
- Rust pre-commit sequence before each task commit: `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features`: passed.

## Known Stubs

None - stub scan found only shell variable initializers inside the helper and fake-command test fixtures (`port=""`, `header_file=""`, `body_file=""`, `data_path=""`, `url=""`, `out=""`). These are not UI-rendered placeholder data or incomplete product behavior.

## Threat Flags

None - the new OTA upload helper covers the plan's expected trust boundary: explicit `DEVICE_URL` to `/api/system/OTA`, manifest `esp-miner.bin` checksum validation, invalid/valid request logging, post-reboot monitor evidence, and redaction review. No unplanned endpoint, auth path, schema change, reference-tree edit, or runtime firmware trust boundary was introduced.

## User Setup Required

None for local tooling. A reachable `DEVICE_URL` for the just-flashed Ultra 205 remains required before live firmware OTA, invalid rejection, reboot identity, rollback, selected partition, and boot-validation evidence can pass.

## Next Phase Readiness

Ready for Plan 13-06. Firmware OTA helper coverage and blocked evidence are in place, but checklist/release docs must keep firmware OTA and rollback-sensitive rows below verified unless a real `DEVICE_URL` is provided and a valid OTA run captures post-reboot identity and `ota_boot_validation=` markers.

## Self-Check: PASSED

- Created files exist: `phase13-firmware-ota-smoke.sh`, `phase13-firmware-ota-smoke-test.sh`, `firmware-ota.md`, `firmware-ota/firmware-ota-smoke.log`, `firmware-ota/post-ota-monitor.log`, and `13-04-SUMMARY.md`.
- Task commits exist: `4e7bd62` and `51023d3`.
- SUMMARY frontmatter delimiter check passed with exactly two standalone `---` delimiters.

*Phase: 13-final-ultra-205-release-evidence*
*Completed: 2026-06-30*
