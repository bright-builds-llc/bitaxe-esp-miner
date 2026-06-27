---
phase: 05-axeos-api-logs-and-telemetry
plan: 03
subsystem: api
tags: [rust, serde-json, axeos, statistics, scoreboard, mining-state]

requires:
  - phase: 05-axeos-api-logs-and-telemetry
    provides: "Plan 05-01 API wire DTO foundation and safe Ultra 205 fixtures"
  - phase: 05-axeos-api-logs-and-telemetry
    provides: "Plan 05-02 settings parser and persistence contract"
  - phase: 04-stratum-v1-and-first-mining-loop
    provides: "MiningRuntimeState and fail-closed mining-loop status"
provides:
  - "Pure system info, ASIC settings, mining-state, statistics, and scoreboard response mappers"
  - "Mining-derived system fields for shares, rejected reasons, pool difficulty, fallback status, hashrate, and safe blocked status"
  - "Empty statistics and scoreboard fixtures for compatible no-history/no-scoreboard states"
affects:
  - 05-04-logs-and-live-telemetry
  - 05-05-firmware-route-websocket-settings-log-adapters
  - 05-07-api-compare-and-static-evidence
  - phase-06-safety

tech-stack:
  added: []
  patterns:
    - "Route response mappers live in focused pure modules and feed handwritten AxeOS wire DTOs"
    - "Empty runtime history serializes as compatible empty arrays rather than fabricated telemetry"

key-files:
  created:
    - crates/bitaxe-api/src/system.rs
    - crates/bitaxe-api/src/asic.rs
    - crates/bitaxe-api/src/mining.rs
    - crates/bitaxe-api/src/statistics.rs
    - crates/bitaxe-api/src/scoreboard.rs
    - crates/bitaxe-api/fixtures/api/statistics-empty-compatible.json
    - crates/bitaxe-api/fixtures/api/scoreboard-empty.json
  modified:
    - crates/bitaxe-api/BUILD.bazel
    - crates/bitaxe-api/src/lib.rs
    - crates/bitaxe-api/src/wire.rs
    - crates/bitaxe-api/fixtures/api/system-info-ultra205-safe.json

key-decisions:
  - "System, ASIC, mining, statistics, and scoreboard mappers stay pure and do not introduce ESP-IDF, HTTP, NVS, file, or hardware effects."
  - "Mining-visible share counters, rejected reasons, pool difficulty, fallback state, hashrate, and blocked status derive from MiningRuntimeState."
  - "Statistics and scoreboard empty states are explicit compatible response shapes, not fake historical mining data."

patterns-established:
  - "MiningStateWire is the shared mining projection consumed by system info and future telemetry mappers."
  - "Statistics column selection follows upstream label keys and always appends timestamp."
  - "Scoreboard response entries omit server-side rank and since because AxeOS derives those client-side."

requirements-completed: [API-02, API-04]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 5-2026-06-27T18-02-49
generated_at: 2026-06-27T20:27:00Z

duration: 12 min
completed: 2026-06-27
---

# Phase 05 Plan 03: System, ASIC, Statistics, Scoreboard, And Mining Summary

**Pure AxeOS read-response mappers backed by ApiSnapshot, MiningRuntimeState, catalog data, and explicit empty fixtures**

## Performance

- **Duration:** 12 min
- **Started:** 2026-06-27T20:14:52Z
- **Completed:** 2026-06-27T20:27:00Z
- **Tasks:** 2
- **Files modified:** 11

## Accomplishments

- Added pure `system`, `asic`, and `mining` modules, including mining-derived rejected-reason aggregation and fail-closed blocked-state visibility.
- Extended system-info wire coverage for rejected reasons, best difficulty, pool connection status, response timing placeholders, and catalog-derived power/core facts.
- Added pure `statistics` and `scoreboard` modules with empty-compatible fixtures and tests for optional columns, timestamp handling, and scoreboard field ownership.

## Task Commits

1. **Task 1: Map system info, ASIC settings, and mining state from ApiSnapshot** - `1f558c2` (feat)
2. **Task 2: Implement statistics and scoreboard response shapes** - `65b8642` (feat)

## Files Created/Modified

- `crates/bitaxe-api/src/system.rs` - Pure `ApiSnapshot` to system-info response entry point and mining-value coverage tests.
- `crates/bitaxe-api/src/asic.rs` - Pure catalog-backed ASIC settings response entry point and Ultra 205 option tests.
- `crates/bitaxe-api/src/mining.rs` - `MiningStateWire`, rejected-reason aggregation, lifecycle labels, fallback status, and safe blocked-state mapping.
- `crates/bitaxe-api/src/statistics.rs` - Statistics labels, optional column selection, timestamp appending, empty history handling, and row projection.
- `crates/bitaxe-api/src/scoreboard.rs` - Scoreboard typed input and six-field upstream wire array mapper.
- `crates/bitaxe-api/fixtures/api/statistics-empty-compatible.json` - Empty statistics response fixture with full labels and no fake rows.
- `crates/bitaxe-api/fixtures/api/scoreboard-empty.json` - Empty scoreboard array fixture.
- `crates/bitaxe-api/src/wire.rs`, `src/lib.rs`, `BUILD.bazel`, and `fixtures/api/system-info-ultra205-safe.json` - Exports, Bazel wiring, and system DTO/fixture coverage.

