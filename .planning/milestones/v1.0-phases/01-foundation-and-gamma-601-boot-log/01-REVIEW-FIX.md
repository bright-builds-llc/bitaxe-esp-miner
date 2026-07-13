---
phase: 01-foundation-and-gamma-601-boot-log
source_review: 01-REVIEW.md
status: fixed
fixed_findings: [WR-01, WR-02, WR-03, WR-04]
generated_by: gsd-code-review-fix
lifecycle_mode: yolo
phase_lifecycle_id: 1-2026-06-21T00-30-20
generated_at: 2026-06-21T04:34:00Z
---

# Phase 01 Review Fix Summary

## Fixes Applied

- WR-01: `firmware/bitaxe/build.rs` now emits Cargo rerun hints for `.git/HEAD` and the active symbolic ref before exporting `BITAXE_FIRMWARE_COMMIT`.
- WR-02: `tools/flash` now captures `flash-monitor` monitor output into `flash-monitor.log` when evidence is requested, and the evidence JSON records `command_kind=flash-monitor` only after the referenced log exists.
- WR-03: `tools/parity` now treats explicit `safety-critical` notes and `PWR-`/`THR-` IDs as hardware-evidence-required rows before allowing `verified`.
- WR-04: `tools/flash` rejects bare `COM` as a likely Windows serial port while still accepting numbered ports such as `COM3`.

## Verification

Passed:

- `cargo fmt --all`
- `cargo test -p bitaxe-parity`
- `cargo test -p bitaxe-flash`
- `bazel test //tools/parity:tests //tools/flash:tests`
- `cargo clippy --workspace --exclude bitaxe-firmware --all-targets --all-features -- -D warnings`
- `cargo build --workspace --exclude bitaxe-firmware --all-targets --all-features`
- `cargo test --workspace --exclude bitaxe-firmware --all-features`
- `source "$HOME/export-esp.sh" && cargo clippy -p bitaxe-firmware --target xtensa-esp32s3-espidf -- -D warnings`
- `source "$HOME/export-esp.sh" && cargo build -p bitaxe-firmware --target xtensa-esp32s3-espidf`

Known limitation remains unchanged: host all-target Cargo checks still exclude `bitaxe-firmware` because `esp-idf-sys` rejects host target `aarch64-apple-darwin`; firmware checks run explicitly against `xtensa-esp32s3-espidf`.
