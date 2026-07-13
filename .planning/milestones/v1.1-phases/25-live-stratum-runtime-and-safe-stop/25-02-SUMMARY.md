---
phase: 25-live-stratum-runtime-and-safe-stop
plan: 02
subsystem: firmware
tags: [rust, esp-idf, stratum-v1, tcp, safe-stop, watchdog, redaction, bazel]
requires:
  - phase: 25-live-stratum-runtime-and-safe-stop
    provides: Pure LiveStratumRuntime, SubmitIntent-tied response classifier, and deterministic fake-pool coverage from plan 25-01
provides:
  - Phase 25 compile-time live Stratum evidence mode and acknowledgment gate
  - Firmware-owned TcpStream adapter with prerequisite gating before settings or socket access
  - Phase 25 safe-stop snapshot refresh and watchdog step categories for socket, ASIC, API, WebSocket, and evidence-capture work
affects: [phase-25-live-stratum-runtime, phase-26-telemetry-and-parity, firmware-bitaxe, bitaxe-safety]
tech-stack:
  added: []
  patterns:
    - compile-time evidence mode gate
    - firmware socket adapter around pure Stratum runtime
    - named Phase 25 post-stop snapshot wrapper
    - watchdog category expansion under existing thresholds
key-files:
  created:
    - firmware/bitaxe/src/live_stratum_runtime.rs
    - .planning/phases/25-live-stratum-runtime-and-safe-stop/25-02-SUMMARY.md
  modified:
    - firmware/bitaxe/src/mining_evidence_mode.rs
    - firmware/bitaxe/src/main.rs
    - firmware/bitaxe/src/controlled_mining_runtime.rs
    - firmware/bitaxe/src/runtime_snapshot.rs
    - crates/bitaxe-safety/src/watchdog.rs
key-decisions:
  - "Kept Phase 25 live Stratum startup behind a distinct compile-time mode and acknowledgment pair so Phase 21 controlled evidence cannot start the socket path."
  - "Evaluated typed Phase 22 production-mining preconditions before NVS pool settings access or TcpStream construction."
  - "Used a named Phase 25 snapshot helper for safe-stop refresh without adding Phase 26 statistics or scoreboard semantics."
patterns-established:
  - "Firmware socket shell owns TcpStream connect/read/write/shutdown while message parsing and lifecycle decisions stay in crates/bitaxe-stratum."
  - "Safe stop converges on stable category labels and a phase25_safe_stop blocked runtime state."
  - "Watchdog proof reuses StepSupervisor thresholds for Phase 25 load categories."
requirements-completed: [STR-08, STR-09, SAFE-12, SAFE-13]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 25-2026-07-05T01-55-45
generated_at: 2026-07-05T02:27:15Z
duration: 5min 22s
completed: 2026-07-05
---

# Phase 25 Plan 02: Firmware Live Socket Adapter and Safe Stop Summary

**Gated ESP-IDF TcpStream shell with redaction-safe Stratum runtime startup, bounded safe stop, and watchdog category proof**

## Performance

- **Duration:** 5min 22s
- **Started:** 2026-07-05T02:21:53Z
- **Completed:** 2026-07-05T02:27:15Z
- **Tasks:** 2 completed
- **Files modified:** 7

## Accomplishments

- Added a distinct `Phase25LiveStratumRuntime` compile-time mode and acknowledgment gate while preserving the Phase 21 controlled runtime path.
- Added `firmware/bitaxe/src/live_stratum_runtime.rs`, a thin firmware adapter that checks production-mining readiness before NVS pool settings access or `TcpStream` construction, uses 100 ms socket read/write timeouts, drives `LiveStratumRuntime`, parses server JSON lines through `parse_server_message`, serializes client messages with `to_json_line`, and shuts down sockets through `Shutdown::Both`.
- Added Phase 25 safe-stop convergence with stable redaction-safe markers, `phase25_safe_stop` post-stop runtime state, a named snapshot helper, and watchdog categories for `Socket`, `Asic`, `Api`, `WebSocket`, and `EvidenceCapture`.

## Task Commits

Each task was committed atomically:

1. **Task 25-02-01: Add firmware live socket adapter** - `bd4cf7c` (`feat`)
2. **Task 25-02-02: Implement safe-stop snapshot and watchdog proof** - `4ff7581` (`feat`)

## Files Created/Modified

