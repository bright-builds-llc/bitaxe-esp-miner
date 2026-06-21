---
phase: 01-foundation-and-gamma-601-boot-log
verified: 2026-06-21T04:34:21Z
status: human_needed
score: 35/36 must-haves verified
generated_by: gsd-verifier
lifecycle_mode: yolo
phase_lifecycle_id: 1-2026-06-21T00-30-20
generated_at: 2026-06-21T04:34:21Z
lifecycle_validated: true
overrides_applied: 0
human_verification:
  - test: "Gamma 601 flash-monitor hardware smoke"
    expected: "Captured log contains boot identity, reset reason, partition/image identity, platform or PSRAM status, firmware commit or Unavailable, pinned reference commit, and the safe no-mining/no-control state."
    why_human: "No Gamma 601 serial port was visible; live boot/log behavior cannot be verified from code or cached build output."
---

# Phase 1: Foundation And Gamma 601 Boot/Log Verification Report

**Phase Goal:** A developer can build, package, flash, and monitor a safe Gamma 601 Rust firmware image that boots and logs identity/status while mining and hardware control remain disabled.
**Verified:** 2026-06-21T04:34:21Z
**Status:** human_needed
**Re-verification:** No - initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|---|---|---|
| 1 | Pinned `reference/esp-miner` is present, clean, and protected by workflows that fail on missing, dirty, or unpinned state. | VERIFIED | `git submodule status --recursive reference/esp-miner` and `just verify-reference` both report `c1915b0a63bfabebdb95a515cedfee05146c1d50`; guard tests exist and pass. |
| 2 | Developer can run `just build`, `just test`, `just package`, `just verify-reference`, and `just parity` with Bazel-backed outputs. | VERIFIED | Reran `just build`, `just test`, `just package`, `just verify-reference`, and `just parity`; all passed. Package outputs are present under `bazel-bin/firmware/bitaxe/`. |
| 3 | Developer can use `just flash`, `just monitor`, and `just flash-monitor` for `board=601` with typed args and command printing. | VERIFIED | Reran dry-run wrappers. `just flash ...` prints `espflash flash --chip esp32s3 --port ... bitaxe-gamma601.elf`; `just monitor` and `just flash-monitor` print monitor command paths without touching hardware. |
| 4 | Bitaxe owner can observe Gamma 601 boot logs with identity/status and safe no-mining/no-control state. | NEEDS HUMAN | Firmware code contains the required log lines and builds for `xtensa-esp32s3-espidf`, but no serial port was visible and no hardware boot log was captured. Checklist rows remain `implemented`/`pending`, not `verified`. |
| 5 | Parity/provenance tooling shows checklist status, evidence gaps, implementation pointers, breadcrumbs, package metadata, and license guardrails without treating implementation as verification. | VERIFIED | `just parity` reports the pinned reference commit and `validation_errors: none`; verified rows have non-pending evidence, and safety-critical rows are not verified without hardware evidence. |

**Score:** 35/36 must-haves verified. The unverified item is live Gamma 601 hardware boot-log observation.

### Plan Must-Haves

| Plan | Truths | Status | Evidence |
|---|---:|---|---|
| 01 | 3 | VERIFIED | Reference submodule, guard script, and Bazel guard target verified. |
| 02 | 3 | VERIFIED | Cargo workspace, ESP toolchain config, target config, and crate_universe wiring verified. |
| 03 | 3 | VERIFIED | `bitaxe-core`, `bitaxe-config`, and `bitaxe-test-support` expose typed Gamma 601/BM1370/safe-state contracts and pass tests. |
| 04 | 3 | VERIFIED | ASIC, Stratum, and API crates expose explicit deferred statuses and no active runtime behavior. |
| 05 | 3 | VERIFIED | Firmware and host tool packages are workspace members; later plans replaced the initial inert entrypoints where intended. |
| 06 | 3 | VERIFIED | Firmware builds for ESP32-S3, logs required status strings, and contains no active Wi-Fi/mining/ASIC/power-control calls. |
| 07 | 4 | VERIFIED | Parity report runs the reference guard, parses checklist rows, emits reference commit, and rejects false verified claims. |
| 08 | 5 | VERIFIED | Package manifest, default ELF flash image, reference guard, typed flash CLI, and `Command` vector execution verified. |
| 09 | 4 | VERIFIED | Just wrappers work; hardware smoke is explicitly recorded as missing evidence because no serial port was visible. |

### Required Artifacts

