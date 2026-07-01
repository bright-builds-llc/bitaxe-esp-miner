---
phase: 15-bm1366-mining-evidence-completion
plan: "02"
subsystem: parity
tags: [bm1366, mining, evidence, diagnostics, flash-wrapper]
requires:
  - phase: 15-bm1366-mining-evidence-completion
    provides: mining allow-manifest gate and Phase 15 evidence/redaction scaffold
provides:
  - Package-backed BM1366 diagnostic firmware builder
  - Detector-gated BM1366 chip-detect hardware smoke evidence
  - Mining allow manifest and redaction-reviewed chip-detect evidence pack
  - Repo-relative flash wrapper manifest/image path handling for `bazel run`
affects: [phase-15, bm1366-mining, parity-evidence, tools-flash]
tech-stack:
  added: []
  patterns:
    - Package-backed diagnostic firmware builds through Bazel action env and `scripts/package-firmware.sh`
    - Hardware evidence uses detector, mining-allow validation, wrapper trust markers, and redaction review before citation
key-files:
  created:
    - scripts/phase15-bm1366-diagnostic-package.sh
    - scripts/phase15-bm1366-diagnostic-package-test.sh
    - docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/chip-detect.md
    - docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/chip-detect/allow-chip-detect.json
    - docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/chip-detect/flash-command-evidence.json
  modified:
    - scripts/BUILD.bazel
    - tools/flash/src/main.rs
    - docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/redaction-review.md
key-decisions:
  - "Kept chip-detect evidence scoped to diagnostic chip-detect smoke and did not promote work/result, production mining, voltage, fan, or share claims."
  - "Resolved explicit flash image and manifest arguments relative to the workspace so `bazel run //tools/flash:flash` can consume repo-relative evidence package paths."
  - "Committed the copied package manifest needed for `mining-allow` validation, while dropping large generated package binaries from the worktree after the hardware run."
patterns-established:
  - "Phase 15 diagnostic packages are built with explicit `BITAXE_ASIC_DIAGNOSTIC` and `BITAXE_HARDWARE_EVIDENCE_ACK` Bazel action env values, then packaged before any flash command."
  - "Chip-detect hardware evidence must cite detector output, package identity, allow validation, wrapper JSON, serial markers, non-claims, and redaction review."
requirements-completed: [ASIC-07, EVD-05]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 15-2026-07-01T02-07-59
generated_at: 2026-07-01T03:47:32Z
duration: 19 min
completed: 2026-07-01
---

# Phase 15 Plan 02: Package-Backed BM1366 Chip-Detect Summary

**BM1366 chip-detect diagnostics now run through the package-backed flash wrapper with detector, allow-manifest, trusted-marker, and redaction evidence.**

## Performance

- **Duration:** 19 min
- **Started:** 2026-07-01T03:28:25Z
- **Completed:** 2026-07-01T03:47:32Z
- **Tasks:** 2
- **Files modified:** 12

## Accomplishments

- Added `scripts/phase15-bm1366-diagnostic-package.sh`, which builds chip-detect or work-result diagnostic firmware through Bazel action env and packages it through `scripts/package-firmware.sh`.
- Captured detector-gated Ultra 205 chip-detect evidence on `/dev/cu.usbmodem1101` with package manifest source commit `804eaa81f1184a35864e5681361733c93242ded9` and reference commit `c1915b0a63bfabebdb95a515cedfee05146c1d50`.
- Validated `allow-chip-detect.json` through `bazel run //tools/parity:report -- mining-allow`, then ran the exact allowed wrapper command and captured `trusted_output: true` with `capture_status: timed_out_after_trusted_output`.
- Updated the Phase 15 redaction review so the `bm1366-chip-detect` pack is passed while later work-result, mining-smoke, bounded-soak, parity-redaction, and final-ledger packs remain pending.
- Fixed `tools/flash` so explicit relative `--image` and `--manifest` arguments resolve under the workspace when invoked through `bazel run`.

## Task Commits

Each task was committed atomically:

