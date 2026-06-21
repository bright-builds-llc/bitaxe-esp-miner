---
phase: 01-foundation-and-gamma-601-boot-log
plan: "04"
subsystem: foundation
tags: [rust, cargo, bazel, pure-crates, deferred-status, asic, stratum, api]
requires:
  - "01-03 foundational pure crates"
provides:
  - "Pure bitaxe-asic crate with DeferredUntilPhase3 status"
  - "Pure bitaxe-stratum crate with DeferredUntilPhase4 status"
  - "Pure bitaxe-api crate with DeferredUntilPhase5 status"
  - "Cargo.lock coverage for all Phase 1 pure crates created so far"
affects: [foundation, firmware, parity, asic, stratum, api]
tech-stack:
  added: [bitaxe-asic, bitaxe-stratum, bitaxe-api]
  patterns:
    - "Deferred Phase 1 surfaces are represented as explicit status enums"
    - "Each pure crate exposes a Cargo package and matching Bazel rust_test target"
key-files:
  created:
    - crates/bitaxe-asic/Cargo.toml
    - crates/bitaxe-asic/BUILD.bazel
    - crates/bitaxe-asic/src/lib.rs
    - crates/bitaxe-stratum/Cargo.toml
    - crates/bitaxe-stratum/BUILD.bazel
    - crates/bitaxe-stratum/src/lib.rs
    - crates/bitaxe-api/Cargo.toml
    - crates/bitaxe-api/BUILD.bazel
    - crates/bitaxe-api/src/lib.rs
  modified:
    - Cargo.toml
    - Cargo.lock
key-decisions:
  - "Represent deferred ASIC, Stratum, and API surfaces as explicit single-variant enums instead of empty modules or active skeletons."
  - "Keep Phase 1 deferred surface crates dependency-free and side-effect-free; later phases add behavior with evidence."
  - "Honor AGENTS.md Rust pre-commit rule by recording TDD RED failures without committing failing intermediate states."
patterns-established:
  - "Deferred pure crates expose only a status enum plus focused unit tests until their owning parity phase begins."
requirements-completed: [FND-05, FND-10]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 1-2026-06-21T00-30-20
generated_at: 2026-06-21T02:41:41Z
duration: 4 min
completed: 2026-06-21
---

# Phase 01 Plan 04: Deferred Pure Crate Contracts Summary

**ASIC, Stratum, and API pure crates with explicit deferred runtime status contracts**

## Performance

- **Duration:** 4 min
- **Started:** 2026-06-21T02:36:59Z
- **Completed:** 2026-06-21T02:41:41Z
- **Tasks:** 2
- **Files modified:** 11

## Accomplishments

- Added `bitaxe-asic` with `AsicRuntimeStatus::DeferredUntilPhase3` and focused unit/Bazel tests.
- Added `bitaxe-stratum` with `StratumRuntimeStatus::DeferredUntilPhase4` and focused unit/Bazel tests.
- Added `bitaxe-api` with `ApiRuntimeStatus::DeferredUntilPhase5` and focused unit/Bazel tests.
- Updated Cargo workspace metadata and `Cargo.lock` for all pure crate members created through this plan.

## Task Commits

Each task was committed atomically:

1. **Task 1: Create ASIC and Stratum deferred-status crates** - `a49578a` (feat)
2. **Task 2: Create API deferred-status crate and refresh lockfile** - `c95aa98` (feat)

## Files Created/Modified

- `Cargo.toml` - Adds `bitaxe-api`, `bitaxe-asic`, and `bitaxe-stratum` workspace members.
- `Cargo.lock` - Locks the new local pure crate packages.
- `crates/bitaxe-asic/Cargo.toml` - Defines the pure `bitaxe-asic` package.
- `crates/bitaxe-asic/BUILD.bazel` - Adds Bazel library and test targets for `bitaxe-asic`.
- `crates/bitaxe-asic/src/lib.rs` - Defines the Phase 3 deferred ASIC runtime status contract and test.
- `crates/bitaxe-stratum/Cargo.toml` - Defines the pure `bitaxe-stratum` package.
- `crates/bitaxe-stratum/BUILD.bazel` - Adds Bazel library and test targets for `bitaxe-stratum`.
- `crates/bitaxe-stratum/src/lib.rs` - Defines the Phase 4 deferred Stratum runtime status contract and test.
- `crates/bitaxe-api/Cargo.toml` - Defines the pure `bitaxe-api` package.
- `crates/bitaxe-api/BUILD.bazel` - Adds Bazel library and test targets for `bitaxe-api`.
- `crates/bitaxe-api/src/lib.rs` - Defines the Phase 5 deferred API runtime status contract and test.

