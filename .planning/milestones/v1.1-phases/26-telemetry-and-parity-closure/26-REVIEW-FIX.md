---
phase: 26-telemetry-and-parity-closure
fixed_at: 2026-07-05T04:54:15Z
review_path: .planning/phases/26-telemetry-and-parity-closure/26-REVIEW.md
iteration: 3
findings_in_scope: 1
fixed: 1
skipped: 0
status: all_fixed
---

# Phase 26: Code Review Fix Report

**Fixed at:** 2026-07-05T04:54:15Z
**Source review:** `.planning/phases/26-telemetry-and-parity-closure/26-REVIEW.md`
**Iteration:** 3

**Summary:**
- Findings in scope: 1
- Fixed: 1
- Skipped: 0

## Fixed Issues

### WR-01: Closed Pool Socket Is Treated As No Data

**Status:** fixed: requires human verification
**Files modified:** `firmware/bitaxe/src/live_stratum_runtime.rs`
**Commit:** `1a5c651`
**Applied fix:** Changed zero-byte firmware TCP reads to return a redaction-safe `stratum socket closed` error so the existing pump error branch publishes reconnect status/classification and stops through `FallbackExhausted`. Extended the scripted socket test harness with an error event and added a focused regression proving a socket read error reaches reconnect lifecycle telemetry instead of verification cleanup.
**Verification:**
- Tier 1 re-read confirmed the EOF error path, scripted socket error branch, and focused reconnect regression are present and surrounding code is intact.
- `rustfmt --edition 2021 --check firmware/bitaxe/src/live_stratum_runtime.rs` passed.
- `cargo test -p bitaxe-firmware socket_read_error_publishes_reconnect_before_fallback_stop` was attempted but blocked before compiling tests because `esp-idf-sys` rejects native target `aarch64-apple-darwin`.
- `bazel build //firmware/bitaxe:firmware` passed.

## Skipped Issues

None.

_Fixed: 2026-07-05T04:54:15Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 3_
