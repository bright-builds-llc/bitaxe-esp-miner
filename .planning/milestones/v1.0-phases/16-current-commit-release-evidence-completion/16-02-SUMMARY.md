---
phase: 16-current-commit-release-evidence-completion
plan: "02"
subsystem: release-evidence
tags:
  - ultra205
  - package
  - release-gate
  - serial
  - hardware-smoke
  - parity
requires:
  - phase: 16-01
    provides: Phase 16 release evidence validator and gate prerequisites
provides:
  - Current release-candidate Ultra 205 package manifest evidence
  - Release-gate transcript for the copied package manifest
  - Detector-gated Ultra 205 serial boot evidence
  - Redaction review for Plan 16-02 artifacts
affects:
  - Phase 16 live HTTP evidence
  - Phase 16 OTA evidence
  - Phase 16 recovery evidence
  - release checklist
tech-stack:
  added: []
  patterns:
    - Wrapper-owned flash-monitor evidence validated against the package manifest
    - Workspace-relative release evidence CLI paths normalized before validation
key-files:
  created:
    - docs/parity/evidence/phase-16-current-commit-release-evidence-completion/package-release-gate.md
    - docs/parity/evidence/phase-16-current-commit-release-evidence-completion/package-release-gate/package-command.log
    - docs/parity/evidence/phase-16-current-commit-release-evidence-completion/package-release-gate/release-gate.log
    - docs/parity/evidence/phase-16-current-commit-release-evidence-completion/package-release-gate/bitaxe-ultra205-package.json
    - docs/parity/evidence/phase-16-current-commit-release-evidence-completion/serial-boot.md
    - docs/parity/evidence/phase-16-current-commit-release-evidence-completion/serial-boot/detect-ultra205.log
    - docs/parity/evidence/phase-16-current-commit-release-evidence-completion/serial-boot/flash-command-evidence.json
    - docs/parity/evidence/phase-16-current-commit-release-evidence-completion/serial-boot/flash-monitor.log
  modified:
    - docs/parity/evidence/phase-16-current-commit-release-evidence-completion/redaction-review.md
    - tools/parity/src/main.rs
key-decisions:
  - "Plan 16-02 records the package-to-hardware identity chain for release-candidate source commit b55d3e68b68060fc6cf271372a75fc86c0a934c6; later evidence and metadata commits advance repository HEAD without changing the flashed firmware evidence."
  - "Detector-gated serial evidence uses only the `just flash-monitor` wrapper output as trusted proof; raw monitor output is not cited."
patterns-established:
  - "Release evidence validation accepts workspace-relative CLI paths and normalizes them before containment checks."
  - "Phase evidence summaries keep later live HTTP, OTA, recovery, and destructive-test claims explicitly out of scope."
requirements-completed:
  - FND-06
  - REL-01
  - REL-04
  - REL-07
  - EVD-05
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 16-2026-07-01T12-36-46
generated_at: 2026-07-01T14:16:32Z
duration: 18min
completed: 2026-07-01
---

# Phase 16 Plan 02: Current Commit Release Evidence Summary

**Ultra 205 package, release-gate, detector, and trusted serial boot evidence tied to release-candidate source commit `b55d3e68b68060fc6cf271372a75fc86c0a934c6`.**

## Performance

- **Duration:** 18min
- **Started:** 2026-07-01T13:58:41Z
- **Completed:** 2026-07-01T14:16:32Z
- **Tasks:** 2
- **Files modified:** 10

## Accomplishments

- Captured `just package` and release-gate logs under `docs/parity/evidence/phase-16-current-commit-release-evidence-completion/package-release-gate/`.
- Copied only the generated Ultra 205 package manifest into evidence; generated binaries stayed out of docs.
- Ran `just detect-ultra205`; it selected exactly one ESP32-S3 port, `/dev/cu.usbmodem1101`, and board-info succeeded before flashing.
- Captured wrapper-owned `just flash-monitor` serial boot evidence with trusted output, safe-state markers, route registration, SPIFFS availability, reset reason, and firmware/reference commit identity.
- Ran the release-evidence validator for the serial JSON and recorded `release_evidence_status: passed`.
- Updated `redaction-review.md` only for Plan 16-02 artifacts while leaving phase-level redaction pending for later live evidence plans.

## Task Commits

Each task was committed atomically:

1. **Task 1: Package current commit and record release-gate evidence** - `b55d3e6` (docs)
2. **Task 2: Capture detector-gated flash-monitor serial boot evidence** - `76eca67` (fix)

## Files Created/Modified

- `docs/parity/evidence/phase-16-current-commit-release-evidence-completion/package-release-gate.md` - Package and release-gate conclusion for source commit `b55d3e68b68060fc6cf271372a75fc86c0a934c6`.
- `docs/parity/evidence/phase-16-current-commit-release-evidence-completion/package-release-gate/package-command.log` - Captured `just package` transcript.
- `docs/parity/evidence/phase-16-current-commit-release-evidence-completion/package-release-gate/release-gate.log` - Captured release-gate transcript.
- `docs/parity/evidence/phase-16-current-commit-release-evidence-completion/package-release-gate/bitaxe-ultra205-package.json` - Committed package manifest copy with source/reference commits and artifact SHA-256 values.
- `docs/parity/evidence/phase-16-current-commit-release-evidence-completion/serial-boot.md` - Detector, flash-monitor, serial boot, validator, and redaction conclusions.
- `docs/parity/evidence/phase-16-current-commit-release-evidence-completion/serial-boot/detect-ultra205.log` - Detector transcript proving one selected ESP32-S3 port before flash.
- `docs/parity/evidence/phase-16-current-commit-release-evidence-completion/serial-boot/flash-command-evidence.json` - Wrapper-owned flash-monitor evidence JSON with `trusted_output: true`.
- `docs/parity/evidence/phase-16-current-commit-release-evidence-completion/serial-boot/flash-monitor.log` - Serial boot log with safe-state and identity markers.
- `docs/parity/evidence/phase-16-current-commit-release-evidence-completion/redaction-review.md` - Plan 16-02 artifact redaction review.
- `tools/parity/src/main.rs` - Release-evidence CLI path normalization and regression coverage.

## Decisions Made

- The same-commit release evidence chain is anchored to release-candidate source commit `b55d3e68b68060fc6cf271372a75fc86c0a934c6`; later task and metadata commits are evidence bookkeeping, not reflashed firmware proof.
- The detector transcript and wrapper JSON are the only trusted hardware evidence sources for this plan.
- Live HTTP, static, recovery, OTA, rollback, failed-update, interrupted-update, and large-erase claims remain out of scope until later Phase 16 plans generate their own evidence.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Refreshed package evidence after the first task commit advanced HEAD**

- **Found during:** Task 2 (Capture detector-gated flash-monitor serial boot evidence)
- **Issue:** Task 1 committed package evidence for source commit `025d371f405d6d60774f26f333eb50c8187e72a6`, then the flash-monitor wrapper rebuilt and flashed commit `b55d3e68b68060fc6cf271372a75fc86c0a934c6`. Release evidence requires package and flash commits to match.
- **Fix:** Reran `just package`, reran release-gate, recopied the generated manifest, and updated `package-release-gate.md` so the manifest, release gate, and serial JSON share source commit `b55d3e68b68060fc6cf271372a75fc86c0a934c6`.
- **Files modified:** `docs/parity/evidence/phase-16-current-commit-release-evidence-completion/package-release-gate.md`, `docs/parity/evidence/phase-16-current-commit-release-evidence-completion/package-release-gate/package-command.log`, `docs/parity/evidence/phase-16-current-commit-release-evidence-completion/package-release-gate/release-gate.log`, `docs/parity/evidence/phase-16-current-commit-release-evidence-completion/package-release-gate/bitaxe-ultra205-package.json`
- **Verification:** Manifest source/reference commits match flash evidence JSON, and release-evidence validation passed.
- **Committed in:** `76eca67`

**2. [Rule 3 - Blocking] Normalized release-evidence validator input paths**

- **Found during:** Task 2 (Capture detector-gated flash-monitor serial boot evidence)
- **Issue:** The exact plan command for `release-evidence` failed when passed relative `--evidence-root` and `--flash-evidence-json` paths because wrapper JSON contains absolute paths and the containment check compared mixed path forms.
- **Fix:** Workspace-resolved `evidence_root` and `flash_evidence_json_path` in `tools/parity/src/main.rs`, and added regression coverage for relative CLI inputs.
- **Files modified:** `tools/parity/src/main.rs`
- **Verification:** `cargo test -p bitaxe-parity --all-features release_evidence` passed, then the exact plan `bazel run //tools/parity:report -- release-evidence ...` command passed.
- **Committed in:** `76eca67`