1. **Task 1: Add the package-backed diagnostic builder** - `804eaa8` (feat)
2. **Task 2: Capture package-backed BM1366 chip-detect evidence** - `15340c7` (fix)

**Plan metadata:** pending final metadata commit

## Files Created/Modified

- `scripts/phase15-bm1366-diagnostic-package.sh` - Package-backed diagnostic builder for chip-detect and work-result modes.
- `scripts/phase15-bm1366-diagnostic-package-test.sh` - Fake Bazel/package test coverage for both modes, unknown modes, summary JSON, and no raw hardware commands.
- `scripts/BUILD.bazel` - Registers the Phase 15 diagnostic package binary and test.
- `tools/flash/src/main.rs` - Resolves explicit relative image/manifest paths against the workspace and tests both path cases.
- `docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/chip-detect.md` - Human-readable chip-detect evidence summary, observed markers, conclusion, non-claims, and residual blockers.
- `docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/chip-detect/detect-ultra205.log` - Detector gate output with exactly one likely Ultra 205 port and board-info success.
- `docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/chip-detect/allow-chip-detect.json` - Mining allow manifest for the exact chip-detect flash-monitor command.
- `docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/chip-detect/diagnostic-package-summary.json` - Diagnostic package summary with mode, action env values, manifest path, and command shape.
- `docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/chip-detect/flash-command-evidence.json` - Wrapper-owned flash-monitor evidence JSON.
- `docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/chip-detect/flash-monitor.log` - Captured serial log with trusted wrapper markers and fail-closed chip-detect result.
- `docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/chip-detect/package/bitaxe-ultra205-package.json` - Copied package manifest used for allow validation and the hardware run.
- `docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/redaction-review.md` - Marks the chip-detect pack passed and leaves later packs pending.

## Decisions Made

- Kept chip-detect evidence scoped to `passed for package-backed chip-detect smoke`; the summary explicitly does not claim production mining, frequency transition, voltage behavior, work-send, result-receive, or accepted shares.
- Fixed `tools/flash` rather than rewriting the evidence command, because the plan required an exact repo-relative `bazel run //tools/flash:flash` command and the wrapper should support that documented path shape.
- Retained the copied package manifest for reproducible `mining-allow` validation, but did not commit the large generated binary package payloads because prior evidence convention records package identity/checksums without storing binaries under `docs/`.

## Deviations from Plan

### Process Adjustments

**1. AGENTS.md pre-commit rule superseded TDD RED commit**
- **Found during:** Task 1 (Add the package-backed diagnostic builder)
- **Issue:** The plan was marked `tdd="true"`, but repo Rust rules require `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features` to pass before every commit.
- **Fix:** Ran the RED Bazel test and captured the expected missing-wrapper failure, then implemented the builder and committed only the passing Task 1 state.
- **Files modified:** `scripts/phase15-bm1366-diagnostic-package.sh`, `scripts/phase15-bm1366-diagnostic-package-test.sh`, `scripts/BUILD.bazel`
- **Verification:** RED `bazel test //scripts:phase15_bm1366_diagnostic_package_test` failed before implementation; post-implementation script, Bazel, and full Rust checks passed.
- **Committed in:** `804eaa8`

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Resolved relative manifest/image paths under `bazel run`**
- **Found during:** Task 2 (Capture package-backed BM1366 chip-detect evidence)
- **Issue:** The exact allowed flash-monitor command failed before flashing because `tools/flash` interpreted the repo-relative manifest path from Bazel's run directory instead of the workspace.
- **Fix:** Resolved explicit `--image` and `--manifest` arguments through the wrapper environment's workspace path helper, and added regression tests for both cases.
- **Files modified:** `tools/flash/src/main.rs`
- **Verification:** `cargo test -p bitaxe-flash --all-features` passed, then the exact allowed flash-monitor command succeeded and wrote trusted evidence.
- **Committed in:** `15340c7`

**Total deviations:** 1 process adjustment, 1 auto-fixed blocking issue.
**Impact on plan:** The fixes preserved the plan's exact command contract and evidence scope. No checklist overclaim or hardware scope expansion was introduced.

