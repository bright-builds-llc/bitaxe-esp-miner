---
phase: 07-ota-filesystem-and-release-packaging
reviewed: 2026-06-28T19:36:25Z
depth: standard
files_reviewed: 39
files_reviewed_list:
  - BUILD.bazel
  - about.hbs
  - about.toml
  - crates/bitaxe-api/BUILD.bazel
  - crates/bitaxe-api/src/lib.rs
  - crates/bitaxe-api/src/route_shell.rs
  - crates/bitaxe-api/src/static_plan.rs
  - crates/bitaxe-api/src/update_plan.rs
  - docs/parity/checklist.md
  - docs/parity/evidence/phase-07-ota-filesystem-release.md
  - docs/parity/evidence/phase-07-ultra-205-ota-hardware-smoke.md
  - docs/release/cargo-about.html
  - docs/release/license-inventory.md
  - docs/release/provenance-manifest.md
  - docs/release/ultra-205.md
  - firmware/bitaxe/BUILD.bazel
  - firmware/bitaxe/partitions-ultra205.csv
  - firmware/bitaxe/sdkconfig.defaults
  - firmware/bitaxe/src/boot_validation.rs
  - firmware/bitaxe/src/filesystem.rs
  - firmware/bitaxe/src/http_api.rs
  - firmware/bitaxe/src/main.rs
  - firmware/bitaxe/src/ota_update.rs
  - firmware/bitaxe/src/static_files.rs
  - firmware/bitaxe/static/recovery_page.html
  - firmware/bitaxe/static/www/assets/app.css
  - firmware/bitaxe/static/www/assets/app.css.gz
  - firmware/bitaxe/static/www/assets/release.json
  - firmware/bitaxe/static/www/index.html
  - scripts/package-firmware.sh
  - tools/flash/src/main.rs
  - tools/parity/BUILD.bazel
  - tools/parity/src/main.rs
  - tools/parity/src/release_gate.rs
  - tools/xtask/BUILD.bazel
  - tools/xtask/Cargo.toml
  - tools/xtask/src/main.rs
  - tools/xtask/src/package_manifest.rs
  - tools/xtask/src/partition_contract.rs
findings:
  critical: 0
  warning: 0
  info: 0
  total: 0
status: clean
---

# Phase 07: Code Review Report

**Reviewed:** 2026-06-28T19:36:25Z
**Depth:** standard
**Files Reviewed:** 39
**Status:** clean

## Summary

Re-reviewed the Phase 07 findings after orchestrator fixes, preserving the original Phase 07 file scope in frontmatter for downstream tooling. The focused re-review covered `scripts/package-firmware.sh`, `tools/xtask/src/main.rs`, `tools/xtask/src/package_manifest.rs`, `tools/parity/src/release_gate.rs`, `firmware/bitaxe/src/ota_update.rs`, and this review artifact.

The prior findings are resolved and no new actionable bugs, security issues, or code quality issues were found in the focused files. No hardware flash, OTA, erase, rollback, monitor, or interrupted-update commands were run.

## Prior Findings Rechecked

- Factory image payloads: resolved. `scripts/package-firmware.sh:276` now runs `overlay-factory-payloads` after the factory image is generated, and `tools/xtask/src/main.rs:317` overlays `www.bin` and `otadata-initial.bin` at the contract offsets. `tools/xtask/src/main.rs:417` validates those same bytes before manifest generation. The orchestrator also reported that `just package` passed and that generated `www.bin` and `otadata-initial.bin` match the factory image bytes at `0x410000` and `0xf10000`.
- Release gate row-level unknown handling: resolved. `tools/parity/src/release_gate.rs:169` now validates each line containing `unknown` rather than the whole section, and `tools/parity/src/release_gate.rs:512` adds a regression test for the section-wide false-pass case. The orchestrator reported the release-gate cargo test and `bazel run //tools/parity:report -- release-gate` passed.
- OTA progress reporting: resolved. `firmware/bitaxe/src/ota_update.rs:130` now emits progress from `progress_after_write`, which subtracts the just-written chunk before computing percent. `firmware/bitaxe/src/ota_update.rs:204` covers the final-chunk `100%` behavior.

## Verification Evidence

Orchestrator-provided evidence accepted for this re-review:

- `cargo test -p xtask --all-features` passed.
- `cargo test -p bitaxe-parity --all-features release_gate -- --nocapture` passed.
- `just package` passed, including `overlay-factory-payloads`.
- Manual generated-artifact check passed: `www.bin` matches factory image bytes at `0x410000`; `otadata-initial.bin` matches factory image bytes at `0xf10000`.
- `bazel run //tools/parity:report -- release-gate` passed.

All reviewed files meet quality standards. No issues found.

***

_Reviewed: 2026-06-28T19:36:25Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: standard_
