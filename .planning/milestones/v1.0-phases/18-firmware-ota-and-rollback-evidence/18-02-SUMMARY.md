---
phase: 18-firmware-ota-and-rollback-evidence
plan: "02"
subsystem: firmware-ota-evidence
tags: [phase-18, package, release-gate, ultra-205, serial-boot, target-lock, redaction]
requires:
  - phase: 18-firmware-ota-and-rollback-evidence
    provides: Phase 18 OTA evidence wrapper and target-lock helper from Plan 01
  - phase: 17-live-http-api-and-static-evidence
    provides: Redacted serial boot and target-lock evidence pattern
provides:
  - Current package manifest and release-gate ledger for the Phase 18 source commit
  - Detector-gated Ultra 205 flash-monitor evidence tied to the copied package manifest
  - Sanitized target lock derived from trusted local raw flash-monitor evidence
affects: [phase-18, firmware-ota, release-evidence, target-lock, redaction, ultra-205]
tech-stack:
  added: []
  patterns:
    - Manifest-backed package gate before live OTA work
    - Detector-first Ultra 205 hardware evidence capture
    - Commit-redacted serial evidence plus untracked raw target extraction under target/
key-files:
  created:
    - docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/package-release-gate.md
    - docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/package-release-gate/bitaxe-ultra205-package.json
    - docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/package-release-gate/package-command.log
    - docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/package-release-gate/release-gate.log
    - docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/serial-boot.md
    - docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/serial-boot/detect-ultra205.log
    - docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/serial-boot/flash-command-evidence.json
    - docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/serial-boot/flash-monitor.log
    - docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/target-lock.json
  modified: []
key-decisions:
  - "Treat `22d02f8e97928f1ec29360552179380b92582e6a` as the Phase 18 package identity because it was HEAD at Task 1 package time."
  - "Run flash-monitor from a temporary package-source clone under `target/` so wrapper-owned flash evidence stays aligned with the copied package manifest after task commits."
  - "Keep raw target/device_url extraction evidence under `target/phase18-firmware-ota-and-rollback-evidence-dev-raw/` and commit only redacted serial and target-lock artifacts."
patterns-established:
  - "Target-lock-only helper scratch output can be routed to `target/` while `--target-lock-out` writes the committed sanitized lock."
  - "Generated serial logs should be normalized for whitespace after redaction before `git diff --check`."
requirements-completed: [REL-02, REL-08, REL-07, EVD-05]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 18-2026-07-03T14-06-29
generated_at: 2026-07-03T15:33:39Z
duration: 20m10s
completed: 2026-07-03
---

# Phase 18 Plan 02: Package, Serial Boot, And Target Lock Summary

**Current package release-gate evidence plus detector-gated Ultra 205 serial boot and sanitized target provenance for OTA follow-up**

## Performance

- **Duration:** 20m10s
- **Started:** 2026-07-03T15:13:29Z
- **Completed:** 2026-07-03T15:33:39Z
- **Tasks:** 2
- **Files modified:** 9

## Accomplishments

- Ran `just package`, copied the current Ultra 205 package manifest, and recorded a manifest-backed release gate with OTA image checksum `c7f4a62872d6662562f89ff8e93e881317d65b8f58003577acfd6a0a50eb6463`.
- Ran `just detect-ultra205`, confirmed exactly one ESP32-S3 board-info target on `/dev/cu.usbmodem1101`, and captured redacted detector output.
- Flashed and monitored board `205` with commit-redacted wrapper evidence; `flash-command-evidence.json` is trusted, board `205`, and source/reference commits match the copied manifest.
- Created `target-lock.json` with `target_status: passed`, `device_url_source: usb_flash_monitor_log`, `device_url_redacted: http://[redacted]`, and `network_scan: disabled`.

## Task Commits

Each task was committed atomically:

1. **Task 1: Refresh current package and release-gate identity** - `914110c` (docs)
2. **Task 2: Capture detector, flash-monitor identity, and target lock** - `c184566` (docs)

**Plan metadata:** pending final metadata commit after SUMMARY and state updates.

## Files Created/Modified

- `docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/package-release-gate.md` - Package/release-gate ledger with manifest identity, checksums, command logs, and `network_scan: disabled`.
- `docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/package-release-gate/bitaxe-ultra205-package.json` - Copied manifest produced by the Plan 18-02 `just package` run.
- `docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/package-release-gate/package-command.log` - Captured `just package` output.
- `docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/package-release-gate/release-gate.log` - Captured manifest-backed release-gate output with `release_gate: passed`.
- `docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/serial-boot.md` - Detector, flash-monitor, target-lock, route marker, safe boot, and non-claim ledger.
- `docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/serial-boot/detect-ultra205.log` - Redacted detector and board-info transcript.
- `docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/serial-boot/flash-command-evidence.json` - Commit-redacted wrapper-owned flash-monitor metadata.
- `docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/serial-boot/flash-monitor.log` - Commit-redacted serial boot log with safe state, route registration, and commit markers.
- `docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/target-lock.json` - Sanitized target provenance derived from trusted raw flash evidence.

## Decisions Made

