---
phase: 01-foundation-and-gamma-601-boot-log
verified: 2026-06-26T14:29:22Z
status: passed
score: 16/16 must-haves verified
generated_by: gsd-verifier
lifecycle_mode: yolo
phase_lifecycle_id: 1-2026-06-21T00-30-20
generated_at: 2026-06-26T14:29:22Z
lifecycle_validated: true
overrides_applied: 0
re_verification:
  previous_status: human_needed
  previous_score: 35/36 must-haves verified
  gaps_closed:
    - "Previous Gamma 601 hardware-smoke human-needed item is superseded by ADR-0014; current Phase 1 target is Ultra 205/BM1366 safe boot/log, with passing Ultra 205 smoke evidence."
  gaps_remaining: []
  regressions: []
deferred:
  - truth: "Gamma 601/BM1370 hardware verification"
    addressed_in: "Future non-205 board phase / V2 board scope"
    evidence: "ADR-0014 supersedes ADR-0007 and states Gamma 601/BM1370 must not inherit Ultra 205 verification."
  - truth: "Ultra 205 config persistence, NVS runtime behavior, BM1366 ASIC init, mining, safety controllers, API, OTA, and release parity"
    addressed_in: "Phases 2-8"
    evidence: "ROADMAP.md assigns config/NVS to Phase 2, ASIC init to Phase 3, mining to Phase 4, API to Phase 5, safety/display/input to Phase 6, OTA/release packaging to Phase 7, and release evidence governance to Phase 8."
---

# Phase 1: Foundation And Ultra 205 Boot/Log Verification Report

**Phase Goal:** A developer can build, package, flash, and monitor a safe Ultra 205 Rust firmware image that boots and logs identity/status while mining and hardware control remain disabled.
**Verified:** 2026-06-26T14:29:22Z
**Status:** passed
**Re-verification:** Yes - refreshed after ADR-0014 and Ultra 205 smoke evidence superseded the original Gamma 601 target.

This verification used the current canonical contract from `.planning/ROADMAP.md`, `.planning/REQUIREMENTS.md`, `.planning/STATE.md`, `01-CONTEXT.md`, ADR-0014, the Ultra 205 smoke report, and `docs/parity/checklist.md`. Historical Phase 1 plans and summaries still contain Gamma 601 wording, but the accepted current target is Ultra 205/BM1366. Gamma 601/BM1370 evidence is superseded/deferred and must not inherit Ultra 205 verification.

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|---|---|---|
| 1 | Developer can verify the pinned `reference/esp-miner` submodule is present, clean, and protected by fail-closed workflows. | VERIFIED | `git submodule status --recursive reference/esp-miner` shows `c1915b0a63bfabebdb95a515cedfee05146c1d50`; `git -C reference/esp-miner status --porcelain --untracked-files=all` is empty; `just verify-reference` passed and printed the pinned commit. |
| 2 | Developer can run `just build`, `just test`, `just package`, `just verify-reference`, and `just parity` with Bazel-backed outputs. | VERIFIED | `just build`, `just test`, `just package`, `just verify-reference`, and `just parity` passed during this verification. Package outputs are `bitaxe-ultra205.elf`, `bitaxe-ultra205-factory.bin`, and `bitaxe-ultra205-package.json`. |
| 3 | Developer can use `just flash`, `just monitor`, and `just flash-monitor` for `board=205` with port handling, build-before-flash behavior, and printed underlying commands. | VERIFIED | `just flash dry-run=true board=205 port=/dev/cu.usbmodem1101` built the package and printed `espflash flash --chip esp32s3 --port ... bitaxe-ultra205.elf`; `just monitor dry-run=true port=/dev/cu.usbmodem1101` printed `espflash monitor --port ...`; `tools/flash` rejects deferred `board=601`. |
| 4 | Bitaxe owner can observe Ultra 205 boot logs with firmware identity, platform status, reset reason, partition/image identity, selected board/ASIC target, and explicit safe no-mining/no-control state. | VERIFIED | `docs/parity/evidence/ultra-205-pivot-safe-state-smoke-2026-06-26.md` records live `just flash-monitor board=205 port=/dev/cu.usbmodem1101` evidence with `board=Ultra 205 asic=BM1366`, safe-state disabled mining/work/control, reset reason, partition, PSRAM status, firmware commit, reference commit, ESP-IDF version, and Rust target. `01-HUMAN-UAT.md` records the Ultra 205 smoke as passed. |
| 5 | Developer can inspect parity/provenance output showing checklist status, evidence gaps, implementation pointers, reference breadcrumbs, package metadata, and license guardrails without treating implementation alone as verification. | VERIFIED | `just parity` passed with `validation_errors: none`; checklist rows use `hardware-smoke` for verified Ultra 205 boot/flash rows, keep safety-critical/control rows below verified, and mark Gamma 601/BM1370 as deferred. |

