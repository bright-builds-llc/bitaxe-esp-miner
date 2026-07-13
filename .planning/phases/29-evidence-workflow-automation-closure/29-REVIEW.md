---
phase: 29-evidence-workflow-automation-closure
reviewed: 2026-07-13T02:10:01Z
generated_at: 2026-07-13T02:10:01Z
depth: standard
status: issues_found
generated_by: gsd-code-reviewer
lifecycle_mode: yolo
phase_lifecycle_id: 29-2026-07-13T00-19-45
files_reviewed: 37
files_reviewed_list:
  - Cargo.toml
  - tools/parity/Cargo.toml
  - tools/parity/BUILD.bazel
  - tools/parity/src/main.rs
  - tools/parity/src/operator_evidence.rs
  - tools/parity/src/operator_evidence/profile.rs
  - tools/parity/src/operator_evidence/generation.rs
  - tools/parity/src/operator_evidence/generation/rendering.rs
  - tools/parity/src/operator_evidence/generation/filesystem.rs
  - tools/parity/src/operator_evidence/generation/tests.rs
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
  warning: 8
  info: 0
  total: 9
---

# Phase 29: Code Review Report

**Reviewed:** 2026-07-13T02:10:01Z  
**Depth:** standard  
**Files Reviewed:** 37  
**Status:** issues_found

## Summary

The Phase 29 unit suite passes (`cargo test -p bitaxe-parity`: 152 passed), but isolated integration probes against the real parity binary show that the documented Phase 25/27 completion and Phase 28 consolidation flows do not interoperate with their actual producers. The review also found a commit-ready redaction bypass, stale generated-state promotion, atomic rollback error masking, and tests/docs that assert behavior through fakes rather than the production generator.

The review was informed by the repo-local hardware/redaction constraints and the Bright Builds architecture, code-shape, verification, testing, and Rust standards. No hardware, credentials, raw private evidence, direct UART, pins, or reference modifications were accessed.

## Critical Issues

### CR-01: Strict validation ignores artifacts that can contain raw runtime values

**File:** `/Users/peterryszkiewicz/Repos/bitaxe-esp-miner/tools/parity/src/operator_evidence.rs:84-106`  
**Issue:** `load_operator_evidence_documents` reads only the eleven named Markdown slots. It does not inventory or scan `summary.md`, `mining-allow.json`, or nested `live-capture-runtime` artifacts. Phase 27 permits hardware mode without requiring `--redact-evidence=true`, and only rewrites `flash-monitor.log` when that flag is present (`scripts/phase27-live-hardware-bridge-evidence.sh:745-760`). The wrapper can therefore end with `redaction_status: passed` and strict operator validation while raw device URLs, IPs, MACs, or other runtime values remain under the same evidence root. The shell tests scan only top-level `*.md` files, so they do not detect this leak.

**Fix:** Make commit-ready validation own a typed, explicit inventory of every allowed file and directory, reject symlinks and unknown entries, and scan all allowed regular artifacts recursively without printing matches. Require `--redact-evidence=true` for Phase 27 hardware evidence before capture starts, and add a test placing a sentinel in a nested runtime log and in `mining-allow.json` that must fail validation.

## Warnings

### WR-01: Phase 23, 25, and 27 wrappers cannot satisfy the new strict profile validator

**File:** `/Users/peterryszkiewicz/Repos/bitaxe-esp-miner/scripts/phase25-live-stratum-evidence.sh:175-212`  
**Issue:** Wrapper-authored slots do not include the now-required `evidence_profile` or `evidence_disposition` fields, while completion preserves every existing slot. The same mismatch exists in Phase 23 and Phase 27. An isolated Phase 25 blocked run with the real binary failed on missing profile/disposition fields for all wrapper-authored slots. It also failed because Phase 25 blocked evidence records `safe_stop_status: complete` and no `asic_bridge_status: blocked`, while the shared blocked-outcome validator requires both blocked markers. A successful Phase 25 run would additionally emit `share_outcome: live_submit_response_observed`, which is not represented by `ShareOutcome`.

**Fix:** Define profile-specific slot renderers and outcome types shared by the wrappers and validator. Emit exactly one profile/disposition/status field in every authored slot, model Phase 25's live-submit and completed-safe-stop semantics explicitly, and run each real wrapper against the real parity binary in tests.

### WR-02: Phase 28 rejects every outcome shape produced by Phase 27

**File:** `/Users/peterryszkiewicz/Repos/bitaxe-esp-miner/tools/parity/src/operator_evidence/generation/rendering.rs:103-139`  
**Issue:** Accepted/rejected consolidation requires `asic_correlation_status: passed` and `safe_stop_status: passed`, but Phase 27 writes `asic_bridge_status: result_correlated` and `safe_stop_status: complete`. Blocked consolidation requires `safe_stop_status: blocked`, while Phase 27's blocked evidence writes `safe_stop_status: complete`. Isolated fixtures matching the real Phase 27 fields failed with `accepted source outcome requires asic_correlation_status: passed` and `blocked_safe_prerequisite source requires safe_stop_status: blocked`.

**Fix:** Parse the actual Phase 27 category contract into a typed source record, normalize `result_correlated`/`complete` deliberately, and add end-to-end tests that feed Phase 27 wrapper output directly into the production Phase 28 command for accepted, rejected, and blocked outcomes.

