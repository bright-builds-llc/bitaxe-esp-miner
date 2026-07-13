---
phase: 04-stratum-v1-and-first-mining-loop
fixed_at: 2026-06-27T15:29:34Z
review_path: .planning/phases/04-stratum-v1-and-first-mining-loop/04-REVIEW.md
iteration: 1
findings_in_scope: 4
fixed: 4
skipped: 0
status: all_fixed
---

# Phase 04: Code Review Fix Report

**Fixed at:** 2026-06-27T15:29:34Z
**Source review:** `.planning/phases/04-stratum-v1-and-first-mining-loop/04-REVIEW.md`
**Iteration:** 1

**Summary:**

- Findings in scope: 4
- Fixed: 4
- Skipped: 0

## Fixed Issues

### WR-01: Clean-Jobs Notifications Do Not Clear Stale Work On Enqueue

**Status:** fixed: requires human verification
**Files modified:** `crates/bitaxe-stratum/src/v1/queue.rs`
**Commit:** eefb961
**Applied fix:** `MiningWorkQueue::enqueue_work` now clears stale queued work and valid-job state before enqueueing a `clean_jobs=true` item. Added a focused unit test proving only the clean job remains queued and valid.

### WR-02: Nonce Results Are Coupled To Queued Work Instead Of Active Work

**Status:** fixed: requires human verification
**Files modified:** `crates/bitaxe-stratum/src/v1/mining_loop.rs`, `crates/bitaxe-stratum/src/v1/queue.rs`
**Commit:** b103499
**Applied fix:** Added active dispatched-work tracking to `MiningWorkQueue` and changed share conversion to use active work independently from pending queue dispatch. Added tests for empty pending queue with matching active work and for a mismatched queued front job.

### WR-03: Version-Rolling Mask Is Parsed But Discarded During Work Construction

**Status:** fixed: requires human verification
**Files modified:** `crates/bitaxe-stratum/src/v1/mining.rs`
**Commit:** ad21225
**Applied fix:** Non-zero version rolling masks now fail closed with `InvalidField` until BM1366 version rolling work generation is implemented. Added a non-zero mask unit test.

### WR-04: Mining Smoke And Soak Evidence Row Overstates Status

**Status:** fixed
**Files modified:** `docs/parity/checklist.md`, `docs/parity/evidence/phase-04-stratum-v1-mining-loop.md`
**Commit:** 5f6467f
**Applied fix:** Split recorded smoke/soak criteria from live mining smoke/soak evidence. The live evidence row is now `not-started | pending`, and the evidence file states that hardware smoke and soak remain pending.

---

_Fixed: 2026-06-27T15:29:34Z_
_Fixer: the agent (gsd-code-fixer)_
_Iteration: 1_
