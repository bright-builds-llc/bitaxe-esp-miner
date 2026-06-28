---
phase: 08-parity-evidence-and-ultra-205-release-gate
plan: 02
subsystem: parity-evidence
tags: [ultra-205, evidence, package, hardware-detection, device-url]
requires:
  - phase: 07-ota-filesystem-and-release-packaging
    provides: Package, OTA, filesystem, recovery, and Phase 7 deferred live-evidence records.
provides:
  - Phase 8 evidence ledger with package manifest paths and artifact SHA-256 values.
  - Ultra 205 detector output proving exactly one ESP32-S3 USB candidate.
  - DEVICE_URL blocker record with sanitized serial boot evidence and no checklist promotion.
affects: [phase-08-release-gate, phase-08-live-http-ota, parity-evidence]
tech-stack:
  added: []
  patterns:
    - Evidence ledger records package, detector, URL, and live-surface status before checklist promotion.
    - Hardware evidence logs are committed only after scanning for private URLs and credentials.
key-files:
  created:
    - docs/parity/evidence/phase-08-ultra-205-release-gate.md
    - docs/parity/evidence/phase-08-ultra-205-release-gate/flash-monitor-noninteractive.log
    - .planning/phases/08-parity-evidence-and-ultra-205-release-gate/08-02-SUMMARY.md
  modified: []
key-decisions:
  - "Record DEVICE_URL as blocked rather than promote live HTTP/OTA/recovery rows without a reachable URL."
  - "Use a bounded non-interactive espflash monitor capture after the repo wrapper failed at monitor startup in this non-interactive environment."
  - "Do not mark Phase 8 requirements globally complete from this precondition-only plan."
patterns-established:
  - "DEVICE_URL evidence uses exactly one status line and explicit no-URL conclusions in every dependent live section."
  - "Phase 8 hardware logs can be committed when sanitized and scoped to boot evidence."
requirements-completed: [REL-08, EVD-05]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 8-2026-06-28T21-51-32
generated_at: 2026-06-28T23:36:07Z
duration: 11 min
completed: 2026-06-28
---

# Phase 08 Plan 02: Ultra 205 Evidence Preconditions Summary

**Phase 8 evidence ledger now records package hashes, passed Ultra 205 USB detection, serial boot evidence, and a sanitized no-DEVICE_URL blocker.**

## Performance

- **Duration:** 11 min
- **Started:** 2026-06-28T23:25:09Z
- **Completed:** 2026-06-28T23:36:07Z
- **Tasks:** 3
- **Files modified:** 3

## Accomplishments

- Created `docs/parity/evidence/phase-08-ultra-205-release-gate.md` with all required sections and parser-safe Markdown.
- Ran `just package` before hardware detection and recorded the Ultra 205 manifest, artifact paths, and SHA-256 values.
- Ran `just detect-ultra205`; it selected `port=/dev/cu.usbmodem1101` and board-info identified ESP32-S3 rev v0.2 with 16 MB flash.
- Captured serial boot evidence showing factory boot, SPIFFS mount, boot-validation state, HTTP route registration, and no discoverable device URL.

## Task Commits

Each task was committed atomically:

1. **Task 1: Create Phase 8 evidence ledger** - `11374c3` (docs)
2. **Task 2: Run Ultra 205 detection and package preflight** - `a0f544b` (docs)
3. **Task 3: Establish DEVICE_URL or record blocker** - `8e8f460` (docs)

## Files Created/Modified

- `docs/parity/evidence/phase-08-ultra-205-release-gate.md` - Phase 8 ledger for package, detector, `DEVICE_URL`, and live release-gate conclusions.
- `docs/parity/evidence/phase-08-ultra-205-release-gate/flash-monitor-noninteractive.log` - Sanitized bounded serial monitor capture.
- `.planning/phases/08-parity-evidence-and-ultra-205-release-gate/08-02-SUMMARY.md` - This execution summary.

## Decisions Made

