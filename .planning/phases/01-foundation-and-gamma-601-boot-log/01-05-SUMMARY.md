---
phase: 01-foundation-and-gamma-601-boot-log
plan: "05"
subsystem: foundation
tags: [rust, cargo, firmware, host-tools, package-contracts, entrypoints]
requires:
  - "01-04 deferred pure crate contracts"
provides:
  - "bitaxe-firmware package contract in firmware/bitaxe"
  - "bitaxe-flash host tool package contract in tools/flash"
  - "bitaxe-parity host tool package contract in tools/parity"
  - "xtask host workflow package contract in tools/xtask"
  - "Cargo workspace membership and lockfile entries for firmware and host tool packages"
affects: [foundation, firmware, package, flash, parity, host-tools]
tech-stack:
  added: [bitaxe-firmware, bitaxe-flash, bitaxe-parity, xtask]
  patterns:
    - "Compile-only package contracts use empty side-effect-free main entrypoints"
    - "Future implementation plan ownership is recorded in source comments"
key-files:
  created:
    - firmware/bitaxe/Cargo.toml
    - firmware/bitaxe/src/main.rs
    - tools/flash/Cargo.toml
    - tools/flash/src/main.rs
    - tools/parity/Cargo.toml
    - tools/parity/src/main.rs
    - tools/xtask/Cargo.toml
    - tools/xtask/src/main.rs
  modified:
    - Cargo.toml
    - Cargo.lock
    - .planning/phases/01-foundation-and-gamma-601-boot-log/01-04-SUMMARY.md
key-decisions:
  - "Keep firmware and host tool entrypoints empty until their owning implementation plans add behavior."
  - "Do not add a firmware Bazel target in Plan 05 because Plan 06 owns the ESP-IDF firmware Bazel integration."
  - "Keep Plan 05 host tools free of process execution, package generation, parity mutation, flashing, monitoring, and hardware-control behavior."
patterns-established:
  - "Package contracts are created as Cargo members first; Bazel targets are added only by their owning implementation plans."
requirements-completed: [FND-03, FND-04, FND-05]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 1-2026-06-21T00-30-20
generated_at: 2026-06-21T02:54:25Z
duration: 8 min
completed: 2026-06-21
---

# Phase 01 Plan 05: Firmware And Host Tool Package Contracts Summary

**Effect-free Cargo package contracts for firmware, flash, parity, and xtask entrypoints**

## Performance

- **Duration:** 8 min
- **Started:** 2026-06-21T02:46:44Z
- **Completed:** 2026-06-21T02:54:25Z
- **Tasks:** 2
- **Files modified:** 11

## Accomplishments

- Added `bitaxe-firmware` under `firmware/bitaxe` as a workspace member with an empty Plan 06-owned entrypoint.
- Added `bitaxe-flash`, `bitaxe-parity`, and `xtask` under `tools/` as workspace members with empty Plan 08, Plan 07, and Plan 08-owned entrypoints.
- Refreshed `Cargo.lock` so all firmware and host tool package contracts resolve through the root workspace.
- Preserved the Plan 05 threat model by adding no ESP-IDF imports, command execution, package generation, parity mutation, flashing, monitoring, mining, ASIC, voltage, fan, thermal, or power behavior.

## Task Commits

Each task was committed atomically:

1. **Task 1: Add firmware package entrypoint contract** - `ac7abab` (feat)
2. **Task 2: Add host tool package entrypoint contracts** - `bc85441` (feat)

Preflight repair:

- `0b61bab` - repaired Plan 04 summary body separators so lifecycle validation could pass before Plan 05 execution.

## Files Created/Modified

- `Cargo.toml` - Adds `firmware/bitaxe`, `tools/flash`, `tools/parity`, and `tools/xtask` workspace members.
- `Cargo.lock` - Locks the new local firmware and host tool package contracts.
- `firmware/bitaxe/Cargo.toml` - Defines the `bitaxe-firmware` package using workspace edition and license.
- `firmware/bitaxe/src/main.rs` - Empty Plan 06-owned firmware entrypoint contract.
- `tools/flash/Cargo.toml` - Defines the `bitaxe-flash` package using workspace edition and license.
- `tools/flash/src/main.rs` - Empty Plan 08-owned host tool entrypoint contract.
- `tools/parity/Cargo.toml` - Defines the `bitaxe-parity` package using workspace edition and license.
- `tools/parity/src/main.rs` - Empty Plan 07-owned host tool entrypoint contract.
- `tools/xtask/Cargo.toml` - Defines the `xtask` package using workspace edition and license.
- `tools/xtask/src/main.rs` - Empty Plan 08-owned workflow helper entrypoint contract.
- `.planning/phases/01-foundation-and-gamma-601-boot-log/01-04-SUMMARY.md` - Converts body separators from `---` to `***` so lifecycle metadata parsing remains anchored to the real frontmatter.

## Decisions Made

