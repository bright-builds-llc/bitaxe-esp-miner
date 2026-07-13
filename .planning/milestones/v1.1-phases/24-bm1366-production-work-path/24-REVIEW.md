---
phase: 24-bm1366-production-work-path
reviewed: 2026-07-05T01:20:00Z
depth: standard
files_reviewed: 19
files_reviewed_list:
  - crates/bitaxe-asic/src/bm1366/production.rs
  - crates/bitaxe-asic/src/bm1366.rs
  - crates/bitaxe-asic/BUILD.bazel
  - crates/bitaxe-stratum/src/v1/production_work.rs
  - crates/bitaxe-stratum/src/v1.rs
  - crates/bitaxe-stratum/BUILD.bazel
  - crates/bitaxe-stratum/src/v1/mining.rs
  - crates/bitaxe-stratum/src/v1/mining_loop.rs
  - crates/bitaxe-stratum/src/v1/controlled_runtime.rs
  - crates/bitaxe-stratum/src/v1/controlled_runtime/tests.rs
  - firmware/bitaxe/src/controlled_mining_runtime.rs
  - firmware/bitaxe/src/asic_adapter.rs
  - firmware/bitaxe/src/asic_adapter/status.rs
  - docs/parity/evidence/phase-24-bm1366-production-work-path/production-work.md
  - docs/parity/evidence/phase-24-bm1366-production-work-path/result-correlation.md
  - docs/parity/evidence/phase-24-bm1366-production-work-path/redaction-review.md
  - docs/parity/evidence/phase-24-bm1366-production-work-path/summary.md
  - docs/parity/checklist.md
  - .planning/phases/24-bm1366-production-work-path/24-VALIDATION.md
findings:
  critical: 0
  warning: 0
  info: 0
  total: 0
status: clean
---

# Phase 24: Code Review Report

**Reviewed:** 2026-07-05T01:20:00Z
**Depth:** standard
**Files Reviewed:** 19
**Status:** clean

## Summary

Reviewed the Phase 24 BM1366 production work path using `24-01-SUMMARY.md` through `24-04-SUMMARY.md` for scope, plus `AGENTS.md`, `AGENTS.bright-builds.md`, `standards/index.md`, `standards/core/architecture.md`, `standards/core/code-shape.md`, `standards/core/verification.md`, `standards/core/testing.md`, and `standards/languages/rust.md` for local review rules. The previously reported generation-stamping bug is fixed, and Phase 24 now explicitly treats nonce-vs-target proof and share-hash validation as non-claims instead of advertised implemented validation.

## Resolved Findings

### CR-01: Guarded loop restamps nonce results as the current generation

**File:** `crates/bitaxe-stratum/src/v1/mining_loop.rs:191-196`

**Resolution:** Fixed. `GuardedMiningLoopInputs` now accepts `maybe_nonce_observation: Option<ProductionNonceObservation>` and passes the stamped observation through to `ProductionWorkRegistry::correlate_nonce_result` without restamping it. The controlled runtime creates the observation before calling the guarded loop, and a regression test proves a stale generation-0 observation is blocked after clean-jobs invalidation and reuse of the same BM1366 job lookup key in generation 1.

### WR-01: Target mismatch guard does not validate the nonce result against the target

**File:** `crates/bitaxe-stratum/src/v1/production_work.rs:325-332`

**Resolution:** Resolved as an explicit non-claim. Phase 24 keeps existing controlled-runtime behavior and does not implement full nonce-vs-target proof or share-hash validation. The code now describes `TargetMismatch` as stored work-context drift, and the Phase 24 evidence/checklist language explicitly records nonce-vs-target proof and share-hash validation as non-claims.

## Review Notes

No raw BM1366 frames, pool endpoints, credential values, device URLs, IP/MAC values, Wi-Fi values, or raw share payloads were found in the Phase 24 evidence claim files. The checklist rows for ASIC-09 through ASIC-12 remain at `implemented` with `unit,workflow` evidence and preserve Phase 25 and Phase 26 non-claims.

_Reviewed: 2026-07-05T01:20:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