- Used the package source commit `22d02f8e97928f1ec29360552179380b92582e6a` as the identity for both package and serial evidence, even after task commits advanced Git HEAD.
- Used a temporary local clone under `target/` at the package source commit for flash-monitor execution because `tools/flash` records `git rev-parse HEAD` as `firmware_commit`.
- Kept raw target extraction evidence under `target/phase18-firmware-ota-and-rollback-evidence-dev-raw/` and committed only redacted serial artifacts plus sanitized `target-lock.json`.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Ran flash-monitor from a package-source clone**

- **Found during:** Task 2 (Capture detector, flash-monitor identity, and target lock)
- **Issue:** The required per-task package evidence commit advanced Git HEAD, while `tools/flash` writes current HEAD into `flash-command-evidence.json`. Running from the main worktree would have made `firmware_commit` differ from the copied package manifest.
- **Fix:** Created a temporary clone under `target/` at `22d02f8e97928f1ec29360552179380b92582e6a`, initialized reference submodules recursively there, ran repo-owned `just flash-monitor` from that clone, and wrote evidence back to the current Phase 18 directory.
- **Files modified:** Phase 18 serial evidence artifacts only.
- **Verification:** `phase18_flash_identity_check: passed`; flash JSON `firmware_commit` and `reference_commit` match the copied manifest.
- **Committed in:** `c184566`

**2. [Rule 3 - Blocking] Routed target-lock helper scratch output to `target/`**

- **Found during:** Task 2 (Capture detector, flash-monitor identity, and target lock)
- **Issue:** `scripts/phase18-firmware-ota-evidence.sh --target-lock-only` still creates a `firmware-ota` scratch log under its `--out-dir`, but Plan 18-02 owns only target-lock and serial artifacts.
- **Fix:** Used `--out-dir target/phase18-firmware-ota-and-rollback-evidence-dev-raw/target-lock-helper` with `--target-lock-out docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/target-lock.json`.
- **Files modified:** `target-lock.json`; scratch helper output remains untracked under `target/`.
- **Verification:** `phase18_target_lock_check: passed`; `git status` shows no untracked docs firmware-ota artifacts.
- **Committed in:** `c184566`

**Total deviations:** 2 auto-fixed blocking issues.

**Impact on plan:** Evidence artifacts satisfy the planned package, detector, flash identity, and target-lock criteria without changing source code or broadening Phase 18 scope.

## Issues Encountered

- The first Task 1 manifest check used a non-exported `TASK1_HEAD` shell variable; rerunning the check with the expected commit as a node argument passed.
- The first detector capture wrapper attempted to assign zsh's read-only `status` variable after `just detect-ultra205`; rerunning with `rc` captured the detector result correctly.
- A detached Git worktree could not build the package because the Bazel source stamp target expects `.git/HEAD` and `.git/logs/HEAD`; the temporary clone resolved this.
- Generated detector and serial logs contained trailing whitespace and CR-style line endings; they were mechanically normalized before commit so `git diff --check` passed.

## Verification

- `test -s package-release-gate/bitaxe-ultra205-package.json`: passed.
- `rg -n "release_gate: passed" package-release-gate/release-gate.log`: passed.
- Package manifest node check for `source_commit`, `reference_commit`, `firmware_ota_image`, `esp-miner.bin`, and 64-character SHA-256: passed.
- Package ledger rg check for package status, release-gate status, source/reference commits, OTA path/checksum, and `network_scan: disabled`: passed.
- `test -s serial-boot/detect-ultra205.log`: passed.
- Serial/target-lock status rg check for detector, flash-monitor, board `205`, selected port, source/reference commits, and `network_scan: disabled`: passed.
- Target-lock node check for board `205`, `network_scan: disabled`, explicit target provenance, and redacted device URL: passed.
- Flash identity node check for board `205`, trusted output, package source commit, and reference commit: passed.
- `git diff --cached --check` for committed Phase 18 Plan 02 evidence files: passed.
- Targeted redaction scan over committed Phase 18 Plan 02 artifacts: passed.
- Rust pre-commit sequence before task commits: `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features`: passed.

## Known Stubs

None. Stub scan found no UI dummy data, unfinished-code markers, or unwired mock evidence in the files created by this plan.

## Threat Flags

None. This plan created evidence files only. It did not introduce new network endpoints, auth paths, file-access code paths, schema changes, or trust-boundary code outside the plan threat model.

## User Setup Required

None for Plan 18-02. The ignored local raw target evidence remains under `target/phase18-firmware-ota-and-rollback-evidence-dev-raw/` for follow-up OTA plans; it was not committed.

## Next Phase Readiness

Ready for Plan 18-03. The current Phase 18 package identity, detector gate, flash-monitor identity, and sanitized target provenance are available before OTA upload evidence work begins.

## Self-Check: PASSED

- Created files exist: package ledger, copied manifest, package log, release-gate log, serial ledger, detector log, flash evidence JSON, redacted flash-monitor log, target lock, and `18-02-SUMMARY.md`.
- Task commits exist: `914110c` and `c184566`.
- SUMMARY frontmatter delimiter check passed with exactly two standalone `---` delimiters.
- Stub scan and summary whitespace check passed.

*Phase: 18-firmware-ota-and-rollback-evidence*
*Completed: 2026-07-03*