**3. [Rule 3 - Blocking] Restored writable normal mode for the copied manifest**

- **Found during:** Task 2 (Capture detector-gated flash-monitor serial boot evidence)
- **Issue:** The evidence manifest inherited a generated executable/read-only file mode, which blocked overwrite during the same-commit package refresh and would have committed the JSON as executable.
- **Fix:** Made the file writable before recopying, then normalized the committed manifest to mode `100644`.
- **Files modified:** `docs/parity/evidence/phase-16-current-commit-release-evidence-completion/package-release-gate/bitaxe-ultra205-package.json`
- **Verification:** Fresh manifest copy succeeded and `git show --stat --oneline --name-only 76eca67` includes the normalized file in the task commit.
- **Committed in:** `76eca67`

**Total deviations:** 3 auto-fixed (3 Rule 3 blocking fixes)
**Impact on plan:** All fixes preserved the plan objective and were required to produce trustworthy same-commit release evidence.

## Issues Encountered

- Release-evidence validation is intentionally current-HEAD-sensitive. It passed for source commit `b55d3e68b68060fc6cf271372a75fc86c0a934c6` before the evidence task commit advanced repository HEAD to `76eca67`; this summary keeps the release-candidate evidence chain explicit rather than implying the later docs/tooling commit was reflashed.
- The hardware detector passed, so no blocked serial evidence path was needed.

## Verification

- `just package` - passed for the release-candidate package evidence and passed again after Task 2 at HEAD `76eca67`.
- `bazel run //tools/parity:report -- release-gate --manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json` - passed.
- `just detect-ultra205` - passed; exactly one port, `/dev/cu.usbmodem1101`, with ESP32-S3 board-info success.
- `just flash-monitor board=205 port=/dev/cu.usbmodem1101 manifest=bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json evidence-dir=docs/parity/evidence/phase-16-current-commit-release-evidence-completion/serial-boot capture-timeout-seconds=35` - passed with trusted wrapper output and captured serial log.
- `bazel run //tools/parity:report -- release-evidence --manifest docs/parity/evidence/phase-16-current-commit-release-evidence-completion/package-release-gate/bitaxe-ultra205-package.json --evidence-root docs/parity/evidence/phase-16-current-commit-release-evidence-completion --flash-evidence-json docs/parity/evidence/phase-16-current-commit-release-evidence-completion/serial-boot/flash-command-evidence.json` - passed for the copied manifest and serial JSON.
- `cargo test -p bitaxe-parity --all-features release_evidence` - passed.
- Task automated `test` and `rg` checks - passed.
- Redaction secret scan - passed for Plan 16-02 artifacts; only expected labels, ESP boot terms, and board identity values were retained.
- `git diff -- reference/esp-miner --exit-code` - passed; the reference tree was not modified.
- Rust pre-commit sequence before task commits - passed: `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features`.

## Known Stubs

None. Stub-pattern scan of files created or modified by this plan found only a Rust report-rendering format string in `tools/parity/src/main.rs`, not placeholder evidence or unwired data.

## Threat Flags

None. The plan touched release evidence files and a parity CLI path-normalization path already covered by the plan threat model; it did not add network endpoints, auth paths, schema changes, or new hardware control surfaces.

## User Setup Required

None.

## Next Phase Readiness

Plan 16-03 can consume the package, release-gate, detector, serial boot, and redaction artifacts for source commit `b55d3e68b68060fc6cf271372a75fc86c0a934c6`.

Remaining claims still need their own evidence before release promotion: live HTTP/static behavior, recovery route behavior, firmware OTA, OTAWWW, rollback, failed-update, interrupted-update, and large-erase behavior. Phase-level redaction status remains pending for those later artifacts.

*Phase: 16-current-commit-release-evidence-completion*
*Completed: 2026-07-01*

## Self-Check: PASSED

- Summary file exists at `.planning/phases/16-current-commit-release-evidence-completion/16-02-SUMMARY.md`.
- Task commit `b55d3e6` exists.
- Task commit `76eca67` exists.
- Summary uses standalone `---` only for the opening and closing YAML frontmatter delimiters.
