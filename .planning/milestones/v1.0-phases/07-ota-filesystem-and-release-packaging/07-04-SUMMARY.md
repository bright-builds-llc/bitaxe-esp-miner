---
phase: 07-ota-filesystem-and-release-packaging
plan: 04
subsystem: firmware-static-filesystem-recovery
tags: [rust, esp-idf, spiffs, static-assets, recovery, firmware-http]
requires:
  - phase: 07-01
    provides: Pure static route resolver and fixture-backed route decisions.
  - phase: 07-02
    provides: Ultra 205 package manifest and partition contracts.
provides:
  - ESP-IDF SPIFFS mount/status adapter for the `www` partition.
  - Firmware static and recovery route handlers wired through the pure static resolver.
  - Rust-owned fallback static assets and re-authored recovery page assets.
affects: [07-05, 07-06, 07-07, 07-08, firmware-packaging, ota-recovery]
tech-stack:
  added: []
  patterns:
    - Thin firmware adapters around `bitaxe-api` pure route decisions.
    - Asset-local provenance comments for Rust-owned firmware static files.
key-files:
  created:
    - firmware/bitaxe/src/filesystem.rs
    - firmware/bitaxe/src/static_files.rs
    - firmware/bitaxe/static/www/index.html
    - firmware/bitaxe/static/www/assets/release.json
    - firmware/bitaxe/static/www/assets/app.css
    - firmware/bitaxe/static/www/assets/app.css.gz
    - firmware/bitaxe/static/recovery_page.html
  modified:
    - firmware/bitaxe/src/main.rs
    - firmware/bitaxe/src/http_api.rs
    - firmware/bitaxe/sdkconfig.defaults
    - firmware/bitaxe/BUILD.bazel
key-decisions:
  - "Serve static HTTP paths through `bitaxe-api::resolve_static_request` before opening SPIFFS files."
  - "Keep `/recovery` explicitly registered and register the static wildcard after API, OTA, and websocket routes."
  - "Use Rust-owned minimal fallback and recovery assets instead of copying upstream AxeOS or recovery HTML."
  - "Use an ESP-IDF-resolvable relative partition CSV path because custom partition filenames are resolved from the generated CMake project."
patterns-established:
  - "Firmware HTTP handlers should delegate route safety and gzip/cache decisions to pure shared logic before adapter I/O."
  - "Recovery assets can be embedded with `include_str!` while normal static assets are sourced from the `www` SPIFFS image tree."
requirements-completed: [REL-01, REL-07, REL-08]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 7-2026-06-28T13-30-15
generated_at: 2026-06-28T16:40:23Z
duration: 19m49s
completed: 2026-06-28
---

# Phase 07 Plan 04: SPIFFS Static Recovery Firmware Surface Summary

**Firmware SPIFFS mount/status, static/recovery route adapters, and Rust-owned fallback assets for Ultra 205 release packaging.**

## Performance

- **Duration:** 19m49s
- **Started:** 2026-06-28T16:20:34Z
- **Completed:** 2026-06-28T16:40:23Z
- **Tasks:** 3
- **Files modified:** 11

## Accomplishments

- Mounted the `www` SPIFFS partition at `/www` with explicit success/failure status logging and no auto-format fallback.
- Added firmware static and `/recovery` HTTP handlers that preserve API, OTA, and websocket route ownership before the static wildcard.
- Added Rust-owned fallback `www` assets, a deterministic gzip smoke asset, and a re-authored recovery page with the required UI-SPEC labels.

## Task Commits

Each task was committed atomically:

1. **Task 1: Add SPIFFS mount/status adapter and SDK config** - `1d173f1` (feat)
2. **Task 2: Add static and recovery HTTP handlers** - `9c37885` (feat)
3. **Task 3: Add Rust-owned static compatibility and recovery assets** - `3c4c536` (feat)

## Files Created/Modified

- `firmware/bitaxe/src/filesystem.rs` - SPIFFS mount/status adapter around ESP-IDF SPIFFS APIs.
- `firmware/bitaxe/src/static_files.rs` - Firmware static and recovery route adapter using the pure resolver.
- `firmware/bitaxe/src/main.rs` - Startup now mounts SPIFFS before registering HTTP routes.
- `firmware/bitaxe/src/http_api.rs` - HTTP setup now receives filesystem status and registers recovery/static handlers in route-safe order.
- `firmware/bitaxe/sdkconfig.defaults` - Partition table, flash size, HTTP URI/header limits, and SPIFFS filename length settings.
- `firmware/bitaxe/BUILD.bazel` - Firmware target tracks partition and static asset inputs.
- `firmware/bitaxe/static/www/index.html` - Rust-owned fallback page for unavailable AxeOS UI.
- `firmware/bitaxe/static/www/assets/release.json` - Minimal release metadata for future packaging evidence.
- `firmware/bitaxe/static/www/assets/app.css` - Rust-owned fallback page styling.
- `firmware/bitaxe/static/www/assets/app.css.gz` - Deterministic gzip counterpart generated with `gzip -n -9 -c`.
- `firmware/bitaxe/static/recovery_page.html` - Re-authored recovery upload page embedded by firmware.

