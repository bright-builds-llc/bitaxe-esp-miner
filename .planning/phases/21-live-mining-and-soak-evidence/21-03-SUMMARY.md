---
phase: 21-live-mining-and-soak-evidence
plan: "03"
subsystem: firmware-evidence
tags: [phase21, ultra205, release-gate, detector, board-info, safe-baseline, redaction]
requires:
  - phase: 21-live-mining-and-soak-evidence
    provides: controlled live-mining runtime package and gate from plans 21-01 and 21-02
provides:
  - Current Phase 21 default-safe package manifest and release-gate evidence.
  - Detector-gated Ultra 205 board-info evidence before hardware use.
  - Redacted safe-baseline flash-monitor evidence with mining, work submission, and hardware control disabled.
affects: [phase21-bm1366-diagnostic, phase21-live-mining-smoke, phase21-bounded-soak, phase21-telemetry]
tech-stack:
  added: []
  patterns: [detector-gated hardware preflight, commit-redacted hardware evidence, package identity refresh after evidence commits]
key-files:
  created:
    - docs/parity/evidence/phase-21-live-mining-and-soak-evidence/preflight.md
    - docs/parity/evidence/phase-21-live-mining-and-soak-evidence/preflight/package-release-gate/bitaxe-ultra205-package.json
    - docs/parity/evidence/phase-21-live-mining-and-soak-evidence/preflight/package-release-gate/package-command.log
    - docs/parity/evidence/phase-21-live-mining-and-soak-evidence/preflight/package-release-gate/release-gate.log
    - docs/parity/evidence/phase-21-live-mining-and-soak-evidence/preflight/safe-baseline/detect-ultra205.log
    - docs/parity/evidence/phase-21-live-mining-and-soak-evidence/preflight/safe-baseline/flash-command-evidence.json
    - docs/parity/evidence/phase-21-live-mining-and-soak-evidence/preflight/safe-baseline/flash-monitor.log
  modified:
    - docs/parity/evidence/phase-21-live-mining-and-soak-evidence/redaction-review.md
    - .planning/phases/21-live-mining-and-soak-evidence/21-VALIDATION.md
key-decisions:
  - "Refresh package/release-gate evidence after the Task 1 evidence commit so flashed safe-baseline firmware and copied manifest share the same source commit."
  - "Commit only redacted detector and flash-monitor evidence; raw detector output stayed under ignored target/ state."
  - "Redact IP-shaped Rust tool-version suffixes in the copied evidence manifest and rerun the release gate against the committed copy."
patterns-established:
  - "Phase 21 preflight evidence must keep package identity, detector, board-info, and safe-state status in one ledger before later hardware tiers consume it."
  - "Broad redaction scans should distinguish package metadata false positives from private endpoints, then remove ambiguity from committed evidence when practical."
requirements-completed: [ASIC-07, STR-06, STR-07, SAFE-09, EVD-05]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 21-2026-07-04T01-35-47
generated_at: 2026-07-04T05:00:57Z
duration: 15min
completed: 2026-07-04
---

# Phase 21 Plan 03: Preflight Evidence Summary

**Detector-gated Ultra 205 preflight package, release-gate, board-info, and redacted safe-baseline flash evidence for Phase 21 hardware tiers**

## Performance

- **Duration:** 15 min
- **Started:** 2026-07-04T04:46:08Z
- **Completed:** 2026-07-04T05:00:57Z
- **Tasks:** 2 completed
- **Files modified:** 10

## Accomplishments

- Captured the current Phase 21 default-safe package manifest and release-gate log.
- Ran `just detect-ultra205` before hardware use and recorded detector-gated board-info evidence.
- Flashed and monitored the connected Ultra 205 through the repo wrapper with `redact-evidence=true`, producing commit-ready safe-state evidence.
- Updated the preflight, redaction-review, and validation ledgers for downstream ASIC diagnostic and live-mining tiers.

## Task Commits

1. **Task 1: Capture current package/release-gate preflight artifacts** - `a19b9e0` (docs)
2. **Task 2: Capture detector-gated safe-baseline evidence** - `0b4622b` (docs)

## Files Created/Modified

- `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/preflight.md` - Package, detector, board-info, safe-baseline status, and explicit non-claims.
- `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/preflight/package-release-gate/bitaxe-ultra205-package.json` - Redacted committed package manifest copy.
- `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/preflight/package-release-gate/package-command.log` - `just package` evidence.
- `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/preflight/package-release-gate/release-gate.log` - Release-gate evidence with `release_gate: passed`.
- `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/preflight/safe-baseline/detect-ultra205.log` - Redacted detector and board-info evidence.
- `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/preflight/safe-baseline/flash-command-evidence.json` - Wrapper-owned flash/monitor evidence JSON.
- `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/preflight/safe-baseline/flash-monitor.log` - Redacted boot log with safe-state markers.
- `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/redaction-review.md` - Marked the preflight pack as redaction-reviewed.
- `.planning/phases/21-live-mining-and-soak-evidence/21-VALIDATION.md` - Added preflight and live-mining enablement citations to 21-W0-02.

## Verification

