---
phase: 04-stratum-v1-and-first-mining-loop
reviewed: 2026-06-27T15:17:22Z
depth: standard
files_reviewed: 23
files_reviewed_list:
  - Cargo.lock
  - MODULE.bazel.lock
  - crates/bitaxe-stratum/BUILD.bazel
  - crates/bitaxe-stratum/Cargo.toml
  - crates/bitaxe-stratum/fixtures/v1/fake-pool-transcripts.json
  - crates/bitaxe-stratum/fixtures/v1/mining-job-cases.json
  - crates/bitaxe-stratum/fixtures/v1/protocol-cases.json
  - crates/bitaxe-stratum/src/error.rs
  - crates/bitaxe-stratum/src/jsonrpc.rs
  - crates/bitaxe-stratum/src/lib.rs
  - crates/bitaxe-stratum/src/v1.rs
  - crates/bitaxe-stratum/src/v1/coinbase.rs
  - crates/bitaxe-stratum/src/v1/fake_pool.rs
  - crates/bitaxe-stratum/src/v1/messages.rs
  - crates/bitaxe-stratum/src/v1/mining.rs
  - crates/bitaxe-stratum/src/v1/mining_loop.rs
  - crates/bitaxe-stratum/src/v1/queue.rs
  - crates/bitaxe-stratum/src/v1/state.rs
  - docs/parity/checklist.md
  - docs/parity/evidence/phase-04-stratum-v1-mining-loop.md
  - firmware/bitaxe/src/asic_adapter.rs
  - firmware/bitaxe/src/asic_adapter/status.rs
  - firmware/bitaxe/src/main.rs
findings:
  critical: 0
  warning: 4
  info: 0
  total: 4
status: issues_found
---

# Phase 04: Code Review Report

**Reviewed:** 2026-06-27T15:17:22Z
**Depth:** standard
**Files Reviewed:** 23
**Status:** issues_found

## Summary

Reviewed the Phase 04 Stratum v1 pure core, fake-pool fixtures, first mining-loop gate, firmware status logging, and parity evidence. The firmware shell remains fail-closed by default, and no critical security issues were found.

The main concerns are mining correctness and parity evidence quality: clean-job notifications are not enforced at the queue boundary, ASIC nonce results are correlated with queued work instead of active work, negotiated version rolling is parsed but discarded during work construction, and one checklist row overstates smoke/soak evidence status.

Local guidance materially considered: `AGENTS.md`, `AGENTS.bright-builds.md`, `standards/index.md`, `standards/core/architecture.md`, `standards/core/code-shape.md`, `standards/core/verification.md`, `standards/core/testing.md`, and `standards/languages/rust.md`. No repo-local project skills were present.

## Warnings

### WR-01: Clean-Jobs Notifications Do Not Clear Stale Work On Enqueue

**File:** `crates/bitaxe-stratum/src/v1/queue.rs:80`
**Issue:** `MiningWork` carries `clean_jobs`, but `MiningWorkQueue::enqueue_work` always appends and marks the new job valid without clearing stale queued work or stale valid-job state. The upstream V1 path clears the queue before enqueueing a `clean_jobs=true` notify. As written, old work can remain dispatchable after a clean-job notify, and a full stale queue can reject the new clean job instead of replacing old work.
**Fix:**
```rust
pub fn enqueue_work(&mut self, work: MiningWork) -> Result<(), StratumV1Error> {
    let asic_job_id = work.asic_job_id;
    if work.clean_jobs {
        self.clear_jobs();
    }

    self.queue.enqueue(work)?;
    self.valid_jobs.insert(asic_job_id);
    Ok(())
}
```
Add a focused unit test that enqueues stale work, then enqueues a `clean_jobs=true` work item and proves only the new job remains queued and valid.

### WR-02: Nonce Results Are Coupled To Queued Work Instead Of Active Work

**File:** `crates/bitaxe-stratum/src/v1/mining_loop.rs:128`
**Issue:** `GuardedMiningLoopInputs::plan` returns before looking at `maybe_nonce_result` when the queue is empty, and when the queue is non-empty it dequeues the front item before converting a nonce result into a share. In real mining, nonce results arrive after work has already been dispatched, often while no new queued work exists. The reference firmware looks up nonce results in an active-jobs table keyed by job ID, not in the pending queue. This can drop valid shares or error when a nonce belongs to a valid job that is not the dequeued front item.
**Fix:** Track dispatched work separately from pending work and process nonce results against active work before or independently from dequeueing new work.
```rust
let maybe_share_submission = self
    .maybe_nonce_result
    .and_then(|result| {
        self.active_work
            .get(&result.job_id.lookup_key())
            .map(|work| (work, result))
    })
    .map(|(work, result)| ShareSubmission::from_nonce_result(work, result))
    .transpose()?;
```
Add tests for a nonce result with an empty pending queue but matching active work, and for a queued front job that differs from a valid active result.

### WR-03: Version-Rolling Mask Is Parsed But Discarded During Work Construction

**File:** `crates/bitaxe-stratum/src/v1/mining.rs:156`
**Issue:** `build_work_fields_with_extranonce2` accepts `_maybe_version_mask` but ignores it. The parser and fixtures expose `mining.configure`, `mining.set_version_mask`, and version-bit submission, while the reference mining path uses the negotiated mask to prepare rolled versions/midstates and configure the ASIC. This creates a parity gap: the Rust core appears to support version rolling but generated BM1366 work does not honor the negotiated mask.
**Fix:** Either thread `VersionMask` into the BM1366 work/dispatch model and test non-zero mask behavior, or fail closed until version rolling is implemented.
```rust
if let Some(mask) = maybe_version_mask {
    if mask.mask != 0 {
        return Err(StratumV1Error::InvalidField {
            field: "version_mask",
            reason: "version rolling work generation is not implemented",
        });
    }
}
```
If implemented instead, add a non-zero-mask fixture proving the work/dispatch plan changes according to the negotiated mask.

### WR-04: Mining Smoke And Soak Evidence Row Overstates Status

**File:** `docs/parity/checklist.md:80`
**Issue:** `STR-007` is marked `implemented | workflow` for "Mining smoke and soak evidence", but the linked evidence file says hardware smoke and soak are "not run - hardware evidence pending." Because this checklist is the parity source of truth, marking the evidence surface implemented can mislead downstream readers into treating live mining evidence as present when only criteria were recorded.
**Fix:** Split the row or downgrade the live-evidence status. For example, keep a separate "smoke/soak criteria recorded" row as implemented, and mark live mining smoke/soak evidence as `not-started` or `in-progress` until a hardware run is recorded.

---

_Reviewed: 2026-06-27T15:17:22Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: standard_
