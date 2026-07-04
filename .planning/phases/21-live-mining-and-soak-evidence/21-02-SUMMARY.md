---
phase: 21-live-mining-and-soak-evidence
plan: "02"
subsystem: firmware
tags: [rust, esp-idf, stratum, bm1366, live-mining, evidence]
requires:
  - phase: 21-live-mining-and-soak-evidence
    provides: "21-01 readiness audit, evidence contract, redaction scaffold"
provides:
  - "Bounded controlled Stratum runtime contract with guarded BM1366 dispatch"
  - "Phase 21 firmware harness gated by compile-time live-mining evidence mode"
  - "Controlled live-mining package helper, ready ledger, and package manifest"
affects: [phase-21-live-smoke, phase-21-bounded-soak, stratum-runtime, firmware-api-telemetry]
tech-stack:
  added: []
  patterns:
    - "Functional-core controlled runtime with firmware shell publication"
    - "Compile-time evidence gates for non-default live mining paths"
    - "Redacted evidence ledger before hardware smoke/soak execution"
key-files:
  created:
    - crates/bitaxe-stratum/src/v1/controlled_runtime.rs
    - crates/bitaxe-stratum/src/v1/controlled_runtime/tests.rs
    - firmware/bitaxe/src/controlled_mining_runtime.rs
    - firmware/bitaxe/src/mining_evidence_mode.rs
    - scripts/phase21-live-mining-package.sh
    - scripts/phase21-live-mining-package-test.sh
    - docs/parity/evidence/phase-21-live-mining-and-soak-evidence/live-mining-enablement.md
    - docs/parity/evidence/phase-21-live-mining-and-soak-evidence/live-mining-enablement/package/bitaxe-ultra205-package.json
  modified:
    - crates/bitaxe-stratum/src/v1/mining_loop.rs
    - crates/bitaxe-stratum/src/v1.rs
    - crates/bitaxe-stratum/BUILD.bazel
    - firmware/bitaxe/build.rs
    - firmware/bitaxe/src/main.rs
    - firmware/bitaxe/src/http_api.rs
    - firmware/bitaxe/src/runtime_snapshot.rs
    - scripts/BUILD.bazel
    - .gitignore
key-decisions:
  - "Default firmware remains fail-closed; controlled mining only activates with BITAXE_MINING_EVIDENCE_MODE=live-mining-runtime and BITAXE_HARDWARE_EVIDENCE_ACK=ultra205-live-mining-runtime-safe-bench."
  - "Firmware parses explicit stored Stratum settings for controlled mode; schema defaults alone do not enable the harness."
  - "Only the package manifest is tracked; generated Phase 21 package binaries are ignored and reproducible through the helper."
patterns-established:
  - "Controlled runtime evidence logs use marker-only redacted status lines."
  - "Package helpers write blocked ledgers before invoking build tools when prerequisites are missing."
requirements-completed: [ASIC-07, STR-06, STR-07, SAFE-09, EVD-05]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 21-2026-07-04T01-35-47
generated_at: 2026-07-04T04:39:48Z
duration: 33min
completed: 2026-07-04
---

# Phase 21 Plan 02: Controlled Live Mining Runtime Summary

**Bounded controlled Stratum runtime, firmware harness, and redacted package enablement ledger for Phase 21 live mining evidence**

## Performance

- **Duration:** 33 min
- **Started:** 2026-07-04T04:06:58Z
- **Completed:** 2026-07-04T04:39:48Z
- **Tasks:** 3 completed
- **Files modified:** 17

## Accomplishments

- Added a pure controlled runtime contract that performs subscribe, authorize, notify, guarded queue dispatch, optional nonce/share handling, runtime state updates, safe-stop evidence, and redacted summaries.
- Wired firmware controlled mode behind compile-time evidence gates and explicit stored Stratum settings, then publishes marker-only logs and API/WebSocket-visible mining state.
- Added `scripts/phase21-live-mining-package.sh`, Bazel test coverage, a ready `live-mining-enablement.md` ledger, and a package manifest with source/reference commits for downstream smoke and soak plans.

