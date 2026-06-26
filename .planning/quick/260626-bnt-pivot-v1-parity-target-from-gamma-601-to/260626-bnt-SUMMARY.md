# Quick Task 260626-bnt Summary: Pivot V1 Parity Target To Ultra 205

## Outcome

Completed the initial parity-target pivot from Gamma 601/BM1370 to Ultra 205/BM1366 for the safe-state slice.

## Implemented

1. Added ADR-0014 and marked ADR-0007/old Gamma-first planning as superseded where active docs would otherwise mislead.
1. Updated active project, roadmap, state, milestone, flash, package, and parity docs for Ultra 205 first parity while preserving historical Gamma-first phase artifacts.
1. Changed Rust identity and tooling defaults to Ultra 205/BM1366:
   - `BoardTarget::Ultra205`
   - `AsicTarget::Bm1366`
   - boot log `bitaxe-rust boot: board=Ultra 205 asic=BM1366`
   - package artifacts `bitaxe-ultra205.elf`, `bitaxe-ultra205-factory.bin`, and `bitaxe-ultra205-package.json`
   - default/supported flash board `board=205`
   - deferred `board=601` rejection
1. Added Ultra 205 config defaults from `reference/esp-miner/config-205.cvs` to `Phase1BoardSelection::ultra_205()`:
   - `devicemodel=ultra`
   - `boardversion=205`
   - `asicmodel=BM1366`
   - `asicfrequency=485`
   - `asicvoltage=1200`
1. Added Cargo workspace `default-members` for host-checkable crates/tools so root Cargo pre-commit commands do not try to build the ESP-IDF firmware crate for the macOS host target. Firmware remains a workspace member and is verified through `just build`.
1. Captured post-pivot hardware evidence in `docs/parity/evidence/ultra-205-pivot-safe-state-smoke-2026-06-26.md`.

## Verification

Passed:

1. `cargo fmt --all`
1. `cargo clippy --all-targets --all-features -- -D warnings`
1. `cargo build --all-targets --all-features`
1. `cargo test --all-features`
1. `just verify-reference`
1. `just build`
1. `just test`
1. `just package`
1. `just parity`
1. `just flash dry-run=true board=205 port=/dev/cu.usbmodem1101`
1. `just monitor dry-run=true port=/dev/cu.usbmodem1101`
1. `just flash-monitor dry-run=true board=205 port=/dev/cu.usbmodem1101`
1. `just flash dry-run=true board=601 port=/dev/cu.usbmodem1101` failed as expected with the deferred-board error.
1. `espflash board-info --port /dev/cu.usbmodem1101`
1. `just flash-monitor board=205 port=/dev/cu.usbmodem1101`, followed by monitor `CTRL+R`, captured Ultra 205/BM1366 identity and safe-state boot logs.

## Residual Risk

This verifies only safe-state boot/flash identity and packaging. ASIC init, mining, voltage, fan, thermal, power, NVS runtime behavior, display, Wi-Fi, API, OTA, and release parity remain later phases and must not inherit this evidence. Root Cargo commands cover host-checkable default members; ESP-IDF firmware compilation remains covered by `just build`.
