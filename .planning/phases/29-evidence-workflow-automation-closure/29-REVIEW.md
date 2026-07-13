---
phase: 29-evidence-workflow-automation-closure
reviewed: 2026-07-13T03:29:35Z
generated_at: 2026-07-13T03:29:35Z
depth: standard
status: issues_found
generated_by: gsd-code-reviewer
lifecycle_mode: yolo
phase_lifecycle_id: 29-2026-07-13T00-19-45
files_reviewed: 40
files_reviewed_list:
  - Cargo.toml
  - tools/parity/Cargo.toml
  - tools/parity/BUILD.bazel
  - tools/parity/src/main.rs
  - tools/parity/src/operator_evidence.rs
  - tools/parity/src/operator_evidence/inventory.rs
  - tools/parity/src/operator_evidence/profile.rs
  - tools/parity/src/operator_evidence/tests.rs
  - tools/parity/src/operator_evidence/generation.rs
  - tools/parity/src/operator_evidence/generation/rendering.rs
  - tools/parity/src/operator_evidence/generation/filesystem.rs
  - tools/parity/src/operator_evidence/generation/tests.rs
  - tools/parity/src/operator_evidence/generation/tests/promotion.rs
  - scripts/phase23-redacted-operator-evidence.sh
  - scripts/phase23-redacted-operator-evidence-test.sh
  - scripts/phase25-live-stratum-evidence.sh
  - scripts/phase25-live-stratum-evidence-test.sh
  - scripts/phase27-live-hardware-bridge-evidence.sh
  - scripts/phase27-live-hardware-bridge-evidence-test.sh
  - scripts/phase28-evidence.sh
  - scripts/phase28-evidence-test.sh
  - scripts/phase29-doc-redaction-check.sh
  - scripts/phase29-doc-redaction-check-test.sh
  - scripts/BUILD.bazel
  - Justfile
  - docs/release/ultra-205.md
  - docs/parity/evidence/phase-29-evidence-workflow-automation-closure/summary.md
  - docs/parity/evidence/phase-29-evidence-workflow-automation-closure/redaction-review.md
  - docs/parity/evidence/phase-29-evidence-workflow-automation-closure/conclusion.md
  - AGENTS.md
  - AGENTS.bright-builds.md
  - standards-overrides.md
  - standards/index.md
  - standards/core/architecture.md
  - standards/core/code-shape.md
  - standards/core/operability.md
  - standards/core/local-guidance.md
  - standards/core/verification.md
  - standards/core/testing.md
  - standards/languages/rust.md
findings:
  critical: 1
  warning: 0
  info: 0
  total: 1
---

# Phase 29: Code Review Report

**Reviewed:** 2026-07-13T03:29:35Z
**Depth:** standard
**Files Reviewed:** 40
**Status:** issues_found

## Summary

The cleanup-durability finding is resolved: post-removal sync failures now use `DurabilityUncertain` without a retained-generation locator, and injected tests prove the reported state matches the destination and staging paths.

The recursive scanner also rejects the exact plain vendor-SSID and Stratum-URI fixtures from iteration 2 without printing their values. However, the new redaction-placeholder exemption is prefix-based and introduces a bypass: appending a raw value after a redaction marker is accepted as safe. A production-binary probe confirmed that both a pool host in URI authority and an SSID suffix pass strict Phase 27 validation. The redaction finding is therefore not fully resolved.

Verification completed without hardware or credential access:

- `cargo fmt --all -- --check` passed.
- `cargo clippy -p bitaxe-parity --all-targets --all-features -- -D warnings` passed.
- `cargo test -p bitaxe-parity` passed: 172 tests.
- `bazel test //scripts:phase27_live_hardware_bridge_evidence_test //scripts:phase28_evidence_test //tools/parity:tests` passed.
- `bash -n scripts/phase27-live-hardware-bridge-evidence-test.sh` passed.
- The category-only production probe returned zero for both redaction-prefix suffix cases; its temporary evidence root and outputs were removed.

## Critical Issues

### CR-01: Redaction-marker prefixes can hide appended raw runtime values

**File:** `/Users/peterryszkiewicz/Repos/bitaxe-esp-miner/tools/parity/src/operator_evidence/inventory.rs:160-187`
**Issue:** `contains_unredacted_wifi_ssid` accepts any value whose remainder starts with `[redacted-ssid]`, and `contains_unredacted_url` accepts any URI whose post-scheme text starts with `[redacted`. Neither function verifies that the placeholder is the complete sensitive field or URI authority. Consequently, strict validation accepts a raw SSID appended to `[redacted-ssid]` and a raw pool host appended after a redacted URI user-info prefix. The production `operator-evidence --profile phase27 --require-redaction-passed` command exited zero for both cases, so commit-ready evidence can still contain private runtime values while reporting redaction passed.

**Fix:** Require an exact, closed redaction token at the field boundary instead of `starts_with`, or reject every URI-shaped value in commit-ready artifacts and retain only scheme-free category labels. For vendor Wi-Fi lines, parse the SSID field through its delimiter and require it to equal `[redacted-ssid]` exactly. Add Rust and production-wrapper regressions for redaction markers followed by suffix text or URI authority data, and continue asserting that failure output contains only the artifact path/category.

***

_Reviewed: 2026-07-13T03:29:35Z_
_Reviewer: gsd-code-reviewer_
_Depth: standard_