- Entrypoints stay empty instead of printing, parsing arguments, invoking commands, or reporting status because Plan 05 is only a package contract layer.
- The firmware package intentionally has no `BUILD.bazel` file; Plan 06 owns the firmware Bazel target and ESP-IDF boot/log integration.
- Host tool Cargo packages intentionally exist before their Bazel/Just behavior so downstream plans can depend on stable package names without inheriting premature side effects.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Repaired prior summary lifecycle parsing**
- **Found during:** Preflight lifecycle validation before Task 1
- **Issue:** `verify lifecycle 01 --require-plans --raw` failed because the Plan 04 summary had body `---` separators that the GSD parser treated as the last frontmatter block.
- **Fix:** Converted the Plan 04 body separators to `***`, preserving rendered Markdown separation while keeping lifecycle metadata machine-readable.
- **Files modified:** `.planning/phases/01-foundation-and-gamma-601-boot-log/01-04-SUMMARY.md`
- **Verification:** Lifecycle validation returned `valid` before Plan 05 execution began.
- **Committed in:** `0b61bab`

**2. [Rule 3 - Blocking] Kept Cargo.lock reproducible after firmware member creation**
- **Found during:** Task 1
- **Issue:** Adding `firmware/bitaxe` caused Cargo verification to add the `bitaxe-firmware` package entry to `Cargo.lock` before Task 2's planned final lockfile refresh.
- **Fix:** Committed the Task 1 lockfile entry with the Task 1 workspace change, then regenerated the lockfile again in Task 2 after adding the host tool members.
- **Files modified:** `Cargo.lock`
- **Verification:** `cargo metadata --format-version=1 --no-deps` resolved all workspace members after both tasks.
- **Committed in:** `ac7abab`, `bc85441`

**Total deviations:** 2 auto-fixed (2 blocking)
**Impact on plan:** Both fixes preserved the required lifecycle gate and reproducible Cargo workspace state. No product behavior or extra runtime surface was added.

## Issues Encountered

None beyond the deviations documented above.

## User Setup Required

None - no external service configuration required.

## Verification

Passed:

- Lifecycle validation before execution: `node /Users/peterryszkiewicz/.codex/get-shit-done/bin/gsd-tools.cjs verify lifecycle 01 --require-plans --raw`
- `cargo metadata --format-version=1 --no-deps`
- `grep -q 'Plan 06 replaces this entrypoint' firmware/bitaxe/src/main.rs`
- `grep -q 'firmware/bitaxe' Cargo.toml`
- `grep -q 'name = "bitaxe-firmware"' firmware/bitaxe/Cargo.toml`
- `rg 'esp_idf|wifi|stratum|asic|voltage|fan|thermal|power|mine' firmware/bitaxe/src/main.rs` returned no matches.
- `cargo check -p bitaxe-firmware -p bitaxe-flash -p bitaxe-parity -p xtask`
- `grep -q 'tools/flash' Cargo.toml`
- `grep -q 'tools/parity' Cargo.toml`
- `grep -q 'tools/xtask' Cargo.toml`
- `grep -q 'Plan 08' tools/flash/src/main.rs`
- `grep -q 'Plan 07' tools/parity/src/main.rs`
- `grep -q 'Plan 08' tools/xtask/src/main.rs`
- `rg 'Command::new|espflash|bazel|cargo|package|flash|monitor|verified|wifi|mine|asic|voltage|fan|thermal|power' tools/flash/src tools/parity/src tools/xtask/src` returned no matches.
- `cargo fmt --all`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo build --all-targets --all-features`
- `cargo test --all-features`

## Known Stubs

These are intentional Plan 05 compile-only contracts and do not block the plan goal:

| File | Line | Reason |
| --- | --- | --- |
| `firmware/bitaxe/src/main.rs` | 1 | Empty Plan 06-owned firmware entrypoint; Plan 06 replaces it with safe ESP-IDF boot/log behavior. |
| `tools/flash/src/main.rs` | 1 | Empty Plan 08-owned host tool entrypoint; Plan 08 adds typed workflow behavior. |
| `tools/parity/src/main.rs` | 1 | Empty Plan 07-owned host tool entrypoint; Plan 07 adds parity report semantics. |
| `tools/xtask/src/main.rs` | 1 | Empty Plan 08-owned workflow helper entrypoint; Plan 08 adds typed workflow glue. |

Stub scan found no `TODO`, `FIXME`, placeholder strings, hardcoded empty UI data, or unwired UI data sources in the files created or modified by this plan.

## Next Phase Readiness

Ready for `01-06-PLAN.md`. The firmware package name and host tool package names now exist in Cargo without side effects, so downstream plans can replace the entrypoints and add Bazel/Just behavior in their own scoped commits.

*Phase: 01-foundation-and-gamma-601-boot-log*
*Completed: 2026-06-21*

## Self-Check: PASSED

- Verified the summary, firmware package, and host tool package files exist on disk.
- Verified commits exist: `0b61bab`, `ac7abab`, `bc85441`.
- Verified lifecycle validation still returns `valid` after summary creation.