**Score:** 5/5 roadmap truths and 11/11 FND requirements verified.

### Deferred Items

| # | Item | Addressed In | Evidence |
|---|---|---|---|
| 1 | Gamma 601/BM1370 hardware verification | Future non-205 board phase / V2 board scope | ADR-0014 states Gamma 601/BM1370 remains deferred and must not inherit Ultra 205 verification. |
| 2 | ASIC init, mining, voltage, fan, thermal, power, NVS runtime behavior, display, Wi-Fi, API, OTA, and release parity | Phases 2-8 | ROADMAP.md assigns these surfaces to later phases; the Ultra 205 smoke report says it does not validate them. |

### Required Artifacts

| Artifact | Expected | Status | Details |
|---|---|---|---|
| `reference/esp-miner` | Pinned clean upstream reference | VERIFIED | Submodule is clean at `c1915b0a63bfabebdb95a515cedfee05146c1d50`; nested submodule is initialized. |
| `scripts/verify-reference-clean.sh` and `scripts/BUILD.bazel` | Fail-closed reference guard and Bazel target | VERIFIED | Guard checks missing, dirty, uninitialized, and mismatched reference state; `just verify-reference` passed. |
| `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `.cargo/config.toml`, `MODULE.bazel` | Rust workspace, ESP-IDF toolchain pins, target config, and Bzlmod graph | VERIFIED | Workspace pins ESP-IDF Rust stack, `xtensa-esp32s3-espidf`, `MCU=esp32s3`, Bazel 9.1.1, and crate mirror wiring. |
| `crates/bitaxe-core`, `crates/bitaxe-config`, `crates/bitaxe-test-support` | Pure Ultra 205 identity and safe-state contracts | VERIFIED | Tests pass; `BoardTarget::Ultra205`, `AsicTarget::Bm1366`, `Phase1SafeState`, and `Phase1BoardSelection::ultra_205()` exist. |
| `crates/bitaxe-asic`, `crates/bitaxe-stratum`, `crates/bitaxe-api` | Deferred pure crate contracts | VERIFIED | Tests pass; these crates expose deferred statuses only and do not enable ASIC, Stratum, or API runtime behavior. |
| `firmware/bitaxe` | Safe ESP-IDF boot/log firmware | VERIFIED | Firmware code logs Ultra 205/BM1366 identity, reset reason, partition, PSRAM status, commit fields, ESP-IDF version, Rust target, and disabled safe state. |
| `firmware/bitaxe:firmware_image`, `scripts/package-firmware.sh`, `tools/xtask` | Machine-readable Ultra 205 package manifest | VERIFIED | `just package` passed; manifest records board, device, ASIC, firmware/reference commits, ESP-IDF version, Rust target, image paths, offsets, and SHA-256 checksums. |
| `tools/flash` and `Justfile` | Bazel-backed flash, monitor, and flash-monitor UX | VERIFIED | `Justfile` is a thin Bazel wrapper; dry-runs print `espflash` commands; `board=601` fails as deferred. |
| `tools/parity` and `docs/parity/checklist.md` | Guarded parity report and evidence ledger | VERIFIED | `just parity` passed; false verified claims are rejected; safety-critical rows require hardware evidence. |

### Key Link Verification

| From | To | Via | Status | Details |
|---|---|---|---|---|
| `Justfile` | Bazel targets | Recipes call `bazel build`, `bazel test`, and `bazel run` | VERIFIED | `just --summary` lists build/test/package/flash/monitor/flash-monitor/verify-reference/parity; core commands passed. |
| `scripts/verify-reference-clean.sh` | `reference/esp-miner` | Git submodule, commit, and dirty-state checks | VERIFIED | Live guard passed only after checking the pinned clean reference. |
| `firmware/bitaxe/src/main.rs` | `bitaxe-core` | Imports `BoardTarget`, `AsicTarget`, `Phase1SafeState` | VERIFIED | Boot and safe-state log strings are derived from typed Ultra 205/BM1366 domain values. |
| `firmware/bitaxe:firmware_image` | Reference guard and firmware ELF | Bazel `tools` and `srcs` | VERIFIED | `bazel query 'deps(//firmware/bitaxe:firmware_image)'` includes `//scripts:verify_reference_clean` and `//firmware/bitaxe:firmware`. |
| `scripts/package-firmware.sh` | `tools/xtask` manifest generation | `cargo run -p xtask -- package-firmware --board 205` | VERIFIED | Package script copies `bitaxe-ultra205.elf`, creates factory bin, then writes the Ultra 205 manifest. |
| `tools/flash` | Generated manifest | `PACKAGE_MANIFEST_RELATIVE_PATH`, package build, JSON parsing | VERIFIED | Flash dry-run built package and resolved `bitaxe-ultra205.elf` as the default flash image. |
| `tools/parity:report` | Reference guard | Bazel dependency and runtime guard execution | VERIFIED | `bazel query 'deps(//tools/parity:report)'` includes `//scripts:verify_reference_clean`; `just parity` passed. |
| `docs/parity/checklist.md` | Ultra 205 smoke evidence | Evidence links in verified rows | VERIFIED | WF/SYS rows point to `ultra-205-pivot-safe-state-smoke-2026-06-26.md`; Gamma 601 rows are deferred. |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|---|---|---|---|---|
| `firmware/bitaxe/src/main.rs` | Board and ASIC identity | `BoardTarget::Ultra205.display_name()`, `AsicTarget::Bm1366.display_name()` | Yes | VERIFIED |
| `firmware/bitaxe/src/main.rs` | Safe state | `Phase1SafeState::default().log_line()` | Yes | VERIFIED |
| `firmware/bitaxe/src/main.rs` | Reset, partition, PSRAM status | ESP-IDF APIs (`esp_reset_reason`, `esp_ota_get_running_partition`, heap caps) | Yes, observed in Ultra 205 smoke | VERIFIED |
| `tools/xtask/src/main.rs` | Package manifest fields | Git commands, reference guard, artifact hashing, fixed board `205` | Yes | VERIFIED |
| `tools/flash/src/main.rs` | Flash image and port | Package manifest, explicit `--port`, or `espflash list-ports` | Yes | VERIFIED |
| `tools/parity/src/main.rs` | Checklist rows and validation errors | Markdown checklist parser plus reference guard | Yes | VERIFIED |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|---|---|---|---|
| Reference guard passes for pinned clean reference | `just verify-reference` | Printed `reference clean: c1915b0a63bfabebdb95a515cedfee05146c1d50` | PASS |
| Firmware build target works | `just build` | Produced `bazel-bin/firmware/bitaxe/bitaxe-firmware.elf` | PASS |
| Bazel test surface works | `just test` | 10 Bazel test targets passed | PASS |
| Package target works | `just package` | Produced Ultra 205 ELF, factory bin, and JSON manifest | PASS |
| Manifest records required package metadata | Node JSON check of `bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json` | Board, device, ASIC, commits, ESP-IDF version, target, default image, artifacts, offsets, and SHA-256 checks all passed | PASS |
| Parity report rejects false verification and passes current checklist | `just parity` | Reported pinned reference commit and `validation_errors: none` | PASS |
| Ultra 205 flash dry-run prints command and builds first | `just flash dry-run=true board=205 port=/dev/cu.usbmodem1101` | Built `//firmware/bitaxe:firmware_image` and printed `espflash flash --chip esp32s3 --port ... bitaxe-ultra205.elf` | PASS |
| Monitor dry-run prints command | `just monitor dry-run=true port=/dev/cu.usbmodem1101` | Printed `espflash monitor --port /dev/cu.usbmodem1101` | PASS |
| Gamma 601 is not accepted as Phase 1 hardware evidence | `just flash dry-run=true board=601 port=/dev/cu.usbmodem1101` | Failed with `board 601 is deferred after the Ultra 205 pivot; Phase 1 supports board=205 only` | PASS |
| Host Rust package tests pass | `cargo test -p bitaxe-core -p bitaxe-config -p bitaxe-test-support -p bitaxe-asic -p bitaxe-stratum -p bitaxe-api -p bitaxe-parity -p bitaxe-flash -p xtask` | 42 tests passed across host-checkable packages | PASS |
| Rust formatting is clean | `cargo fmt --all -- --check` | Passed | PASS |

