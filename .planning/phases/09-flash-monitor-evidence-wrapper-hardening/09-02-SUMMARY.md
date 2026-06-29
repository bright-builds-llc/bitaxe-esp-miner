---
phase: 09-flash-monitor-evidence-wrapper-hardening
plan: 02
subsystem: tooling
tags: [rust, espflash, evidence, flash-monitor, docs, hardware]
requires:
  - phase: 09-flash-monitor-evidence-wrapper-hardening
    provides: Plan 09-01 wrapper-owned noninteractive flash-monitor capture contract.
provides:
  - Fresh Ultra 205 wrapper-owned serial evidence captured through `just flash-monitor`.
  - Phase 9 machine-readable flash-monitor evidence JSON and serial log artifacts.
  - Fail-closed monitor trust gates for exact safe-state markers and observed boot commit identity.
  - WF-005 checklist citation to wrapper-produced evidence.
  - Ultra 205 release-guide recovery guidance for fail-closed monitor evidence capture.
affects: [flash-monitor, evidence, release-gate, parity, ultra-205]
tech-stack:
  added: []
  patterns:
    - Workspace-anchored evidence paths for repo-owned commands launched through Bazel.
    - Observed serial firmware/reference identity must match expected source/reference commit prefixes before output is trusted.
    - Serial-scope evidence ledgers that explicitly preserve unsupported HTTP, OTA, recovery, and rollback boundaries.
key-files:
  created:
    - docs/parity/evidence/phase-09-flash-monitor-evidence-wrapper-hardening.md
    - docs/parity/evidence/phase-09-flash-monitor-evidence-wrapper-hardening/flash-command-evidence.json
    - docs/parity/evidence/phase-09-flash-monitor-evidence-wrapper-hardening/flash-monitor.log
    - .planning/phases/09-flash-monitor-evidence-wrapper-hardening/09-02-SUMMARY.md
  modified:
    - tools/flash/src/main.rs
    - docs/parity/checklist.md
    - docs/release/ultra-205.md
key-decisions:
  - "Phase 9 proof now uses wrapper-produced JSON and serial logs; raw espflash monitor output remains historical only."
  - "Relative evidence directories are resolved against the repository workspace so Bazel-run commands write committed evidence where operators requested it."
  - "WF-005 can cite wrapper evidence, but FS-001, OTA-001, OTA-002, REL-001, REL-002, and REL-003 remain below verified release parity."
patterns-established:
  - "Hardware evidence ledgers record detector gate, board-info summary, JSON contract, log markers, scope boundary, secret review, and final conclusion."
  - "Operator recovery docs distinguish trusted wrapper evidence from diagnostic-only monitor output."
requirements-completed: [FND-07, FND-08, REL-07, EVD-05]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: "09-2026-06-29T13-16-47"
generated_at: 2026-06-29T14:30:11Z
duration: 17 min
completed: 2026-06-29
---

# Phase 09 Plan 02: Wrapper-Owned Ultra 205 Evidence Summary

**Ultra 205 flash-monitor evidence captured through the repo wrapper, with checklist and release docs pointing to trusted JSON/log artifacts**

## Performance

- **Duration:** 17 min
- **Started:** 2026-06-29T14:08:40Z
- **Completed:** 2026-06-29T14:25:48Z
- **Tasks:** 2
- **Files modified:** 6

## Accomplishments

- Ran the Ultra 205 detector gate and captured fresh flash-monitor evidence through `just flash-monitor board=205 port=/dev/cu.usbmodem1101 evidence-dir=docs/parity/evidence/phase-09-flash-monitor-evidence-wrapper-hardening`.
- Committed wrapper-generated `flash-command-evidence.json` and `flash-monitor.log` with noninteractive capture status `timed_out_after_trusted_output` and all seven trusted serial boot markers.
- Added a human-readable Phase 9 evidence ledger with run identity, detector output, JSON fields, monitor markers, scope boundaries, secret review, and final conclusion.
- Tightened the wrapper trust predicate after code review so future captures require the exact safe no-op state, reset/ESP-IDF provenance, and 12-character-or-longer observed commit prefixes matching the expected source/reference commits.
- Updated WF-005 to cite Phase 9 wrapper evidence while keeping HTTP, static, recovery, OTA, rollback, and release-sensitive rows below verified.
- Updated the Ultra 205 release guide with the wrapper command, timeout override, trusted status values, fail-closed wording, and diagnostic-only recovery flow.

## Task Commits

Each task was committed atomically:

1. **Task 1: Capture wrapper-owned Ultra 205 evidence** - `b611415` (feat)
2. **Task 2: Refresh checklist and release docs** - `1575187` (docs)
3. **Code review follow-up: Validate monitor evidence identity** - `74dff26` (fix)
4. **Code review follow-up: Require stable commit marker prefixes** - `ef7c185` (fix)
5. **Code review follow-up: Anchor monitor evidence markers** - `7b0d2cd` (fix)
6. **Code review follow-up: Require exact trusted boot markers** - `424b8ec` (fix)

## Files Created/Modified

- `docs/parity/evidence/phase-09-flash-monitor-evidence-wrapper-hardening.md` - Records the Phase 9 hardware run, detector gate, evidence artifacts, trusted markers, secret review, and serial-scope conclusion.
- `docs/parity/evidence/phase-09-flash-monitor-evidence-wrapper-hardening/flash-command-evidence.json` - Stores the wrapper-produced machine-readable evidence contract.
- `docs/parity/evidence/phase-09-flash-monitor-evidence-wrapper-hardening/flash-monitor.log` - Stores the wrapper-owned noninteractive serial capture.
- `tools/flash/src/main.rs` - Anchors relative evidence paths under the workspace when launched through Bazel, verifies exact safe-state/provenance markers, rejects stale or truncated observed boot commits, and adds regression tests.
- `docs/parity/checklist.md` - Updates WF-005 with Phase 9 wrapper evidence citations.
- `docs/release/ultra-205.md` - Documents trusted wrapper evidence capture, timeout, statuses, fail-closed guidance, and recovery commands.
- `.planning/phases/09-flash-monitor-evidence-wrapper-hardening/09-02-SUMMARY.md` - Plan execution summary.

