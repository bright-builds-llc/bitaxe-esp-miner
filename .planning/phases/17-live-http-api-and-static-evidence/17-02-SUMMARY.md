---
phase: 17-live-http-api-and-static-evidence
plan: "02"
subsystem: parity-evidence
tags:
  - docs
  - bazel
  - espflash
  - hardware
  - release-gate
requires:
  - phase: 17-live-http-api-and-static-evidence
    provides: Phase 17 helper, WebSocket capture, and redaction scaffold from Plan 17-01
provides:
  - Phase 17 Ultra 205 package manifest copy and release-gate evidence
  - Detector-gated wrapper-owned flash-monitor identity evidence
  - Package-to-board source/reference commit chain for downstream HTTP and WebSocket plans
affects:
  - phase-17-live-evidence
  - parity-evidence
  - release-docs
tech-stack:
  added: []
  patterns:
    - Wrapper-owned flash-monitor JSON is the trusted serial proof source
    - Package evidence is refreshed to the pre-serial source commit before route probes consume it
key-files:
  created:
    - docs/parity/evidence/phase-17-live-http-api-and-static-evidence/package-release-gate.md
    - docs/parity/evidence/phase-17-live-http-api-and-static-evidence/package-release-gate/bitaxe-ultra205-package.json
    - docs/parity/evidence/phase-17-live-http-api-and-static-evidence/package-release-gate/package-command.log
    - docs/parity/evidence/phase-17-live-http-api-and-static-evidence/package-release-gate/release-gate.log
    - docs/parity/evidence/phase-17-live-http-api-and-static-evidence/serial-boot.md
    - docs/parity/evidence/phase-17-live-http-api-and-static-evidence/serial-boot/detect-ultra205.log
    - docs/parity/evidence/phase-17-live-http-api-and-static-evidence/serial-boot/flash-command-evidence.json
    - docs/parity/evidence/phase-17-live-http-api-and-static-evidence/serial-boot/flash-monitor.log
  modified: []
key-decisions:
  - "Trust only wrapper-owned flash-monitor JSON and log artifacts for serial identity evidence."
  - "Refresh package/release-gate evidence after the Task 1 commit so the copied manifest matches the flashed source commit."
  - "Keep live HTTP, static route, WebSocket frame, OTA, rollback, boot-validation, and OTAWWW update behavior as explicit non-claims."
patterns-established:
  - "Identity chain: copied package manifest source/reference commits must match flash-command-evidence.json before downstream probes can claim just-flashed evidence."
requirements-completed:
  - REL-01
  - REL-07
  - EVD-05
generated_by: gsd-execute-plan
lifecycle_mode: interactive
phase_lifecycle_id: 17-2026-07-02T01-09-48
generated_at: 2026-07-02T02:59:39Z
duration: 10 min
completed: 2026-07-02
---

# Phase 17 Plan 02: Package And Flash-Monitor Identity Summary

**Ultra 205 package, release-gate, detector, and wrapper-owned serial identity evidence are captured for source commit `d9e471c9699eb0140749127416640aa1bf077d26`.**

## Performance

- **Duration:** 10 min
- **Started:** 2026-07-02T02:49:26Z
- **Completed:** 2026-07-02T02:59:39Z
- **Tasks:** 2
- **Files modified:** 8

## Accomplishments

- Captured Phase 17 `just package` and release-gate logs plus a copied Ultra 205 package manifest.
- Ran `just detect-ultra205`, selected exactly one board `205` port, and captured the detector transcript.
- Ran wrapper-owned `just flash-monitor` and committed trusted `flash-command-evidence.json` plus serial boot log evidence.

## Task Commits

Each task was committed atomically:

1. **Task 1: Record current package and release-gate evidence** - `d9e471c` (`docs`)
2. **Task 2: Capture detector-gated flash-monitor identity or blocker** - `d551e75` (`docs`)

## Files Created/Modified

- `docs/parity/evidence/phase-17-live-http-api-and-static-evidence/package-release-gate.md` - Package status, release-gate status, source/reference identity, artifact checksums, and non-claims.
- `docs/parity/evidence/phase-17-live-http-api-and-static-evidence/package-release-gate/bitaxe-ultra205-package.json` - Copied package manifest consumed by downstream identity checks.
- `docs/parity/evidence/phase-17-live-http-api-and-static-evidence/package-release-gate/package-command.log` - Captured `just package` output.
- `docs/parity/evidence/phase-17-live-http-api-and-static-evidence/package-release-gate/release-gate.log` - Captured release-gate output.
- `docs/parity/evidence/phase-17-live-http-api-and-static-evidence/serial-boot.md` - Detector, flash-monitor, serial marker, conclusion, and non-claim ledger.
- `docs/parity/evidence/phase-17-live-http-api-and-static-evidence/serial-boot/detect-ultra205.log` - Detector and board-info transcript.
- `docs/parity/evidence/phase-17-live-http-api-and-static-evidence/serial-boot/flash-command-evidence.json` - Wrapper-owned flash-monitor evidence JSON.
- `docs/parity/evidence/phase-17-live-http-api-and-static-evidence/serial-boot/flash-monitor.log` - Captured noninteractive serial boot log.

## Decisions Made