## Task Commits

| Task | Name | Commit |
| --- | --- | --- |
| 1 | Controlled runtime core | `b1124d3` |
| 2 | Firmware bounded harness | `2753016` |
| 3a | Runtime marker alignment fix | `f34b50d` |
| 3 | Package helper and enablement ledger | `78ab8b0` |

## Verification

- `cargo test -p bitaxe-stratum --all-features controlled_runtime`
- `cargo test -p bitaxe-stratum --all-features mining_loop`
- `cargo test -p bitaxe-api --all-features mining`
- `cargo test -p bitaxe-api --all-features telemetry`
- `cargo test -p bitaxe-firmware --target xtensa-esp32s3-espidf --all-features mining_evidence_mode --no-run`
- `cargo test -p bitaxe-firmware --target xtensa-esp32s3-espidf --all-features controlled_mining_runtime --no-run`
- `cargo build --package bitaxe-firmware --target xtensa-esp32s3-espidf`
- `bazel test //crates/bitaxe-stratum:tests --test_filter=controlled_runtime`
- `bazel test //scripts:phase21_live_mining_package_test`
- `scripts/phase21-live-mining-package.sh --out-dir docs/parity/evidence/phase-21-live-mining-and-soak-evidence/live-mining-enablement --readiness-audit docs/parity/evidence/phase-21-live-mining-and-soak-evidence/readiness-audit.md`
- `just parity`
- `just verify-reference`
- `git diff -- reference/esp-miner --exit-code`
- Required Rust pre-commit sequence passed before each commit: `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, `cargo test --all-features`.

## Files Created/Modified

- `crates/bitaxe-stratum/src/v1/controlled_runtime.rs` - Pure controlled runtime contract and redacted evidence summary.
- `crates/bitaxe-stratum/src/v1/controlled_runtime/tests.rs` - Runtime lifecycle, dispatch, redaction, share outcome, and watchdog tests.
- `crates/bitaxe-stratum/src/v1/mining_loop.rs` - Guarded queued work now emits typed `Bm1366Command::SendDiagnosticWork`.
- `firmware/bitaxe/src/mining_evidence_mode.rs` - Compile-time Phase 21 evidence gate.
- `firmware/bitaxe/src/controlled_mining_runtime.rs` - Firmware shell for settings-derived bounded harness execution and marker publication.
- `firmware/bitaxe/src/runtime_snapshot.rs` - Evidence-only mining state replacement hook.
- `firmware/bitaxe/src/http_api.rs` - Settings PATCH refresh hook for controlled mode.
- `scripts/phase21-live-mining-package.sh` - Readiness-gated package helper that writes ready/blocked ledgers.
- `scripts/phase21-live-mining-package-test.sh` - Shell test coverage for blocked and ready helper paths.
- `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/live-mining-enablement.md` - Ready enablement ledger with required runtime markers and redaction reviewer status.
- `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/live-mining-enablement/package/bitaxe-ultra205-package.json` - Package manifest for committed firmware source `f34b50d`.

## Decisions Made

- Controlled live mining remains opt-in at build time, not a runtime toggle, so default firmware cannot accidentally start mining.
- Missing explicit stored pool settings blocks controlled mode even though config defaults exist; later live plans must intentionally PATCH disposable Stratum values.
- Package evidence tracks the manifest and ledger only; generated image artifacts are ignored because they are reproducible and large.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug/Clippy] Simplified option handling in controlled runtime**
- **Found during:** Task 1
- **Issue:** `cargo clippy -D warnings` rejected a manual `let Some(...) else` path in share response handling.
- **Fix:** Replaced it with `let share = maybe_share?;`.
- **Files modified:** `crates/bitaxe-stratum/src/v1/controlled_runtime.rs`
- **Verification:** Full required Rust pre-commit sequence passed.
- **Committed in:** `b1124d3`

**2. [Rule 2 - Missing Critical] Added firmware build rerun hints for evidence env vars**
- **Found during:** Task 2
- **Issue:** The compile-time evidence gate needed Cargo rebuild tracking for `BITAXE_MINING_EVIDENCE_MODE` and `BITAXE_HARDWARE_EVIDENCE_ACK`.
- **Fix:** Added `cargo:rerun-if-env-changed` lines in `firmware/bitaxe/build.rs`.
- **Files modified:** `firmware/bitaxe/build.rs`
- **Verification:** Firmware target build and package helper build passed.
- **Committed in:** `2753016`

**3. [Rule 2 - Missing Critical] Aligned runtime log marker names with downstream ledger**
- **Found during:** Task 3
- **Issue:** The package ledger requires `api_websocket_telemetry_update_status` and `safe_stop_status`, but the initial firmware log strings used different names.
- **Fix:** Updated firmware logs to emit the exact downstream marker names before regenerating the package manifest.
- **Files modified:** `firmware/bitaxe/src/controlled_mining_runtime.rs`
- **Verification:** Firmware package regenerated with source commit `f34b50d`; plan-level verification passed.
- **Committed in:** `f34b50d`

**4. [Rule 1 - Bug] Corrected enablement ledger path**
- **Found during:** Task 3
- **Issue:** The helper initially wrote the ledger under the package directory, while the plan verification expects `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/live-mining-enablement.md`.
- **Fix:** Changed the helper to write `${out_dir}.md` and updated tests.
- **Files modified:** `scripts/phase21-live-mining-package.sh`, `scripts/phase21-live-mining-package-test.sh`
- **Verification:** `bazel test //scripts:phase21_live_mining_package_test` passed and the real helper wrote the expected ledger.
- **Committed in:** `78ab8b0`