## Decisions Made

- Deferred surfaces use explicit enum variants, not empty crates, so downstream code can report planned status without implying active behavior.
- The new crates stay dependency-free in Phase 1. ESP-IDF, transport, serving, mining, and hardware control behavior remain later-phase work.
- TDD RED failures were captured by test runs, but not committed as failing revisions because the repo-level Rust commit rule requires format, Clippy, build, and tests to pass before every commit.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Preserved Rust passing-commit rule during TDD**
- **Found during:** Task 1 and Task 2
- **Issue:** The plan's `tdd="true"` flow called for committing failing RED tests, but `AGENTS.md` requires `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features` to pass before every Rust-project commit.
- **Fix:** Wrote RED tests and ran them to confirm the expected unresolved API failures, then committed only the passing task states after GREEN implementation and verification.
- **Files modified:** None beyond the planned task files.
- **Verification:** RED failures were observed for unresolved `AsicRuntimeStatus`, `StratumRuntimeStatus`, and `ApiRuntimeStatus`; both task commits passed the Rust pre-commit sequence before commit.
- **Committed in:** `a49578a`, `c95aa98`

**2. [Rule 3 - Blocking] Kept Cargo.lock reproducible across task commits**
- **Found during:** Task 1
- **Issue:** Running Cargo verification after adding `bitaxe-asic` and `bitaxe-stratum` updated `Cargo.lock` before Task 2's planned final lockfile refresh.
- **Fix:** Committed the Task 1 lockfile entries with the Task 1 workspace change, then ran `cargo generate-lockfile` again in Task 2 after adding `bitaxe-api`.
- **Files modified:** `Cargo.lock`
- **Verification:** `cargo metadata --format-version=1 --no-deps` resolved all workspace members after Task 2.
- **Committed in:** `a49578a`, `c95aa98`

---

**Total deviations:** 2 auto-fixed (1 missing critical, 1 blocking)
**Impact on plan:** The deviations preserved repo-level commit guarantees and kept each committed workspace state reproducible. No active ASIC, Stratum, or API behavior was added.

## Issues Encountered

None beyond the deviations documented above.

## User Setup Required

None - no external service configuration required.

## Verification

Passed:

- Lifecycle validation: `node /Users/peterryszkiewicz/.codex/get-shit-done/bin/gsd-tools.cjs verify lifecycle 01 --require-plans --raw`
- `cargo test -p bitaxe-asic -p bitaxe-stratum`
- `bazel test //crates/bitaxe-asic:tests //crates/bitaxe-stratum:tests`
- `cargo metadata --format-version=1 --no-deps`
- `cargo test -p bitaxe-api`
- `bazel test //crates/bitaxe-api:tests`
- `cargo fmt --all`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo build --all-targets --all-features`
- `cargo test --all-features`
- `cargo test -p bitaxe-asic -p bitaxe-stratum -p bitaxe-api`
- `bazel test //crates/bitaxe-asic:tests //crates/bitaxe-stratum:tests //crates/bitaxe-api:tests`

Acceptance checks passed:

- `grep -R "DeferredUntilPhase3" crates/bitaxe-asic/src/lib.rs`
- `grep -R "DeferredUntilPhase4" crates/bitaxe-stratum/src/lib.rs`
- `grep -R "DeferredUntilPhase5" crates/bitaxe-api/src/lib.rs`
- `grep -R "esp-idf" crates/bitaxe-asic/Cargo.toml crates/bitaxe-stratum/Cargo.toml crates/bitaxe-api/Cargo.toml` returned no matches.
- `grep -q 'crates/bitaxe-api' Cargo.toml`
- `test -f Cargo.lock`
- `rg 'init|send_work|socket|mine|voltage|fan|thermal|power|route|WebSocket|websocket|settings|OTA|static asset|static' crates/bitaxe-asic/src crates/bitaxe-stratum/src crates/bitaxe-api/src` returned no active behavior matches.

## Known Stubs

None.

## Next Phase Readiness

Ready for `01-05-PLAN.md`. The planned pure crate package surface now exists for core, config, test-support, ASIC, Stratum, and API contracts, while active mining, transport, hardware control, HTTP, OTA, and web asset behavior remain deferred to their owning later phases.

---

*Phase: 01-foundation-and-gamma-601-boot-log*
*Completed: 2026-06-21*

## Self-Check: PASSED

- Verified created summary and key crate files exist on disk.
- Verified task commits exist: `a49578a`, `c95aa98`.