- Wrapper-owned `flash-command-evidence.json` and `flash-monitor.log` are the only trusted serial proof; raw monitor fallback output is not cited.
- The copied manifest and wrapper JSON are matched on full source commit `d9e471c9699eb0140749127416640aa1bf077d26` and reference commit `c1915b0a63bfabebdb95a515cedfee05146c1d50`.
- This plan intentionally generated no HTTP/static/API route artifacts, WebSocket frame artifacts, OTA artifacts, rollback artifacts, or redaction sign-off update.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Refreshed package evidence after Task 1 commit advanced HEAD**
- **Found during:** Task 2 (Capture detector-gated flash-monitor identity or blocker)
- **Issue:** The flash-monitor wrapper rebuilt the package after the Task 1 commit, so the flashed firmware source commit became `d9e471c9699eb0140749127416640aa1bf077d26` while the first copied package manifest still recorded pre-Task-1 commit `1a1648158c3b8c1a509c73a72f87e4005a1aeff9`.
- **Fix:** Re-ran `just package`, re-ran release-gate validation, recopied `bitaxe-ultra205-package.json`, updated `package-release-gate.md`, and verified the copied manifest matches `flash-command-evidence.json`.
- **Files modified:** `package-release-gate.md`, `package-command.log`, `release-gate.log`, `bitaxe-ultra205-package.json`
- **Verification:** `plan_identity_check: passed` and both Task 1 and Task 2 `rg` acceptance checks passed.
- **Committed in:** `d551e75`

**Total deviations:** 1 auto-fixed (1 bug)
**Impact on plan:** The fix preserves the plan's identity-chain requirement without adding route, WebSocket, OTA, rollback, or redaction scope.

## Issues Encountered

- `just flash-monitor` continued past the first 30-second polling window but exited successfully after the wrapper wrote trusted output.
- A sensitive-token scan matched benign `PSRAM memory pool` log text; no credentials, private endpoints, pool credentials, tokens, or NVS secret values were found.

## Authentication Gates

None.

## User Setup Required

None - no external service configuration required.

## Known Stubs

None.

## Verification

- `node /Users/peterryszkiewicz/.codex/get-shit-done/bin/gsd-tools.cjs verify lifecycle 17 --require-plans --raw`
- `just package`
- `bazel run //tools/parity:report -- release-gate --manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json`
- `just detect-ultra205`
- `just flash-monitor board=205 port=/dev/cu.usbmodem1101 manifest=bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json evidence-dir=docs/parity/evidence/phase-17-live-http-api-and-static-evidence/serial-boot capture-timeout-seconds=35`
- `test -f docs/parity/evidence/phase-17-live-http-api-and-static-evidence/package-release-gate/bitaxe-ultra205-package.json && test -f docs/parity/evidence/phase-17-live-http-api-and-static-evidence/package-release-gate/package-command.log && test -f docs/parity/evidence/phase-17-live-http-api-and-static-evidence/package-release-gate/release-gate.log && rg -n "package_status|release_gate_status|source_commit|reference_commit|does not prove live HTTP|does not prove WebSocket frames|does not prove valid OTA|whole-.www. OTAWWW" docs/parity/evidence/phase-17-live-http-api-and-static-evidence/package-release-gate.md`
- `test -f docs/parity/evidence/phase-17-live-http-api-and-static-evidence/serial-boot/detect-ultra205.log && test -f docs/parity/evidence/phase-17-live-http-api-and-static-evidence/serial-boot.md && rg -n "detector_status|board.*205|flash_monitor_status|network_scan: disabled|source_commit|reference_commit|does not prove live HTTP|does not prove WebSocket frames|does not prove valid OTA" docs/parity/evidence/phase-17-live-http-api-and-static-evidence/serial-boot.md`
- `node -e 'const fs=require("fs"); const manifest=JSON.parse(fs.readFileSync("docs/parity/evidence/phase-17-live-http-api-and-static-evidence/package-release-gate/bitaxe-ultra205-package.json","utf8")); const flash=JSON.parse(fs.readFileSync("docs/parity/evidence/phase-17-live-http-api-and-static-evidence/serial-boot/flash-command-evidence.json","utf8")); if (manifest.source_commit !== flash.firmware_commit) throw new Error("source mismatch"); if (manifest.reference_commit !== flash.reference_commit) throw new Error("reference mismatch"); if (flash.trusted_output !== true) throw new Error("untrusted flash output"); console.log("plan_identity_check: passed");'`
- `rg -n "package_status|release_gate_status|detector_status|flash_monitor_status|network_scan: disabled" docs/parity/evidence/phase-17-live-http-api-and-static-evidence/package-release-gate.md docs/parity/evidence/phase-17-live-http-api-and-static-evidence/serial-boot.md`

## Next Phase Readiness

Ready for `17-03-PLAN.md`. Downstream HTTP/static/API evidence can consume the copied package manifest and wrapper-owned flash evidence. Live route or WebSocket claims still require later plans and an explicit origin-only `DEVICE_URL`; this plan did not infer or record one.

## Self-Check: PASSED

- Created files exist on disk.
- Task commits `d9e471c` and `d551e75` exist in git history.
- Summary frontmatter fields are readable and use only the opening and closing YAML delimiters.

*Phase: 17-live-http-api-and-static-evidence*
*Completed: 2026-07-02*