## Decisions Made

- Treated `timed_out_after_trusted_output` as passing evidence because the wrapper captured every trusted Phase 9 boot marker before stopping the continuous monitor.
- Kept raw `espflash monitor` as diagnostic-only language in docs; the trusted Phase 9 proof is the wrapper-produced JSON/log pair.
- Preserved the release boundary: serial boot evidence does not verify live HTTP, static assets, recovery UI, firmware OTA, invalid image rejection, rollback, failed update recovery, large erase, interrupted update, or OTAWWW behavior.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Anchored relative evidence paths under the workspace**
- **Found during:** Task 1 (capture wrapper-owned Ultra 205 evidence)
- **Issue:** `bazel run` executes the flash tool outside the repository root, so a relative `evidence-dir=docs/parity/evidence/...` could write generated JSON/log artifacts outside the workspace.
- **Fix:** Added `FlashEnvironment::workspace_path`, resolved evidence directories through it, preserved user-facing recovery guidance paths, and added `relative_evidence_dir_writes_under_workspace_dir`.
- **Files modified:** `tools/flash/src/main.rs`
- **Verification:** Wrapper-generated Phase 9 JSON and log landed under `docs/parity/evidence/phase-09-flash-monitor-evidence-wrapper-hardening/`; Rust and parity verification were run during plan execution.
- **Committed in:** `b611415`

**2. [Rule 3 - Blocking] Tightened trusted monitor marker and boot identity checks**
- **Found during:** Code review gate after Task 2
- **Issue:** The original trusted-output predicate accepted loose `safe_state:` text and did not compare the captured `firmware_commit=` or `reference_commit=` markers against the expected source/reference commits.
- **Fix:** Required exact safe no-op state plus reset and ESP-IDF provenance markers, parsed observed boot identity into the JSON record, and rejected captures whose observed source/reference prefixes do not match the expected commits.
- **Files modified:** `tools/flash/src/main.rs`
- **Verification:** `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features` passed.
- **Committed in:** `74dff26`

**3. [Rule 3 - Blocking] Required stable commit marker prefix length**
- **Found during:** Second code review pass
- **Issue:** Commit prefix comparison accepted any prefix length, so a one-character observed marker could match the start of `HEAD`.
- **Fix:** Required observed commit markers to be 12+ hexadecimal characters, no longer than the expected full hash, and matching by prefix; added truncated-marker regression coverage.
- **Files modified:** `tools/flash/src/main.rs`
- **Verification:** `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features` passed.
- **Committed in:** `ef7c185`

**4. [Rule 3 - Blocking] Anchored key/value monitor markers to token boundaries**
- **Found during:** Third code review pass
- **Issue:** Key/value marker extraction could match substrings inside larger field names such as `not_firmware_commit=...`.
- **Fix:** Changed monitor marker extraction to split log lines into whitespace-delimited tokens and only accept tokens with the expected marker prefix; added prefixed-marker regression coverage.
- **Files modified:** `tools/flash/src/main.rs`
- **Verification:** `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features` passed.
- **Committed in:** `7b0d2cd`

**5. [Rule 3 - Blocking] Required exact phrase and token trusted boot markers**
- **Found during:** Fourth code review pass
- **Issue:** Non-key trusted markers still used raw substring matching, so misleading strings such as `unsafe_state:` or `not_spiffs_mount=available` could satisfy the trusted evidence predicate.
- **Fix:** Added line-boundary message matching for boot and safe-state phrases, exact-token matching for SPIFFS and route-shell markers, and regression tests for prefixed marker variants.
- **Files modified:** `tools/flash/src/main.rs`
- **Verification:** `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features` passed.
- **Committed in:** `424b8ec`

**Total deviations:** 5 auto-fixed (5 blocking)
**Impact on plan:** The adjustments were required for the planned evidence command to produce committed evidence in the requested repo path and to keep future trusted evidence fail-closed. No parity scope was expanded.

## Issues Encountered

None beyond the auto-fixed issues above.

## User Setup Required

None - no external service configuration required.

## Verification

- `just detect-ultra205` passed with exactly one likely Ultra 205 serial port.
- `just flash-monitor board=205 port=/dev/cu.usbmodem1101 evidence-dir=docs/parity/evidence/phase-09-flash-monitor-evidence-wrapper-hardening` produced `flash-command-evidence.json` and `flash-monitor.log`.
- The serial log contains the required Phase 9 markers: `bitaxe-rust boot: board=Ultra 205 asic=BM1366`, `safe_state:`, `ota_boot_validation=`, `spiffs_mount=available`, `axeos_api_route_shell=started`, `reset_reason=`, `firmware_commit=`, `reference_commit=`, and `esp_idf_version=`.
- Code review follow-up verification passed: `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features`.
- `just parity` passed during documentation refresh.

## Next Phase Readiness

Phase 9 now has wrapper-owned serial proof for the flash-monitor path. Later release work still needs live HTTP, static asset, recovery, OTA, rollback, failed-update recovery, large erase, interrupted-update, and OTAWWW evidence before those rows can be verified.

*Phase: 09-flash-monitor-evidence-wrapper-hardening*
*Completed: 2026-06-29*
