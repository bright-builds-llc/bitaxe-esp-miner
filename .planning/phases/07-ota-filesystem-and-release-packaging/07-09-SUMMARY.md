---
phase: 07-ota-filesystem-and-release-packaging
plan: 09
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 7-2026-06-28T13-30-15
generated_at: 2026-06-28T18:20:13Z
subsystem: parity-release-gates
tags: [parity, ota, release-gate, hardware-evidence, checklist]

requires:
  - phase: 07-08
    provides: Release operator evidence docs and Ultra 205 OTA hardware smoke template
provides:
  - Release and OTA verified-claim guard in parity tooling
  - Phase 7 checklist rows updated with package, workflow, gap, and pending hardware evidence
  - Ultra 205 hardware checkpoint updated with recovered factory-boot evidence and Phase 8 live HTTP/OTA deferral
  - Final automated package, parity, test, and release-gate verification
affects: [phase-08, release-packaging, ota-recovery, parity-evidence]

tech-stack:
  added: []
  patterns:
    - Verified release and OTA rows require evidence classes that match the claim severity.
    - Hardware checkpoint evidence records Phase 7 serial proof separately from Phase 8 live HTTP/OTA/recovery evidence.

key-files:
  created:
    - .planning/phases/07-ota-filesystem-and-release-packaging/07-09-SUMMARY.md
  modified:
    - tools/parity/src/main.rs
    - docs/parity/checklist.md
    - docs/parity/evidence/phase-07-ultra-205-ota-hardware-smoke.md

key-decisions:
  - "Keep release and OTA checklist rows below verified unless the evidence class satisfies the parity guard."
  - "Keep OTAWWW as explicit REL-03 gap until interrupted-update hardware regression evidence exists."
  - "Treat the recovered Ultra 205 run as Phase 7 hardware evidence for factory flash, boot, SPIFFS, and route registration; defer live HTTP/OTA/recovery verification to Phase 8."
  - "Do not commit the failing TDD RED state because the repo Rust pre-commit rule requires passing checks before every commit."

patterns-established:
  - "Claim guards: verified parity rows for release-sensitive behavior must check both evidence tokens and notes."
  - "Hardware reachability handling: serial boot evidence can complete a packaging phase, but checklist rows remain below verified until Phase 8 exercises live HTTP/OTA/recovery behavior."

requirements-completed: [REL-01, REL-02, REL-03, REL-04, REL-05, REL-06, REL-07]
requirements-deferred: [REL-08]
duration: 12m 8s
completed: 2026-06-28
---

# Phase 07 Plan 09: OTA Filesystem And Release Packaging Summary

**Release and OTA parity guards now prevent verified claims without matching hardware, interrupted-update, release-gate, provenance, and package evidence.**

## Performance

- **Duration:** 12m 8s
- **Started:** 2026-06-28T18:08:05Z
- **Completed:** 2026-06-28T18:20:13Z
- **Tasks:** 3
- **Files modified:** 3

## Accomplishments

- Extended the parity report validator so FS, OTA, and release rows cannot be marked `verified` from package/workflow/unit evidence alone.
- Updated Phase 7 checklist rows for `FS-001`, `OTA-001`, `OTA-002`, `REL-001`, `REL-002`, and `REL-003` with implementation pointers and evidence references while keeping unsupported claims below `verified`.
- Updated the Ultra 205 hardware checkpoint after a recovered board run: corrected factory flashing, partition layout, SPIFFS mount, PSRAM, boot validation entry, display startup, safe-state logging, and HTTP route registration were captured on serial evidence.
- Deferred live HTTP, OTA, static-route, recovery, rollback, large-erase, and interrupted-update evidence to Phase 8 because no reachable device URL was exposed by the current firmware run.
- Completed automated verification for Rust tests, Bazel test/package paths, parity validation, and the release gate.

## Hardware Continuation

After the initial no-port checkpoint, the operator recovered the Ultra 205 and authorized hardware verification. The continuation used `/dev/cu.usbmodem1101`, passed `just detect-ultra205`, rebuilt the package, confirmed the dry-run flash path selected `espflash write-bin 0x0 ...bitaxe-ultra205-factory.bin`, and ran `just flash-monitor board=205 port=/dev/cu.usbmodem1101 evidence-dir=docs/parity/evidence/phase-07-ultra-205-ota-hardware-smoke`.

The first continuation passes exposed three firmware startup issues, all fixed before the final capture:

- HTTP server socket configuration exceeded the ESP-IDF maximum after internal sockets, so `max_open_sockets` was reduced from 8 to 7.
- HTTP startup needed ESP-IDF netif and default event-loop initialization before `EspHttpServer::new`.
- The live telemetry thread needed an explicit 16 KiB stack to avoid immediate pthread stack overflow.

The final hardware capture showed the Phase 7 partition table with `www`, `ota_0`, `ota_1`, `otadata`, and `coredump`; `spiffs_mount=available`; `ota_boot_validation=not_pending state=factory`; `psram_status=available`; and `axeos_api_route_shell=started registered_routes=15`. It did not show an IP address, DHCP event, Wi-Fi association, AP address, mDNS name, or other reachable URL, so HTTP request and OTA upload checks are deferred to Phase 8.

## Task Commits

Each task was committed atomically:

1. **Task 1: Extend parity guard for release and OTA evidence** - `6cb1953` (feat)
2. **Task 2: Update Phase 7 checklist rows** - `e32c47f` (docs)
3. **Task 3: Verify live Ultra 205 OTA and recovery behavior** - `1505bca` (docs)

