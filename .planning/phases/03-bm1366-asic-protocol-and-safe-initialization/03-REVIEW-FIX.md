---
phase: 03-bm1366-asic-protocol-and-safe-initialization
fixed_at: 2026-06-27T01:56:22Z
review_path: .planning/phases/03-bm1366-asic-protocol-and-safe-initialization/03-REVIEW.md
iteration: 2
findings_in_scope: 4
fixed: 4
skipped: 0
status: all_fixed
---

# Phase 03: Code Review Fix Report

**Fixed at:** 2026-06-27T01:56:22Z
**Source review:** `.planning/phases/03-bm1366-asic-protocol-and-safe-initialization/03-REVIEW.md`
**Iteration:** 2

**Summary:**
- Findings in scope: 4
- Fixed: 4
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

### WR-03: Adapter setup failures can bypass fail-closed reset/status before the action loop

**Status:** fixed: requires human verification
**Commit:** `7a5d25b` (`fix(03): fail closed on adapter setup errors`)
**Files modified:** `crates/bitaxe-asic/src/bm1366/chip_detect.rs`, `firmware/bitaxe/src/asic_adapter.rs`
**Applied fix:** Added pure setup-failure actions for reset and UART adapter initialization failures. The firmware adapter now initializes reset before UART, publishes `reset_adapter_unavailable` when reset setup fails, and best-effort holds reset low plus publishes `uart_adapter_unavailable` when UART setup fails.
**Verification:** Added a host seam test proving setup failures publish visible fail-closed status and, when reset is available, include hold-reset-low. The full Rust pre-commit gate, firmware `cargo check`, and `just parity` passed before commit.
**Residual risk:** The fail-closed transition is unit-tested through the pure seam and firmware-compiles, but physical GPIO behavior still requires live hardware evidence.

### IN-01: Phase 3 API still exposes a stale deferred runtime status

**Status:** fixed
**Commit:** `77b682e` (`refactor(03): update ASIC runtime gate status`)
**Files modified:** `crates/bitaxe-asic/src/lib.rs`
**Applied fix:** Renamed the stale `DeferredUntilPhase3` placeholder to `FailClosedDiagnosticGate` and updated the unit test to match Phase 03 behavior.
**Verification:** The full Rust pre-commit gate passed before commit.
**Residual risk:** None.

## Verification

- `cargo fmt --all` passed before all fix commits.
- `cargo clippy --all-targets --all-features -- -D warnings` passed before all fix commits.
- `cargo build --all-targets --all-features` passed before all fix commits.
- `cargo test --all-features` passed before all fix commits and after the warning fixes.
- `cargo test -p bitaxe-asic --all-features` passed after the warning fixes.
- `source "$HOME/export-esp.sh" && cargo check -p bitaxe-firmware --target xtensa-esp32s3-espidf` passed.
- `just parity` passed with `validation_errors: none`.
- `git status --short reference/esp-miner` produced no output.

## Skipped Issues

None.

___

_Fixed: 2026-06-27T01:56:22Z_
_Fixer: the agent (gsd-code-fixer)_
_Iteration: 2_