| Artifact Group | Expected | Status | Details |
|---|---|---|---|
| Reference guard | `.gitmodules`, `reference/esp-miner`, `scripts/verify-reference-clean.sh`, `scripts/BUILD.bazel` | VERIFIED | GSD artifact verifier passed 4/4; live guard passed. |
| Workspace and toolchain | `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `.cargo/config.toml`, `MODULE.bazel` | VERIFIED | Pins Cargo deps, ESP toolchain, `xtensa-esp32s3-espidf`, Bzlmod, and crate_universe. |
| Pure crates | `crates/bitaxe-{core,config,test-support,asic,stratum,api}` | VERIFIED | Cargo and Bazel tests pass; active deferred surfaces stay side-effect-free. |
| Firmware | `firmware/bitaxe/src/main.rs`, `firmware/bitaxe/BUILD.bazel`, `scripts/build-firmware.sh`, `sdkconfig.defaults` | VERIFIED | `just build` produced `bitaxe-firmware.elf`; explicit ESP32-S3 clippy/build passed. |
| Package and flash tooling | `tools/xtask`, `tools/flash`, `scripts/package-firmware.sh`, `//firmware/bitaxe:firmware_image` | VERIFIED | `just package` produced ELF, factory bin, and manifest; flash dry-runs print command vectors. |
| Parity and evidence | `tools/parity`, `docs/parity/checklist.md`, `docs/parity/evidence/phase-01-gamma-601-boot-log.md` | VERIFIED WITH HUMAN PENDING | Tooling and missing-evidence semantics verified; live hardware log not captured. |
| Just surface | `Justfile` | VERIFIED | Recipes are thin Bazel wrappers for build, test, package, flash, monitor, flash-monitor, verify-reference, and parity. |

### Key Link Verification

| From | To | Via | Status | Details |
|---|---|---|---|---|
| `scripts/verify-reference-clean.sh` | `reference/esp-miner` | submodule status, commit, and dirty checks | VERIFIED | GSD key-link verifier passed. |
| `MODULE.bazel` | `Cargo.toml` | crate_universe manifest wiring | VERIFIED | GSD key-link verifier passed. |
| Pure crates | `bitaxe-core` | shared `BoardTarget`, `AsicTarget`, and `Phase1SafeState` | VERIFIED | Imports and tests verify usage. |
| `firmware/bitaxe/src/main.rs` | `bitaxe-core` | `Phase1SafeState` and board/ASIC display names | VERIFIED | Grep and build verify the data path. |
| `firmware/bitaxe/BUILD.bazel` | `scripts/build-firmware.sh` | `//scripts:build_firmware` tool | VERIFIED | GSD literal pattern missed `build-firmware.sh`, but manual inspection shows `cmd = "$(location //scripts:build_firmware)"` and `scripts/BUILD.bazel` maps that target to `build-firmware.sh`. |
| `firmware/bitaxe:firmware_image` | reference guard and firmware target | Bazel deps/tools | VERIFIED | `bazel query` shows `//scripts:verify_reference_clean` and `//firmware/bitaxe:firmware`. |
| `tools/parity:report` | reference guard | Bazel data/runfile plus CLI guard execution | VERIFIED | `bazel query` shows guard dependency; `just parity` passed. |
| `Justfile` | Bazel targets | thin wrapper recipes | VERIFIED | Direct dry-run and build/test/package/parity commands passed. |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|---|---|---|---|---|
| `firmware/bitaxe/src/main.rs` | board, ASIC, safe-state log line | `bitaxe-core` enums and `Phase1SafeState::default()` | Yes, static typed domain values | VERIFIED |
| `firmware/bitaxe/src/main.rs` | reset, partition, PSRAM/platform status | ESP-IDF APIs via firmware code | Build verified; runtime value needs hardware | NEEDS HUMAN |
| `tools/xtask/src/main.rs` | package manifest metadata | git commands, reference guard, tool versions, SHA-256 artifact reads | Yes | VERIFIED |
| `tools/flash/src/main.rs` | default flash image | generated manifest top-level `default_flash_image` | Yes, validates `bitaxe-gamma601.elf` and rejects factory bin default | VERIFIED |
| `tools/parity/src/main.rs` | checklist rows and validation errors | Markdown checklist parser plus reference guard | Yes | VERIFIED |
| `docs/parity/checklist.md` | boot/log evidence state | evidence document and command summaries | Explicitly pending, not false-verified | VERIFIED |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|---|---|---|---|
| Reference guard passes only for clean pinned reference | `just verify-reference` | Printed `reference clean: c1915b0a63bfabebdb95a515cedfee05146c1d50` | PASS |
| Public build target works | `just build` | Produced `bazel-bin/firmware/bitaxe/bitaxe-firmware.elf` | PASS |
| Public test target works | `just test` | 10 Bazel test targets passed from cache | PASS |
| Public package target works | `just package` | Produced `bitaxe-gamma601.elf`, `bitaxe-gamma601-factory.bin`, and package JSON | PASS |
| Package manifest has required metadata | Node JSON check | Found schema, board, device, ASIC, firmware/reference commits, ESP-IDF version, target, default ELF, offsets, and SHA-256 values | PASS |
| Public parity target works | `just parity` | Reported reference commit and `validation_errors: none` | PASS |
| Flash dry-run works through Just | `just flash board=601 dry-run=true port=/dev/cu.usbmodem101 image=/tmp/bitaxe-gamma601.elf` | Printed exact `espflash flash` command | PASS |
| Monitor dry-run works through Just | `just monitor dry-run=true port=/dev/cu.usbmodem101` | Printed exact `espflash monitor` command | PASS |
| Flash-monitor dry-run works through Just | `just flash-monitor board=601 dry-run=true port=/dev/cu.usbmodem101 image=/tmp/bitaxe-gamma601.elf` | Printed flash and monitor commands | PASS |
| Hardware availability precheck | `espflash list-ports` | Exited 0 with `No known serial ports found.` | HUMAN PENDING |
| Host Rust verification | `cargo fmt --all -- --check`; host clippy/build/test excluding firmware | Passed | PASS |
| Firmware target verification | `source "$HOME/export-esp.sh" && cargo clippy/build -p bitaxe-firmware --target xtensa-esp32s3-espidf` | Passed | PASS |
| Review-fix verification evidence | `01-REVIEW-FIX.md` command list | Records post-fix `cargo fmt --all`, `cargo test -p bitaxe-parity`, `cargo test -p bitaxe-flash`, `bazel test //tools/parity:tests //tools/flash:tests`, host checks excluding firmware, explicit firmware clippy/build | PASS |