## Issues Encountered

- The first exact flash-monitor attempt failed on repo-relative manifest resolution under `bazel run`; this is documented above as the Rule 3 auto-fix.
- Generated package binaries were removed from the worktree after capture to avoid committing large build payloads under `docs/`; the package manifest with checksums remains committed because `mining-allow` validation depends on it.
- Safe-boot restore artifacts were not created because the diagnostic package produced trusted output and did not require restore handling.

## Known Stubs

None. The stub scan matched only intentional future-pack `pending` rows in `redaction-review.md`, observed firmware status strings such as `hardware_evidence_pending` and `ota_boot_validation=not_pending`, and Bash variable initialization used before argument parsing.

## User Setup Required

None - no external service configuration required.

## Verification

- `node "$HOME/.codex/get-shit-done/bin/gsd-tools.cjs" verify lifecycle 15 --require-plans --raw` - passed.
- `bash -n scripts/phase15-bm1366-diagnostic-package.sh scripts/phase15-bm1366-diagnostic-package-test.sh` - passed.
- `bazel test //scripts:phase15_bm1366_diagnostic_package_test` - passed.
- `just detect-ultra205` - passed and wrote `chip-detect/detect-ultra205.log` with one port, `/dev/cu.usbmodem1101`.
- `scripts/phase15-bm1366-diagnostic-package.sh --mode chip-detect --out-dir docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/chip-detect` - passed.
- `bazel run //tools/parity:report -- mining-allow --manifest docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/chip-detect/allow-chip-detect.json --surface bm1366-chip-detect --allowed-command "bazel run //tools/flash:flash -- flash-monitor --board 205 --port /dev/cu.usbmodem1101 --manifest docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/chip-detect/package/bitaxe-ultra205-package.json --evidence-dir docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/chip-detect --capture-timeout-seconds 35"` - passed.
- `bazel run //tools/flash:flash -- flash-monitor --board 205 --port /dev/cu.usbmodem1101 --manifest docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/chip-detect/package/bitaxe-ultra205-package.json --evidence-dir docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/chip-detect --capture-timeout-seconds 35` - passed with `trusted_output: true`.
- `rg -n "bm1366-chip-detect|chip_detect_status|trusted_output|spiffs_mount=available|axeos_api_route_shell=started|safe_state: mining=disabled|redaction" docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/chip-detect.md docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/redaction-review.md` - passed.
- `rg -n "just detect-ultra205|bazel run //tools/flash:flash|passed for package-backed chip-detect smoke|non_claims: production mining, frequency transition, voltage behavior, work-send, result-receive, accepted shares" docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/chip-detect.md` - passed.
- `git diff -- docs/parity/checklist.md --exit-code` - passed; no broad ASIC/mining row was promoted.
- `cargo test -p bitaxe-parity --all-features mining_allow` - passed.
- `cargo test -p bitaxe-flash --all-features` - passed.
- `just parity` - passed with `validation_errors: none`.
- `git diff -- reference/esp-miner --exit-code` - passed.
- Task commit pre-checks: `cargo fmt --all`; `cargo clippy --all-targets --all-features -- -D warnings`; `cargo build --all-targets --all-features`; `cargo test --all-features` - passed before each task commit.

## Next Phase Readiness

Ready for `15-03-PLAN.md`: the package-backed diagnostic path and chip-detect trust root are in place, with explicit non-claims. Work-result diagnostics and any later mining smoke/soak evidence must still run through their own allow manifests, redaction review, and safe-stop evidence before citation.

*Phase: 15-bm1366-mining-evidence-completion*
*Completed: 2026-07-01*

## Self-Check: PASSED

- Summary file exists.
- Created files exist: diagnostic builder/test, chip-detect evidence summary, allow manifest, wrapper evidence JSON, and copied package manifest.
- Task commits exist: `804eaa8` and `15340c7`.
- Summary frontmatter delimiter check passed with standalone `---` only at the opening and closing frontmatter lines.
