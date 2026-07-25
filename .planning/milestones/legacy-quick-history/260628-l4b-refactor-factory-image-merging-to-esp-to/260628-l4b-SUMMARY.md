---
status: completed
created: 2026-06-28
quick_id: 260628-l4b
title: Refactor Factory Image Merging To ESP Tooling
---

# Summary

Production factory image assembly now uses ESP tooling instead of the removed `xtask overlay-factory-payloads` byte overlay path.

## Changes

- `scripts/package-firmware.sh` creates a temporary base factory image with `espflash save-image --merge --skip-padding`, then uses `esptool.py merge_bin` to assemble the final `bitaxe-ultra205-factory.bin` with:
  - base image at `0x0`
  - `www.bin` at `0x410000`
  - `otadata-initial.bin` at `0xf10000`
- `esptool.py` resolves from `PATH` first, then from managed `.embuild/espressif` ESP-IDF locations.
- The production `xtask overlay-factory-payloads` command and byte-copy helpers were removed.
- Rust package validation still rejects factory images that do not contain `www.bin` and `otadata-initial.bin` at the contract offsets.
- README, CONTRIBUTING, AGENTS repo guidance, and release inventory now document `.embuild/` as local generated ESP-IDF/esp-rs tool state that repo automation may use.
- `just doctor` reports managed `.embuild` ESP tools when present without requiring them before the first build.
- Added `scripts/package-firmware-test.sh` coverage for the `merge_bin` command and no-overlay behavior.

## Verification

- `bash scripts/esp-doctor-test.sh` passed.
- `bash scripts/package-firmware-test.sh` passed.
- `just doctor` passed and reported managed `spiffsgen.py`, `gen_esp32part.py`, and `esptool.py`.
- `cargo fmt --all` passed.
- `cargo clippy --all-targets --all-features -- -D warnings` passed.
- `cargo build --all-targets --all-features` passed.
- `cargo test --all-features` passed.
- `just test` passed after adding `--skip-padding` to the base factory image generation.
- `just package` passed.
- Byte validation passed for `www.bin` at `0x410000` and `otadata-initial.bin` at `0xf10000` in `bazel-bin/firmware/bitaxe/bitaxe-ultra205-factory.bin`.
- `git diff --check` passed.

## Residual Risk

- `.embuild/` remains local generated state; fresh contributors still need a first firmware build before the managed ESP-IDF tools exist locally.