## Decisions Made

- Static file safety remains in `bitaxe-api` so firmware opens files only after traversal, missing-file, gzip, and cache decisions are resolved.
- `/recovery` is registered explicitly and before the wildcard so recovery remains available even when SPIFFS is unavailable.
- The static wildcard is registered after API, OTA, and websocket routes to avoid capturing Phase 5 firmware API paths.
- The fallback and recovery assets are intentionally small and Rust-owned; they do not claim full AxeOS UI parity and are not copied from upstream ESP-Miner.

## Verification

- `node /Users/peterryszkiewicz/.codex/get-shit-done/bin/gsd-tools.cjs verify lifecycle 07 --require-plans` - passed before implementation.
- Task 1 acceptance greps for SPIFFS mount behavior and SDK config - passed.
- Task 2 acceptance greps for static/recovery handler behavior and route ownership - passed.
- Task 3 asset/provenance/gzip verification command - passed.
- `cargo fmt --all` - passed before task commits.
- `cargo clippy --all-targets --all-features -- -D warnings` - passed before task commits.
- `cargo build --all-targets --all-features` - passed before task commits.
- `cargo test --all-features` - passed before task commits.
- `cargo test -p bitaxe-api --all-features static_plan` - passed, 7 tests.
- `cargo build -p bitaxe-firmware --target xtensa-esp32s3-espidf` - passed.
- `bazel build //firmware/bitaxe:firmware` - passed and produced `bazel-bin/firmware/bitaxe/bitaxe-firmware.elf`.
- `git diff --check` - passed.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Fixed ESP-IDF custom partition CSV path resolution**
- **Found during:** Task 1 (Add SPIFFS mount/status adapter and SDK config)
- **Issue:** ESP-IDF resolved `CONFIG_PARTITION_TABLE_CUSTOM_FILENAME` from the generated `esp-idf-sys` CMake project and looked for `target/.../out/partitions-ultra205.csv`.
- **Fix:** Set the SDK config value to the relative path back to the checked-in firmware partition CSV: `../../../../../../firmware/bitaxe/partitions-ultra205.csv`.
- **Files modified:** `firmware/bitaxe/sdkconfig.defaults`
- **Verification:** `cargo build -p bitaxe-firmware --target xtensa-esp32s3-espidf` and `bazel build //firmware/bitaxe:firmware` both passed.
- **Committed in:** `1d173f1`

**Total deviations:** 1 auto-fixed Rule 3 blocking issue.
**Impact on plan:** The fix was required for the planned custom partition table to build; no scope was added beyond build correctness.

## Issues Encountered

- Task 2 needed minor Rust lifetime and mutable request adjustments while wiring ESP-IDF HTTP request path access. These were implementation details inside the planned handler and were verified by firmware builds and the full Rust pre-commit sequence.
- TDD red signals used targeted missing-file/acceptance failures instead of committed failing tests because the affected work was firmware adapter/config wiring and repo Rust pre-commit rules require passing checks before commits.

## Known Stubs

None. The `AxeOS unavailable` text is the required fallback UI copy for this plan, not a placeholder for full AxeOS parity.

## Threat Flags

None. The new HTTP/filesystem surfaces match the plan threat model: path traversal is rejected by the shared resolver, SPIFFS failure keeps recovery available, and API/update/websocket route registration precedes the static wildcard.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan 07-05 can package the tracked static source tree into `www.bin` inputs.
- Plan 07-06 can record provenance for the Rust-owned static and recovery assets.
- Hardware smoke plans can now verify visible SPIFFS mount status, `/recovery`, gzip serving, missing-file redirect behavior, and protected API route reachability.

## Self-Check: PASSED

- Created summary and firmware/static asset files exist.
- Task commits `1d173f1`, `9c37885`, and `3c4c536` exist in git history.

***
*Phase: 07-ota-filesystem-and-release-packaging*
*Completed: 2026-06-28*
