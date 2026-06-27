---
phase: 03-bm1366-asic-protocol-and-safe-initialization
fixed_at: 2026-06-27T01:46:23Z
review_path: .planning/phases/03-bm1366-asic-protocol-and-safe-initialization/03-REVIEW.md
iteration: 1
findings_in_scope: 2
fixed: 2
skipped: 0
status: all_fixed
---

# Phase 03: Code Review Fix Report

**Fixed at:** 2026-06-27T01:46:23Z
**Source review:** `.planning/phases/03-bm1366-asic-protocol-and-safe-initialization/03-REVIEW.md`
**Iteration:** 1

**Summary:**
- Findings in scope: 2
- Fixed: 2
- Skipped: 0

## Fixed Issues

### WR-01: Chip-detect success can be published without validating the chip-id response

**Status:** fixed: requires human verification
**Commit:** `d11667f` (`fix(03): validate BM1366 chip-detect responses`)
**Files modified:** `crates/bitaxe-asic/BUILD.bazel`, `crates/bitaxe-asic/src/bm1366.rs`, `crates/bitaxe-asic/src/bm1366/chip_detect.rs`, `crates/bitaxe-asic/src/bm1366/command.rs`, `crates/bitaxe-asic/src/bm1366/init_plan.rs`, `crates/bitaxe-asic/src/bm1366/transcript.rs`, `firmware/bitaxe/src/asic_adapter.rs`
**Applied fix:** Added reusable BM1366 chip-id response validation, replaced the init plan's raw exact read plus unconditional `chip_detected` publish with a typed `ReadChipId` adapter action, and changed the firmware interpreter to publish `chip_detected` only after preamble, CRC, chip-id, and expected chip-count validation pass.
**Verification:** Added host tests proving an exact-length bad-preamble response produces fail-closed actions and no `ChipDetectedNoMining` status; `cargo test -p bitaxe-asic --all-features` passed; the required pre-commit gate passed before commit.
**Residual risk:** Host tests and firmware compile prove the control-flow contract, but live UART behavior still needs hardware verification before claiming verified chip-detect parity.

### WR-02: Adapter I/O failures escape without visible fail-closed status after reset release

**Status:** fixed: requires human verification
**Commit:** `5e5a0ed` (`fix(03): fail closed on adapter I/O errors`)
**Files modified:** `crates/bitaxe-asic/src/bm1366/chip_detect.rs`, `firmware/bitaxe/src/asic_adapter.rs`
**Applied fix:** Added pure fail-closed adapter I/O failure actions and changed the firmware action loop so interpreter errors publish `asic_status=fail_closed reason=chip_detect_adapter_error`, attempt best-effort reset-low, and return without silently escaping through `main`.
**Verification:** Added a host seam test proving partial-read and UART-write failure categories produce hold-reset-low plus fail-closed status actions; `cargo test -p bitaxe-asic --all-features` passed; the required pre-commit gate passed before commit.
**Residual risk:** The fail-closed transition is unit-tested through the pure seam and firmware-compiles, but the physical reset-low effect still requires live hardware evidence.

## Verification

- `cargo fmt --all` passed before both fix commits.
- `cargo clippy --all-targets --all-features -- -D warnings` passed before both fix commits.
- `cargo build --all-targets --all-features` passed before both fix commits.
- `cargo test --all-features` passed before both fix commits and again after both commits.
- `cargo test -p bitaxe-asic --all-features` passed after both commits.
- `source "$HOME/export-esp.sh" && cargo check -p bitaxe-firmware --target xtensa-esp32s3-espidf` passed.
- `just parity` passed with `validation_errors: none`.
- `git status --short reference/esp-miner` produced no output.

## Skipped Issues

None.

---

_Fixed: 2026-06-27T01:46:23Z_
_Fixer: the agent (gsd-code-fixer)_
_Iteration: 1_
