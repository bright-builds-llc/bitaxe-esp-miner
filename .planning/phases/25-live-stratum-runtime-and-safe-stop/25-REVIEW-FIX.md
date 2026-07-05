---
phase: 25-live-stratum-runtime-and-safe-stop
fixed_at: 2026-07-05T02:44:27Z
review_path: .planning/phases/25-live-stratum-runtime-and-safe-stop/25-REVIEW.md
iteration: 1
findings_in_scope: 4
fixed: 4
skipped: 0
status: all_fixed
---

# Phase 25: Code Review Fix Report

**Fixed at:** 2026-07-05T02:44:27Z
**Source review:** `.planning/phases/25-live-stratum-runtime-and-safe-stop/25-REVIEW.md`
**Iteration:** 1

**Summary:**
- Findings in scope: 4
- Fixed: 4
- Skipped: 0

## Fixed Issues

### WR-01: Firmware Live Socket Path Stops After One Server Line

**Status:** fixed: requires human verification
**Files modified:** `firmware/bitaxe/src/live_stratum_runtime.rs`
**Commit:** 050437a
**Applied fix:** Replaced the single socket read with a bounded socket pump that alternates queued runtime writes and socket reads, checkpoints the watchdog, handles fallback on socket errors, and safe-stops through verification cleanup after the bounded loop. Added a fake-socket test that feeds subscribe, authorize, set-difficulty, notify, and response lines and verifies subscribe/authorize writes plus the active marker before cleanup stop.
**Verification:** Re-read the changed runtime and test sections. `cargo test -p bitaxe-firmware live_socket_loop_progresses_through_notify_before_cleanup_stop` could not run in this host environment because `esp-idf-sys` rejects target `aarch64-apple-darwin`; no source rollback was required because this was an environment limitation before firmware test execution.

### WR-02: Hardware Evidence Mode Never Attempts Live Capture

**Status:** fixed: requires human verification
**Files modified:** `scripts/phase25-live-stratum-evidence.sh`, `scripts/phase25-live-stratum-evidence-test.sh`
**Commit:** 75600a1
**Applied fix:** Split hardware-ready behavior so detector, board-info, local pool credentials, and explicit origin now trigger a bounded repo-owned live capture helper before writing `live_socket_response_not_observed`. Added redacted success and attempted-but-not-observed evidence paths plus tests proving the helper is invoked in the ready case and skipped when prerequisites are missing.
**Verification:** Re-read the changed script and test sections. `bash -n scripts/phase25-live-stratum-evidence.sh scripts/phase25-live-stratum-evidence-test.sh` passed. `bash scripts/phase25-live-stratum-evidence-test.sh` passed.

### WR-03: Fake Pool Counts Any Failed Response As Rejected Share

**Status:** fixed: requires human verification
**Files modified:** `crates/bitaxe-stratum/src/v1/fake_pool.rs`
**Commit:** 64ddb4a
**Applied fix:** Derived the expected client kind before failure handling and now records rejected-share counters only for failed `SubmitShare` responses. Failed authorize/subscribe/non-submit responses block work submission or mark the lifecycle error without incrementing rejected-share counters. Added a regression test for authorize failure.
**Verification:** Re-read the changed fake-pool section. `cargo test -p bitaxe-stratum fake_pool` passed with 10 tests.

### WR-04: `--device-url` Validation Does Not Enforce Origin-Only Input

**Status:** fixed: requires human verification
**Files modified:** `scripts/phase25-live-stratum-evidence.sh`, `scripts/phase25-live-stratum-evidence-test.sh`
**Commit:** b6e45db
**Applied fix:** Replaced prefix-only URL validation with origin-only parsing that requires `http` or `https`, a host, optional numeric port, and no userinfo, path beyond a single trailing slash, query, fragment, bracketed/malformed host, or invalid port. Added wrapper tests for accepted origins and rejected path/userinfo/query/fragment/bad-port examples.
**Verification:** Re-read the changed validation and tests. `bash -n scripts/phase25-live-stratum-evidence.sh scripts/phase25-live-stratum-evidence-test.sh` passed. `bash scripts/phase25-live-stratum-evidence-test.sh` passed.

## Skipped Issues

None - all in-scope findings were fixed.

***

_Fixed: 2026-07-05T02:44:27Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
