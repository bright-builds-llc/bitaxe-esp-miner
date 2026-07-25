---
quick_id: 260628-kri
description: Add ESP-IDF contributor dependency workflow
completed: 2026-06-28
status: complete
---

# Quick Task 260628-kri: Add ESP-IDF Contributor Dependency Workflow - Summary

## Result

Implemented a first-class ESP-IDF contributor dependency workflow:

- Added `just doctor` as a read-only local dependency check.
- Added `just bootstrap-esp` as the explicit opt-in installer for `espup`, the ESP Rust toolchain, `ldproxy`, and `espflash`.
- Added shell tests covering missing-tool diagnostics, passing fake toolchains, and bootstrap dry-run output.
- Updated build/package failures to point contributors at the new workflow.
- Documented quickstart setup and repo-local ESP-IDF tooling preference.

## Verification

- `bash scripts/esp-doctor-test.sh` passed.
- `just doctor` passed.
- `bash -n scripts/esp-doctor.sh scripts/bootstrap-esp.sh scripts/esp-doctor-test.sh` passed.
- `bazel test //scripts:esp_doctor_test` passed.
- `cargo fmt --all` passed.
- `cargo clippy --all-targets --all-features -- -D warnings` passed.
- `cargo build --all-targets --all-features` passed.
- `cargo test --all-features` passed.
- `just test` passed.

## Residual Risk

CI remains unchanged by design. Firmware package/build CI can be added later once cache/runtime costs are accepted.