### WR-03: Generated failure placeholders survive later successful reruns

**File:** `/Users/peterryszkiewicz/Repos/bitaxe-esp-miner/tools/parity/src/operator_evidence/generation.rs:118-136`  
**Issue:** Completion skips every existing file, including its own `generated_provenance: phase29-completion` placeholders. A failed run can create blocked slots with `workflow_status: failed`; a later successful run preserves those bytes. The validator allows blocked/deferred dispositions broadly and only rejects `CrossLinked` for observation-required slots, so the wrapper can return success with stale failed/blocked generated slots. Phase 25 is especially exposed because it never authors `conclusion.md` and will retain an earlier generated blocked conclusion.

**Fix:** On rerun, replace only generator-owned slots atomically using their provenance marker and current workflow state, preserve only genuinely observed wrapper-owned slots, and require a passed workflow to contain no stale `workflow_status: failed` generated slots. Add failed-to-passed and blocked-to-passed rerun tests.

### WR-04: Production Phase 28 output omits the summary asserted by its shell test

**File:** `/Users/peterryszkiewicz/Repos/bitaxe-esp-miner/tools/parity/src/operator_evidence/generation/rendering.rs:156-200`  
**Issue:** The production generator writes eleven slots plus the manifest, but never writes `summary.md`. The shell test expects thirteen files including `summary.md`, yet uses a fake parity implementation that manufactures that file. A direct production consolidation produced 12 files and `summary_exists=no`, so the test and Phase 29 closure evidence overstate the real inventory.

**Fix:** Either generate and validate a deterministic `summary.md` in production and include it in the managed inventory, or remove the summary contract consistently. Replace the fake-generator inventory assertion with a test invoking the production parity binary.

### WR-05: Rollback hides the original parent-sync failure and can underreport recovery state

**File:** `/Users/peterryszkiewicz/Repos/bitaxe-esp-miner/tools/parity/src/operator_evidence/generation/filesystem.rs:63-117`  
**Issue:** On a real parent-directory sync failure, `rollback_exchange(...).and(Err(error))` returns the rollback helper's synthetic `Injected(DuringParentSync)` error rather than the original I/O error. If the sync after exchanging back fails, the function returns a plain `Io` before cleanup even though both generations remain and durability is uncertain, rather than reporting `RecoveryRequired` with the retained path.

**Fix:** Make rollback return a success/recovery-state value instead of an injected error, return the original sync error after a fully successful rollback, and wrap any post-rollback sync or cleanup failure in `RecoveryRequired` with both paths and explicit durability state. Test with an injectable filesystem/sync adapter rather than mapping an enum directly to rollback.

### WR-06: Profile validation accepts contradictory status fields and can omit share outcome entirely

**File:** `/Users/peterryszkiewicz/Repos/bitaxe-esp-miner/tools/parity/src/operator_evidence.rs:160-193`  
**Issue:** `slot_status`, required metadata, and `redaction_status` are checked with substring searches, so a document containing both `slot_status: passed` and `slot_status: blocked` can satisfy validation. `validate_share_outcome_slot` parses an outcome only when one happens to be present, so Phase 27/28 share evidence can omit the field without a specific missing-outcome error.

**Fix:** Parse every singleton field with `parse_single_field`, use typed enums for status/redaction/outcome, reject duplicates and contradictions, and make outcome presence profile-specific and mandatory for Phase 27/28. Add duplicate-field and missing-outcome regression tests.

### WR-07: Path checks and destination ownership are vulnerable to replacement between validation and exchange

**File:** `/Users/peterryszkiewicz/Repos/bitaxe-esp-miner/tools/parity/src/operator_evidence/generation/filesystem.rs:16-45`  
**Issue:** Destination ownership is established by the existence of any regular file named `.phase28-evidence-manifest`; its content is never verified. Symlink checks and destination inventory checks are path-based and occur before later opens and the atomic exchange. A concurrent replacement can therefore swap a validated destination or path component before exchange/cleanup, defeating the fail-closed check and potentially deleting a substituted directory.

**Fix:** Validate exact manifest schema/content, reject non-regular and symlink entries, capture destination identity, and perform subsequent operations relative to no-follow directory handles (or hold a repo-owned lock and revalidate inode/device identity immediately before exchange and cleanup). Add manifest-content, allowed-name-directory, symlink-entry, and replacement-race tests.

### WR-08: Operator guide promises preservation that the final validation path does not provide

**File:** `/Users/peterryszkiewicz/Repos/bitaxe-esp-miner/docs/release/ultra-205.md:501-507`  
**Issue:** The guide says generation and strict Phase 28 validation occur in staging and that either failure leaves the prior destination unchanged. The wrapper performs another strict validation only after consolidation has promoted the destination (`scripts/phase28-evidence.sh:55-58`), and the shell test explicitly exempts `operator-failure` from its destination-preservation assertion (`scripts/phase28-evidence-test.sh:321-325`). Thus the documented failure guarantee is broader than the implementation and test.

**Fix:** Make the promoted command return only after the exact final validator has passed in staging, or roll back on post-promotion validation failure. Then remove the test exemption and assert byte-identical preservation. If post-promotion validation is intentionally diagnostic only, narrow the guide's guarantee accordingly.

_Reviewed: 2026-07-13T02:10:01Z_  
_Reviewer: gsd-code-reviewer_  
_Depth: standard_