### Requirements Coverage

| Requirement | Source | Status | Evidence |
|---|---|---|---|
| FND-01 | REQUIREMENTS.md | SATISFIED | `reference/esp-miner` exists as a pinned submodule at `c1915b0a63bfabebdb95a515cedfee05146c1d50`. |
| FND-02 | REQUIREMENTS.md | SATISFIED | Reference guard script, guard tests, package target, parity target, and `just verify-reference` all enforce clean pinned reference state. |
| FND-03 | REQUIREMENTS.md | SATISFIED | `.bazelversion`, `MODULE.bazel`, `BUILD.bazel`, crate/tool Bazel targets, and Just wrappers route build/test/package/flash/parity workflows through Bazel or Bazel-represented repo scripts. |
| FND-04 | REQUIREMENTS.md | SATISFIED | Rust workspace pins ESP-IDF Rust dependencies and firmware metadata; `.cargo/config.toml` uses `xtensa-esp32s3-espidf`, `ldproxy`, and `MCU=esp32s3`; firmware metadata pins ESP-IDF `v5.5.4`. |
| FND-05 | REQUIREMENTS.md | SATISFIED | Planned pure crates exist for core state, config, ASIC, Stratum, API, and test support; targeted package tests passed. |
| FND-06 | REQUIREMENTS.md | SATISFIED | Firmware code logs required Ultra 205/BM1366 identity and safe state; live Ultra 205 smoke evidence and UAT prove boot/log observation on connected hardware. |
| FND-07 | REQUIREMENTS.md | SATISFIED | `just --summary` exposes build, test, package, flash, monitor, flash-monitor, verify-reference, and parity; tested commands pass or intentionally reject deferred board. |
| FND-08 | REQUIREMENTS.md | SATISFIED | `tools/flash` supports `board=205`, optional `port`, port discovery errors, build-before-flash, printed `espflash` commands, and deferred `board=601` rejection. |
| FND-09 | REQUIREMENTS.md | SATISFIED | Package manifest records image paths, factory offset, SHA-256 checksums, tool/version fields, firmware commit, and reference commit. |
| FND-10 | REQUIREMENTS.md | SATISFIED | Reference tree stays read-only and clean; package/parity outputs record reference commit; `PROVENANCE.md` and checklist maintain MIT-first/GPL guardrail posture without false verification. |
| FND-11 | REQUIREMENTS.md | SATISFIED | `just parity` reports checklist status, evidence gaps, implementation pointers, and reference breadcrumbs; verified rows require evidence and safety-critical rows require hardware evidence. |