- Kept live HTTP, OTA, rollback, recovery, large erase, and interrupted-update sections below verified evidence because no reachable `DEVICE_URL` was found.
- Recorded the exact blocker: no IP, DHCP, Wi-Fi association, AP address, mDNS, hostname, or operator-supplied reachable URL.
- Left `.planning/STATE.md` and global requirement completion untouched because `.planning/STATE.md` was dirty before this run and REL-08/EVD-05 are not globally complete after a precondition-only plan.

## Validation Results

| Command / Check | Result |
| --- | --- |
| Fixed-string ledger check | Passed. |
| `grep -n '^---$' docs/parity/evidence/phase-08-ultra-205-release-gate.md \| wc -l` | Passed with `0`. |
| `just package` before detector | Passed and produced the expected Ultra 205 artifacts. |
| `just detect-ultra205` | Passed with exactly one port, `port=/dev/cu.usbmodem1101`. |
| `just flash-monitor board=205 port=/dev/cu.usbmodem1101 evidence-dir=docs/parity/evidence/phase-08-ultra-205-release-gate` | Ran after detector; factory flash path executed, monitor exited nonzero with `Failed to initialize input reader`. |
| `timeout 25 espflash monitor --chip esp32s3 --port /dev/cu.usbmodem1101 --non-interactive` | Captured bounded serial boot evidence; exited by timeout as expected for a continuous monitor. |
| Branch-aware `DEVICE_URL status` check | Passed with exactly one supported blocked status. |
| Final `just package` | Passed. |
| Final `just detect-ultra205` | Passed. |
| Rust pre-commit sequence before each task commit | Passed: `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, `cargo test --all-features`. |
| Secret/redaction scan | Passed; hits were limited to explicit redaction rows and `esp_psram` log wording. |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Captured monitor output with non-interactive espflash**
- **Found during:** Task 3 (Establish DEVICE_URL or record blocker)
- **Issue:** `just flash-monitor ... evidence-dir=...` reached the monitor step but `espflash monitor --port /dev/cu.usbmodem1101` exited with `Failed to initialize input reader` in this non-interactive execution environment.
- **Fix:** Used the same detected port with a bounded `espflash monitor --chip esp32s3 --port /dev/cu.usbmodem1101 --non-interactive` capture into the plan-owned evidence directory.
- **Files modified:** `docs/parity/evidence/phase-08-ultra-205-release-gate.md`, `docs/parity/evidence/phase-08-ultra-205-release-gate/flash-monitor-noninteractive.log`
- **Verification:** Branch-aware `DEVICE_URL` check passed; log scan found no URL or credential leakage.
- **Committed in:** `8e8f460`

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** The workaround preserved the plan's safety boundary: no HTTP, OTA, erase, or interruption commands ran without a reachable URL.

## Issues Encountered

- `DEVICE_URL` remains blocked. The serial log has route registration and boot evidence, but no IP, DHCP, Wi-Fi association, AP address, mDNS, hostname, or operator-supplied reachable URL.
- `just flash-monitor` wrote its failed monitor log under Bazel runfiles, not the repo evidence directory. The committed log is from the bounded non-interactive monitor capture.

## Known Stubs

None. Stub scan found no placeholder/TODO/FIXME patterns in the created evidence files.

## Threat Flags

None beyond the plan's documented trust boundaries. This plan adds evidence files only; it introduces no endpoint, auth path, file-access adapter, firmware effect, or schema change.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Plan 08-03 should treat live HTTP/OTA/recovery probes as blocked until a reachable `DEVICE_URL` is established or a later plan adds a documented network setup path. No checklist rows were promoted by this plan.

## Self-Check: PASSED

- Found evidence ledger: `docs/parity/evidence/phase-08-ultra-205-release-gate.md`
- Found monitor log: `docs/parity/evidence/phase-08-ultra-205-release-gate/flash-monitor-noninteractive.log`
- Found summary file: `.planning/phases/08-parity-evidence-and-ultra-205-release-gate/08-02-SUMMARY.md`
- Found task commits: `11374c3`, `a0f544b`, and `8e8f460`
- Stub scan found no blocking stubs.
- Threat surface scan found no unplanned security-relevant surfaces.

***
*Phase: 08-parity-evidence-and-ultra-205-release-gate*
*Completed: 2026-06-28*