- `firmware/bitaxe/src/live_stratum_runtime.rs` - Phase 25 firmware socket shell, prerequisite/settings gates, category-only lifecycle markers, safe-stop convergence, and module tests.
- `firmware/bitaxe/src/mining_evidence_mode.rs` - Distinct Phase 25 mode constants, enum variant, and fail-closed tests.
- `firmware/bitaxe/src/main.rs` - Phase 25 starter is called only after the Wi-Fi/network setup path reports success or a category failure.
- `firmware/bitaxe/src/controlled_mining_runtime.rs` - Phase 21 controlled runtime ignores the Phase 25 mode instead of matching it as Phase 21.
- `firmware/bitaxe/src/runtime_snapshot.rs` - Named post-stop snapshot helper for Phase 25.
- `crates/bitaxe-safety/src/watchdog.rs` - Phase 25 watchdog step kinds and threshold tests.

## Decisions Made

- Kept production firmware fail-closed by making the default Phase 25 firmware preconditions blocked until live safety prerequisites are supplied by a later detector-gated evidence path.
- Used category-only retained logs for lifecycle, prerequisite, watchdog, and safe-stop markers; no raw pool endpoint, user, password, socket detail, share payload, or BM1366 frame is retained.
- Limited API/WebSocket-visible behavior to replacing `MiningRuntimeState` after safe stop; no Phase 26 statistics, scoreboard, or share-counter projection was added.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Handled the new Phase 25 mode in the Phase 21 controlled runtime match**
- **Found during:** Task 25-02-01 (Add firmware live socket adapter)
- **Issue:** Adding `Phase25LiveStratumRuntime` made the existing `MiningEvidenceMode` match in `controlled_mining_runtime.rs` non-exhaustive.
- **Fix:** Added an explicit Phase 25 arm that does nothing in the Phase 21 controlled runtime path.
- **Files modified:** `firmware/bitaxe/src/controlled_mining_runtime.rs`
- **Verification:** `bazel build //firmware/bitaxe:firmware`
- **Committed in:** `bd4cf7c`

**2. [Rule 1 - Bug] Cloned runtime state before snapshot replacement**
- **Found during:** Task 25-02-01 (Add firmware live socket adapter)
- **Issue:** The first safe-stop snapshot update attempted to move `MiningRuntimeState` out of a shared runtime reference.
- **Fix:** Cloned the runtime state before snapshot replacement, then refined the path in Task 25-02-02 to use `replace_mining_runtime_state_after_phase25_safe_stop`.
- **Files modified:** `firmware/bitaxe/src/live_stratum_runtime.rs`
- **Verification:** `bazel build //firmware/bitaxe:firmware`
- **Committed in:** `bd4cf7c`, refined in `4ff7581`

**Total deviations:** 2 auto-fixed bugs
**Impact on plan:** Both fixes were required for the planned firmware adapter to compile and remain scoped to Phase 25 mode/safe-stop behavior.

## Issues Encountered

- The firmware app does not expose a separate Bazel `rust_test` target, so the adapter tests are module-local and the enforceable task verification was the planned `bazel build //firmware/bitaxe:firmware` plus pure crate tests.
- The Task 25-02-01 forbidden scan over `main.rs` has a pre-existing broad-pattern false positive on the boot marker `rust_target`; the final plan-level scan over the Phase 25 adapter and runtime snapshot files passed.

## Known Stubs

None. The stub scan only matched normal Rust `format!` placeholders in existing boot/status logging.

## Threat Flags

None. The new socket/settings/stop/watchdog trust surfaces are covered by plan threats T-25-01 through T-25-05.

## Verification

- `bazel test //crates/bitaxe-safety:tests //crates/bitaxe-stratum:tests`
- `bazel build //firmware/bitaxe:firmware`
- `rg "phase25_live_stratum_status|phase25_safe_stop_status" firmware/bitaxe/src/live_stratum_runtime.rs`
- Forbidden-token scan returned no matches across `firmware/bitaxe/src/live_stratum_runtime.rs` and `firmware/bitaxe/src/runtime_snapshot.rs`.
- `git diff --check`

## Auth Gates

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Plan 25-03 can add the detector-gated evidence wrapper, mining allow-manifest updates, redaction review, and checklist closure around the Phase 25 live-or-blocked runtime path. Live accepted/rejected STR-09 proof still depends on Ultra 205 detection, runtime-only local pool credentials, safe prerequisites, and redacted hardware evidence.

## Self-Check: PASSED

- Found created files: `firmware/bitaxe/src/live_stratum_runtime.rs` and `25-02-SUMMARY.md`.
- Found task commits: `bd4cf7c` and `4ff7581`.

*Phase: 25-live-stratum-runtime-and-safe-stop*
*Completed: 2026-07-05*