- `just package`
- `bazel run //tools/parity:report -- release-gate --manifest docs/parity/evidence/phase-21-live-mining-and-soak-evidence/preflight/package-release-gate/bitaxe-ultra205-package.json`
- `just detect-ultra205`
- `just flash-monitor board=205 port=/dev/cu.usbmodem1101 evidence-dir=docs/parity/evidence/phase-21-live-mining-and-soak-evidence/preflight/safe-baseline capture-timeout-seconds=45 redact-evidence=true wifi-credentials=wifi-credentials.json`
- Task 2 evidence checks for required files, safe-state markers, release gate pass, validation citations, and strict raw-value redaction scan.
- Before each task commit: `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, `cargo test --all-features`.
- Overall verification: `just parity`, `just verify-reference`, `git diff -- reference/esp-miner --exit-code`.

## Hardware Detection Outcome

- Detector gate: passed.
- Selected port: `/dev/cu.usbmodem1101`.
- Board-info: passed through `espflash board-info --chip esp32s3 --port /dev/cu.usbmodem1101 --non-interactive`.
- Safe baseline: passed with `capture_status: timed_out_after_trusted_output`.
- Safe-state markers observed: `mining=disabled`, `hardware_control=disabled`, `work_submission=disabled`.

## Decisions Made

- Refreshed the package manifest and release gate after Task 1 committed, because the original package copy referenced the previous source commit while Task 2 would flash the current source commit.
- Preserved raw detector output only in ignored `target/` scratch space and committed a redacted detector log.
- Redacted IP-shaped Rust tool-version suffixes in the committed manifest copy rather than documenting them as false positives, then reran release-gate verification.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Refreshed package identity after Task 1 commit**
- **Found during:** Task 2 (safe-baseline evidence)
- **Issue:** The Task 1 package evidence was generated before the Task 1 evidence commit, so the manifest `source_commit` did not match the firmware commit that Task 2 would flash.
- **Fix:** Reran `just package`, recopied the manifest, reran the release gate, and updated the preflight ledger to `a19b9e0660a315e8f0e1aa08d16e4822fd6937a6`.
- **Files modified:** `preflight.md`, `preflight/package-release-gate/bitaxe-ultra205-package.json`, `package-command.log`, `release-gate.log`
- **Verification:** Release gate passed against the refreshed committed manifest; flash evidence reports the same firmware commit.
- **Committed in:** `0b4622b`

**2. [Rule 2 - Missing Critical] Committed redacted detector output instead of raw detector output**
- **Found during:** Task 2 (detector capture)
- **Issue:** The plan required detector output, but user and repo rules prohibit committing raw IP, MAC, target URL, credential, or secret-bearing evidence.
- **Fix:** Kept raw detector output under ignored `target/` scratch space and committed a redacted `detect-ultra205.log`.
- **Files modified:** `preflight/safe-baseline/detect-ultra205.log`, `redaction-review.md`
- **Verification:** Strict raw-value redaction scan passed for the committed preflight pack.
- **Committed in:** `0b4622b`

**3. [Rule 2 - Missing Critical] Redacted IP-shaped tool-version metadata in the committed package manifest**
- **Found during:** Task 2 (redaction scan)
- **Issue:** The copied manifest contained Rust tool-version suffixes shaped like four-octet addresses, which would keep broad raw-IP scans ambiguous.
- **Fix:** Replaced those suffixes with `[redacted-ip-shaped-tool-version]` in the committed evidence copy and reran the release gate.
- **Files modified:** `preflight/package-release-gate/bitaxe-ultra205-package.json`, `release-gate.log`
- **Verification:** Release gate still passed; strict raw-value redaction scan passed.
- **Committed in:** `0b4622b`

**4. [Rule 3 - Blocking] Normalized evidence manifest permissions**
- **Found during:** Task 2 (package refresh)
- **Issue:** The copied Bazel manifest was read-only/executable and blocked replacement.
- **Fix:** Normalized the committed evidence copy to mode `100644`.
- **Files modified:** `preflight/package-release-gate/bitaxe-ultra205-package.json`
- **Verification:** Manifest replacement succeeded and the release gate passed.
- **Committed in:** `0b4622b`

**Total deviations:** 4 auto-fixed (3 missing critical, 1 blocking)
**Impact on plan:** All changes preserve the plan objective, improve evidence correctness, and avoid committing raw or ambiguous hardware data.

## Issues Encountered

- The flash/monitor command printed hardware metadata to the local terminal while executing, but committed evidence artifacts were produced with `redact-evidence=true` and passed the strict redaction scan.

## Auth Gates

None.

## Known Stubs

None.

## Threat Flags

None - this plan added evidence artifacts only. No new network endpoints, auth paths, file-access surfaces, or schema changes were introduced.

## User Setup Required

None - no external service configuration required. The existing ignored `wifi-credentials.json` file was passed only as a path to the repo wrapper; contents were not read or committed.

## Next Phase Readiness

Phase 21 diagnostic, live-mining smoke, soak, and telemetry plans can consume the preflight ledger, copied package manifest, detector evidence, board-info status, and safe-baseline safe-state proof. The plan does not claim BM1366 chip detection, production mining, accepted/rejected shares, live API/WebSocket telemetry, frequency transition, voltage control, fan control, OTA, erase, rollback, or interrupted-update behavior.

## Self-Check: PASSED

- Required files exist.
- Task commits `a19b9e0` and `0b4622b` are reachable.
- Summary uses only the required YAML frontmatter delimiter pair.

*Phase: 21-live-mining-and-soak-evidence*
*Completed: 2026-07-04*