All FND-01 through FND-11 requirements are verified for the current Ultra 205/BM1366 safe boot/log Phase 1 target.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|---|---:|---|---|---|
| None | - | - | - | Stub/placeholder scan across Phase 1 source, scripts, tools, and checklist found no blocking anti-patterns. Matches for mining, voltage, fan, thermal, power, NVS, Wi-Fi, API, and OTA are scope comments, tests, or deferred checklist rows rather than active behavior. |

### Human Verification Required

None for the current Phase 1 target. The required human-visible behavior is covered by `01-HUMAN-UAT.md`, which records the Ultra 205 flash-monitor safe-state smoke as passed with evidence in `docs/parity/evidence/ultra-205-pivot-safe-state-smoke-2026-06-26.md`.

### Gaps Summary

No Phase 1 implementation gaps remain for the current accepted target. The previous Gamma 601 hardware-smoke item is not a live gap because ADR-0014 supersedes the Gamma-first target. Gamma 601/BM1370 remains deferred and must not inherit Ultra 205 verification.

The verified Phase 1 scope is intentionally narrow: repository foundation, reference guardrails, Bazel/Cargo/Just workflow, package/flash/monitor tooling, parity/provenance reporting, and Ultra 205/BM1366 safe boot/log with mining and hardware control disabled. ASIC init, mining, voltage, fan, thermal, power, NVS runtime behavior, display, Wi-Fi, API, OTA, and release parity remain later-phase or unverified surfaces.

### Disconfirmation Notes

- The old plans/summaries contain historical Gamma 601 wording; the current roadmap, requirements, context, ADR-0014, UAT, and parity evidence supersede that target.
- Passing package and flash tests do not prove NVS persistence, BM1366 initialization, Stratum mining, safety control, API, OTA, or release parity. The checklist correctly leaves those rows pending, implemented-only, deferred, or assigned to later phases.
- Unit tests prove typed identity, safe-state, parser, manifest, and validation behavior, while the actual hardware boot/log claim relies on the captured Ultra 205 smoke evidence.

______________________________________________________________________

_Verified: 2026-06-26T14:29:22Z_
_Verifier: the agent (gsd-verifier)_
