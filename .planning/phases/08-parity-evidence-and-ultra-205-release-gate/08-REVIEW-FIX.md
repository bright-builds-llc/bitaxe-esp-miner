---
phase: 08-parity-evidence-and-ultra-205-release-gate
fixed_at: 2026-06-29T00:26:00Z
review_path: .planning/phases/08-parity-evidence-and-ultra-205-release-gate/08-REVIEW.md
iteration: 1
findings_in_scope: 2
fixed: 2
skipped: 0
status: all_fixed
---

# Phase 08: Code Review Fix Report

**Fixed at:** 2026-06-29T00:26:00Z
**Source review:** .planning/phases/08-parity-evidence-and-ultra-205-release-gate/08-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 2
- Fixed: 2
- Skipped: 0

## Fixed Issues

### WR-01: Verified-row guards accept negated live-evidence text

**Files modified:** `tools/parity/src/main.rs`
**Commit:** 5b19207
**Commit status:** fixed: requires human verification
**Applied fix:** Added a verified-row blocker guard for `FS-001`, `OTA-001`, `OTA-002`, and `REL-003` so live evidence text containing `not run`, `blocked`, `pending`, `no reachable DEVICE_URL`, or `unverified` cannot satisfy release evidence requirements. Added regression coverage where required terms appear only inside blocker language.

### WR-02: Manifest gate accepts wrong-board metadata

**Files modified:** `tools/parity/src/release_gate.rs`
**Commit:** e5303d8
**Commit status:** fixed: requires human verification
**Applied fix:** Tightened manifest-backed release-gate validation to require exact Ultra 205 release metadata and exact artifact kind/path/offset tuples parsed from structured JSON. Added a regression test proving a Gamma 601/BM1370 board 601 manifest is rejected.

***

_Fixed: 2026-06-29T00:26:00Z_
_Fixer: the agent (gsd-code-fixer)_
_Iteration: 1_