## Decisions Made

- Kept response construction in pure Rust crate modules; firmware route, WebSocket, and hardware adapters remain future plan work.
- Inferred `hardware_evidence_ack_missing` for a `SafeBlocked` mining state with blocked work submission so the safe state remains visible without unlocking work.
- Used upstream statistics label keys and an empty `statistics` array for unavailable history; no hashrate, fan, power, or temperature history is fabricated.
- Kept scoreboard output to `difficulty`, `job_id`, `extranonce2`, `ntime`, `nonce`, and `version_bits` only.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Extended existing system wire DTO and safe fixture**
- **Found during:** Task 1
- **Issue:** The new system mapper needed to expose mining-derived rejected reasons, best difficulty, fallback/pool status, and timing fields through the existing `SystemInfoWire`; adding only wrapper modules would have left the public DTO unable to carry required runtime values.
- **Fix:** Added the missing upstream-compatible fields to `SystemInfoWire` and updated the safe Ultra 205 system-info fixture.
- **Files modified:** `crates/bitaxe-api/src/wire.rs`, `crates/bitaxe-api/fixtures/api/system-info-ultra205-safe.json`
- **Verification:** `bazel test //crates/bitaxe-api:tests --test_filter='system|asic|mining'`, full Rust commit gate, `just test`
- **Committed in:** `1f558c2`

### Process Adjustments

**AGENTS.md precedence over TDD RED commits**
- **Found during:** Task 1 and Task 2
- **Issue:** The plan requested TDD RED commits, but repo instructions require `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features` to pass before every Rust-project commit.
- **Adjustment:** Ran failing RED tests for both tasks and committed only the passing GREEN implementations after the required Rust gate.
- **Impact:** Preserved the higher-priority repo rule without changing feature scope.

---

**Total deviations:** 1 auto-fixed (1 missing critical), 1 process adjustment.
**Impact on plan:** The auto-fix completed the planned system mapper contract without adding route, firmware, hardware-control, or persistence scope.

## Issues Encountered

None beyond the documented deviation and process adjustment.

## Known Stubs

None blocking. The empty arrays in `statistics-empty-compatible.json`, `scoreboard-empty.json`, and `sharesRejectedReasons` are intentional compatible empty states for no statistics history, no scoreboard entries, and no rejected shares. Safe zero hardware telemetry remains Phase 6-owned and unchanged from Plan 05-01.

## Threat Flags

None. The new modules are pure response mappers and fixtures for planned API surfaces; they introduce no new network endpoint, authentication path, file access pattern, schema migration, or hardware-control effect.

## Authentication Gates

None.

## User Setup Required

None - no external service configuration required.

## Verification

- `bazel test //crates/bitaxe-api:tests --test_filter='system|asic|mining'` - failed in RED, then passed
- `bazel test //crates/bitaxe-api:tests //crates/bitaxe-stratum:tests //crates/bitaxe-asic:tests` - passed
- `bazel test //crates/bitaxe-api:tests --test_filter='statistics|scoreboard'` - failed in RED, then passed
- `bazel test //crates/bitaxe-api:tests` - passed
- `cargo fmt --all` - passed before each task commit
- `cargo clippy --all-targets --all-features -- -D warnings` - passed before each task commit
- `cargo build --all-targets --all-features` - passed before each task commit
- `cargo test --all-features` - passed before each task commit
- `just test` - passed
- `git status --short reference/esp-miner` - clean output

## Next Phase Readiness

Ready for Plan 05-04. The API crate now has pure read-response mappers for system, ASIC, mining, statistics, and scoreboard surfaces that later telemetry, route, and comparison plans can reuse without duplicating runtime state models.

## Self-Check: PASSED

- Found `.planning/phases/05-axeos-api-logs-and-telemetry/05-03-SUMMARY.md`
- Found `crates/bitaxe-api/src/system.rs`
- Found `crates/bitaxe-api/src/asic.rs`
- Found `crates/bitaxe-api/src/mining.rs`
- Found `crates/bitaxe-api/src/statistics.rs`
- Found `crates/bitaxe-api/src/scoreboard.rs`
- Found `crates/bitaxe-api/fixtures/api/statistics-empty-compatible.json`
- Found `crates/bitaxe-api/fixtures/api/scoreboard-empty.json`
- Found task commit `1f558c2`
- Found task commit `65b8642`
- Reference implementation remains unmodified.

---
*Phase: 05-axeos-api-logs-and-telemetry*
*Completed: 2026-06-27*