**5. [Rule 3 - Blocking] Made package helper repeatable after generated ELF output**
- **Found during:** Task 3
- **Issue:** A previous package run left a non-writable generated ELF, causing repeat packaging to fail before ledger regeneration.
- **Fix:** The helper removes known generated package outputs before invoking `scripts/package-firmware.sh`; generated package binaries are ignored while the manifest is tracked.
- **Files modified:** `scripts/phase21-live-mining-package.sh`, `.gitignore`
- **Verification:** Real helper reran successfully and produced ready ledger plus manifest.
- **Committed in:** `78ab8b0`

**Total deviations:** 5 auto-fixed (2 Rule 1, 2 Rule 2, 1 Rule 3)
**Impact on plan:** All fixes were needed for correctness, repeatability, or downstream evidence matching. No architectural changes or live hardware actions were introduced.

## Issues Encountered

- Host-target firmware unit tests cannot compile `esp-idf-sys` on `aarch64-apple-darwin`; firmware tests were compiled with `--target xtensa-esp32s3-espidf --no-run`, and the firmware package build verified the target path.

## Known Stubs

None. Stub scan found only shell variable initialization and pre-existing API route format strings, not UI/data placeholders.

## Threat Review

The plan-defined mitigations were implemented: compile-time evidence gates, redacted pool handling, bounded single-cycle harness behavior, typed BM1366 dispatch only, package ledger provenance, and no `DEVICE_URL` inference or network scanning. No additional unplanned network endpoint, auth path, file-access boundary, or schema trust boundary was introduced beyond the planned package helper and evidence files.

## User Setup Required

None. No external service credentials or manual setup were required.

## Next Phase Readiness

Plans 21-03 and later can consume the ready enablement ledger and package manifest. Live smoke and bounded soak still must satisfy detector gates, redaction review, disposable Stratum settings, and hardware evidence requirements before claiming live mining behavior.

## Self-Check: PASSED

- Created files exist: summary, controlled runtime core, firmware harness, package helper, enablement ledger, and package manifest.
- Commits exist: `b1124d3`, `2753016`, `f34b50d`, `78ab8b0`.
- Markdown frontmatter check passed: only the opening and closing frontmatter delimiters use standalone `---`.

*Phase: 21-live-mining-and-soak-evidence*
*Completed: 2026-07-04*