**Plan metadata:** created after this summary self-check in the final docs commit. The hardware continuation evidence was added later after the recovered Ultra 205 became available.

## Files Created/Modified

- `tools/parity/src/main.rs` - Adds release/OTA verified-claim validation and focused regression tests.
- `docs/parity/checklist.md` - Updates Phase 7 release, filesystem, and OTA rows with package/workflow evidence and pending hardware caveats.
- `docs/parity/evidence/phase-07-ultra-205-ota-hardware-smoke.md` - Records automated checkpoint commands, package manifest artifacts, recovered Ultra 205 serial evidence, and the remaining HTTP reachability blocker.
- `.planning/phases/07-ota-filesystem-and-release-packaging/07-09-SUMMARY.md` - Captures plan execution results and verification evidence.

## Decisions Made

- Verified release/OTA claims must be blocked unless the row has the right evidence class and supporting notes for the claim.
- Package and workflow evidence can support `implemented` rows, but not `verified` live OTA, rollback, recovery, or interrupted-update behavior.
- `OTA-002` remains the explicit REL-03 D-16 gap with `deferred` evidence until OTAWWW static update and interrupted-update hardware regression evidence exists.
- The hardware checkpoint passes the narrowed Phase 7 serial scope: corrected factory flash, partition table, SPIFFS, and route registration are proven. Live HTTP/OTA/recovery behavior is assigned to Phase 8 until a reachable device URL exists.

## Deviations from Plan

### Workflow Adjustments

**1. AGENTS.md pre-commit rule overrode the TDD RED commit split**
- **Found during:** Task 1 (Extend parity guard for release and OTA evidence)
- **Issue:** The TDD workflow normally commits the failing RED tests separately, but repo Rust rules require `cargo fmt`, `cargo clippy`, `cargo build`, and `cargo test` to pass before every commit.
- **Fix:** Ran and recorded the failing RED test signal, then committed the tests with the passing implementation in the Task 1 commit.
- **Files modified:** `tools/parity/src/main.rs`
- **Verification:** `cargo test -p bitaxe-parity --all-features release_ota_verified_guard` failed before implementation and passed after implementation.
- **Committed in:** `6cb1953`

***

**Total deviations:** 1 workflow adjustment.
**Impact on plan:** No implementation scope change. The TDD failure signal was preserved in execution evidence while respecting the repo's stricter commit requirements.

## Issues Encountered

- The original Task 3 checkpoint had no Ultra 205 serial port. A later recovered-board continuation completed the narrowed hardware checkpoint: corrected factory flashing and serial boot evidence now pass.
- No reachable HTTP URL was available from logs, known local network state, or user input. Live HTTP, OTA, recovery, rollback, large-erase, and interrupted-update evidence is deferred to Phase 8, and affected checklist rows stay below `verified`.

## Verification

- `node /Users/peterryszkiewicz/.codex/get-shit-done/bin/gsd-tools.cjs verify lifecycle 07 --require-plans` passed before execution.
- `cargo test -p bitaxe-parity --all-features release_ota_verified_guard` passed.
- `cargo fmt --all` passed.
- `cargo clippy --all-targets --all-features -- -D warnings` passed.
- `cargo build --all-targets --all-features` passed.
- `cargo test --all-features` passed.
- `just test` passed.
- `just package` passed.
- `bazel run //tools/parity:report -- release-gate` passed with `release_gate: passed`.
- `just parity` passed with `validation_errors: none`.
- `git diff --check` passed.
- Hardware continuation spot-checks passed: `just detect-ultra205`, `just flash dry-run=true board=205 port=/dev/cu.usbmodem1101`, final `just flash-monitor board=205 port=/dev/cu.usbmodem1101 evidence-dir=docs/parity/evidence/phase-07-ultra-205-ota-hardware-smoke`, `cargo test -p bitaxe-api --all-features update_plan`, `cargo test -p bitaxe-api --all-features static_plan`, and `cargo test -p bitaxe-api --all-features logs`.

## Known Stubs

- `docs/parity/evidence/phase-07-ultra-205-ota-hardware-smoke.md` still records live HTTP/OTA/static/recovery/rollback/large-erase/interrupted-update sections as deferred because no reachable HTTP URL was available.
- `docs/parity/evidence/phase-07-ultra-205-ota-hardware-smoke.md` records `passed for Phase 7 serial scope` in the final conclusion. This is not a live HTTP/OTA verified parity claim.

## Auth Gates

None.

## User Setup Required

Phase 8 live HTTP/OTA verification needs either firmware network bring-up that logs a reachable device address or a user-provided reachable `DEVICE_URL`. Destructive rollback, erase, and interrupted-update checks still need phase-gated recovery instructions and evidence capture.

## Next Phase Readiness

Phase 7 now has automated parity and release-gate safeguards against unsupported release claims plus Ultra 205 serial hardware evidence for the corrected factory image. Follow-on work can consume the checklist safely: package/workflow evidence is recorded, factory boot/SPIFFS/route registration evidence is recorded, live OTA/recovery/rollback/interrupted-update evidence is assigned to Phase 8, and no affected release row is marked `verified` without the missing HTTP evidence.

## Self-Check: PASSED

- Found `.planning/phases/07-ota-filesystem-and-release-packaging/07-09-SUMMARY.md`.
- Found `tools/parity/src/main.rs`.
- Found `docs/parity/checklist.md`.
- Found `docs/parity/evidence/phase-07-ultra-205-ota-hardware-smoke.md`.
- Found task commits `6cb1953`, `e32c47f`, and `1505bca`.

***
*Phase: 07-ota-filesystem-and-release-packaging*
*Completed: 2026-06-28*