### Requirements Coverage

| Requirement | Source Plan(s) | Description | Status | Evidence |
|---|---|---|---|---|
| FND-01 | 01 | Pinned upstream submodule | SATISFIED | Submodule exists and is clean at `c1915b0a63bfabebdb95a515cedfee05146c1d50`. |
| FND-02 | 01, 07, 08, 09 | Workflows fail on missing/dirty/unpinned reference | SATISFIED | Guard tests and live `just verify-reference`; package/parity deps include guard. |
| FND-03 | 01, 02, 05, 06, 07, 08, 09 | Bazel/Bzlmod canonical automation graph | SATISFIED | `.bazelversion`, `MODULE.bazel`, Bazel targets, and Just wrappers verified. |
| FND-04 | 02, 03, 05, 06 | Rust workspace and firmware toolchain pins | SATISFIED | Cargo workspace, ESP target config, ESP-IDF `v5.5.4`, and explicit firmware target checks pass. |
| FND-05 | 03, 04, 05 | Planned pure crates and test support exist | SATISFIED | All pure crate Cargo and Bazel tests pass. |
| FND-06 | 06, 09 | Firmware can boot/log on Gamma 601 with mining/control disabled | NEEDS HUMAN | Firmware builds and contains required logs; live boot cannot be claimed without captured hardware log. Checklist correctly stays below `verified`. |
| FND-07 | 09 | Required Just commands available and routed through Bazel | SATISFIED | `just --summary` surface inspected; build/test/package/parity and dry-run flash/monitor/flash-monitor passed. |
| FND-08 | 08, 09 | USB flashing ergonomics | SATISFIED | Typed board parser, port discovery errors, dry-run command printing, and key/value aliases verified. |
| FND-09 | 08, 09 | Machine-readable package manifest | SATISFIED | Manifest records image paths, offsets, checksums, tool versions, firmware commit, and reference commit. |
| FND-10 | 01, 04, 07, 08 | Provenance and license guardrails | SATISFIED | Reference remains read-only; parity/package outputs include reference commit; checklist avoids false verified claims. |
| FND-11 | 07, 09 | Parity report distinguishes status, evidence gaps, pointers, breadcrumbs | SATISFIED | `just parity` emits rows and validates no false `verified` claims. |

All Phase 1 requirement IDs listed by the prompt are accounted for. No additional Phase 1 requirement IDs were orphaned in `.planning/REQUIREMENTS.md`.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|---|---:|---|---|---|
| None | - | - | - | No blocking or warning anti-patterns found. Grep hits were format strings, temp directory setup, or expected test/evidence literals. |

### Human Verification Required

#### 1. Gamma 601 Flash-Monitor Smoke

**Test:** Connect a Gamma 601 over USB, run `espflash list-ports`, choose the port, then run `just flash-monitor board=601 port=<port> evidence-dir=docs/parity/evidence/phase-01-gamma-601-boot-log`.
**Expected:** Captured log contains `bitaxe-rust boot: board=Gamma 601 asic=BM1370`, `safe_state: mining=disabled asic_work_submission=disabled hardware_control=disabled`, `reset_reason=`, `partition=` or `image_partition=`, `platform_status=` or `psram_status=`, `firmware_commit=` with an actual commit or `Unavailable`, and `reference_commit=c1915b0a63bfabebdb95a515cedfee05146c1d50`.
**Why human:** No compatible serial port was visible. Hardware boot/log observation cannot be proven from code, manifest, or dry-run output.

### Gaps Summary

No blocking implementation gaps were found. The only unverified phase outcome is live Gamma 601 boot-log observation. This is recorded honestly as missing hardware-smoke evidence in `docs/parity/evidence/phase-01-gamma-601-boot-log.md`, and the checklist does not falsely mark boot/log or safety-critical hardware rows as `verified`.

The evidence document still records the Phase 09 commit from the no-hardware checkpoint; the current package manifest records the current HEAD after review fixes. Because no boot log was captured and no hardware row was verified, this is a residual evidence note, not a false parity claim.

---

_Verified: 2026-06-21T04:34:21Z_
_Verifier: the agent (gsd-verifier)_
